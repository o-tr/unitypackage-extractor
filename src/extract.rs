use crate::progress_window::{ProgressMsg};
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::mpsc::Sender;
use tar::Archive;

pub const ASSET_FILE_NAME: &str = "asset";
pub const ASSET_META_FILENAME: &str = "asset.meta";
pub const PATHNAME_FILENAME: &str = "pathname";

pub fn extract_objects(
    archive_path: &Path,
    output_dir: &Path,
    objects: &mut HashMap<String, HashMap<String, String>>,
    tx: &Sender<ProgressMsg>,
) -> Result<(), String> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).map_err(|e| format!("出力ディレクトリの作成に失敗しました: {}", e))?;
    }

    // まずエントリ数をカウント
    let mut total = 0u32;
    {
        let file = File::open(archive_path).map_err(|e| format!("ファイルの読み込みに失敗しました: {}", e))?;
        let reader = BufReader::new(file);
        let gz = GzDecoder::new(reader);
        let mut archive = Archive::new(gz);

        for entry in archive
            .entries()
            .map_err(|e| format!("アーカイブのエントリの取得に失敗しました: {}", e))?
        {
            entry.map_err(|e| format!("アーカイブのエントリの読み込みに失敗しました: {}", e))?;
            total += 1;
        }
    }


    // 実際の処理用にアーカイブを再度開く
    let file = File::open(archive_path).map_err(|e| format!("ファイルの読み込みに失敗しました: {}", e))?;
    let reader = BufReader::new(file);
    let gz = GzDecoder::new(reader);
    let mut archive = Archive::new(gz);

    let mut idx = 0u32;
    for entry in archive
        .entries()
        .map_err(|e| format!("アーカイブのエントリの取得に失敗しました: {}", e))?
    {
        idx += 1;
        let mut entry = entry.map_err(|e| format!("アーカイブのエントリの読み込みに失敗しました: {}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("パスの取得に失敗しました: {}", e))?
            .to_path_buf();
        tx.send(ProgressMsg::Progress { value: (idx as f32)/(total as f32), text: path.display().to_string(), done: false }).ok();
        fltk::app::awake();

        if path.components().count() < 2 {
            continue;
        }

        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let folder = if let Some(parent) = path.parent() {
            parent.to_str().unwrap().to_string()
        } else {
            "".to_string()
        };

        if file_name == ASSET_META_FILENAME || file_name == PATHNAME_FILENAME {
            let mut string_entry = String::new();
            entry
                .read_to_string(&mut string_entry)
                .map_err(|e| format!("ファイルの読み込みに失敗しました: {}", e))?;

            objects
                .entry(folder)
                .or_default()
                .insert(file_name, string_entry);
            continue;
        }
        if file_name != ASSET_FILE_NAME {
            println!("unknown file: {}", file_name);
            continue;
        }
        let out_path = output_dir.join(&folder);
        if let Some(parent) = out_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| format!("ディレクトリ作成失敗: {}", e))?;
            }
        }
        let mut outfile = std::fs::File::create(&out_path)
            .map_err(|e| format!("ファイルの作成に失敗しました: {}", e))?;
        std::io::copy(&mut entry, &mut outfile).map_err(|e| format!("ファイルの書き込みに失敗しました: {}", e))?;
    }

    Ok(())
}
