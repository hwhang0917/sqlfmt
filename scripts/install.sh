#!/bin/sh
set -e

INSTALL_DIR="/usr/local/bin"
BINARY_NAME="sqlfmt"
REPO="hwhang0917/sqlfmt"

main() {
    need_cmd curl
    need_cmd tar
    need_cmd uname

    arch=$(uname -m)
    os=$(uname -s | tr '[:upper:]' '[:lower:]')

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) err "Unsupported architecture: $arch" ;;
    esac

    case "$os" in
        linux) target="${arch}-unknown-linux-gnu" ;;
        darwin) target="${arch}-apple-darwin" ;;
        *) err "Unsupported OS: $os" ;;
    esac

    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    tag=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | cut -d'"' -f4)

    if [ -z "$tag" ]; then
        err "Failed to fetch latest release from github.com/${REPO}"
    fi

    url="https://github.com/${REPO}/releases/download/${tag}/${BINARY_NAME}-${tag}-${target}.tar.gz"

    printf "Downloading %s %s for %s...\n" "$BINARY_NAME" "$tag" "$target"
    curl -sL "$url" -o "${tmpdir}/${BINARY_NAME}.tar.gz" || err "Download failed: $url"

    tar -xzf "${tmpdir}/${BINARY_NAME}.tar.gz" -C "$tmpdir"

    if [ ! -f "${tmpdir}/${BINARY_NAME}" ]; then
        err "Binary not found in archive"
    fi

    if [ -w "$INSTALL_DIR" ]; then
        mv "${tmpdir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        printf "Installing to %s (requires sudo)...\n" "$INSTALL_DIR"
        sudo mv "${tmpdir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    printf "Installed %s %s to %s/%s\n" "$BINARY_NAME" "$tag" "$INSTALL_DIR" "$BINARY_NAME"
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "Required command not found: $1"
    fi
}

err() {
    printf "error: %s\n" "$1" >&2
    exit 1
}

main
