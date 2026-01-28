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

## ユースケース

- Webページ上のリンクからローカルファイルを直接開く
- 社内ドキュメントサイトから共有フォルダにアクセス
- 検索結果ページからファイルの場所へ移動

**特徴:** クロスプラットフォーム、シングルバイナリ、セキュア（ディレクトリのみ開く）
[Chrome拡張機能](qmpo-extension/)を使えば、既存の `file://` リンクがそのまま動作します。🔄

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
./target/release/qmpo-lau register   # Linux/macOS
.\target\release\qmpo-lau.exe register   # Windows
```

### Arch Linux

```bash
git clone https://github.com/tagawa0525/qmpo.git
cd qmpo/aur
makepkg -si
```

### NixOS / Home Manager

Flake入力として追加:

```nix
# flake.nix
{
  inputs = {
    qmpo = {
      url = "github:tagawa0525/qmpo";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
```

#### 方法1: Home Managerモジュールを使用（推奨）

```nix
# home.nix または home-manager設定
{ inputs, ... }:
{
  imports = [ inputs.qmpo.homeManagerModules.default ];

  programs.qmpo.enable = true;
}
```

#### 方法2: 手動設定

```nix
# flake.nix のオーバーレイに追加
nixpkgs.overlays = [ qmpo.overlays.default ];

# home-manager設定
{ pkgs, ... }:
{
  xdg.mimeApps.defaultApplications = {
    "x-scheme-handler/directory" = "qmpo.desktop";
  };

  xdg.desktopEntries.qmpo = {
    name = "qmpo";
    exec = "${pkgs.qmpo}/bin/qmpo %u";
    terminal = false;
    noDisplay = true;
    mimeType = [ "x-scheme-handler/directory" ];
  };
}
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
| Arch Linux | `/usr/lib/qmpo/` | PKGBUILD + pacmanフック |
| NixOS | `/nix/store/...` | Flake + Home Manager |

## ライセンス

MIT
