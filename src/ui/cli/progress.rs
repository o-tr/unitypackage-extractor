use crate::ui::{UiHandler, OverwriteMode, OverwriteAction};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct CliProgressHandler {
    overwrite_mode: OverwriteMode,
    cancelled: Arc<AtomicBool>,
    last_progress: f32,
}

impl CliProgressHandler {
    pub fn new(overwrite_mode: OverwriteMode) -> Self {
        Self {
            overwrite_mode,
            cancelled: Arc::new(AtomicBool::new(false)),
            last_progress: 0.0,
        }
    }
}

impl UiHandler for CliProgressHandler {
    fn update_progress(&mut self, value: f32, text: &str) {
        // 進捗率が1%以上変わった場合のみ表示
        if (value - self.last_progress) >= 0.01 || value >= 1.0 {
            println!("[{:>3.0}%] {}", value * 100.0, text);
            self.last_progress = value;
        }
    }

    fn finish(&mut self) {
        println!("[100%] 完了");
    }

    fn confirm_overwrite(&mut self, path: &str) -> Result<OverwriteAction, String> {
        match self.overwrite_mode {
            OverwriteMode::Overwrite => {
                println!("上書き: {}", path);
                Ok(OverwriteAction::Overwrite)
            }
            OverwriteMode::Skip => {
                println!("スキップ: {}", path);
                Ok(OverwriteAction::Skip)
            }
            OverwriteMode::Rename => {
                println!("リネーム: {}", path);
                Ok(OverwriteAction::Rename)
            }
            OverwriteMode::Ask => {
                // CLI版では Ask は使用しない
                Err("CLI版では OverwriteMode::Ask はサポートされていません".to_string())
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}
