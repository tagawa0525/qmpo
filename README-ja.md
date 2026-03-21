# qmpo

ファイルマネージャーでディレクトリを開く `directory://` URIハンドラー。

[English](README.md)

## なぜ必要？

ブラウザはセキュリティ上の理由で `file://` リンクをブロックします。
しかし、社内Wiki、ドキュメントサイト、共有フォルダへの検索結果など、
Webページからローカルディレクトリを開きたい場面があります。

qmpoは `directory://` URIスキームを提供し、ファイルマネージャーで
ディレクトリを安全に開きます（ファイルは開きません）。
[Chrome拡張機能](qmpo-extension/)を使えば、既存の `file://` リンクもそのまま動作します。

## URI形式

| OS | パス | URI |
| --- | --- | --- |
| Windows | `C:\Users\tagawa` | `directory://C:/Users/tagawa` |
| Windows (UNC) | `\\server\share` | `directory://server/share` |
| macOS/Linux | `/home/tagawa` | `directory:///home/tagawa` |

## 仕組み

qmpoは3つのコンポーネントで構成されています：

1. **qmpo** - `directory://` URIを受け取り、ファイルマネージャーで開く
   URIハンドラー
2. **qmpo-lau** - qmpoをシステムの `directory://` プロトコルハンドラー
   として登録するツール
3. **Chrome拡張機能** - Webページ上の `file://` リンクを `directory://`
   に変換

### Windows（実装）

- Windowsレジストリにプロトコルハンドラーを登録（`HKCU\Software\Classes\directory`）
- バイナリを `%PROGRAMFILES%\qmpo\qmpo.exe` にインストール
- `explorer.exe` でディレクトリを開く

### macOS（実装）

- `/Applications/qmpo.app` にアプリバンドルを作成
- `Info.plist` で `directory://` URLスキームを登録
- `lsregister` でLaunch Servicesに登録
- `open` コマンドでディレクトリを開く

### Linux（実装）

- `~/.local/share/applications/qmpo.desktop` にデスクトップエントリを作成
- バイナリを `/usr/local/bin/qmpo` にインストール
- `xdg-mime` でMIMEハンドラーとして登録
- `xdg-open` でディレクトリを開く

### Chrome拡張機能（仕組み）

- Content scriptが各ページで `file://` リンクを検出
- クリック時に `file://` URLを `directory://` URLに変換
- 隠しiframeでシステムのプロトコルハンドラーをトリガー
- 変換されたリンクにフォルダアイコン（📂）を表示
- ドメインの許可リスト/ブロックリストに対応

## インストール

### Windows

[GitHub Releases](https://github.com/tagawa0525/qmpo/releases)から最新版をダウンロード：

1. `qmpo-windows-x64.zip` をダウンロード
2. アーカイブを展開
3. `qmpo-lau.exe` を実行（ダブルクリックまたはコマンドプロンプトから実行）

これにより、qmpoが `directory://` プロトコルハンドラーとして自動的に登録されます。

### Arch Linux

```bash
git clone https://github.com/tagawa0525/qmpo.git
cd qmpo/aur
makepkg -si
```

### NixOS / Home Manager

```nix
# flake.nix
inputs.qmpo.url = "github:tagawa0525/qmpo";

# home.nix
imports = [ inputs.qmpo.homeManagerModules.default ];
programs.qmpo.enable = true;
```

### その他（ソースからビルド）

```bash
git clone https://github.com/tagawa0525/qmpo.git
cd qmpo
cargo build --release
sudo ./target/release/qmpo-lau register  # 管理者権限が必要
```

### Chrome拡張機能

Chrome拡張機能は `file://` リンクを自動的に `directory://` に変換します。

1. Chromeで `chrome://extensions/` を開く
2. 右上の「デベロッパーモード」を有効にする
3. 「パッケージ化されていない拡張機能を読み込む」をクリック
4. `qmpo-extension` フォルダを選択

拡張機能は変換されたリンクにフォルダアイコンを追加し、拡張機能のポップアップから設定ができます。

## テストページ

qmpoのインストール確認用テストページ:

| OS | English | 日本語 |
| --- | --- | --- |
| Linux | [test-linux.html](docs/test-linux.html) | [test-linux-ja.html](docs/test-linux-ja.html) |
| macOS | [test-macos.html](docs/test-macos.html) | [test-macos-ja.html](docs/test-macos-ja.html) |
| Windows | [test-windows.html](docs/test-windows.html) | [test-windows-ja.html](docs/test-windows-ja.html) |

## ライセンス

MIT

---

**O**pen **D**irectory **W**ith **B**rowser → odwb 🔄 qmpo
