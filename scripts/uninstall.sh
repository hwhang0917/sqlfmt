#!/bin/sh
set -e

INSTALL_DIR="${SQLFMT_INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="sqlfmt"

main() {
    if [ ! -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        printf "%s is not installed at %s\n" "$BINARY_NAME" "$INSTALL_DIR"
        exit 0
    fi

    rm "${INSTALL_DIR}/${BINARY_NAME}"
    printf "Uninstalled %s from %s\n" "$BINARY_NAME" "$INSTALL_DIR"
}

main
