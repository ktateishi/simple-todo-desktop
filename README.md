<div align="center">

# Simple TODO

**シンプルで、強力なデスクトップ TODO アプリ**

[Tauri](https://tauri.app/) + [SvelteKit](https://kit.svelte.dev/) + [TypeScript](https://www.typescriptlang.org/) + [Rust](https://www.rust-lang.org/) で構築されたクロスプラットフォームのタスク管理アプリです。

[![Release](https://img.shields.io/github/v/release/ktateishi/simple-todo-desktop)](https://github.com/ktateishi/simple-todo-desktop/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-informational)](#-ダウンロード)

[ダウンロード](#-ダウンロード) ·
[機能](#-主な機能) ·
[開発](#-開発) ·
[Webサイト](https://ktateishi.github.io/simple-todo-desktop/)

</div>

---

## 概要

Simple TODO は、日々のタスク管理をシンプルかつパワフルにこなすためのデスクトップアプリです。今日やるべきことを最上段に自動集約し、グループ・タグ・ステータス・優先度で整理しながら、期限切れタスクの見逃しを防ぎます。ガントチャート形式のスケジュール表示にも対応し、期間のあるタスクを俯瞰できます。

ローカルの SQLite データベースにすべてのデータを保存するため、外部サービスへの登録は不要です。

## 主な機能

### タスク管理
- タスクの作成・編集・削除、優先度（高 / 中 / 低）の設定
- 期日（due_at）・開始日（start_at）によるタスク期間の管理
- 繰り返しタスク（毎日 / 毎週 / 隔週 / 毎月 / 毎年）
- リマインダー通知（ネイティブ通知 + バックグラウンドスケジューラー）
- クイック追加バーからの音声入力・`#タグ名` インライン指定
- タスクのコピー & ペースト

### ステータス管理
- デフォルト4ステータス（未着手 / 進行中 / 保留 / 完了）に加え、任意のカスタムステータスを作成可能
- ステータスごとに「今日のタスク」への表示可否を設定可能

### 表示・整理
- **今日のタスク**: 期日が今日のタスクを自動集約
- **期限切れ**: 期日を過ぎた未完了タスクを自動集約、経過時間を表示
- **グループ表示** / **一覧表示** / **スケジュール表示（ガントチャート）** の3ビュー
- グループ・タグによる分類とサイドバーからのフィルタリング
- 手動並び替え（ドラッグ&ドロップ）、期日・優先度によるソート

### アプリ体験
- 5種類のカラーテーマ（Airy / Bloom / Noir / Dusk / Ash）
- システムトレイ常駐、ウィンドウを閉じても終了せずバックグラウンド動作
- ログイン時の自動起動（autostart）
- 署名付きパッケージによる自動アップデート機能

## スクリーンショット

アプリの詳細な紹介は [Webサイト](https://ktateishi.github.io/simple-todo-desktop/) をご覧ください。

## 📥 ダウンロード

最新版は [Releases](https://github.com/ktateishi/simple-todo-desktop/releases/latest) から入手できます。

| プラットフォーム | ファイル |
|---|---|
| macOS — Apple Silicon (M1 / M2 / M3) | `*_aarch64.dmg` |
| macOS — Intel Mac | `*_x86_64.dmg` |
| Windows — インストーラー（推奨） | `*_x64-setup.exe` |
| Windows — MSI パッケージ | `*_x64_en-US.msi` |

> **macOS をご利用の方へ**: ダウンロード後に Gatekeeper の警告が表示された場合は、システム設定 → プライバシーとセキュリティ →「このまま開く」を選択してください。

### 動作環境

| プラットフォーム | 要件 |
|---|---|
| macOS | macOS 12 Monterey 以降 |
| Windows | Windows 10 / 11 (64-bit) |

## 🛠 技術スタック

| レイヤー | 技術 |
|---|---|
| デスクトップランタイム | [Tauri v2](https://tauri.app/) |
| フロントエンド | [SvelteKit](https://kit.svelte.dev/) ([Svelte 5](https://svelte.dev/) / Runes) + TypeScript |
| バックエンド | Rust |
| データベース | SQLite（[rusqlite](https://github.com/rusqlite/rusqlite) / バンドル版） |
| ビルドツール | [Vite](https://vitejs.dev/) |
| パッケージマネージャー | [pnpm](https://pnpm.io/) |

## 💻 開発

### 前提条件

- [Node.js](https://nodejs.org/) 22 以上
- [pnpm](https://pnpm.io/) 9 以上
- [Rust](https://www.rust-lang.org/tools/install)（stable）
- Tauri の[プラットフォーム別の前提パッケージ](https://tauri.app/start/prerequisites/)

### セットアップ

```bash
git clone https://github.com/ktateishi/simple-todo-desktop.git
cd simple-todo-desktop
pnpm install
```

### 開発サーバーの起動

```bash
pnpm tauri dev
```

### 本番ビルド

```bash
pnpm tauri build
```

### その他のスクリプト

| コマンド | 内容 |
|---|---|
| `pnpm dev` | SvelteKit のフロントエンドのみ起動（ブラウザプレビュー用） |
| `pnpm build` | フロントエンドをビルド |
| `pnpm check` | `svelte-check` による型チェック |
| `cargo check`（`src-tauri/` 内） | Rust コードの型チェック |

### 推奨 IDE 設定

[VS Code](https://code.visualstudio.com/) + [Svelte for VS Code](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 📁 プロジェクト構成

```
.
├── src/                      # SvelteKit フロントエンド
│   ├── lib/
│   │   ├── components/       # UI コンポーネント（TaskItem, EditModal, ScheduleView など）
│   │   ├── api.ts            # Tauri invoke のラッパー
│   │   ├── stores.ts         # Svelte ストア（フィルター・ソート・派生ストアなど）
│   │   ├── types.ts          # 型定義
│   │   └── dateUtils.ts      # 日付計算ユーティリティ
│   └── routes/                # ページ（+page.svelte が実質的なメイン画面）
├── src-tauri/                # Rust バックエンド
│   ├── src/
│   │   ├── commands.rs       # フロントエンドから呼ばれる Tauri コマンド
│   │   ├── db/                # SQLite スキーマ・リポジトリ層
│   │   ├── domain.rs          # ドメインモデル
│   │   └── scheduler.rs       # リマインダー通知のスケジューラー
│   └── tauri.conf.json        # Tauri アプリ設定
├── docs/                     # GitHub Pages 用ランディングページ
└── CHANGELOG.md
```

## 🤝 コントリビュート

Issue・Pull Request を歓迎します。

1. このリポジトリを Fork する
2. 作業用ブランチを作成する（`git checkout -b feat/your-feature`）
3. 変更をコミットする（`git commit -m 'feat: 追加した機能の説明'`）
4. ブランチをプッシュする（`git push origin feat/your-feature`）
5. Pull Request を作成する

バグ報告・機能要望は [Issues](https://github.com/ktateishi/simple-todo-desktop/issues) からお願いします。

## 📄 ライセンス

このプロジェクトは [MIT License](LICENSE) の下で公開されています。
