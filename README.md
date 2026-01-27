# qmpo

**Open Directory With Browser** - A cross-platform `directory://` URI scheme
handler that opens directories in your file manager.

[日本語](README-ja.md)

## Overview

qmpo enables you to open directory with browser using a custom URI scheme. When
you click a `directory://` link or enter it in your browser's address bar,
qmpo opens the corresponding directory in your system's file manager.

This tool bridges the gap between web browsers and local file systems, making
it easy to open directory with browser navigation.

## Use Cases

- Open local files directly from links on web pages
- Access shared folders from internal documentation sites
- Navigate to files from search result pages

**Why qmpo?** Cross-platform, single binary, secure (opens directories only).
With the [Chrome extension](qmpo-extension/), existing `file://` links work. 🔄

## URI Format

| OS | File Path | URI Format |
| --- | --- | --- |
| Windows (local) | `C:\Users\tagawa` | `directory://C:/Users/tagawa` |
| Windows (UNC) | `\\server\share\folder` | `directory://server/share/folder` |
| macOS/Linux | `/home/tagawa` | `directory:///home/tagawa` |

**Note:** If a file path is specified, qmpo opens the parent directory
containing that file.

## Installation

### Build from Source

```bash
git clone https://github.com/tagawa0525/qmpo.git
cd qmpo
cargo build --release
```

### Register URI Handler

```bash
# Linux
./target/release/qmpo-lau register

# macOS
./target/release/qmpo-lau register

# Windows
.\target\release\qmpo-lau.exe register
```

## Usage

### Direct Execution

```bash
qmpo "directory:///home/user/Documents"
```

### Open Directory With Browser

Enter a URI in your browser's address bar to open directory with browser:

```text
directory:///home/user/Documents
```

### Management Commands

```bash
# Register as URI handler
qmpo-lau register

# Check registration status
qmpo-lau status

# Unregister
qmpo-lau unregister
```

## Project Structure

```text
qmpo/
├── qmpo-core/    # Core library (URI parsing)
├── qmpo/         # Main application (URI handler)
└── qmpo-lau/     # Registration utility
```

## Platform Support

| Platform | Handler Location | Registration Method |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\qmpo\` | Registry (HKCU) |
| macOS | `~/Applications/qmpo.app/` | Launch Services |
| Linux | `~/.local/bin/` | XDG MIME + Desktop file |

## License

MIT

---

## Why "qmpo"?

**O**pen **D**irectory **W**ith **B**rowser → odwb 🔄 qmpo

Ideally, browsers handle `directory://` links natively. Until then, qmpo
fills the gap.
