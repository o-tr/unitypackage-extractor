use crate::extract::{ASSET_META_FILENAME, PATHNAME_FILENAME};
use yaml_rust::{YamlLoader};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use crate::progress_window::ProgressMsg;

pub fn rebuild_objects(
    objects: &HashMap<String, HashMap<String, String>>,
    output_dir: &Path,
    source_dir: &Path,
    tx: &Sender<ProgressMsg>,
) -> Result<(), String> {
    let total = objects.len() as f32;
    tx.send(ProgressMsg::Progress { value: 0.0, text: "開始".to_string(), done: false }).ok();
    let mut idx = 0u32;
    for (folder, files) in objects {
        idx += 1;
        let pathname = files.get(PATHNAME_FILENAME).ok_or("pathnameが見つかりません")?;
        let asset_meta = files.get(ASSET_META_FILENAME).ok_or("asset.metaが見つかりません")?;
        tx.send(ProgressMsg::Progress { value: idx as f32 /total , text: pathname.clone(), done: false }).ok();
        fltk::app::awake();

        let asset_meta_yaml = YamlLoader::load_from_str(asset_meta)
            .map_err(|e| format!("{}のmetaファイルのパースに失敗しました: {}", pathname, e))?;
        let asset_meta_yaml = asset_meta_yaml.get(0).ok_or("metaファイルのルートが見つかりません")?;
        let is_dir = asset_meta_yaml["folderAsset"].as_str().unwrap_or("false") == "yes";

        let source_file_path = source_dir.join(&folder);
        if is_dir || !source_file_path.exists() {
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
        let mut skip_asset = false;
        let mut asset_rename: Option<String> = None;
        if meta_path.exists() {
            let (resp_tx, resp_rx) = std::sync::mpsc::channel();
            let meta_path_display = PathBuf::from(pathname).parent().unwrap().join(format!("{}.meta", file_name));
            tx.send(ProgressMsg::ConfirmOverwrite {
                path: meta_path_display.display().to_string(),
                resp_tx,
            }).ok();
            let resp = resp_rx.recv();
            if let Ok(false) = resp {
                // falseの場合は「スキップ」または「自動リネーム」
                // ProgressWindowのshow_overwrite_dialogでSome(4)（自動リネーム）の場合はfalseが返る
                // ファイル名を自動リネーム
                let mut new_meta_path = meta_path.clone();
                let mut count = 1;
                let mut new_asset_name = file_name.to_string();
                while new_meta_path.exists() {
                    let file_stem = file_name;
                    if let Some((stem, ext)) = file_stem.rsplit_once('.') {
                        let new_name = format!("{}_copy{}.{}", stem, count, ext);
                        new_meta_path = output_file_path.parent().unwrap().join(format!("{}.meta", new_name));
                        new_asset_name = new_name;
                    } else {
                        let new_name = format!("{}_copy{}", file_stem, count);
                        new_meta_path = output_file_path.parent().unwrap().join(format!("{}.meta", new_name));
                        new_asset_name = new_name;
                    }
                    count += 1;
                }
                let mut meta_file = File::create(&new_meta_path).map_err(|e| format!("metaファイル作成失敗: {}", e))?;
                meta_file
                    .write_all(asset_meta.as_bytes())
                    .map_err(|e| format!("Failed to write file meta: {}", e))?;
                asset_rename = Some(new_asset_name);
            } else if let Ok(true) = resp {
                let mut meta_file = File::create(meta_path).map_err(|e| format!("metaファイル作成失敗: {}", e))?;
                meta_file
                    .write_all(asset_meta.as_bytes())
                    .map_err(|e| format!("Failed to write file meta: {}", e))?;
            } else {
                skip_asset = true;
                println!("スキップ: {}", meta_path.display());
            }
        } else {
            let mut meta_file = File::create(meta_path).map_err(|e| format!("metaファイル作成失敗: {}", e))?;
            meta_file
                .write_all(asset_meta.as_bytes())
                .map_err(|e| format!("Failed to write file meta: {}", e))?;
        }

        // 実体ファイルの処理
        let mut final_output_file_path = output_file_path.clone();
        if let Some(new_name) = asset_rename {
            final_output_file_path = output_file_path.parent().unwrap().join(new_name);
        }
        if final_output_file_path.exists() {
            let (resp_tx, resp_rx) = std::sync::mpsc::channel();
            tx.send(ProgressMsg::ConfirmOverwrite {
                path: final_output_file_path.file_name().unwrap().to_string_lossy().to_string(),
                resp_tx,
            }).ok();
            if !resp_rx.recv().unwrap_or(false) {
                println!("スキップ: {}", final_output_file_path.display());
                skip_asset = true;
            }
        }
        if !skip_asset {
            std::fs::rename(source_file_path, final_output_file_path)
                .map_err(|e| format!("Failed to rename source file to output file: {}", e))?;
        }
    }
    tx.send(ProgressMsg::Progress { value: total, text: "完了".to_string(), done: true }).ok();
    Ok(())
}
