use crate::ui::{UiHandler, OverwriteAction};
use yaml_rust::YamlLoader;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

const ASSET_META_FILENAME: &str = "asset.meta";
const PATHNAME_FILENAME: &str = "pathname";

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

        ui_handler.update_progress(idx as f32 / total, pathname);

        let asset_meta_yaml = YamlLoader::load_from_str(asset_meta)
            .map_err(|e| format!("{}のmetaファイルのパースに失敗しました: {}", pathname, e))?;
        let asset_meta_yaml = asset_meta_yaml.get(0)
            .ok_or("metaファイルのルートが見つかりません")?;
        let is_dir = asset_meta_yaml["folderAsset"].as_str().unwrap_or("false") == "yes";

        let source_file_path = source_dir.join(&folder);

        if is_dir || !source_file_path.exists() {
            handle_directory(output_dir, pathname, asset_meta)?;
            continue;
        }

        handle_file(
            output_dir,
            pathname,
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
    pathname: &str,
    asset_meta: &str,
) -> Result<(), String> {
    let output_path = output_dir.join(pathname);
    if !output_path.exists() {
        std::fs::create_dir_all(&output_path)
            .map_err(|e| format!("Output directory creation failed: {}", e))?;
    }

    let folder_name = pathname.split('/').last().unwrap_or("");
    let meta_path = output_path
        .parent()
        .unwrap()
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
    pathname: &str,
    asset_meta: &str,
    source_file_path: &Path,
    ui_handler: &mut U,
) -> Result<(), String> {
    let output_file_path = output_dir.join(pathname);
    let output_basedir = output_file_path.parent().unwrap();

    if !output_basedir.exists() {
        std::fs::create_dir_all(&output_basedir)
            .map_err(|e| format!("Output directory creation failed: {}", e))?;
    }

    let file_name = pathname.split('/').last().unwrap_or("");
    let meta_path = output_file_path
        .parent()
        .unwrap()
        .join(format!("{}.meta", file_name));

    let mut skip_asset = false;
    let mut asset_rename: Option<String> = None;

    // meta ファイルの処理
    if meta_path.exists() {
        let meta_path_display = PathBuf::from(pathname)
            .parent()
            .unwrap()
            .join(format!("{}.meta", file_name));

        let action = ui_handler.confirm_overwrite(&meta_path_display.display().to_string());

        match action {
            OverwriteAction::Overwrite => {
                write_meta_file(&meta_path, asset_meta)?;
            }
            OverwriteAction::Rename => {
                let new_name = find_unique_name(&output_file_path, file_name);
                let new_meta_path = output_file_path
                    .parent()
                    .unwrap()
                    .join(format!("{}.meta", new_name));
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
            final_output_file_path = output_file_path.parent().unwrap().join(new_name);
        }

        if final_output_file_path.exists() {
            let action = ui_handler.confirm_overwrite(
                &final_output_file_path.file_name().unwrap().to_string_lossy()
            );

            match action {
                OverwriteAction::Overwrite => {
                    // そのまま上書き（既存の挙動）
                }
                OverwriteAction::Rename => {
                    // ユニーク名を生成
                    let file_name = final_output_file_path.file_name().unwrap().to_string_lossy();
                    let new_name = find_unique_name(&final_output_file_path, &file_name);

                    // meta fileも既に書き込まれている場合、一緒にリネーム
                    let old_meta_path = final_output_file_path.parent().unwrap().join(format!("{}.meta", file_name));
                    if old_meta_path.exists() {
                        let new_meta_path = final_output_file_path.parent().unwrap().join(format!("{}.meta", new_name));
                        std::fs::rename(&old_meta_path, &new_meta_path)
                            .map_err(|e| format!("Failed to rename meta file: {}", e))?;
                    }

                    // asset fileのパスを更新
                    final_output_file_path = final_output_file_path.parent().unwrap().join(new_name);
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

fn find_unique_name(base_path: &Path, original_name: &str) -> String {
    let parent = base_path.parent().unwrap();
    let mut count = 1;

    loop {
        let new_name = if let Some((stem, ext)) = original_name.rsplit_once('.') {
            format!("{}_copy{}.{}", stem, count, ext)
        } else {
            format!("{}_copy{}", original_name, count)
        };

        let test_path = parent.join(format!("{}.meta", new_name));
        if !test_path.exists() {
            return new_name;
        }
        count += 1;
    }
}
