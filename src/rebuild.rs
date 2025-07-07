use crate::dialog::confirm_overwrite;
use crate::extract::{ASSET_META_FILENAME, PATHNAME_FILENAME};
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub fn rebuild_objects(
    objects: &HashMap<String, HashMap<String, String>>,
    output_dir: &Path,
    source_dir: &Path,
) -> Result<(), String> {
    for (folder, files) in objects {
        let pathname = files.get(PATHNAME_FILENAME).ok_or("pathnameが見つかりません")?;
        let asset_meta = files.get(ASSET_META_FILENAME).ok_or("asset.metaが見つかりません")?;

        let asset_meta_yaml: serde_yaml::Value =
            serde_yaml::from_str(asset_meta).map_err(|e| format!("asset.metaのYAMLパースに失敗しました: {}", e))?;
        let is_dir = asset_meta_yaml
            .get("folderAsset")
            .and_then(|v| v.as_str())
            .unwrap_or("false")
            == "yes";

        if is_dir {
            let output_path = output_dir.join(&pathname);
            if !output_path.exists() {
                std::fs::create_dir_all(&output_path).map_err(|e| format!("Output directory creation failed: {}", e))?;
            }
            let folder_name = pathname.split('/').last().unwrap_or("");
            let meta_path = output_path
                .parent()
                .unwrap()
                .join(format!("{}.meta", folder_name));
            if !meta_path.exists() {
                let mut meta_file = File::create(meta_path).map_err(|e| format!("metaファイル作成失敗: {}", e))?;
                meta_file
                    .write_all(asset_meta.as_bytes())
                    .map_err(|e| format!("Failed to write folder meta file: {}", e))?;
            }
            continue;
        }

        let source_file_path = source_dir.join(&folder);
        if !source_file_path.exists() {
            return Err(format!("Source path does not exist: {}", source_file_path.display()));
        }
        let output_file_path = output_dir.join(&pathname);
        let output_basedir = output_file_path.parent().unwrap();
        if !output_basedir.exists() {
            std::fs::create_dir_all(&output_basedir).map_err(|e| format!("Output directory creation failed: {}", e))?;
        }

        let file_name = pathname.split('/').last().unwrap_or("");
        let meta_path = output_file_path
            .parent()
            .unwrap()
            .join(format!("{}.meta", file_name));
        if meta_path.exists() && !confirm_overwrite(&meta_path) {
            println!("スキップ: {}", meta_path.display());
        } else {
            let mut meta_file = File::create(meta_path).map_err(|e| format!("metaファイル作成失敗: {}", e))?;
            meta_file
                .write_all(asset_meta.as_bytes())
                .map_err(|e| format!("Failed to write file meta: {}", e))?;
        }

        if output_file_path.exists() {
            if !confirm_overwrite(&output_file_path) {
                println!("スキップ: {}", output_file_path.display());
                continue;
            }
        }
        std::fs::rename(source_file_path, output_file_path)
            .map_err(|e| format!("Failed to rename source file to output file: {}", e))?;
    }
    Ok(())
}
