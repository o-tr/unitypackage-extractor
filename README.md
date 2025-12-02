# unitypackage-extractor

Rust製CLIツール「unitypackage-extractor」は、Unityの .unitypackage ファイルの展開・圧縮を行うWindows向けユーティリティです。

## 機能
- .unitypackageファイルのコンテキストメニューからアセット、metaファイルの展開
- Unityプロジェクトディレクトリから.unitypackageファイルの生成（CLI版のみ）

## 使い方
### インストール
- [リリースページ](https://github.com/o-tr/unitypackage-extractor/releases)から最新のインストーラーをダウンロードし、実行してください

### GUI版の使用方法
- エクスプローラーで .unitypackage ファイルを右クリックし、「unitypackage-extractorで展開」を選択します。
- 展開先のディレクトリを選択すると、アセットとmetaファイルが展開されます。

### CLI版の使用方法

#### 展開（Extract）モード
```bash
# 基本的な使い方
unitypackage-extractor.exe input.unitypackage --output-dir ./output

# 上書きモードを指定
unitypackage-extractor.exe input.unitypackage --output-dir ./output --overwrite-mode=rename
```

#### 圧縮（Compress）モード
```bash
# Unityプロジェクトディレクトリから.unitypackageファイルを生成
unitypackage-extractor.exe compress ./Assets/MyPackage output.unitypackage

# または --output オプションを使用
unitypackage-extractor.exe compress ./Assets/MyPackage --output output.unitypackage

# プロジェクトルートを指定（相対パスの基準を変更）
# この場合、パッケージ内では "MyPackage/" として配置される
unitypackage-extractor.exe compress ./MyUnityProject/Assets/MyPackage output.unitypackage --project-root ./MyUnityProject/Assets
```

**注意事項:**
- 圧縮モードでは、各アセットファイルに対応する`.meta`ファイルが必須です
- `.meta`ファイルが存在しないファイル/ディレクトリは警告が表示され、スキップされます
- `.meta`ファイルから既存のGUIDを読み取り、パッケージに含めます
- `--project-root`を指定しない場合、入力ディレクトリの親ディレクトリが基準となります


## 開発
### ビルド
```
cargo build --release
```

## ファイル構成
- `src/main.rs`: エントリーポイント
- `src/args.rs`: コマンドライン引数解析
- `src/cli_main.rs`: CLI版メインロジック
- `src/gui_main.rs`: GUI版メインロジック
- `src/core/extract.rs`: 抽出ロジック
- `src/core/rebuild.rs`: 再構築ロジック
- `src/core/compress.rs`: 圧縮ロジック
- `src/ui/`: UI処理（CLI/GUI共通インターフェース）

## 開発方針
- Rust標準ライブラリを優先利用
- Windows環境前提
- エラー時は詳細なメッセージを表示
- metaファイル仕様はUnity準拠

## ライセンス
MIT License

