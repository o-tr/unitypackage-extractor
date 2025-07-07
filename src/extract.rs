use flate2::read::GzDecoder;
// アーカイブからの抽出処理
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
) {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).expect("出力ディレクトリの作成に失敗しました");
    }
    for entry in archive
        .entries()
        .expect("アーカイブのエントリの取得に失敗しました")
    {
        let mut entry = entry.expect("エントリの読み込みに失敗しました");
        let path = entry
            .path()
            .expect("パスの取得に失敗しました")
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
                .expect("ファイルの読み込みに失敗しました");

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
        let mut outfile = std::fs::File::create(output_dir.join(&folder).join(&file_name))
            .expect("ファイルの作成に失敗しました");
        std::io::copy(&mut entry, &mut outfile).expect("ファイルの書き込みに失敗しました");
    }
}
