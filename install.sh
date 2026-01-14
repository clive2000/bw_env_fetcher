#!/bin/bash
set -euo pipefail

REPO="x68huang/bw_env_fetcher"
BINARY_NAME="bw_env_fetcher"
INSTALL_DIR="/usr/local/bin"

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
            echo "Error: Unsupported OS: $os" >&2
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
            echo "Error: Unsupported architecture: $arch" >&2
            exit 1
            ;;
    esac

    echo "${arch}-${os}"
}

main() {
    echo "==> Detecting platform..."
    local platform
    platform=$(detect_platform)
    echo "    Platform: $platform"

    echo "==> Fetching latest release..."
    local version
    version=$(get_latest_release)
    if [[ -z "$version" ]]; then
        echo "Error: Could not determine latest version" >&2
        exit 1
    fi
    echo "    Version: $version"

    local download_url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${platform}.tar.gz"
    echo "==> Downloading ${BINARY_NAME}..."
    echo "    URL: $download_url"

    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    if ! curl -fsSL "$download_url" -o "${tmp_dir}/archive.tar.gz"; then
        echo "Error: Failed to download release" >&2
        echo "URL: $download_url" >&2
        exit 1
    fi

    echo "==> Extracting..."
    tar -xzf "${tmp_dir}/archive.tar.gz" -C "$tmp_dir"

    echo "==> Installing to ${INSTALL_DIR}..."
    if [[ -w "$INSTALL_DIR" ]]; then
        mv "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/"
        chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    else
        echo "    (requires sudo)"
        sudo mv "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/"
        sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    echo "==> Done!"
    echo ""
    echo "${BINARY_NAME} ${version} has been installed to ${INSTALL_DIR}/${BINARY_NAME}"
    echo ""
    echo "Run '${BINARY_NAME} --help' to get started."
}

main "$@"
