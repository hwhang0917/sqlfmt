#!/bin/sh
set -e

INSTALL_DIR="/usr/local/bin"
BINARY_NAME="sqlfmt"

main() {
    if [ ! -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        printf "%s is not installed at %s\n" "$BINARY_NAME" "$INSTALL_DIR"
        exit 0
    fi

    if [ -w "$INSTALL_DIR" ]; then
        rm "${INSTALL_DIR}/${BINARY_NAME}"
    else
        printf "Removing %s/%s (requires sudo)...\n" "$INSTALL_DIR" "$BINARY_NAME"
        sudo rm "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    printf "Uninstalled %s from %s\n" "$BINARY_NAME" "$INSTALL_DIR"
}

main
