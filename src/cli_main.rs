use crate::args::Args;
use crate::core::{extract_objects, rebuild_objects};
use crate::ui::cli::CliProgressHandler;
use std::collections::HashMap;

const TMP_OUTPUT_DIR: &str = ".jp.ootr.unitypackage-extractor";

pub fn run() -> Result<(), String> {
    let args = Args::parse()?;

    let input_file = &args.input_file;
    if !input_file.exists() {
        return Err(format!("指定されたファイルが存在しません: {}", input_file.display()));
    }

    let output_dir = args.output_dir
        .ok_or_else(|| "--output-dir is required in CLI mode".to_string())?;

    let tmp_output_dir = output_dir.join(TMP_OUTPUT_DIR);
    if tmp_output_dir.exists() {
        std::fs::remove_dir_all(&tmp_output_dir)
            .map_err(|e| format!("一時ディレクトリの削除に失敗しました: {}", e))?;
    }

    println!("解凍を開始します: {} -> {}", input_file.display(), output_dir.display());

    let mut objects = HashMap::new();
    let mut ui_handler = CliProgressHandler::new(args.overwrite_mode);

    // 抽出
    extract_objects(input_file, &tmp_output_dir, &mut objects, &mut ui_handler)?;

    // 再構築
    rebuild_objects(&objects, &output_dir, &tmp_output_dir, &mut ui_handler)?;

    // クリーンアップ
    if tmp_output_dir.exists() {
        std::fs::remove_dir_all(&tmp_output_dir)
            .map_err(|e| format!("一時ディレクトリの削除に失敗しました: {}", e))?;
    }

    println!("解凍が完了しました。");

    Ok(())
}
