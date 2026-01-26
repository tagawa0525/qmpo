# qmpo Browser Extension

**Open Directory With Browser** - Convert `file://` links to `directory://`
scheme, enabling local file/folder access via the qmpo handler.

qmpo is a cross-platform `directory://` URI scheme handler that opens
directories in your file manager. This extension bridges `file://` links to
qmpo, making it a superior alternative to existing solutions like Local
Explorer.

## Features

- Automatically converts `file://` links to `directory://`
- Works on any website (configurable domain allowlist/blocklist)
- Visual indicator (folder icon) on converted links
- No external dependencies - pure JavaScript

## Requirements

- **qmpo** must be installed and registered as the `directory://` URI handler
- Chrome or Chromium-based browser (Edge, Brave, etc.)

### Installing qmpo

Download qmpo from
[GitHub Releases](https://github.com/tagawa0525/qmpo/releases):

| OS | Download |
| --- | --- |
| Windows | `qmpo-windows-x64.zip` |
| macOS (Intel) | `qmpo-macos-x64.tar.gz` |
| macOS (Apple Silicon) | `qmpo-macos-arm64.tar.gz` |
| Linux | `qmpo-linux-x64.tar.gz` |

Or use the install script:

```bash
# macOS/Linux
curl -sSL \
  https://raw.githubusercontent.com/tagawa0525/qmpo/main/scripts/install.sh \
  | bash

# Windows (PowerShell)
irm \
  https://raw.githubusercontent.com/tagawa0525/qmpo/main/scripts/install.ps1 \
  | iex
```

For more details, see the [qmpo repository](https://github.com/tagawa0525/qmpo).

## Installation

### From Source (Developer Mode)

1. Clone or download this repository
2. Open Chrome and navigate to `chrome://extensions/`
3. Enable "Developer mode" (toggle in top-right)
4. Click "Load unpacked"
5. Select the `qmpo-extension` folder

### From Chrome Web Store

(Coming soon)

## Usage

1. Ensure qmpo is installed and registered:

   ```bash
   qmpo-lau status
   # If not registered:
   qmpo-lau register
   ```

2. Visit any page with `file://` links (e.g., internal wiki, Confluence)

3. Click a `file://` link - it will open in your file manager with the file
   selected

## Configuration

Click the extension icon or go to Settings:

- **Enable conversion**: Toggle the extension on/off
- **Show folder icon**: Display a visual indicator on converted links
- **Allowed Domains**: Only convert links on these domains (empty = all)
- **Blocked Domains**: Never convert links on these domains

### Example: Corporate Intranet Only

```text
Allowed Domains:
wiki.company.com
confluence.company.com
intranet.internal
```

## How It Works

```text
User clicks file://path/to/file
        ↓
Extension intercepts click
        ↓
Converts to directory://path/to/file
        ↓
Browser triggers URI handler
        ↓
qmpo opens file manager with file selected
```

## Comparison with Local Explorer

| Feature | Local Explorer | qmpo + Extension |
| --- | --- | --- |
| Platform | Windows only | Windows/macOS/Linux |
| Browser | Chrome/Edge | All browsers |
| Stability | Breaks on Chrome updates | OS-level handler, stable |
| File selection | Limited | Full support |
| Dependencies | Helper app required | Single binary |

## Development

```bash
# Watch for changes (optional)
npm install -g web-ext
web-ext run --source-dir=.
```

## License

MIT
