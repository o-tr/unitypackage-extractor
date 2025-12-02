use crate::ui::UiHandler;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Builder;
use yaml_rust::YamlLoader;

/// Unityプロジェクトディレクトリから.unitypackageファイルを生成
pub fn compress_directory<U: UiHandler>(
    input_dir: &Path,
    output_file: &Path,
    ui_handler: &mut U,
) -> Result<(), String> {
    // 入力ディレクトリ内のアセットファイルとmetaファイルを収集
    let entries = collect_entries(input_dir, ui_handler)?;

    if entries.is_empty() {
        return Err("圧縮対象のファイルが見つかりませんでした".to_string());
    }

    println!("{}個のファイルを圧縮します...", entries.len());

    // 出力ファイルを作成
    let output_file_handle = File::create(output_file)
        .map_err(|e| format!("出力ファイルの作成に失敗しました: {}", e))?;

    let gz_encoder = GzEncoder::new(output_file_handle, Compression::default());
    let mut tar_builder = Builder::new(gz_encoder);

    let total = entries.len() as f32;

    // 各エントリをアーカイブに追加
    for (idx, entry) in entries.iter().enumerate() {
        // キャンセルチェック
        if ui_handler.is_cancelled() {
            return Err("キャンセルされました".to_string());
        }

        ui_handler.update_progress(
            (idx as f32) / total,
            &entry.pathname
        );

        add_entry_to_archive(&mut tar_builder, entry)?;
    }

    // アーカイブを完了
    tar_builder.finish()
        .map_err(|e| format!("アーカイブの完了に失敗しました: {}", e))?;

    ui_handler.finish();
    Ok(())
}

/// アーカイブエントリ情報
struct ArchiveEntry {
    guid: String,
    pathname: String,
    asset_path: Option<PathBuf>,
    meta_content: String,
}

/// 入力ディレクトリから圧縮対象のエントリを収集
fn collect_entries<U: UiHandler>(
    input_dir: &Path,
    ui_handler: &mut U,
) -> Result<Vec<ArchiveEntry>, String> {
    let mut entries = Vec::new();

    // ディレクトリを再帰的に走査
    collect_entries_recursive(input_dir, input_dir, &mut entries, ui_handler)?;

    Ok(entries)
}

/// 再帰的にディレクトリを走査してエントリを収集
fn collect_entries_recursive<U: UiHandler>(
    base_dir: &Path,
    current_dir: &Path,
    entries: &mut Vec<ArchiveEntry>,
    ui_handler: &mut U,
) -> Result<(), String> {
    let read_dir = std::fs::read_dir(current_dir)
        .map_err(|e| format!("ディレクトリの読み込みに失敗しました: {}", e))?;

    for entry_result in read_dir {
        // キャンセルチェック
        if ui_handler.is_cancelled() {
            return Err("キャンセルされました".to_string());
        }

        let entry = entry_result
            .map_err(|e| format!("ディレクトリエントリの読み込みに失敗しました: {}", e))?;

        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // .metaファイルはスキップ（後で対応するアセットと一緒に処理）
        if file_name_str.ends_with(".meta") {
            continue;
        }

        // 隠しファイル/ディレクトリをスキップ
        if file_name_str.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            // ディレクトリの場合
            let meta_path = path.with_extension("meta");

            if !meta_path.exists() {
                println!("警告: ディレクトリのmetaファイルが見つかりません（スキップ）: {}", path.display());
                // ディレクトリ内を再帰的に走査
                collect_entries_recursive(base_dir, &path, entries, ui_handler)?;
                continue;
            }

            // metaファイルからGUIDを読み取る
            let meta_content = std::fs::read_to_string(&meta_path)
                .map_err(|e| format!("metaファイルの読み込みに失敗しました: {}", e))?;

            let guid = extract_guid_from_meta(&meta_content)?;
            let pathname = get_relative_path(base_dir, &path)?;

            entries.push(ArchiveEntry {
                guid,
                pathname,
                asset_path: None, // ディレクトリの場合はassetファイルなし
                meta_content,
            });

            // ディレクトリ内を再帰的に走査
            collect_entries_recursive(base_dir, &path, entries, ui_handler)?;
        } else {
            // ファイルの場合
            let meta_path = PathBuf::from(format!("{}.meta", path.display()));

            if !meta_path.exists() {
                println!("警告: ファイルのmetaファイルが見つかりません（スキップ）: {}", path.display());
                continue;
            }

            // metaファイルからGUIDを読み取る
            let meta_content = std::fs::read_to_string(&meta_path)
                .map_err(|e| format!("metaファイルの読み込みに失敗しました: {}", e))?;

            let guid = extract_guid_from_meta(&meta_content)?;
            let pathname = get_relative_path(base_dir, &path)?;

            entries.push(ArchiveEntry {
                guid,
                pathname,
                asset_path: Some(path.clone()),
                meta_content,
            });
        }
    }

    Ok(())
}

/// metaファイルからGUIDを抽出
fn extract_guid_from_meta(meta_content: &str) -> Result<String, String> {
    let docs = YamlLoader::load_from_str(meta_content)
        .map_err(|e| format!("metaファイルのパースに失敗しました: {}", e))?;

    let doc = docs.get(0)
        .ok_or("metaファイルのルートが見つかりません")?;

    let guid = doc["guid"].as_str()
        .ok_or("metaファイルにguidフィールドが見つかりません")?;

    Ok(guid.to_string())
}

/// ベースディレクトリからの相対パスを取得
fn get_relative_path(base_dir: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(base_dir)
        .map_err(|e| format!("相対パスの取得に失敗しました: {}", e))?;

    // Unixスタイルのパス区切りに変換
    let pathname = relative.to_string_lossy()
        .replace('\\', "/");

    Ok(pathname)
}

/// エントリをアーカイブに追加
fn add_entry_to_archive(
    tar_builder: &mut Builder<GzEncoder<File>>,
    entry: &ArchiveEntry,
) -> Result<(), String> {
    let guid = &entry.guid;

    // pathname ファイルを追加
    {
        let pathname_path = format!("{}/pathname", guid);
        let pathname_data = entry.pathname.as_bytes();

        let mut header = tar::Header::new_gnu();
        header.set_path(&pathname_path)
            .map_err(|e| format!("pathnameパスの設定に失敗しました: {}", e))?;
        header.set_size(pathname_data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        tar_builder.append(&header, pathname_data)
            .map_err(|e| format!("pathnameの追加に失敗しました: {}", e))?;
    }

    // asset.meta ファイルを追加
    {
        let meta_path = format!("{}/asset.meta", guid);
        let meta_data = entry.meta_content.as_bytes();

        let mut header = tar::Header::new_gnu();
        header.set_path(&meta_path)
            .map_err(|e| format!("asset.metaパスの設定に失敗しました: {}", e))?;
        header.set_size(meta_data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        tar_builder.append(&header, meta_data)
            .map_err(|e| format!("asset.metaの追加に失敗しました: {}", e))?;
    }

    // asset ファイルを追加（ファイルの場合のみ）
    if let Some(asset_path) = &entry.asset_path {
        let asset_archive_path = format!("{}/asset", guid);

        let mut file = File::open(asset_path)
            .map_err(|e| format!("アセットファイルのオープンに失敗しました: {}", e))?;

        tar_builder.append_file(&asset_archive_path, &mut file)
            .map_err(|e| format!("アセットファイルの追加に失敗しました: {}", e))?;
    }

    Ok(())
}

