# ![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dialog;
mod extract;
mod rebuild;
mod progress_window;

use dialog::pick_output_dir;
use extract::extract_objects;
use rebuild::rebuild_objects;
use progress_window::{ProgressWindow};

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::mpsc;
use rfd::MessageDialog;

const TMP_OUTPUT_DIR: &str = ".jp.ootr.unitypackage-extractor";

fn main() {
    if let Err(e) = run() {
        MessageDialog::new()
            .set_title("エラー")
            .set_description(&e)
            .show();
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("Usage: {} <file.tar.gz> [output_dir]", args[0]));
    }

    let filepath_str = &args[1];
    let filepath = Path::new(filepath_str);
    if !filepath.exists() {
        return Err(format!("指定されたファイルが存在しません: {}", filepath.display()));
    }

    let output_dir = if args.len() == 3 {
        args[2].clone()
    } else {
        pick_output_dir(filepath)?
    };
    let output_dir = Path::new(&output_dir);
    let tmp_output_dir = Path::new(&output_dir).join(TMP_OUTPUT_DIR);
    if tmp_output_dir.exists() {
        std::fs::remove_dir_all(&tmp_output_dir).map_err(|e| format!("一時ディレクトリの削除に失敗しました: {}", e))?;
    }

    let mut progress = ProgressWindow::new("処理中...");
    let (tx, rx) = mpsc::channel();
    let archive_path = filepath.to_path_buf();
    let output_dir_for_extract = tmp_output_dir.to_path_buf();
    let objects_arc = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
    let objects_arc_extract = std::sync::Arc::clone(&objects_arc);
    let output_dir2 = output_dir.to_path_buf();
    std::thread::spawn(move || {
        let mut objects = objects_arc_extract.lock().unwrap();
        let _ = extract_objects(&archive_path, &output_dir_for_extract, &mut *objects, &tx);
        let _ = rebuild_objects(&objects, &output_dir2, &output_dir_for_extract, &tx);
    });
    println!("解凍を開始します");
    progress.run_loop(rx);
    println!("解凍が完了しました。");
    if tmp_output_dir.exists() {
        std::fs::remove_dir_all(&tmp_output_dir).map_err(|e| format!("一時ディレクトリの削除に失敗しました: {}", e))?;
    }

    if let Err(e) = std::process::Command::new("explorer").arg(&output_dir).status() {
        eprintln!("エクスプローラーの起動に失敗しました: {}", e);
    }
    Ok(())
}
