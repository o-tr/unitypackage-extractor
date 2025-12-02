/// 上書きモード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwriteMode {
    /// 常に上書き
    Overwrite,
    /// 常にスキップ
    Skip,
    /// 自動リネーム
    Rename,
    /// 毎回確認 (GUI版のみ)
    Ask,
}

/// 上書き確認の応答
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwriteAction {
    /// 上書きする
    Overwrite,
    /// スキップする
    Skip,
    /// リネームする
    Rename,
}

/// UI操作の抽象化トレイト
pub trait UiHandler: Send {
    /// 進捗状況を更新
    /// value: 0.0 ~ 1.0 の進捗率
    /// text: 進捗状況を表すテキスト（現在処理中のファイル名など）
    fn update_progress(&mut self, value: f32, text: &str);

    /// 処理完了を通知
    fn finish(&mut self);

    /// ファイル上書き確認
    /// path: 上書き対象のファイルパス
    /// 戻り値: 上書きアクション
    fn confirm_overwrite(&mut self, path: &str) -> OverwriteAction;

    /// キャンセルされたかチェック
    fn is_cancelled(&self) -> bool;
}

// feature flagに応じてモジュールを公開
#[cfg(feature = "gui")]
pub mod gui;

#[cfg(not(feature = "gui"))]
pub mod cli;
