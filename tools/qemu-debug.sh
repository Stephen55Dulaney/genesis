#!/bin/bash
# Project Genesis - QEMU Debug Script
# Launches the kernel with GDB debugging enabled
#
# Usage: ./tools/qemu-debug.sh <kernel-binary>
# Then in another terminal: rust-gdb -ex "target remote :1234"

set -e

KERNEL_PATH="$1"

if [ -z "$KERNEL_PATH" ]; then
    echo "Usage: $0 <kernel-binary>"
    exit 1
fi

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           Project Genesis - QEMU Debug Launcher                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "  Kernel: $KERNEL_PATH"
echo "  GDB Server: localhost:1234"
echo ""
echo "  To connect debugger:"
echo "    rust-gdb -ex 'target remote :1234' $KERNEL_PATH"
echo ""
echo "  Press Ctrl+A, X to exit QEMU"
echo ""

# Detect architecture and choose QEMU binary
if [[ "$(uname -m)" == "arm64" ]]; then
    QEMU_BINARY="qemu-system-x86_64"
else
    QEMU_BINARY="qemu-system-x86_64"
fi

# Check if QEMU is installed
if ! command -v $QEMU_BINARY &> /dev/null; then
    echo "ERROR: $QEMU_BINARY not found!"
    echo ""
    echo "Install QEMU with: brew install qemu"
    exit 1
fi

# Run QEMU with GDB server
# -s: Shorthand for -gdb tcp::1234
# -S: Freeze CPU at startup (wait for debugger)
$QEMU_BINARY \
    -drive format=raw,file="$KERNEL_PATH" \
    -serial stdio \
    -m 128M \
    -no-reboot \
    -no-shutdown \
    -s \
    -S

