#!/bin/bash
#
# qmpo installer for macOS and Linux
#
# Usage:
#   Interactive:  ./install.sh
#   Silent:       ./install.sh --silent
#   Specific ver: ./install.sh --version v0.2.0
#
# Or via curl:
#   curl -sSL https://raw.githubusercontent.com/tagawa0525/qmpo/main/scripts/install.sh | bash
#

set -e

REPO_OWNER="tagawa0525"
REPO_NAME="qmpo"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"
SILENT=false

# Colors (disabled in silent mode)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    if [ "$SILENT" = true ] && [ "$level" != "ERROR" ]; then
        return
    fi

    case "$level" in
        INFO)  echo -e "${GREEN}[$timestamp] [INFO]${NC} $message" ;;
        WARN)  echo -e "${YELLOW}[$timestamp] [WARN]${NC} $message" ;;
        ERROR) echo -e "${RED}[$timestamp] [ERROR]${NC} $message" >&2 ;;
    esac
}

detect_platform() {
    local os arch artifact_name

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin)
            case "$arch" in
                x86_64)  artifact_name="qmpo-macos-x64.tar.gz" ;;
                arm64)   artifact_name="qmpo-macos-arm64.tar.gz" ;;
                aarch64) artifact_name="qmpo-macos-arm64.tar.gz" ;;
                *)
                    log ERROR "Unsupported macOS architecture: $arch"
                    exit 1
                    ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64) artifact_name="qmpo-linux-x64.tar.gz" ;;
                *)
                    log ERROR "Unsupported Linux architecture: $arch"
                    exit 1
                    ;;
            esac
            ;;
        *)
            log ERROR "Unsupported OS: $os"
            exit 1
            ;;
    esac

    echo "$artifact_name"
}

get_latest_version() {
    local url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    local version

    if command -v curl &> /dev/null; then
        version=$(curl -sL "$url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    elif command -v wget &> /dev/null; then
        version=$(wget -qO- "$url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    else
        log ERROR "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    if [ -z "$version" ]; then
        log ERROR "Failed to get latest version"
        exit 1
    fi

    echo "$version"
}

download_file() {
    local url="$1"
    local output="$2"

    log INFO "Downloading from: $url"

    if command -v curl &> /dev/null; then
        curl -sL "$url" -o "$output"
    elif command -v wget &> /dev/null; then
        wget -q "$url" -O "$output"
    fi
}

install_qmpo() {
    log INFO "Starting qmpo installation..."

    # Detect platform
    local artifact_name
    artifact_name=$(detect_platform)
    log INFO "Detected platform: $artifact_name"

    # Get version
    if [ "$VERSION" = "latest" ]; then
        log INFO "Fetching latest version..."
        VERSION=$(get_latest_version)
    fi
    log INFO "Installing version: $VERSION"

    # Create install directory
    mkdir -p "$INSTALL_DIR"
    log INFO "Install directory: $INSTALL_DIR"

    # Download
    local download_url="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$VERSION/$artifact_name"
    local tmp_file
    tmp_file=$(mktemp)

    download_file "$download_url" "$tmp_file"

    # Extract
    log INFO "Extracting..."
    tar -xzf "$tmp_file" -C "$INSTALL_DIR"
    rm -f "$tmp_file"

    # Make executable
    chmod +x "$INSTALL_DIR/qmpo" "$INSTALL_DIR/qmpo-lau"

    # Register URI scheme
    register_uri_scheme

    # Check PATH
    check_path

    log INFO "Installation completed successfully!"
}

register_uri_scheme() {
    local os
    os="$(uname -s)"

    case "$os" in
        Darwin)
            register_macos_uri_scheme
            ;;
        Linux)
            register_linux_uri_scheme
            ;;
    esac
}

register_macos_uri_scheme() {
    log INFO "Registering directory:// URI scheme for macOS..."

    # Run qmpo-lau to register (it handles the plist creation)
    if [ -x "$INSTALL_DIR/qmpo-lau" ]; then
        "$INSTALL_DIR/qmpo-lau" register 2>/dev/null || true
        log INFO "URI scheme registered via qmpo-lau"
    else
        log WARN "qmpo-lau not found, skipping URI registration"
    fi
}

register_linux_uri_scheme() {
    log INFO "Registering directory:// URI scheme for Linux..."

    local desktop_file="$HOME/.local/share/applications/qmpo.desktop"
    mkdir -p "$(dirname "$desktop_file")"

    cat > "$desktop_file" << EOF
[Desktop Entry]
Name=qmpo
Comment=Open Directory With Browser
Exec=$INSTALL_DIR/qmpo %u
Type=Application
NoDisplay=true
MimeType=x-scheme-handler/directory;
EOF

    # Update desktop database
    if command -v update-desktop-database &> /dev/null; then
        update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    fi

    # Set as default handler
    if command -v xdg-mime &> /dev/null; then
        xdg-mime default qmpo.desktop x-scheme-handler/directory 2>/dev/null || true
    fi

    log INFO "URI scheme registered"
}

check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        log WARN "$INSTALL_DIR is not in your PATH"

        local shell_config=""
        case "$SHELL" in
            */bash) shell_config="$HOME/.bashrc" ;;
            */zsh)  shell_config="$HOME/.zshrc" ;;
            *)      shell_config="$HOME/.profile" ;;
        esac

        log INFO "Add the following line to $shell_config:"
        log INFO "  export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

show_help() {
    cat << EOF
qmpo installer

Usage: $0 [OPTIONS]

Options:
    --silent          Run in silent mode
    --version VER     Install specific version (e.g., v0.2.0)
    --install-dir DIR Installation directory (default: ~/.local/bin)
    --help            Show this help message

Examples:
    $0                          # Interactive installation
    $0 --silent                 # Silent installation
    $0 --version v0.2.0         # Install specific version
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --silent)
            SILENT=true
            shift
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            log ERROR "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main
install_qmpo
