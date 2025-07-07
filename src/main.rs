mod dialog;
mod extract;
mod rebuild;
mod progress_window;

use dialog::pick_output_dir;
use extract::extract_objects;
use rebuild::rebuild_objects;
use progress_window::ProgressWindow;

use std::collections::HashMap;
use std::env;
use std::path::Path;
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
    
    let mut objects = HashMap::new();
    // 進捗ウィンドウ生成
    let progress = ProgressWindow::new("処理中...", 1); // 仮のmax値、後で関数内で調整
    extract_objects(filepath, &tmp_output_dir, &mut objects, &progress)?;
    println!("解凍が完了しました。");
    rebuild_objects(&objects, &output_dir, &tmp_output_dir, &progress)?;
    progress.close();
    if tmp_output_dir.exists() {
        std::fs::remove_dir_all(&tmp_output_dir).map_err(|e| format!("一時ディレクトリの削除に失敗しました: {}", e))?;
    }
    Ok(())
}
