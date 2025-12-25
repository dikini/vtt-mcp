#!/bin/bash

# vtt-mcp Deployment Script (Temporary Testing Mode)

# Installation Paths
LOCAL_BIN="$HOME/.local/bin/vtt-mcp"
LOCAL_SHARE="$HOME/.local/share/vtt-mcp"

# Local Testing: Replace <url> with local tarball path
TARBALL_URL="$(pwd)/test_vtt_mcp.tar.gz"

# Helper Function: Print Usage
function print_usage() {
    echo "Usage: $0 [--uninstall]"
}

# Uninstall Logic
if [[ "$1" == "--uninstall" ]]; then
    echo "Removing vtt-mcp files..."
    rm -rf "$LOCAL_BIN"
    rm -rf "$LOCAL_SHARE"
    if [[ $? -eq 0 ]]; then
        echo "Uninstall completed successfully."
    else
        echo "Error occurred while uninstalling."
    fi
    exit 0
fi

# Check Prerequisites
if ! command -v curl &> /dev/null; then
    echo "Error: curl is required to simulate the download."
    exit 1
fi
if ! command -v tar &> /dev/null; then
    echo "Error: tar is required to extract files. Please install tar."
    exit 1
fi

# Create Installation Directories
mkdir -p "$LOCAL_BIN"
mkdir -p "$LOCAL_SHARE"

# Simulate Download and Extract the fake tarball
cat "$TARBALL_URL" | tar -xz -C "$LOCAL_BIN" --strip-components=1
if [[ $? -ne 0 ]]; then
    echo "Error occurred during simulated download or extraction."
    exit 1
fi

echo "vtt-mcp has been successfully simulated for installation at $LOCAL_BIN."
exit 0
