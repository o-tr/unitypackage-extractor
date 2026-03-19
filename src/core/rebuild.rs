use crate::core::path_safety::parse_pathname;
use crate::ui::{OverwriteAction, UiHandler};
use yaml_rust::YamlLoader;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

const ASSET_META_FILENAME: &str = "asset.meta";
const PATHNAME_FILENAME: &str = "pathname";
// 衝突が延々と続いた場合の無限ループ防止
const MAX_ATTEMPTS: u32 = 10_000;

pub fn rebuild_objects<U: UiHandler>(
    objects: &HashMap<String, HashMap<String, String>>,
    output_dir: &Path,
    source_dir: &Path,
    ui_handler: &mut U,
) -> Result<(), String> {
    let total = objects.len() as f32;
    ui_handler.update_progress(0.0, "開始");

    let mut idx = 0u32;
    for (folder, files) in objects {
        // キャンセルチェック
        if ui_handler.is_cancelled() {
            return Err("キャンセルされました".to_string());
        }

        idx += 1;
        let pathname = files.get(PATHNAME_FILENAME)
            .ok_or("pathnameが見つかりません")?;
        let asset_meta = files.get(ASSET_META_FILENAME)
            .ok_or("asset.metaが見つかりません")?;
        let (pathname_path, pathname_file_name) = parse_pathname(pathname)?;

        ui_handler.update_progress(idx as f32 / total, pathname);

        let asset_meta_yaml = YamlLoader::load_from_str(asset_meta)
            .map_err(|e| format!("{}のmetaファイルのパースに失敗しました: {}", pathname, e))?;
        let asset_meta_yaml = asset_meta_yaml.first()
            .ok_or("metaファイルのルートが見つかりません")?;

        let folder_path = Path::new(folder);
        // `folder` は extract 側で validate_relative_archive_folder 済みのキー
        let source_file_path = source_dir.join(folder_path);

        // フォルダかどうかの判定:
        // 1. metaファイルにfolderAsset: yesがある
        // 2. source_file_pathが存在しない（フォルダアセットは空ファイルなので展開時に存在しない）
        // 3. source_file_pathがディレクトリとして存在する
        let is_folder_by_meta = asset_meta_yaml["folderAsset"].as_str().unwrap_or("false") == "yes";
        let is_folder_by_fs = source_file_path.exists() && source_file_path.is_dir();
        let is_dir = is_folder_by_meta || !source_file_path.exists() || is_folder_by_fs;

        if is_dir {
            handle_directory(output_dir, &pathname_path, asset_meta)?;
            continue;
        }

        handle_file(
            output_dir,
            &pathname_path,
            &pathname_file_name,
            asset_meta,
            &source_file_path,
            ui_handler,
        )?;
    }

    ui_handler.finish();
    Ok(())
}

fn handle_directory(
    output_dir: &Path,
    pathname: &Path,
    asset_meta: &str,
) -> Result<(), String> {
    let output_path = output_dir.join(pathname);
    if !output_path.exists() {
        std::fs::create_dir_all(&output_path)
            .map_err(|e| format!("Output directory creation failed: {}", e))?;
    }

    let folder_name = pathname
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("ディレクトリ名の取得に失敗しました: {}", pathname.display()))?;
    let output_parent = output_path
        .parent()
        .ok_or_else(|| format!("出力先親ディレクトリの取得に失敗しました: {}", output_path.display()))?;
    let meta_path = output_parent
        .join(format!("{}.meta", folder_name));

    if !meta_path.exists() {
        let mut meta_file = File::create(meta_path)
            .map_err(|e| format!("metaファイル作成失敗: {}", e))?;
        meta_file
            .write_all(asset_meta.as_bytes())
            .map_err(|e| format!("Failed to write folder meta file: {}", e))?;
    }

    Ok(())
}

