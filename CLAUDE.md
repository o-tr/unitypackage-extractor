# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust製CLIツール「unitypackage-extractor」は、Unityの .unitypackage ファイル（実体はtar.gz）を展開し、元のディレクトリ構造とmetaファイルを復元するWindows向けユーティリティです。エクスプローラーの右クリックメニューから実行できます。

## Build Commands

```bash
# リリースビルド
cargo build --release

# デバッグビルド
cargo build

# 実行（引数指定）
cargo run -- <file.unitypackage> [output_dir]
```

## Architecture

### 処理フロー

1. **main.rs**: エントリーポイント。引数解析、出力先ディレクトリ選択、一時ディレクトリ管理
2. **extract.rs**: tar.gzアーカイブから`asset`、`asset.meta`、`pathname`ファイルを抽出し、一時ディレクトリに展開
3. **rebuild.rs**: pathnameファイルに従って元のディレクトリ構造を再構築し、metaファイルを生成
4. **progress_window.rs**: fltk-rsを使用した進捗表示と上書き確認ダイアログ（7zip風UI）
5. **dialog.rs**: rfdを使用したディレクトリ選択ダイアログ

### 一時ディレクトリ管理

- 一時展開先: `{output_dir}/.jp.ootr.unitypackage-extractor`
- 処理完了後に自動削除される
- エラー時や中断時も削除を試みる

### マルチスレッド処理

- メインスレッド: fltk UIイベントループ
- ワーカースレッド: extract → rebuild を実行
- `mpsc::channel`でプログレス情報と上書き確認のやり取りを行う
- `Arc<Mutex<HashMap>>`で抽出したオブジェクト情報を共有

### 上書き処理ロジック

rebuild.rs:64-133で実装される上書き確認処理:

1. metaファイルが既存の場合、ユーザーに確認（上書き/スキップ/自動リネーム/すべてに適用）
2. 自動リネーム選択時は`filename_copy1.ext`のような命名で連番を付ける
3. 実体ファイルについても同様に確認
4. `overwrite_all`フラグで「すべてに適用」を管理

### Unityパッケージ構造

.unitypackageの内部構造:
```
{guid}/
  asset          # 実体ファイル
  asset.meta     # Unity metaファイル（YAML）
  pathname       # 元のファイルパス
```

- `folderAsset: yes`の場合はディレクトリとして扱う
- metaファイルはUnity仕様のYAMLフォーマット

## Development Guidelines

### コーディング方針

- Rustの標準ライブラリを優先的に使用
- Windows環境での動作を前提（エクスプローラーのコンテキストメニュー統合）
- エラー処理はResult型で詳細なメッセージを付与（日本語）
- コメントは日本語で記述

### 命名規則

- Rustの一般的な命名規則（snake_case）に従う
- 定数は大文字スネークケース（例: `ASSET_FILE_NAME`）

### 依存クレート

- `tar` + `flate2`: tar.gz展開
- `yaml-rust`: metaファイルパース
- `fltk`: クロスプラットフォームGUI（進捗表示）
- `rfd`: ネイティブファイルダイアログ

### Windows統合

- リリース時は`windows_subsystem = "windows"`でコンソールウィンドウを非表示
- デバッグ時はコメントアウトしてコンソール出力を確認可能（main.rs:1）
- レジストリ登録でエクスプローラーの右クリックメニューに統合

## Important Notes

- Unity metaファイル仕様に準拠すること
- 既存ファイルの上書き確認は必須
- 処理完了後は必ず一時ディレクトリをクリーンアップ
- マルチスレッド処理時は`fltk::app::awake()`でUI更新を通知

# ExecPlans

When writing complex features or significant refactors, use an ExecPlan (as described in .agent/PLANS.md) from design to implementation.
