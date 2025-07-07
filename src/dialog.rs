use rfd::FileDialog;
use std::env;
// 出力先ディレクトリ選択ダイアログ関連
use std::path::Path;

pub fn pick_output_dir(archive_path: &Path) -> String {
    let abs_path = env::current_dir().unwrap().join(archive_path);
    let parent_path = abs_path.parent().unwrap_or_else(|| Path::new(""));
    let dir = FileDialog::new()
        .set_title("解凍先フォルダーを選択してください")
        .set_directory(parent_path)
        .pick_folder();
    match dir {
        Some(path) => path.display().to_string(),
        None => {
            eprintln!("解凍先フォルダーが選択されませんでした。");
            std::process::exit(1);
        }
    }
}
