# qmpo

A `directory://` URI handler that opens directories in your file manager.

[日本語](README-ja.md)

## Why?

Browsers block `file://` links for security reasons. But sometimes you need to
open local directories from web pages—internal wikis, documentation sites,
or search results pointing to shared folders.

qmpo solves this by providing a `directory://` URI scheme that safely opens
directories (not files) in your file manager. With the
[Chrome extension](qmpo-extension/), existing `file://` links work seamlessly.

## URI Format

| OS | Path | URI |
| --- | --- | --- |
| Windows | `C:\Users\tagawa` | `directory://C:/Users/tagawa` |
| Windows (UNC) | `\\server\share` | `directory://server/share` |
| macOS/Linux | `/home/tagawa` | `directory:///home/tagawa` |

## Installation

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

### Other (Build from Source)

```bash
git clone https://github.com/tagawa0525/qmpo.git
cd qmpo
cargo build --release
./target/release/qmpo-lau register
```

## License

MIT

---

**O**pen **D**irectory **W**ith **B**rowser → odwb 🔄 qmpo
