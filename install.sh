#!/bin/sh
set -e

REPO="tengu-apps/tengu-init"
INSTALL_DIR="${TENGU_INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="tengu-init"

# Colors (if terminal supports them)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    CYAN='\033[0;36m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    CYAN=''
    NC=''
fi

info() { printf "${CYAN}INFO${NC} %s\n" "$1"; }
success() { printf "${GREEN}OK${NC}   %s\n" "$1"; }
warn() { printf "${YELLOW}WARN${NC} %s\n" "$1"; }
error() { printf "${RED}ERR${NC}  %s\n" "$1" >&2; exit 1; }

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64)  ARTIFACT="tengu-init-linux-amd64" ;;
                aarch64) ARTIFACT="tengu-init-linux-arm64" ;;
                arm64)   ARTIFACT="tengu-init-linux-arm64" ;;
                *)       error "Unsupported Linux architecture: $ARCH" ;;
            esac
            ;;
        Darwin)
            case "$ARCH" in
                arm64)   ARTIFACT="tengu-init-apple-silicon" ;;
                x86_64)  error "Intel Mac not supported. Use ARM64 (Apple Silicon)." ;;
                *)       error "Unsupported macOS architecture: $ARCH" ;;
            esac
            ;;
        *)
            error "Unsupported operating system: $OS"
            ;;
    esac

    info "Detected platform: $OS $ARCH -> $ARTIFACT"
}

# Get latest release tag from GitHub API
get_latest_version() {
    info "Fetching latest release..."

    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    if [ -z "$VERSION" ]; then
        error "Failed to fetch latest version"
    fi

    success "Latest version: $VERSION"
}

# Download binary
download_binary() {
    URL="https://github.com/$REPO/releases/download/$VERSION/$ARTIFACT"
    TMPFILE=$(mktemp)

    info "Downloading $URL"

    if command -v curl >/dev/null 2>&1; then
        curl -fSL --progress-bar "$URL" -o "$TMPFILE"
    else
        wget --show-progress -qO "$TMPFILE" "$URL"
    fi

    success "Downloaded to $TMPFILE"
}

# Install binary
install_binary() {
    mkdir -p "$INSTALL_DIR"

    mv "$TMPFILE" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    success "Installed to $INSTALL_DIR/$BINARY_NAME"
}

# Check if install dir is in PATH
check_path() {
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            warn "$INSTALL_DIR is not in your PATH"
            echo ""
            echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
            echo ""
            echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
            echo ""
            ;;
    esac
}

# Verify installation
verify_install() {
    if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
        echo ""
        success "Installation complete!"
        echo ""
        "$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null || true
    else
        error "Installation failed"
    fi
}

main() {
    echo ""
    echo "  ${CYAN}tengu-init${NC} installer"
    echo "  ────────────────────"
    echo ""

    detect_platform
    get_latest_version
    download_binary
    install_binary
    check_path
    verify_install
}

main
