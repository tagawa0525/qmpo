# qmpo Browser Extension

**Open Directory With Browser** - Convert `file://` links to `directory://`
scheme, enabling local file/folder access via the qmpo handler.

Solves the problem where clicking `file://` links in browsers doesn't work.

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

1. **Clone or download this repository**

   ```bash
   git clone https://github.com/tagawa0525/qmpo.git
   cd qmpo
   ```

2. **Open Chrome Extensions page**

   - Navigate to `chrome://extensions/` in Chrome
   - Or: Menu (⋮) → Extensions → Manage Extensions

3. **Enable Developer Mode**

   - Toggle "Developer mode" switch in the top-right corner

4. **Load the extension**

   - Click "Load unpacked" button
   - Select the `qmpo-extension` folder (not the parent `qmpo` folder)
   - The extension should now appear in your extensions list

5. **Verify installation**

   - You should see "qmpo - Open Local Files" in the extensions list
   - The extension icon should appear in the toolbar (you may need to pin it)

### Troubleshooting

#### Manifest file is missing or unreadable

- Make sure you selected the `qmpo-extension` folder, not the parent directory
- Verify `manifest.json` exists in the selected folder

#### Extension not working on a page

- Refresh the page after installing the extension
- Check if the domain is in the blocked list (click extension icon → Settings)
- Ensure qmpo is installed: run `qmpo-lau status` in terminal

#### Failed to open directory error

- Install qmpo: see [Installing qmpo](#installing-qmpo) section
- Register the protocol handler: `qmpo-lau register`

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
Converts to directory://path/to/file 🔄
        ↓
Browser triggers URI handler
        ↓
qmpo opens file manager with file selected
```

## Comparison with Local Explorer

| Feature | Local Explorer | qmpo + Extension |
| --- | --- | --- |
| Platform | Windows only | Windows/macOS/Linux |
| Browser | Chrome/Edge | Chrome/Chromium-based |
| Communication | Native Messaging API | Custom URI scheme |
| Behavior | Opens files directly | Opens directory with file selected |
| Security | Can execute files | Directory only (safer) |

## File Structure

```text
qmpo-extension/
├── manifest.json    # Extension configuration (permissions, scripts, etc.)
├── background.js    # Service Worker (background processing)
├── content.js       # Content script (runs in web pages)
├── popup.html       # Popup UI when clicking toolbar icon
├── popup.js         # Popup logic
├── options.html     # Settings page UI
├── options.js       # Settings page logic
├── icons/           # Extension icons
│   ├── icon16.png
│   ├── icon48.png
│   └── icon128.png
├── test.html        # Local test page
├── README.md        # English documentation
└── README-ja.md     # Japanese documentation
```

### File Descriptions

#### manifest.json

Chrome extension configuration file. Defines:

- Extension name, version, description
- Required permissions (`storage`, `activeTab`, `tabs`)
- Content script injection rules
- Background Service Worker registration

#### content.js

Script that runs within web pages. Main functions:

- Detects links starting with `file://`
- Intercepts click events
- Converts `file://` to `directory://`
- Triggers protocol handler via hidden iframe
- Adds visual indicator (📂) to links

#### background.js

Service Worker running in extension background:

- Handles messages from content script
- (Currently unused since migration to hidden iframe approach)

#### popup.html / popup.js

UI displayed when clicking the extension icon in toolbar:

- Toggle extension on/off
- Toggle folder icon display
- Link to settings page

#### options.html / options.js

Extension settings page:

- Manage allowed domains list
- Manage blocked domains list
- Import/export settings

## Development

```bash
# Watch for changes (optional)
npm install -g web-ext
web-ext run --source-dir=.
```

## License

MIT
