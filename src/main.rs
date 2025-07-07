mod dialog;
mod extract;
mod rebuild;

use dialog::pick_output_dir;
use extract::extract_objects;
use rebuild::rebuild_objects;

use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tar::Archive;

const TMP_OUTPUT_DIR: &str = ".jp.ootr.unitypackage-extractor";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <file.tar.gz> [output_dir]", args[0]);
        std::process::exit(1);
    }

    let filepath_str = &args[1];
    let filepath = Path::new(filepath_str);
    if !filepath.exists() {
        eprintln!("指定されたファイルが存在しません: {}", filepath.display());
        std::process::exit(1);
    }

    let output_dir = if args.len() == 3 {
        args[2].clone()
    } else {
        pick_output_dir(filepath)
    };

    let output_dir = Path::new(&output_dir);
    let tmp_output_dir = Path::new(&output_dir).join(TMP_OUTPUT_DIR);
    let file = File::open(filepath_str).expect("ファイルを開けませんでした");
    let buf_reader = BufReader::new(file);
    let gz_decoder = GzDecoder::new(buf_reader);
    let mut archive = Archive::new(gz_decoder);
    let mut objects = HashMap::new();
    extract_objects(&mut archive, &tmp_output_dir, &mut objects);
    println!("解凍が完了しました。");
    rebuild_objects(&objects, &output_dir, &tmp_output_dir).expect("TODO: panic message");
}
