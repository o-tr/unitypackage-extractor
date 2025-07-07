use flate2::read::GzDecoder;
use rfd::FileDialog;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::Path;
use tar::Archive;
use std::collections::HashMap;
use std::fmt::format;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <file.tar.gz> [output_dir]", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    let output_dir = if args.len() == 3 {
        args[2].clone()
    } else {
        let archive_path = Path::new(file_path);
        let joined_path = env::current_dir().unwrap().join(archive_path);
        let abs_path = joined_path.parent().unwrap_or_else(|| Path::new(""));
        eprintln!("解凍先フォルダーを選択してください。{}", abs_path.display());
        let dir = FileDialog::new()
            .set_title("解凍先フォルダーを選択してください")
            .set_directory(abs_path)
            .pick_folder();
        match dir {
            Some(path) => path.display().to_string(),
            None => {
                eprintln!("解凍先フォルダーが選択されませんでした。");
                std::process::exit(1);
            }
        }
    };
    let output_dir = Path::new(&output_dir);
    let tmp_output_dir = Path::new(&output_dir).join(".jp.ootr.unitypackage-extractor");
    let file = File::open(file_path).expect("ファイルを開けませんでした");
    let buf_reader = BufReader::new(file);
    let gz_decoder = GzDecoder::new(buf_reader);
    let mut archive = Archive::new(gz_decoder);
    let mut objects = HashMap::new();
    extract_objects(&mut archive, &tmp_output_dir, &mut objects);
    println!("解凍が完了しました。");
    rebuild_objects(&objects, &output_dir, &tmp_output_dir).expect("TODO: panic message");
}

fn extract_objects(
    archive: &mut Archive<GzDecoder<BufReader<File>>>,
    output_dir: &Path,
    objects: &mut HashMap<String, HashMap<String, String>>,
){
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).expect("出力ディレクトリの作成に失敗しました");
    }
    for entry in archive.entries().expect("アーカイブのエントリの取得に失敗しました") {
        let mut entry = entry.expect("エントリの読み込みに失敗しました");
        let path = entry.path().expect("パスの取得に失敗しました").to_path_buf();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let folder = if let Some(parent) = path.parent() {
            parent.to_str().unwrap().to_string()
        } else {
            "".to_string()
        };
        if file_name == "asset.meta" || file_name == "pathname" {
            // read files
            let mut string_entry = String::new();
            entry.read_to_string(&mut string_entry).expect("ファイルの読み込みに失敗しました");

            objects.entry(folder).or_default().insert(file_name, string_entry);
            continue;
        }
        if file_name != "asset"{
            println!("unknown file: {}", file_name);
            continue;
        }
        let output_path = Path::new(&output_dir).join(&folder);
        let mut outfile = std::fs::File::create(&output_path).expect("ファイルの作成に失敗しました");
        io::copy(&mut entry, &mut outfile).expect("ファイルの書き込みに失敗しました");
    }
}

fn rebuild_objects(
    objects: &HashMap<String, HashMap<String, String>>,
    output_dir: &Path,
    source_dir: &Path,
)-> Result<(), io::Error> {
    for (folder, files) in objects {
        let pathname = files.get("pathname").unwrap();
        let asset_meta = files.get("asset.meta").unwrap();

        let asset_meta_yaml: serde_yaml::Value = serde_yaml::from_str(asset_meta)
            .expect("asset.metaのYAMLパースに失敗しました");
        let is_dir = asset_meta_yaml.get("folderAsset")
            .and_then(|v| v.as_str())
            .unwrap_or("false") == "yes";


        if is_dir {
            let output_path = output_dir.join(&pathname);
            if !output_path.exists() {
                std::fs::create_dir_all(&output_path).expect("Output directory creation failed");
            }
            let folder_name = pathname.split('/').last().unwrap_or("");
            let meta_path = output_path.parent().unwrap().join(format!("{}.meta", folder_name));
            if !meta_path.exists() {
                let mut meta_file = std::fs::File::create(meta_path)?;
                meta_file.write_all(asset_meta.as_bytes()).expect("Failed to write folder meta file");
            }
            continue;
        }

        let source_file_path = source_dir.join(&folder);
        if !source_file_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path does not exist: {}", source_file_path.display())
            ))
        }
        let output_file_path = output_dir.join(&pathname);
        let output_basedir = output_file_path.parent().unwrap();
        if !output_basedir.exists() {
            std::fs::create_dir_all(&output_basedir).expect("Output directory creation failed");
        }

        let file_name = pathname.split('/').last().unwrap_or("");
        let meta_path = output_file_path.parent().unwrap().join(format!("{}.meta", file_name));
        if !meta_path.exists() {
            let mut meta_file = std::fs::File::create(meta_path)?;
            meta_file.write_all(asset_meta.as_bytes()).expect("Failed to write file meta");
        }
        std::fs::rename(source_file_path, output_file_path)
            .expect("Failed to rename source file to output file");
    }
    Ok(())
}