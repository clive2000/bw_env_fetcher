#!/bin/bash
set -euo pipefail

REPO="clive2000/bw_env_fetcher"
BINARY_NAME="bw_env_fetcher"
INSTALL_DIR="${HOME}/.local/bin"

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

info() { echo -e "${GREEN}==>${NC} $1"; }
warn() { echo -e "${YELLOW}warning:${NC} $1"; }
error() { echo -e "${RED}error:${NC} $1" >&2; }

get_latest_release() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

detect_platform() {
    local os arch

    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)
            os="unknown-linux-gnu"
            ;;
        darwin)
            os="apple-darwin"
            ;;
        *)
            error "Unsupported OS: $os"
            echo "This installer only supports macOS and Linux." >&2
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        arm64|aarch64)
            arch="aarch64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    echo "${arch}-${os}"
}

check_path() {
    if [[ ":$PATH:" == *":${INSTALL_DIR}:"* ]]; then
        return 0
    fi
    return 1
}

print_path_instructions() {
    echo ""
    warn "${INSTALL_DIR} is not in your PATH"
    echo ""
    echo -e "${BOLD}Add it to your shell configuration:${NC}"
    echo ""
    
    case "${SHELL##*/}" in
        bash)
            echo -e "  ${BOLD}bash${NC} (~/.bashrc):"
            echo -e "    echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            echo ""
            echo "  Then run: source ~/.bashrc"
            ;;
        zsh)
            echo -e "  ${BOLD}zsh${NC} (~/.zshrc):"
            echo -e "    echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
            echo ""
            echo "  Then run: source ~/.zshrc"
            ;;
        fish)
            echo -e "  ${BOLD}fish${NC} (~/.config/fish/config.fish):"
            echo -e "    fish_add_path \$HOME/.local/bin"
            ;;
        *)
            echo -e "  Add the following to your shell's config file:"
            echo -e "    export PATH=\"\$HOME/.local/bin:\$PATH\""
            ;;
    esac
    echo ""
}

main() {
    info "Detecting platform..."
    local platform
    platform=$(detect_platform)
    echo "    Platform: $platform"

    info "Fetching latest release..."
    local version
    version=$(get_latest_release)
    if [[ -z "$version" ]]; then
        error "Could not determine latest version"
        exit 1
    fi
    echo "    Version: $version"

    local download_url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${platform}.tar.gz"
    info "Downloading ${BINARY_NAME}..."
    echo "    URL: $download_url"

    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    if ! curl -fsSL "$download_url" -o "${tmp_dir}/archive.tar.gz"; then
        error "Failed to download release"
        echo "URL: $download_url" >&2
        exit 1
    fi

    info "Extracting..."
    tar -xzf "${tmp_dir}/archive.tar.gz" -C "$tmp_dir"

    info "Installing to ${INSTALL_DIR}..."
    mkdir -p "${INSTALL_DIR}"
    mv "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    echo ""
    echo -e "${GREEN}✓${NC} ${BOLD}${BINARY_NAME}${NC} ${version} installed to ${INSTALL_DIR}/${BINARY_NAME}"

    if ! check_path; then
        print_path_instructions
    else
        echo ""
        echo "Run '${BINARY_NAME} --help' to get started."
    fi
}

main "$@"
