use crate::args::{Args, Command};
use crate::core::{extract_objects, rebuild_objects};
use crate::ui::gui::{GuiProgressHandler, ProgressWindow, pick_output_dir};
use crate::ui::UiHandler;
use std::collections::HashMap;
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};
use std::path::Path;

const TMP_OUTPUT_DIR: &str = ".jp.ootr.unitypackage-extractor";

pub fn run() -> Result<(), String> {
    let args = Args::parse()?;

    // GUI版は現在extractのみサポート
    match &args.command {
        Command::Extract { input_file, output_dir, overwrite_mode } => {
            run_extract(input_file, output_dir.as_ref(), *overwrite_mode)
        }
        Command::Compress { .. } => {
            Err("GUI版ではcompressコマンドはサポートされていません。CLI版を使用してください。".to_string())
        }
    }
}

fn lock_or_error<'a, T>(mutex: &'a Mutex<T>, context: &str) -> Result<MutexGuard<'a, T>, String> {
    mutex
        .lock()
        .map_err(|e| format!("{}: {}", context, e))
}

fn run_extract(
    input_file: &std::path::PathBuf,
    output_dir: Option<&std::path::PathBuf>,
    overwrite_mode: crate::ui::OverwriteMode,
) -> Result<(), String> {
    if !input_file.exists() {
        return Err(format!("指定されたファイルが存在しません: {}", input_file.display()));
    }

    let output_dir = if let Some(dir) = output_dir {
        dir.clone()
    } else {
        let dir_str = pick_output_dir(input_file)?;
        std::path::PathBuf::from(dir_str)
    };

    let tmp_output_dir = output_dir.join(TMP_OUTPUT_DIR);
    if tmp_output_dir.exists() {
        std::fs::remove_dir_all(&tmp_output_dir)
            .map_err(|e| format!("一時ディレクトリの削除に失敗しました: {}", e))?;
    }

    let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut progress = ProgressWindow::new("処理中...", Arc::clone(&cancelled));
    let (mut ui_handler, rx) = GuiProgressHandler::new(Arc::clone(&cancelled), overwrite_mode);

    let objects = Arc::new(Mutex::new(HashMap::new()));
    let objects_clone = Arc::clone(&objects);

    // ワーカーの結果を共有するための変数
    let worker_result: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
    let worker_result_clone = Arc::clone(&worker_result);

    let input_file = input_file.to_path_buf();
    let tmp_output_dir_clone = tmp_output_dir.clone();
    let output_dir_clone = output_dir.clone();

    // 処理スレッド起動
    let worker_handle = std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let mut objects = lock_or_error(&objects_clone, "共有オブジェクトのロックに失敗しました")?;

            // extractionを実行
            extract_objects(&input_file, &tmp_output_dir_clone, &mut *objects, &mut ui_handler)?;

            // キャンセルチェック
            if ui_handler.is_cancelled() {
                return Err("キャンセルされました".to_string());
            }

            // rebuildを実行
            rebuild_objects(&objects, &output_dir_clone, &tmp_output_dir_clone, &mut ui_handler)?;

            Ok(())
        })();

        // 結果を共有メモリに保存
        match worker_result_clone.lock() {
            Ok(mut worker_result_slot) => {
                *worker_result_slot = Some(result);
            }
            Err(e) => {
                // mutex poison は致命的ではあるが、少なくとも結果が `None` になって失われるのは避ける
                let lock_err = format!("{}", e);
                let mut worker_result_slot = e.into_inner();
                let stored = match result {
                    Ok(()) => Err(format!(
                        "ワーカースレッドの結果保存で mutex が poison されました: {}",
                        lock_err
                    )),
                    Err(worker_err) => Err(format!(
                        "ワーカースレッド処理に失敗しました: {}; さらに結果保存の mutex が poison されました: {}",
                        worker_err, lock_err
                    )),
                };
                *worker_result_slot = Some(stored);
            }
        }
    });

    progress.run_loop(rx);

    // ワーカースレッドの完了を待機
    if let Err(payload) = worker_handle.join() {
        let panic_message = if let Some(msg) = payload.downcast_ref::<&str>() {
            msg.to_string()
        } else if let Some(msg) = payload.downcast_ref::<String>() {
            msg.clone()
        } else {
            "ワーカースレッドがpanicしました".to_string()
        };
        return Err(format!("内部エラー: {}", panic_message));
    }

    // ワーカーの結果を確認
    // poison で lock() 自体が失敗しても、結果が `None` になるのは避ける
    let mut poisoned_lock_msg: Option<String> = None;
    let result = match worker_result.lock() {
        Ok(mut slot) => slot.take(),
        Err(e) => {
            poisoned_lock_msg = Some(format!("ワーカー結果の mutex lock が poison されました: {}", e));
            e.into_inner().take()
        }
    };

    let (success, worker_error) = match result {
        Some(Ok(())) => {
            (true, None)
        }
        Some(Err(e)) => {
            // キャンセルとエラーを区別
            // エラー文言ではなくキャンセルフラグを唯一の判定根拠にする
            let is_cancelled = cancelled.load(std::sync::atomic::Ordering::SeqCst);
            if !is_cancelled {
                eprintln!("エラー: {}", e);
                (false, Some(e))
            } else {
                (false, None)
            }
        }
        None => {
            eprintln!("警告: ワーカースレッドの結果が取得できませんでした");
            let msg = poisoned_lock_msg.unwrap_or_else(|| {
                "ワーカースレッドの結果が取得できませんでした（保存失敗の可能性）".to_string()
            });
            (false, Some(msg))
        }
    };

    // クリーンアップ（常に実行）
    if tmp_output_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&tmp_output_dir) {
            eprintln!("警告: 一時ディレクトリの削除に失敗しました: {}", e);
        }
    }

    // 成功時のみディレクトリを開く
    if success {
        open_directory(&output_dir)?;
    }

    if let Some(e) = worker_error {
        Err(e)
    } else {
        Ok(())
    }
}

fn open_directory(path: &Path) -> Result<(), String> {
    let open_result = match std::env::consts::OS {
        "windows" => std::process::Command::new("explorer").arg(&path).status(),
        "macos" => std::process::Command::new("open").arg(&path).status(),
        _ => std::process::Command::new("xdg-open").arg(&path).status(),
    };

    if let Err(e) = open_result {
        eprintln!("ディレクトリのオープンに失敗しました: {}", e);
    }

    Ok(())
}
