use crate::ui::OverwriteMode;
use std::path::PathBuf;

/// コマンドの種類
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// 抽出モード（デフォルト）
    Extract {
        input_file: PathBuf,
        output_dir: Option<PathBuf>,
        overwrite_mode: OverwriteMode,
    },
    /// 圧縮モード
    Compress {
        input_dir: PathBuf,
        output_file: PathBuf,
        project_root: Option<PathBuf>,
    },
}

pub struct Args {
    pub command: Command,
}

// 後方互換性のため、古いAPIも維持
impl Args {
    #[allow(dead_code)]
    pub fn input_file(&self) -> &PathBuf {
        match &self.command {
            Command::Extract { input_file, .. } => input_file,
            Command::Compress { input_dir, .. } => input_dir,
        }
    }

    #[allow(dead_code)]
    pub fn output_dir(&self) -> Option<&PathBuf> {
        match &self.command {
            Command::Extract { output_dir, .. } => output_dir.as_ref(),
            Command::Compress { .. } => None,
        }
    }

    #[allow(dead_code)]
    #[cfg_attr(feature = "gui", allow(dead_code))]
    pub fn overwrite_mode(&self) -> OverwriteMode {
        match &self.command {
            Command::Extract { overwrite_mode, .. } => *overwrite_mode,
            Command::Compress { .. } => OverwriteMode::Rename,
        }
    }
}

impl Args {
    pub fn parse() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().collect();

