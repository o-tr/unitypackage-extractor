pub mod extract;
pub mod rebuild;
mod path_safety;

#[cfg(not(feature = "gui"))]
pub mod compress;

pub use extract::extract_objects;
pub use rebuild::rebuild_objects;

#[cfg(not(feature = "gui"))]
pub use compress::compress_directory;

/// キャンセル時のエラーメッセージ（全モジュールで共通）
pub const CANCEL_ERROR_MSG: &str = "キャンセルされました";
