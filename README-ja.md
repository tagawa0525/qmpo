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
./target/release/qmpo-lau register
```

## ライセンス

MIT

---

**O**pen **D**irectory **W**ith **B**rowser → odwb 🔄 qmpo
