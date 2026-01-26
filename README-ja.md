# qmpo

**Open Directory With Browser** - ブラウザから `directory://` URIスキームで
ディレクトリをファイルマネージャーで開くクロスプラットフォームツール。

[English](README.md)

## 概要

qmpoはカスタムURIスキームを使用して、ブラウザからディレクトリを開くことができます
（Open Directory With Browser）。`directory://` リンクをクリックするか、
ブラウザのアドレスバーに入力すると、システムのファイルマネージャーで
対応するディレクトリが開きます。

Webブラウザとローカルファイルシステムの橋渡しをするツールです。

## URI形式

| OS | ファイルパス | URI形式 |
| --- | --- | --- |
| Windows (ローカル) | `C:\Users\tagawa` | `directory://C:/Users/tagawa` |
| Windows (UNC) | `\\server\share\folder` | `directory://server/share/folder` |
| macOS/Linux | `/home/tagawa` | `directory:///home/tagawa` |

**注意:** ファイルパスを指定した場合、そのファイルが存在する親
ディレクトリが開きます。

## インストール

### ソースからビルド

```bash
git clone https://github.com/tagawa0525/qmpo.git
cd qmpo
cargo build --release
```

### URIハンドラーの登録

```bash
# Linux
./target/release/qmpo-lau register

# macOS
./target/release/qmpo-lau register

# Windows
.\target\release\qmpo-lau.exe register
```

## 使い方

### 直接実行

```bash
qmpo "directory:///home/user/Documents"
```

### ブラウザからディレクトリを開く (Open Directory With Browser)

ブラウザのアドレスバーにURIを入力:

```text
directory:///home/user/Documents
```

### 管理コマンド

```bash
# URIハンドラーとして登録
qmpo-lau register

# 登録状態を確認
qmpo-lau status

# 登録解除
qmpo-lau unregister
```

## プロジェクト構成

```text
qmpo/
├── qmpo-core/    # コアライブラリ（URI解析）
├── qmpo/         # メインアプリケーション（URIハンドラー）
└── qmpo-lau/     # 登録ユーティリティ
```

## プラットフォーム対応

| プラットフォーム | ハンドラー配置場所 | 登録方法 |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\qmpo\` | レジストリ (HKCU) |
| macOS | `~/Applications/qmpo.app/` | Launch Services |
| Linux | `~/.local/bin/` | XDG MIME + Desktopファイル |

## ライセンス

MIT
