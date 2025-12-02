use crate::ui::OverwriteMode;
use std::path::PathBuf;

pub struct Args {
    pub input_file: PathBuf,
    pub output_dir: Option<PathBuf>,
    #[cfg_attr(feature = "gui", allow(dead_code))]
    pub overwrite_mode: OverwriteMode,
}

impl Args {
    pub fn parse() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().collect();

        let mut input_file: Option<PathBuf> = None;
        let mut output_dir: Option<PathBuf> = None;

        // デフォルト値: GUI版はAsk、CLI版はRename
        #[cfg(feature = "gui")]
        let mut overwrite_mode = OverwriteMode::Ask;

        #[cfg(not(feature = "gui"))]
        let mut overwrite_mode = OverwriteMode::Rename;

        let mut i = 1;
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
            input_file,
            output_dir,
            overwrite_mode,
        })
    }

    fn usage(program: &str) -> String {
        format!(
            "Usage: {} <input.unitypackage> [options]

Arguments:
  <input.unitypackage>    Input .unitypackage file

Options:
  --output-dir <dir>      Output directory{}
  --overwrite-mode <mode> Overwrite mode: overwrite, skip, rename, ask
                          Default: ask{}
  -h, --help              Show this help message

Examples:
  # CLI mode (GUI feature disabled)
  {} input.unitypackage --output-dir ./output --overwrite-mode=rename

  # GUI mode (GUI feature enabled)
  {} input.unitypackage
  {} input.unitypackage --output-dir ./output",
            program,
            if cfg!(not(feature = "gui")) {
                " (required in CLI mode)"
            } else {
                ""
            },
            if cfg!(not(feature = "gui")) {
                " (ask not available in CLI mode)"
            } else {
                ""
            },
            program,
            program,
            program
        )
    }
}
