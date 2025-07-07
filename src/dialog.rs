use rfd::{FileDialog, MessageDialog};
use std::env;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static OVERWRITE_ALL: OnceLock<Mutex<Option<bool>>> = OnceLock::new();

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

pub fn confirm_overwrite(path: &Path) -> bool {
    let overwrite_all = OVERWRITE_ALL.get_or_init(|| Mutex::new(None));
    if let Some(val) = *overwrite_all.lock().unwrap() {
        return val;
    }
    let msg = format!("既にファイルが存在します。上書きしますか？\n{}", path.display());
    let result = rfd::MessageDialog::new()
        .set_title("上書き確認")
        .set_description(&msg)
        .set_buttons(rfd::MessageButtons::YesNoCancel)
        .set_level(rfd::MessageLevel::Warning)
        .show();
    let apply_all = || {
        rfd::MessageDialog::new()
            .set_title("確認")
            .set_description("今後すべてのファイルにこの操作を適用しますか？")
            .set_buttons(rfd::MessageButtons::YesNo)
            .set_level(rfd::MessageLevel::Info)
            .show() == rfd::MessageDialogResult::Yes
    };
    match result {
        rfd::MessageDialogResult::Yes => {
            if apply_all() {
                *overwrite_all.lock().unwrap() = Some(true);
            }
            true
        },
        rfd::MessageDialogResult::No => {
            if apply_all() {
                *overwrite_all.lock().unwrap() = Some(false);
            }
            false
        },
        _ => false,
    }
}
