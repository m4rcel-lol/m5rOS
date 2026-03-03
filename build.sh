#!/usr/bin/env bash
# Build script for m5rOS
set -e

echo "Building m5rOS..."

# Build the kernel
echo "Building kernel..."
cd kernel
cargo build --release
cd ..

# Build the bootloader
echo "Building bootloader..."
cd bootloader
cargo build --release
cd ..

# Build userspace (will be implemented in later phases)
# echo "Building userspace..."
# make -C libc
# make -C shell
# make -C userland

echo "Build complete!"
echo "Kernel: target/x86_64-unknown-none/release/m5r-kernel"
echo "Bootloader: target/x86_64-unknown-none/release/bootloader"
