# m5rOS Build Guide

## Prerequisites

### Required Tools

1. **Rust Toolchain**
   ```bash
   # Install rustup if not already installed
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Add x86_64 bare metal target
   rustup target add x86_64-unknown-none
   ```

2. **C Compiler** (for userspace - future phases)
   ```bash
   # On Ubuntu/Debian
   sudo apt install clang lld

   # On Arch Linux
   sudo pacman -S clang lld

   # On macOS
   xcode-select --install
   ```

3. **Build Tools**
   ```bash
   # On Ubuntu/Debian
   sudo apt install make

   # On Arch Linux
   sudo pacman -S make
   ```

4. **QEMU** (for testing)
   ```bash
   # On Ubuntu/Debian
   sudo apt install qemu-system-x86

   # On Arch Linux
   sudo pacman -S qemu

   # On macOS
   brew install qemu
   ```

5. **OVMF** (UEFI firmware for QEMU - future phase)
   ```bash
   # On Ubuntu/Debian
   sudo apt install ovmf

   # On Arch Linux
   sudo pacman -S edk2-ovmf
   ```

## Building on Linux (Arch/Ubuntu)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/m4rcel-lol/m5rOS.git
cd m5rOS

# Build everything
make all

# Or use the convenience script
./build.sh
```

### Build Targets

```bash
# Build kernel only
make kernel

# Build bootloader only
make bootloader

# Build userspace (not yet implemented)
make userland

# Build shell (not yet implemented)
make shell

# Clean all artifacts
make clean

# Run tests
make test
```

### Running in QEMU

```bash
# Build and run in QEMU
make qemu

# Or manually
./qemu.sh

# Run with additional QEMU options
./qemu.sh -d int,cpu_reset
```

## Building on Windows

### Using WSL2 (Recommended)

1. Install WSL2 with Ubuntu:
   ```powershell
   wsl --install
   ```

2. Inside WSL, follow the Linux build instructions above

### Native Windows Build (Alternative)

1. Install Rust for Windows from https://rustup.rs/

2. Install LLVM/Clang for Windows

3. Add x86_64 target:
   ```powershell
   rustup target add x86_64-unknown-none
   ```

4. Build using cargo:
   ```powershell
   cargo build --release
   ```

## Build Artifacts

After a successful build, you'll find:

- **Kernel**: `target/x86_64-unknown-none/release/m5r-kernel`
- **Bootloader**: `target/x86_64-unknown-none/release/bootloader`
- **Debug symbols**: `target/x86_64-unknown-none/release/*.pdb` (Windows) or embedded in ELF

## Build Configuration

### Rust Build Configuration

The build uses `stable` Rust with the following features:

- **Target**: `x86_64-unknown-none` (freestanding x86_64)
- **Standard library**: Custom built `core` and `alloc` crates
- **No standard library**: `#![no_std]`
- **Panic**: Abort (no unwinding)

Configuration files:
- `.cargo/config.toml` - Cross-compilation settings
- `Cargo.toml` - Workspace configuration
- `kernel/Cargo.toml` - Kernel dependencies
- `bootloader/Cargo.toml` - Bootloader dependencies

### Linker Script

The kernel uses a custom linker script (`linker.ld`) that:
- Places kernel at higher half (0xFFFFFFFF80000000)
- Aligns sections to 4KB page boundaries
- Separates .text, .rodata, .data, and .bss sections

## Troubleshooting

### "can't find crate for `core`"

Install the x86_64 bare metal target:
```bash
rustup target add x86_64-unknown-none
```

### "rust-lld: error: unknown argument"

Update your Rust toolchain:
```bash
rustup update
```

### QEMU not found

Install QEMU for your platform (see Prerequisites above).

### Build fails with linker errors

Ensure you have the LLVM linker (lld) installed:
```bash
# Ubuntu/Debian
sudo apt install lld

# Arch Linux
sudo pacman -S lld
```

## Development Workflow

### Iterative Development

```bash
# Edit source files
vim kernel/src/main.rs

# Build and test quickly
cargo build --release && ./qemu.sh

# Or use make
make qemu
```

### Debugging

```bash
# Run with QEMU debugging enabled
qemu-system-x86_64 -kernel target/x86_64-unknown-none/release/m5r-kernel \
    -serial stdio -m 256M -no-reboot -s -S

# In another terminal, connect with GDB
gdb target/x86_64-unknown-none/release/m5r-kernel
(gdb) target remote localhost:1234
(gdb) continue
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy (lints)
cargo clippy

# Check without building
cargo check
```

## Continuous Integration

The project is configured for CI with:
- Automatic build verification
- Code formatting checks
- Clippy lints

## Next Steps

Once the kernel is more complete, additional build targets will include:
- Bootable ISO generation (`make iso`)
- Disk image creation
- Installation to USB drive
- Network boot support

## Further Reading

- [Architecture.md](Architecture.md) - System architecture
- [Memory.md](Memory.md) - Memory layout details
- [Syscalls.md](Syscalls.md) - System call interface
