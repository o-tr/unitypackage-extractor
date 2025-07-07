# unitypackage-extractor

Rust製CLIツール「unitypackage-extractor」は、Unityの .unitypackage ファイルの展開を行うWindows向けユーティリティです。

## 機能
- .unitypackageファイルのコンテキストメニューからアセット、metaファイルの展開

## 使い方
### ビルド
```
cargo build --release
```

## ファイル構成
- `src/main.rs`: CLIエントリーポイント
- `src/extract.rs`: 抽出ロジック
- `src/rebuild.rs`: 再構築ロジック
- `src/dialog.rs`: ユーザー確認・ダイアログ
- `src/progress_window.rs`: 進捗表示

## 開発方針
- Rust標準ライブラリを優先利用
- Windows環境前提
- エラー時は詳細なメッセージを表示
- metaファイル仕様はUnity準拠

## ライセンス
MIT License

