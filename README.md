# Libe Desk

リベシティと関連サービスを複数タブで利用する、macOS・Windows向けの非公式デスクトップアプリです。

> 本アプリはリベシティ公式アプリではありません。利用時はリベシティの利用規約・ガイドラインに従ってください。

## 対応サービス

- リベシティ
- ノウハウ図書館
- スキルマーケット
- リベシティ市場

## 開発

前提としてNode.js、npm、Rust、Tauri 2のOS別依存環境が必要です。

```sh
npm install
npm run tauri dev
```

## 検査

```sh
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

## ダウンロード

配布版はGitHub Releasesからダウンロードできます。

- macOS（Apple Silicon）: `aarch64` と記載されたDMG
- macOS（Intel）: `x86_64` と記載されたDMG
- Windows（64ビット）: `x64` と記載されたインストーラー

本アプリはコード署名を行っていないため、初回起動時にOSの警告が表示される場合があります。起動方法は各Releaseの説明を確認してください。

`main`ブランチへの変更はGitHub Actionsで自動検査・ビルドされ、成功するとGitHub Releasesへ公開されます。

## ライセンス

[MIT License](./LICENSE)
