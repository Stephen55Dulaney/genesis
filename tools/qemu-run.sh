#!/bin/bash
# Project Genesis - QEMU Runner Script
# Launches the kernel in QEMU for testing on Mac
#
# Usage: ./tools/qemu-run.sh <kernel-binary>
# Or via cargo: cargo run

set -e

KERNEL_PATH="$1"

if [ -z "$KERNEL_PATH" ]; then
    echo "Usage: $0 <kernel-binary>"
    exit 1
fi

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              Project Genesis - QEMU Launcher                   ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "  Kernel: $KERNEL_PATH"
echo "  Press Ctrl+A, X to exit QEMU"
echo ""

# Detect architecture and choose QEMU binary
if [[ "$(uname -m)" == "arm64" ]]; then
    # Apple Silicon Mac - use x86_64 emulation
    QEMU_BINARY="qemu-system-x86_64"
else
    # Intel Mac
    QEMU_BINARY="qemu-system-x86_64"
fi

# Check if QEMU is installed
if ! command -v $QEMU_BINARY &> /dev/null; then
    echo "ERROR: $QEMU_BINARY not found!"
    echo ""
    echo "Install QEMU with: brew install qemu"
    exit 1
fi

# Run QEMU with the bootloader
# -serial stdio: Redirect serial port to terminal for debug output
# -display default: Use default display (SDL/Cocoa on Mac)
# -m 128M: 128MB of RAM
# -no-reboot: Exit on triple fault instead of rebooting
$QEMU_BINARY \
    -drive format=raw,file="$KERNEL_PATH" \
    -serial stdio \
    -m 128M \
    -no-reboot \
    -no-shutdown

