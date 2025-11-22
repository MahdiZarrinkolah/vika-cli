#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

REPO="MahdiZarrinkolah/vika-cli"
BINARY_NAME="vika-cli"
INSTALL_DIR=""

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    
    case "$OS" in
        Linux*)
            PLATFORM="linux"
            ;;
        Darwin*)
            PLATFORM="macos"
            ;;
        *)
            echo -e "${RED}Error: Unsupported OS: $OS${NC}"
            exit 1
            ;;
    esac
    
    case "$ARCH" in
        x86_64)
            ARCH_SUFFIX="x86_64"
            ;;
        arm64|aarch64)
            if [ "$PLATFORM" = "macos" ]; then
                ARCH_SUFFIX="arm64"
            else
                echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
                exit 1
            fi
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
            exit 1
            ;;
    esac
    
    if [ "$PLATFORM" = "macos" ] && [ "$ARCH_SUFFIX" = "x86_64" ]; then
        ARCH_SUFFIX="x86_64"
    fi
    
    ASSET_NAME="${BINARY_NAME}-${PLATFORM}-${ARCH_SUFFIX}"
}

# Determine install directory
determine_install_dir() {
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    elif [ -w "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    else
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
        echo -e "${YELLOW}Warning: Installing to $INSTALL_DIR (not in PATH)${NC}"
        echo -e "${YELLOW}Add this to your PATH: export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    fi
}

# Get latest release version
get_latest_version() {
    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        echo -e "${RED}Error: curl or wget is required${NC}"
        exit 1
    fi
    
    if [ -z "$VERSION" ]; then
        echo -e "${RED}Error: Could not determine latest version${NC}"
        exit 1
    fi
}

# Download binary
download_binary() {
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET_NAME}"
    CHECKSUM_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET_NAME}.sha256"
    
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    echo -e "${GREEN}Downloading ${BINARY_NAME} ${VERSION}...${NC}"
    
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$TEMP_DIR/${ASSET_NAME}" "$DOWNLOAD_URL"
        curl -fsSL -o "$TEMP_DIR/${ASSET_NAME}.sha256" "$CHECKSUM_URL"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$TEMP_DIR/${ASSET_NAME}" "$DOWNLOAD_URL"
        wget -q -O "$TEMP_DIR/${ASSET_NAME}.sha256" "$CHECKSUM_URL"
    fi
    
    # Verify checksum
    echo -e "${GREEN}Verifying checksum...${NC}"
    cd "$TEMP_DIR"
    if shasum -a 256 -c "${ASSET_NAME}.sha256" >/dev/null 2>&1; then
        echo -e "${GREEN}Checksum verified${NC}"
    else
        echo -e "${RED}Error: Checksum verification failed${NC}"
        exit 1
    fi
    
    # Install
    echo -e "${GREEN}Installing to ${INSTALL_DIR}...${NC}"
    cp "$TEMP_DIR/${ASSET_NAME}" "$INSTALL_DIR/${BINARY_NAME}"
    chmod +x "$INSTALL_DIR/${BINARY_NAME}"
    
    # Check if in PATH
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo -e "${GREEN}✓ ${BINARY_NAME} installed successfully!${NC}"
        echo -e "${GREEN}Run '${BINARY_NAME} --help' to get started${NC}"
    else
        echo -e "${GREEN}✓ ${BINARY_NAME} installed to ${INSTALL_DIR}${NC}"
        echo -e "${YELLOW}Note: ${INSTALL_DIR} is not in your PATH${NC}"
        echo -e "${YELLOW}Add it with: export PATH=\"${INSTALL_DIR}:\$PATH\"${NC}"
    fi
}

# Main
main() {
    echo -e "${GREEN}Installing ${BINARY_NAME}...${NC}"
    detect_platform
    determine_install_dir
    get_latest_version
    download_binary
}

main "$@"

