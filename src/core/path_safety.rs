use std::path::{Component, Path, PathBuf};

pub fn validate_relative_archive_folder(folder: &Path) -> Result<(), String> {
    if folder.as_os_str().is_empty() {
        return Ok(());
    }

    if folder.is_absolute() {
        return Err(format!(
            "アーカイブ内フォルダパスが不正です（絶対パスは不可）: {}",
            folder.display()
        ));
    }

    for component in folder.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!(
                    "アーカイブ内フォルダパスが不正です（.. は不可）: {}",
                    folder.display()
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "アーカイブ内フォルダパスが不正です（ルート指定は不可）: {}",
                    folder.display()
                ));
            }
        }
    }

    Ok(())
}

pub fn validate_relative_unix_path(pathname: &str) -> Result<(), String> {
    if pathname.is_empty() {
        return Err("pathnameが空です".to_string());
    }

    if pathname.starts_with('/') || pathname.starts_with('\\') {
        return Err(format!(
            "pathnameが不正です（絶対パスは不可）: {}",
            pathname
        ));
    }

    let path = Path::new(pathname);
    if path.is_absolute() {
        return Err(format!(
            "pathnameが不正です（絶対パスは不可）: {}",
            pathname
        ));
    }

    for segment in pathname.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return Err(format!(
                "pathnameが不正です（.. は不可）: {}",
                pathname
            ));
        }
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!(
                    "pathnameが不正です（.. は不可）: {}",
                    pathname
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "pathnameが不正です（ルート指定は不可）: {}",
                    pathname
                ));
            }
        }
    }

    Ok(())
}

pub fn parse_pathname(pathname: &str) -> Result<(PathBuf, String), String> {
    validate_relative_unix_path(pathname)?;

    let path = PathBuf::from(pathname);

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("pathnameのファイル名を取得できません: {}", pathname))?
        .to_string();

    Ok((path, file_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_relative_unix_path_ok() {
        assert!(validate_relative_unix_path("Assets/MyPackage/file.txt").is_ok());
    }

    #[test]
    fn validate_relative_unix_path_reject_parent() {
        assert!(validate_relative_unix_path("../evil.txt").is_err());
    }

    #[test]
    fn validate_relative_unix_path_reject_absolute() {
        assert!(validate_relative_unix_path("/etc/passwd").is_err());
    }

    #[test]
    fn validate_relative_archive_folder_ok() {
        assert!(validate_relative_archive_folder(Path::new("abc123")).is_ok());
    }

    #[test]
    fn validate_relative_archive_folder_reject_parent() {
        // 区切り文字に依存しない形で親ディレクトリ参照を拒否することを確認
        assert!(validate_relative_archive_folder(Path::new("a/../b")).is_err());

        // Windows では `\` 区切りでも parent 判定されることを追加確認
        #[cfg(windows)]
        {
            assert!(validate_relative_archive_folder(Path::new("a\\..\\b")).is_err());
        }
    }

    #[test]
    fn parse_pathname_extracts_filename() {
        let (path, file_name) = parse_pathname("Assets/MyPackage/file.txt")
            .unwrap_or_else(|e| panic!("parse failed: {}", e));
        assert_eq!(path, PathBuf::from("Assets/MyPackage/file.txt"));
        assert_eq!(file_name, "file.txt");
    }
}
