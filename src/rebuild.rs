use crate::extract::{ASSET_META_FILENAME, PATHNAME_FILENAME};
use serde_yaml;
// オブジェクト再構築処理
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub fn rebuild_objects(
    objects: &HashMap<String, HashMap<String, String>>,
    output_dir: &Path,
    source_dir: &Path,
) -> Result<(), io::Error> {
    for (folder, files) in objects {
        let pathname = files.get(PATHNAME_FILENAME).unwrap();
        let asset_meta = files.get(ASSET_META_FILENAME).unwrap();

        let asset_meta_yaml: serde_yaml::Value =
            serde_yaml::from_str(asset_meta).expect("asset.metaのYAMLパースに失敗しました");
        let is_dir = asset_meta_yaml
            .get("folderAsset")
            .and_then(|v| v.as_str())
            .unwrap_or("false")
            == "yes";

        if is_dir {
            let output_path = output_dir.join(&pathname);
            if !output_path.exists() {
                std::fs::create_dir_all(&output_path).expect("Output directory creation failed");
            }
            let folder_name = pathname.split('/').last().unwrap_or("");
            let meta_path = output_path
                .parent()
                .unwrap()
                .join(format!("{}.meta", folder_name));
            if !meta_path.exists() {
                let mut meta_file = File::create(meta_path)?;
                meta_file
                    .write_all(asset_meta.as_bytes())
                    .expect("Failed to write folder meta file");
            }
            continue;
        }

        let source_file_path = source_dir.join(&folder);
        if !source_file_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path does not exist: {}", source_file_path.display()),
            ));
        }
        let output_file_path = output_dir.join(&pathname);
        let output_basedir = output_file_path.parent().unwrap();
        if !output_basedir.exists() {
            std::fs::create_dir_all(&output_basedir).expect("Output directory creation failed");
        }

        let file_name = pathname.split('/').last().unwrap_or("");
        let meta_path = output_file_path
            .parent()
            .unwrap()
            .join(format!("{}.meta", file_name));
        if !meta_path.exists() {
            let mut meta_file = File::create(meta_path)?;
            meta_file
                .write_all(asset_meta.as_bytes())
                .expect("Failed to write file meta");
        }
        std::fs::rename(source_file_path, output_file_path)
            .expect("Failed to rename source file to output file");
    }
    Ok(())
}
