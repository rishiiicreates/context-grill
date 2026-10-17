#!/bin/sh
set -e

# context-grill one-line installer
# Usage: curl -fsSL https://raw.githubusercontent.com/rishiicreates/context-grill/main/install.sh | sh

REPO="rishiicreates/context-grill"
DEFAULT_VERSION="0.1.0"
VERSION="${CGRILL_VERSION:-$DEFAULT_VERSION}"
INSTALL_DIR="${CGRILL_INSTALL_DIR:-$HOME/.cgrill/bin}"

# Colors (if terminal supports it)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED=''
    GREEN=''
    BLUE=''
    BOLD=''
    RESET=''
fi

log_info() {
    printf "${BLUE}==>${RESET} ${BOLD}%s${RESET}\n" "$1"
}

log_success() {
    printf "${GREEN}==>${RESET} ${BOLD}%s${RESET}\n" "$1"
}

log_error() {
    printf "${RED}Error:${RESET} %s\n" "$1" >&2
}

# Parse command line flags if provided
DRY_RUN=false
for arg in "$@"; do
    case "$arg" in
        --dry-run)
            DRY_RUN=true
            ;;
        --help|-h)
            echo "context-grill installer"
            echo "Usage: install.sh [--dry-run]"
            echo "Environment variables:"
            echo "  CGRILL_INSTALL_DIR  Installation directory (default: ~/.cgrill/bin)"
            echo "  CGRILL_VERSION      Version to install (default: 0.1.0)"
            exit 0
            ;;
    esac
done

# Detect OS
OS_UNAME="$(uname -s)"
case "$OS_UNAME" in
    Darwin)
        OS="apple-darwin"
        OS_NAME="macOS"
        ;;
    Linux)
        OS="unknown-linux-gnu"
        OS_NAME="Linux"
        ;;
    *)
        log_error "Unsupported operating system: $OS_UNAME (context-grill supports macOS and Linux)"
        exit 1
        ;;
esac

# Detect Architecture
ARCH_UNAME="$(uname -m)"
case "$ARCH_UNAME" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    arm64|aarch64)
        ARCH="aarch64"
        ;;
    *)
        log_error "Unsupported architecture: $ARCH_UNAME (context-grill supports x86_64 and arm64/aarch64)"
        exit 1
        ;;
esac

TARGET="${ARCH}-${OS}"
BINARY_NAME="cgrill"
ASSET_NAME="cgrill-${TARGET}.tar.gz"

log_info "Detected environment: ${OS_NAME} (${ARCH}) -> Target: ${TARGET}"

if [ "$DRY_RUN" = true ]; then
    echo "Dry run mode active. No changes made."
    echo "  Target: $TARGET"
    echo "  Install Directory: $INSTALL_DIR"
    echo "  Download Asset: $ASSET_NAME"
    exit 0
fi

mkdir -p "$INSTALL_DIR"

# Check if local release build exists in current directory (e.g. installing from cloned source)
SCRIPT_DIR="$(cd "$(dirname "$0")" 2>/dev/null && pwd || echo "")"
LOCAL_BIN="${SCRIPT_DIR}/target/release/${BINARY_NAME}"

if [ -f "$LOCAL_BIN" ]; then
    log_info "Found local release binary at $LOCAL_BIN. Copying to $INSTALL_DIR..."
    cp "$LOCAL_BIN" "$INSTALL_DIR/$BINARY_NAME"
else
    # Download from GitHub Releases
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ASSET_NAME}"
    TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'cgrill-install')"
    TMP_ARCHIVE="${TMP_DIR}/${ASSET_NAME}"

    log_info "Downloading context-grill v${VERSION} for ${TARGET}..."
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "$TMP_ARCHIVE"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$DOWNLOAD_URL" -O "$TMP_ARCHIVE"
    else
        log_error "Neither curl nor wget found in PATH."
        rm -rf "$TMP_DIR"
        exit 1
    fi

    log_info "Extracting ${ASSET_NAME}..."
    tar -xzf "$TMP_ARCHIVE" -C "$INSTALL_DIR"
    rm -rf "$TMP_DIR"
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Check if ~/.agents/skills directory exists, and link/install SKILL.md
AGENTS_SKILL_DIR="$HOME/.agents/skills/context-grill"
if [ -d "$HOME/.agents/skills" ]; then
    mkdir -p "$AGENTS_SKILL_DIR"
    if [ -f "${SCRIPT_DIR}/SKILL.md" ]; then
        cp "${SCRIPT_DIR}/SKILL.md" "$AGENTS_SKILL_DIR/SKILL.md"
        log_info "Installed agent skill to $AGENTS_SKILL_DIR/SKILL.md"
    fi
fi

log_success "Successfully installed context-grill binary to $INSTALL_DIR/$BINARY_NAME"

# Check if install dir is in PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*)
        log_success "cgrill is already in your PATH."
        ;;
    *)
        echo ""
        printf "${BOLD}Notice:${RESET} $INSTALL_DIR is not in your PATH.\n"
        printf "Add it to your shell configuration file (~/.zshrc, ~/.bashrc, or ~/.config/fish/config.fish):\n\n"
        printf "  export PATH=\"%s:\$PATH\"\n\n" "$INSTALL_DIR"
        ;;
esac

# Quick verification run
if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
    echo ""
    "$INSTALL_DIR/$BINARY_NAME" --help >/dev/null 2>&1 && log_success "Verification passed: cgrill is ready to use!"
fi
