use crate::args::{Args, Command};
use crate::core::{extract_objects, rebuild_objects, compress_directory};
use crate::ui::cli::CliProgressHandler;
use std::collections::HashMap;
use std::path::PathBuf;

const TMP_OUTPUT_DIR: &str = ".jp.ootr.unitypackage-extractor";

pub fn run() -> Result<(), String> {
    let args = Args::parse()?;

    match &args.command {
        Command::Extract { input_file, output_dir, overwrite_mode } => {
            run_extract(input_file, output_dir.as_ref(), *overwrite_mode)
        }
        Command::Compress { input_dir, output_file } => {
            run_compress(input_dir, output_file)
        }
    }
}

fn run_extract(
    input_file: &PathBuf,
    output_dir: Option<&PathBuf>,
    overwrite_mode: crate::ui::OverwriteMode,
) -> Result<(), String> {
    if !input_file.exists() {
        return Err(format!("指定されたファイルが存在しません: {}", input_file.display()));
    }

    let output_dir = output_dir
        .ok_or_else(|| "--output-dir is required in CLI mode".to_string())?;

    let tmp_output_dir = output_dir.join(TMP_OUTPUT_DIR);
    if tmp_output_dir.exists() {
        std::fs::remove_dir_all(&tmp_output_dir)
            .map_err(|e| format!("一時ディレクトリの削除に失敗しました: {}", e))?;
    }

    // 失敗時も含めて確実に一時ディレクトリを削除するためのガード
    struct TempDirGuard {
        path: PathBuf,
    }
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            if self.path.exists() {
                if let Err(e) = std::fs::remove_dir_all(&self.path) {
                    eprintln!("警告: 一時ディレクトリの削除に失敗しました: {}", e);
                }
            }
        }
    }
    let _tmp_guard = TempDirGuard { path: tmp_output_dir.clone() };

    println!("解凍を開始します: {} -> {}", input_file.display(), output_dir.display());

    let mut objects = HashMap::new();
    let mut ui_handler = CliProgressHandler::new(overwrite_mode);

    // 抽出
    extract_objects(input_file, &tmp_output_dir, &mut objects, &mut ui_handler)?;

    // 再構築
    rebuild_objects(&objects, &output_dir, &tmp_output_dir, &mut ui_handler)?;

    // 明示的なクリーンアップは不要（Dropガードで常に削除される）

    println!("解凍が完了しました。");

    Ok(())
}

fn run_compress(
    input_dir: &PathBuf,
    output_file: &PathBuf,
) -> Result<(), String> {
    if !input_dir.exists() {
        return Err(format!("指定されたディレクトリが存在しません: {}", input_dir.display()));
    }

    if !input_dir.is_dir() {
        return Err(format!("指定されたパスはディレクトリではありません: {}", input_dir.display()));
    }

    println!("圧縮を開始します: {} -> {}", input_dir.display(), output_file.display());

    // 圧縮モードではOverwriteModeは不要（常にRenameで良い）
    let mut ui_handler = CliProgressHandler::new(crate::ui::OverwriteMode::Rename);

    // 圧縮実行
    compress_directory(input_dir, output_file, &mut ui_handler)?;

    println!("圧縮が完了しました。");

    Ok(())
}

