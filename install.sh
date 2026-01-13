#!/usr/bin/env bash
set -euo pipefail

REPO="dbrekelmans/claude-worktree"
BINARY_NAME="worktree"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}$1${NC}"; }
warn() { echo -e "${YELLOW}$1${NC}"; }
error() { echo -e "${RED}$1${NC}" >&2; exit 1; }

# Detect OS and architecture
detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin) os="macos" ;;
        Linux) os="linux" ;;
        *) error "Unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) error "Unsupported architecture: $arch" ;;
    esac

    # Linux builds are not currently provided; only macOS releases are available
    if [[ "$os" == "linux" ]]; then
        error "Linux builds are not currently provided. Only macOS releases are available. See README for building from source."
    fi

    echo "${BINARY_NAME}-${os}-${arch}"
}

# Get download URL for a version
get_download_url() {
    local version="$1"
    local asset_name="$2"

    if [[ "$version" == "latest" ]]; then
        echo "https://github.com/${REPO}/releases/latest/download/${asset_name}.tar.gz"
    else
        # Remove 'v' prefix if present for consistency
        version="${version#v}"
        echo "https://github.com/${REPO}/releases/download/v${version}/${asset_name}.tar.gz"
    fi
}

# Download and install
install() {
    local version="${1:-latest}"
    local asset_name
    local download_url
    local tmp_dir=""
    local existing_binary="$INSTALL_DIR/$BINARY_NAME"

    info "Installing worktree..."

    # Detect platform
    asset_name="$(detect_platform)"
    info "Detected platform: $asset_name"

    # Check if already installed
    if [[ -f "$existing_binary" ]]; then
        local current_version
        current_version="$("$existing_binary" --version 2>/dev/null | awk '{print $2}' || echo "unknown")"
        warn ""
        warn "worktree is already installed (version: $current_version)"
        warn "Location: $existing_binary"
        warn ""
        printf "Do you want to override the existing installation? [y/N]: "
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            info "Installation cancelled."
            exit 0
        fi
        info ""
    fi

    # Get download URL
    download_url="$(get_download_url "$version" "$asset_name")"
    info "Downloading from: $download_url"

    # Create temp directory
    tmp_dir="$(mktemp -d)"
    trap '[[ -n "$tmp_dir" ]] && rm -rf "$tmp_dir"' EXIT

    # Download
    if command -v curl &> /dev/null; then
        curl -fsSL "$download_url" -o "$tmp_dir/worktree.tar.gz" || error "Download failed. Check if the version exists."
    elif command -v wget &> /dev/null; then
        wget -q "$download_url" -O "$tmp_dir/worktree.tar.gz" || error "Download failed. Check if the version exists."
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    # Verify checksum if available
    if command -v sha256sum &> /dev/null || command -v shasum &> /dev/null; then
        checksum_url="${download_url}.sha256"
        checksum_file="$tmp_dir/worktree.tar.gz.sha256"

        # Try to download checksum file
        checksum_downloaded=false
        if command -v curl &> /dev/null; then
            if curl -fsSL "$checksum_url" -o "$checksum_file" 2>/dev/null; then
                checksum_downloaded=true
            fi
        elif command -v wget &> /dev/null; then
            if wget -q "$checksum_url" -O "$checksum_file" 2>/dev/null; then
                checksum_downloaded=true
            fi
        fi

        if [[ "$checksum_downloaded" == "true" ]]; then
            expected_checksum="$(awk 'NR==1 {print $1}' "$checksum_file")"
            if [[ -n "$expected_checksum" ]]; then
                info "Verifying checksum..."
                if command -v sha256sum &> /dev/null; then
                    echo "$expected_checksum  $tmp_dir/worktree.tar.gz" | sha256sum -c - >/dev/null 2>&1 || error "Checksum verification failed. Aborting installation."
                else
                    # macOS uses shasum
                    echo "$expected_checksum  $tmp_dir/worktree.tar.gz" | shasum -a 256 -c - >/dev/null 2>&1 || error "Checksum verification failed. Aborting installation."
                fi
                info "Checksum verified."
            else
                warn "Checksum file downloaded but could not be parsed. Skipping checksum verification."
            fi
        else
            warn "Checksum file not available. Skipping checksum verification."
        fi
    else
        warn "sha256sum/shasum not available; skipping checksum verification."
    fi

    # Extract
    tar -xzf "$tmp_dir/worktree.tar.gz" -C "$tmp_dir"

    # Create install directory if needed
    mkdir -p "$INSTALL_DIR"

    # Install binary
    mv "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    info "Installed to: $INSTALL_DIR/$BINARY_NAME"

    # Check if install dir is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn ""
        warn "Note: $INSTALL_DIR is not in your PATH."
        warn "Add it to your shell config:"
        warn ""
        warn "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
        warn "  # or for zsh:"
        warn "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
        warn ""
    fi

    info ""
    info "Installation complete! Run 'worktree --help' to get started."
}

# Parse arguments
VERSION="latest"
if [[ $# -gt 0 ]]; then
    VERSION="$1"
fi

install "$VERSION"
