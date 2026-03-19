# Copilot Instructions for unity-package

このリポジトリはRust製のCLIツールで、Unityパッケージ（.unitypackage）の再構築や抽出を行います。

## コーディング方針
- Rustの標準ライブラリを優先的に使用してください。
- ファイル操作（コピー、移動、metaファイル生成）は既存の実装に倣ってください。
- ユーザーへの確認（上書き確認など）は `src/ui` 配下のUI抽象（`UiHandler`）を利用してください。
- Windows環境での動作を前提とします。

## 処理の流れ
1. ユーザーからのコマンドライン引数を解析します。
2. 指定されたUnityパッケージからmetaファイルとpathnameファイルを読み取ります。assetファイルはディスクに展開します。
3. pathnameファイルを元に、assetファイルを適切なディレクトリに配置します。

## ファイル構成
- src/main.rs: エントリーポイント。featureに応じてCLI/GUIを分岐。
- src/cli_main.rs: CLI版メインロジック。
- src/gui_main.rs: GUI版メインロジック。
- src/core/extract.rs: パッケージ抽出ロジック。
- src/core/rebuild.rs: パッケージ再構築ロジック。
- src/core/compress.rs: パッケージ圧縮ロジック。
- src/ui/: UI抽象とCLI/GUI実装。

## 命名規則・スタイル
- Rustの一般的な命名規則（snake_case）に従ってください。
- エラー処理はResult型、map_err等で詳細なエラーメッセージを付与してください。
- コメントは日本語で簡潔に記述してください。

## 注意事項
- Unityのmetaファイル仕様に準拠してください。
- 既存のconfirm_overwrite等の関数を再利用してください。
- 既存コードのロジックを壊さないように注意してください。

