#!/bin/sh
set -e

REPO="TattvaOrg/vibe"
DEFAULT_VERSION="v0.0.1"
INSTALL_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"

echo "=========================================="
echo " vibe - Ambient Lighting Suite for Linux  "
echo "=========================================="

# Check operating system
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    echo "Error: vibe is designed exclusively for Linux systems."
    exit 1
fi

# Detect system architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        BIN_NAME="vibe-x86_64-unknown-linux-gnu"
        ;;
    aarch64|arm64)
        BIN_NAME="vibe-aarch64-unknown-linux-gnu"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Resolve release version (latest from GitHub API or fallback to default)
if command -v curl >/dev/null 2>&1; then
    LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/')
elif command -v wget >/dev/null 2>&1; then
    LATEST_TAG=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/')
fi

VERSION="${LATEST_TAG:-$DEFAULT_VERSION}"

# Create target directories
mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_DIR"

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BIN_NAME}"
TARGET_PATH="${INSTALL_DIR}/vibe"

echo "Downloading ${VERSION} binary for ${ARCH}..."
if command -v curl >/dev/null 2>&1; then
    if ! curl -fL "$DOWNLOAD_URL" -o "$TARGET_PATH"; then
        # Fallback to generic binary name
        FALLBACK_URL="https://github.com/${REPO}/releases/download/${VERSION}/vibe"
        curl -fL "$FALLBACK_URL" -o "$TARGET_PATH"
    fi
elif command -v wget >/dev/null 2>&1; then
    if ! wget -qO "$TARGET_PATH" "$DOWNLOAD_URL"; then
        FALLBACK_URL="https://github.com/${REPO}/releases/download/${VERSION}/vibe"
        wget -qO "$TARGET_PATH" "$FALLBACK_URL"
    fi
else
    echo "Error: curl or wget is required to download vibe."
    exit 1
fi

chmod +x "$TARGET_PATH"
echo "Installed binary to ${TARGET_PATH}"

# Install desktop icon
ICON_URL="https://raw.githubusercontent.com/${REPO}/main/assets/icons/logo.svg"
if command -v curl >/dev/null 2>&1; then
    curl -sL "$ICON_URL" -o "${ICON_DIR}/vibe.svg" || true
elif command -v wget >/dev/null 2>&1; then
    wget -qO "${ICON_DIR}/vibe.svg" "$ICON_URL" || true
fi

# Install desktop entry
cat << 'EOF' > "${DESKTOP_DIR}/vibe.desktop"
[Desktop Entry]
Name=vibe
Comment=Ambient Lighting & Now-Playing Overlay for Linux
Exec=vibe settings
Icon=vibe
Terminal=false
Type=Application
Categories=AudioVideo;Utility;
Keywords=ambient;lighting;visualizer;music;mpris;
EOF

chmod 644 "${DESKTOP_DIR}/vibe.desktop"
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
fi

echo ""
echo "Installation complete!"
echo ""
echo "Verify PATH:"
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo "Note: ${INSTALL_DIR} is not in your current PATH."
        echo "Add the following line to your shell configuration (~/.bashrc or ~/.zshrc):"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        ;;
esac

echo "Usage:"
echo "  vibe daemon    # Start background ambient lighting overlay"
echo "  vibe settings  # Open matte-black configuration window"
echo "  vibe toggle    # Toggle glow/tracer on or off"
echo "  vibe next-mode # Cycle animation mode"
echo "=========================================="
