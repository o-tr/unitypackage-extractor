use crate::dialog::confirm_overwrite;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tar::Archive;

pub const ASSET_FILE_NAME: &str = "asset";
pub const ASSET_META_FILENAME: &str = "asset.meta";
pub const PATHNAME_FILENAME: &str = "pathname";

pub fn extract_objects(
    archive: &mut Archive<GzDecoder<BufReader<File>>>,
    output_dir: &Path,
    objects: &mut HashMap<String, HashMap<String, String>>,
) -> Result<(), String> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).map_err(|e| format!("出力ディレクトリの作成に失敗しました: {}", e))?;
    }
    for entry in archive
        .entries()
        .map_err(|e| format!("アーカイブのエントリの取得に失敗しました: {}", e))?
    {
        let mut entry = entry.map_err(|e| format!("エントリの読み込みに失敗しました: {}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("パスの取得に失敗しました: {}", e))?
            .to_path_buf();
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
        // 上書き確認処理を削除（extractでは不要）
        let mut outfile = std::fs::File::create(&out_path)
            .map_err(|e| format!("ファイルの作成に失敗しました: {}", e))?;
        std::io::copy(&mut entry, &mut outfile).map_err(|e| format!("ファイルの書き込みに失敗しました: {}", e))?;
    }
    Ok(())
}