fn handle_file<U: UiHandler>(
    output_dir: &Path,
    pathname: &Path,
    pathname_file_name: &str,
    asset_meta: &str,
    source_file_path: &Path,
    ui_handler: &mut U,
) -> Result<(), String> {
    let output_file_path = output_dir.join(pathname);
    let output_basedir = output_file_path
        .parent()
        .ok_or_else(|| format!("出力先親ディレクトリの取得に失敗しました: {}", output_file_path.display()))?;

    if !output_basedir.exists() {
        std::fs::create_dir_all(&output_basedir)
            .map_err(|e| format!("Output directory creation failed: {}", e))?;
    }

    let meta_path = output_basedir.join(format!("{}.meta", pathname_file_name));

    let mut skip_asset = false;
    let mut asset_rename: Option<String> = None;

    // meta ファイルの処理
    if meta_path.exists() {
        let meta_path_display = build_meta_display_path(pathname, pathname_file_name);

        let action = ui_handler.confirm_overwrite(&meta_path_display.display().to_string())?;

        match action {
            OverwriteAction::Overwrite => {
                write_meta_file(&meta_path, asset_meta)?;
            }
            OverwriteAction::Rename => {
                let new_name = find_unique_name(&output_file_path, pathname_file_name)?;
                let new_meta_path = output_basedir.join(format!("{}.meta", new_name));
                write_meta_file(&new_meta_path, asset_meta)?;
                asset_rename = Some(new_name);
            }
            OverwriteAction::Skip => {
                skip_asset = true;
                println!("スキップ: {}", meta_path.display());
            }
        }
    } else {
        write_meta_file(&meta_path, asset_meta)?;
    }

    // 実体ファイルの処理
    if !skip_asset {
        let mut final_output_file_path = output_file_path.clone();
        if let Some(new_name) = asset_rename {
            final_output_file_path = output_basedir.join(new_name);
        }

        if final_output_file_path.exists() {
            let display_name = final_output_file_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| format!("ファイル名の取得に失敗しました: {}", final_output_file_path.display()))?;
            let action = ui_handler.confirm_overwrite(display_name)?;

            match action {
                OverwriteAction::Overwrite => {
                    std::fs::remove_file(&final_output_file_path)
                        .map_err(|e| format!("既存ファイルの削除に失敗しました: {}", e))?;
                }
                OverwriteAction::Rename => {
                    // ユニーク名を生成
                    let file_name = final_output_file_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| format!("ファイル名の取得に失敗しました: {}", final_output_file_path.display()))?;
                    let new_name = find_unique_name(&final_output_file_path, file_name)?;

                    // meta fileも既に書き込まれている場合、一緒にリネーム
                    let old_meta_path = output_basedir.join(format!("{}.meta", file_name));
                    if old_meta_path.exists() {
                        let new_meta_path = output_basedir.join(format!("{}.meta", new_name));
                        std::fs::rename(&old_meta_path, &new_meta_path)
                            .map_err(|e| format!("Failed to rename meta file: {}", e))?;
                    }

                    // asset fileのパスを更新
                    final_output_file_path = output_basedir.join(new_name);
                }
                OverwriteAction::Skip => {
                    println!("スキップ: {}", final_output_file_path.display());
                    return Ok(());
                }
            }
        }

        std::fs::rename(source_file_path, final_output_file_path)
            .map_err(|e| format!("Failed to rename source file to output file: {}", e))?;
    }

    Ok(())
}

fn write_meta_file(path: &Path, content: &str) -> Result<(), String> {
    let mut meta_file = File::create(path)
        .map_err(|e| format!("metaファイル作成失敗: {}", e))?;
    meta_file
        .write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write file meta: {}", e))?;
    Ok(())
}

fn build_meta_display_path(pathname: &Path, file_name: &str) -> PathBuf {
    let meta_name = format!("{}.meta", file_name);
    match pathname.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(meta_name),
        _ => PathBuf::from(meta_name),
    }
}

fn find_unique_name(base_path: &Path, original_name: &str) -> Result<String, String> {
    let parent = base_path
        .parent()
        .ok_or_else(|| format!("親ディレクトリの取得に失敗しました: {}", base_path.display()))?;
    let mut count = 1;

    loop {
        if count >= MAX_ATTEMPTS {
            return Err(format!(
                "一意な名前を作成できませんでした: {}（衝突が続いたため上限{}に到達）",
                base_path.display(),
                MAX_ATTEMPTS
            ));
        }

        let new_name = if let Some((stem, ext)) = original_name.rsplit_once('.') {
            format!("{}_copy{}.{}", stem, count, ext)
        } else {
            format!("{}_copy{}", original_name, count)
        };

        // Asset と meta の両方を確認する
        let asset_path = parent.join(&new_name);
        let meta_path = parent.join(format!("{}.meta", new_name));

        if !asset_path.exists() && !meta_path.exists() {
            return Ok(new_name);
        }
        count += 1;
    }
}
