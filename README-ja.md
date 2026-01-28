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

**注意:** ファイルパスを指定した場合、そのファイルが存在する親ディレクトリが開きます。

## インストール

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
./target/release/qmpo-lau register   # Linux/macOS
```

## ライセンス

MIT

---

## なぜ "qmpo"?

**O**pen **D**irectory **W**ith **B**rowser → odwb 🔄 qmpo

理想的にはブラウザが `directory://` リンクをネイティブに処理すべきです。
それまでの間、qmpoがその橋渡しをします。
