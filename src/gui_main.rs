use crate::args::Args;
use crate::core::{extract_objects, rebuild_objects};
use crate::ui::gui::{GuiProgressHandler, ProgressWindow, pick_output_dir};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;

const TMP_OUTPUT_DIR: &str = ".jp.ootr.unitypackage-extractor";

pub fn run() -> Result<(), String> {
    let args = Args::parse()?;

    let input_file = &args.input_file;
    if !input_file.exists() {
        return Err(format!("指定されたファイルが存在しません: {}", input_file.display()));
    }

    let output_dir = if let Some(dir) = args.output_dir {
        dir
    } else {
        let dir_str = pick_output_dir(input_file)?;
        std::path::PathBuf::from(dir_str)
    };

    let tmp_output_dir = output_dir.join(TMP_OUTPUT_DIR);
    if tmp_output_dir.exists() {
        std::fs::remove_dir_all(&tmp_output_dir)
            .map_err(|e| format!("一時ディレクトリの削除に失敗しました: {}", e))?;
    }

    let mut progress = ProgressWindow::new("処理中...");
    let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (mut ui_handler, rx) = GuiProgressHandler::new(Arc::clone(&cancelled));

    let objects = Arc::new(Mutex::new(HashMap::new()));
    let objects_clone = Arc::clone(&objects);

    let input_file = input_file.to_path_buf();
    let tmp_output_dir_clone = tmp_output_dir.clone();
    let output_dir_clone = output_dir.clone();

    // 処理スレッド起動
    std::thread::spawn(move || {
        let mut objects = objects_clone.lock().unwrap();
        let _ = extract_objects(&input_file, &tmp_output_dir_clone, &mut *objects, &mut ui_handler);
        let _ = rebuild_objects(&objects, &output_dir_clone, &tmp_output_dir_clone, &mut ui_handler);
    });

    println!("解凍を開始します");
    progress.run_loop(rx);
    println!("解凍が完了しました。");

    // クリーンアップ
    if tmp_output_dir.exists() {
        std::fs::remove_dir_all(&tmp_output_dir)
            .map_err(|e| format!("一時ディレクトリの削除に失敗しました: {}", e))?;
    }

    // ディレクトリを開く
    open_directory(&output_dir)?;

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