        // ヘルプチェック
        if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
            println!("{}", Self::usage(&args[0]));
            std::process::exit(0);
        }

        // サブコマンドの検出
        let subcommand: String;
        let start_idx: usize;

        if args.len() > 1 && !args[1].starts_with("--") {
            match args[1].as_str() {
                "extract" | "compress" => {
                    subcommand = args[1].clone();
                    start_idx = 2;
                }
                _ => {
                    // サブコマンドがない場合はextractとして扱う（後方互換性）
                    subcommand = "extract".to_string();
                    start_idx = 1;
                }
            }
        } else {
            // 引数が足りない、またはオプションから始まる場合
            subcommand = "extract".to_string();
            start_idx = 1;
        }

        match subcommand.as_str() {
            "extract" => Self::parse_extract(&args, start_idx),
            "compress" => Self::parse_compress(&args, start_idx),
            _ => Err(format!("Unknown subcommand: {}\n\n{}", subcommand, Self::usage(&args[0]))),
        }
    }

    fn parse_extract(args: &[String], start_idx: usize) -> Result<Self, String> {
        let mut input_file: Option<PathBuf> = None;
        let mut output_dir: Option<PathBuf> = None;

        // デフォルト値: GUI版はAsk、CLI版はRename
        #[cfg(feature = "gui")]
        let mut overwrite_mode = OverwriteMode::Ask;

        #[cfg(not(feature = "gui"))]
        let mut overwrite_mode = OverwriteMode::Rename;

        let mut i = start_idx;
        while i < args.len() {
            let arg = &args[i];

            if arg.starts_with("--overwrite-mode=") {
                let mode = arg.strip_prefix("--overwrite-mode=").unwrap();
                overwrite_mode = match mode {
                    "overwrite" => OverwriteMode::Overwrite,
                    "skip" => OverwriteMode::Skip,
                    "rename" => OverwriteMode::Rename,
                    "ask" => OverwriteMode::Ask,
                    _ => return Err(format!("Invalid overwrite mode: {}. Use: overwrite, skip, rename, or ask", mode)),
                };
            } else if arg.starts_with("--output-dir=") {
                let dir = arg.strip_prefix("--output-dir=").unwrap();
                output_dir = Some(PathBuf::from(dir));
            } else if arg == "--output-dir" {
                i += 1;
                if i >= args.len() {
                    return Err("--output-dir requires a value".to_string());
                }
                output_dir = Some(PathBuf::from(&args[i]));
            } else if arg == "--help" || arg == "-h" {
                println!("{}", Self::usage(&args[0]));
                std::process::exit(0);
            } else if !arg.starts_with("--") {
                // 位置引数
                if input_file.is_none() {
                    input_file = Some(PathBuf::from(arg));
                } else if output_dir.is_none() {
                    output_dir = Some(PathBuf::from(arg));
                } else {
                    return Err(format!("Unknown argument: {}", arg));
                }
            } else {
                return Err(format!("Unknown option: {}", arg));
            }

            i += 1;
        }

        let input_file = input_file.ok_or_else(|| {
            format!("Input file is required\n\n{}", Self::usage(&args[0]))
        })?;

        // CLI版では output_dir が必須
        #[cfg(not(feature = "gui"))]
        if output_dir.is_none() {
            return Err(format!(
                "--output-dir is required in CLI mode\n\n{}",
                Self::usage(&args[0])
            ));
        }

        // CLI版では Ask モードを明示的に拒否
        #[cfg(not(feature = "gui"))]
        if overwrite_mode == OverwriteMode::Ask {
            return Err(format!(
                "Error: --overwrite-mode=ask is not supported in CLI mode.\nPlease use: overwrite, skip, or rename.\n\n{}",
                Self::usage(&args[0])
            ));
        }

        Ok(Args {
            command: Command::Extract {
                input_file,
                output_dir,
                overwrite_mode,
            },
        })
    }

    fn parse_compress(args: &[String], start_idx: usize) -> Result<Self, String> {
        let mut input_dir: Option<PathBuf> = None;
        let mut output_file: Option<PathBuf> = None;
        let mut project_root: Option<PathBuf> = None;

        let mut i = start_idx;
        while i < args.len() {
            let arg = &args[i];

            if arg.starts_with("--output=") {
                let file = arg.strip_prefix("--output=").unwrap();
                output_file = Some(PathBuf::from(file));
            } else if arg == "--output" || arg == "-o" {
                i += 1;
                if i >= args.len() {
                    return Err("--output requires a value".to_string());
                }
                output_file = Some(PathBuf::from(&args[i]));
            } else if arg.starts_with("--project-root=") {
                let root = arg.strip_prefix("--project-root=").unwrap();
                project_root = Some(PathBuf::from(root));
            } else if arg == "--project-root" {
                i += 1;
                if i >= args.len() {
                    return Err("--project-root requires a value".to_string());
                }
                project_root = Some(PathBuf::from(&args[i]));
            } else if arg == "--help" || arg == "-h" {
                println!("{}", Self::usage(&args[0]));
                std::process::exit(0);
            } else if !arg.starts_with("--") {
                // 位置引数
                if input_dir.is_none() {
                    input_dir = Some(PathBuf::from(arg));
                } else if output_file.is_none() {
                    output_file = Some(PathBuf::from(arg));
                } else {
                    return Err(format!("Unknown argument: {}", arg));
                }
            } else {
                return Err(format!("Unknown option: {}", arg));
            }

            i += 1;
        }

        let input_dir = input_dir.ok_or_else(|| {
            format!("Input directory is required for compress command\n\n{}", Self::usage(&args[0]))
        })?;

        let output_file = output_file.ok_or_else(|| {
            format!("Output file is required for compress command\n\n{}", Self::usage(&args[0]))
        })?;

        Ok(Args {
            command: Command::Compress {
                input_dir,
                output_file,
                project_root,
            },
        })
    }

    fn usage(program: &str) -> String {
        format!(
            "Usage: {} [COMMAND] [OPTIONS]

Commands:
  extract                 Extract .unitypackage file (default)
  compress                Compress directory to .unitypackage file

EXTRACT MODE:
  Usage: {} [extract] <input.unitypackage> [OPTIONS]

  Arguments:
    <input.unitypackage>    Input .unitypackage file

  Options:
    --output-dir <dir>      Output directory{}
    --overwrite-mode <mode> Overwrite mode: overwrite, skip, rename, ask
                            Default: {}
    -h, --help              Show this help message

  Examples:
    # CLI mode (GUI feature disabled)
    {} input.unitypackage --output-dir ./output --overwrite-mode=rename

    # GUI mode (GUI feature enabled)
    {} input.unitypackage
    {} input.unitypackage --output-dir ./output

COMPRESS MODE:
  Usage: {} compress <input-dir> <output.unitypackage> [OPTIONS]
  Usage: {} compress <input-dir> --output <output.unitypackage>

  Arguments:
    <input-dir>             Input directory to compress
    <output.unitypackage>   Output .unitypackage file

  Options:
    --output, -o <file>     Output .unitypackage file (alternative)
    --project-root <dir>    Project root directory (for relative paths in package)
                            If not specified, uses parent of input-dir
    -h, --help              Show this help message

  Examples:
    # シンプルな使い方（input-dirが基準）
    {} compress ./Assets/MyPackage output.unitypackage

    # プロジェクトルートを指定（Assets/MyPackageの内容をMyPackage/として圧縮）
    {} compress ./MyUnityProject/Assets/MyPackage output.unitypackage --project-root ./MyUnityProject/Assets
",
            program,
            program,
            if cfg!(not(feature = "gui")) {
                " (required in CLI mode)"
            } else {
                ""
            },
            if cfg!(feature = "gui") {
                "ask"
            } else {
                "rename (ask not available in CLI mode)"
            },
            program,
            program,
            program,
            program,
            program,
            program,
            program
        )
    }
}
