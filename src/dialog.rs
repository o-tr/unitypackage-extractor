use rfd::{FileDialog, MessageDialog};
use std::env;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

// 出力先ディレクトリ選択ダイアログ関連
pub fn pick_output_dir(archive_path: &Path) -> Result<String, String> {
    let abs_path = env::current_dir().map_err(|e| format!("カレントディレクトリ取得失敗: {}", e))?.join(archive_path);
    let parent_path = abs_path.parent().unwrap_or_else(|| Path::new(""));
    let dir = FileDialog::new()
        .set_title("解凍先フォルダーを選択してください")
        .set_directory(parent_path)
        .pick_folder();
    match dir {
        Some(path) => Ok(path.display().to_string()),
        None => {
            MessageDialog::new()
                .set_title("エラー")
                .set_description("解凍先フォルダーが選択されませんでした。")
                .show();
            Err("解凍先フォルダーが選択されませんでした。".to_string())
        }
    }
}
