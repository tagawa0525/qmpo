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

## How It Works

qmpo consists of three components:

1. **qmpo** - The URI handler that receives `directory://` URIs and opens
   them in your file manager
2. **qmpo-lau** - A registration tool that sets up qmpo as the system's
   `directory://` protocol handler
3. **Chrome Extension** - Converts `file://` links to `directory://` on web
   pages

### Windows (Implementation)

- Registers a protocol handler in the Windows Registry (`HKCU\Software\Classes\directory`)
- Installs the binary to `%PROGRAMFILES%\qmpo\qmpo.exe`
- Opens directories using `explorer.exe`

### macOS (Implementation)

- Creates an app bundle at `/Applications/qmpo.app`
- Registers the `directory://` URL scheme via `Info.plist`
- Registers with Launch Services using `lsregister`
- Opens directories using the `open` command

### Linux (Implementation)

- Creates a desktop entry at `~/.local/share/applications/qmpo.desktop`
- Installs the binary to `/usr/local/bin/qmpo`
- Registers as MIME handler using `xdg-mime`
- Opens directories using `xdg-open`

### Chrome Extension (How It Works)

- Content script runs on every page and detects `file://` links
- Converts `file://` URLs to `directory://` URLs when clicked
- Triggers the system protocol handler via a hidden iframe
- Adds a folder icon (📂) to converted links as a visual indicator
- Supports domain allowlist/blocklist for fine-grained control

## Installation

### Windows

Download the latest release from [GitHub Releases](https://github.com/tagawa0525/qmpo/releases):

1. Download `qmpo-windows-x64.zip`
2. Extract the archive
3. Run `qmpo-lau.exe` (double-click or run from command prompt)

This will automatically register qmpo as the `directory://` protocol handler.

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
sudo ./target/release/qmpo-lau register  # requires admin privileges
```

### Chrome Extension

The Chrome extension converts `file://` links to `directory://` automatically.

1. Open `chrome://extensions/` in Chrome
2. Enable "Developer mode" (toggle in top right)
3. Click "Load unpacked"
4. Select the `qmpo-extension` folder

The extension adds a folder icon to converted links and provides settings
via the extension popup.

## Test Pages

Test pages are available for verifying qmpo installation:

| OS | English | 日本語 |
| --- | --- | --- |
| Linux | [test-linux.html](docs/test-linux.html) | [test-linux-ja.html](docs/test-linux-ja.html) |
| macOS | [test-macos.html](docs/test-macos.html) | [test-macos-ja.html](docs/test-macos-ja.html) |
| Windows | [test-windows.html](docs/test-windows.html) | [test-windows-ja.html](docs/test-windows-ja.html) |

## License

MIT

---

**O**pen **D**irectory **W**ith **B**rowser → odwb 🔄 qmpo
