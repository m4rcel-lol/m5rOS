#!/usr/bin/env bash
# QEMU launch script for m5rOS
set -e

KERNEL="target/x86_64-unknown-none/release/m5r-kernel"

if [ ! -f "$KERNEL" ]; then
    echo "Kernel not found. Run ./build.sh first."
    exit 1
fi

# Check if qemu is available
if ! command -v qemu-system-x86_64 &> /dev/null; then
    echo "qemu-system-x86_64 not found. Please install QEMU."
    exit 1
fi

echo "Launching m5rOS in QEMU..."

# For now, we'll use a simple QEMU command
# UEFI support will be added in Phase 2 when bootloader is complete
qemu-system-x86_64 \
    -kernel "$KERNEL" \
    -serial stdio \
    -m 256M \
    -no-reboot \
    -display none \
    "$@"
