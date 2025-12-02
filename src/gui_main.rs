use crate::args::{Args, Command};
use crate::core::{extract_objects, rebuild_objects};
use crate::ui::gui::{GuiProgressHandler, ProgressWindow, pick_output_dir};
use crate::ui::UiHandler;
use std::collections::HashMap;
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
            let mut objects = objects_clone.lock().unwrap();

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
        *worker_result_clone.lock().unwrap() = Some(result);
    });

    println!("解凍を開始します");
    progress.run_loop(rx);

    // ワーカースレッドの完了を待機
    worker_handle.join().expect("Worker thread panicked");

    // ワーカーの結果を確認
    let result = worker_result.lock().unwrap().take();
    let (success, was_cancelled) = match result {
        Some(Ok(())) => {
            println!("解凍が完了しました。");
            (true, false)
        }
        Some(Err(e)) => {
            // キャンセルとエラーを区別
            let is_cancelled = e.contains("キャンセルされました");

            if is_cancelled {
                println!("処理がキャンセルされました。");
            } else {
                use rfd::MessageDialog;
                MessageDialog::new()
                    .set_title("エラー")
                    .set_description(&format!("処理中にエラーが発生しました: {}", e))
                    .show();
                eprintln!("エラー: {}", e);
            }
            (false, is_cancelled)
        }
        None => {
            eprintln!("警告: ワーカースレッドの結果が取得できませんでした");
            (false, false)
        }
    };

    // クリーンアップ（常に実行）
    if tmp_output_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&tmp_output_dir) {
            eprintln!("警告: 一時ディレクトリの削除に失敗しました: {}", e);
        }
    }

    // 成功時またはキャンセル時にディレクトリを開く
    if success || was_cancelled {
        open_directory(&output_dir)?;
    }

    Ok(())
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
