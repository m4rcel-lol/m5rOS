# m5rOS

A custom operating system built from first principles in Rust and C, inspired by early UNIX and Linux.

## Current Status: Advanced Feature Phase (v0.3.0)

m5rOS has progressed to include a professional boot animation, vi-like text editor, virtual filesystem, comprehensive file operations, and system installer framework.

### ✅ Implemented Components

#### Boot Animation & Service Manager
- **Professional boot animation** with m5rOS ASCII art logo
- **Service initialization display** mimicking Linux distros
- **Visual status indicators** (STARTING → OK/FAILED)
- **10 tracked services**: GDT, IDT, PIC, PIT, Keyboard, Serial, RTC, ATA, Memory Manager, Virtual Memory

#### Text Editor (m5edit)
- **Vi-like editor** with Normal, Insert, and Command modes
- **Navigation**: hjkl keys for cursor movement
- **Editing**: character insertion, deletion, line operations
- **Commands**: :w (save), :q (quit), :wq (save & quit), :q! (force quit)
- **Status line**: mode indicator and file modification state

#### Virtual Filesystem (SimpleFS)
- **In-memory filesystem** with 64 file/directory entries
- **File operations**: create, read, write, delete, find
- **Directory support**: create directories, list contents
- **Storage**: 32KB per file, 2MB total capacity

#### Build System & Infrastructure
- Cargo workspace for Rust components (kernel, bootloader)
- Makefile for orchestration
- Build scripts (build.sh, qemu.sh, iso.sh)
- Cross-compilation to x86_64 bare metal target
- Linker script for kernel memory layout

#### Kernel Core
- **Kernel entry point** with comprehensive initialization
- **Serial port driver** (16550 UART) for debugging output via COM1
- **VGA text mode driver** (80x25, 16 colors) with optimized scrolling
- **Enhanced panic handler** with CPU register dump for debugging
- **Port I/O operations** for hardware access (inb/outb/inw/outw/inl/outl/insw)
- **Physical frame allocator** using bitmap-based allocation (4 KiB frames)
- **Kernel heap allocator** (linked-list) with 16MB heap space
- **CPU feature detection** (CPUID) for vendor and capabilities
- **GDT & IDT** with 21 exception handlers and hardware interrupt support
- **PIC, PIT, PS/2 keyboard** drivers fully functional
- **Complete page table mapper** with map/unmap/translate operations

#### Device Drivers
- **Serial driver** (COM1) for debugging
- **VGA text mode** with color support
- **PS/2 Keyboard** with full US QWERTY layout
- **PIT** (Programmable Interval Timer) at 100 Hz
- **RTC driver** for reading real-time clock
- **ATA PIO driver** for IDE hard disk access (identify, read, write sectors)
- **Framebuffer graphics** with RGB/BGR pixel format support

#### Interactive Command System (22 commands)
- **Command parser** with keyboard input buffering
- **System info**: fetch, help, about, version, uptime
- **Hardware**: cpuinfo, meminfo, stats, heap
- **Time**: date, time (using RTC)
- **File operations**: ls, cat, mkdir, touch, rm, edit
- **Utilities**: clear, echo
- **Power**: reboot, shutdown
- **Installation**: install-m5ros
- **Color-coded categorized help** display
- **Statistics tracking** for IRQs and exceptions

#### System Installer
- **Installation wizard** with professional UI
- **ATA drive detection** and identification
- **Installation steps** outlined (partition, format, bootloader, copy)
- **Framework ready** for bootloader integration

### 🚧 In Progress

- Virtual memory management with 4-level paging
- VGA text mode driver
- Interrupt descriptor table (IDT)
- Exception handlers

### 📋 Roadmap

#### Phase 2-3: Core Kernel (Next)
- Complete memory management (paging, heap allocator)
- Interrupt and exception handling (IDT, PIC, timer, keyboard)
- VGA text output driver

#### Phase 4-5: Process & System Calls
- Process management (scheduler, context switching)
- System call interface
- PID allocation

#### Phase 6-7: Filesystem & Drivers
- m5fs custom filesystem
- VFS abstraction layer
- ATA PIO disk driver

#### Phase 8-9: Userspace
- Custom minimal libc
- Init process (PID 1)
- Core utilities (ls, cat, echo, mkdir, rm, ps, kill)

#### Phase 10: Shell
- m5sh interactive shell with:
  - Command parsing and execution
  - Pipelines and I/O redirection
  - Environment variables
  - Command history and tab completion

## Building

### Prerequisites

- Rust toolchain (stable, edition 2021)
- x86_64 bare metal target: `rustup target add x86_64-unknown-none`
- clang/LLVM
- make
- QEMU (for testing)

### Build Commands

```bash
# Build everything
make all

# Build kernel only
make kernel

# Build and run in QEMU
make qemu

# Clean build artifacts
make clean
```

### Manual Build

```bash
# Build kernel and bootloader
./build.sh

# Run in QEMU
./qemu.sh
```

## Architecture

- **Language**: Kernel in Rust, userspace utilities and shell in C
- **Architecture**: x86_64
- **Boot**: UEFI
- **Kernel Type**: Hybrid monolithic
- **Memory**: 4-level paging (PML4 → PDPT → PD → PT)

## Design Principles

1. **Memory Safety**: Rust's ownership system prevents entire classes of bugs
2. **Explicit Unsafe**: All `unsafe` blocks documented with `// SAFETY:` comments
3. **No GPL Dependencies**: Completely original implementation
4. **Minimalism**: Lean, auditable codebase
5. **Modularity**: Clear subsystem boundaries

## Documentation

- [Architecture.md](Architecture.md) - System design (coming soon)
- [Build.md](Build.md) - Build instructions (coming soon)
- [Syscalls.md](Syscalls.md) - System call reference (coming soon)
- [Memory.md](Memory.md) - Memory layout and management (coming soon)

## Contributing

m5rOS is in early development. Once the core kernel stabilizes, contribution guidelines will be established.

## License

MIT License - See LICENSE file for details

## Acknowledgments

Inspired by:
- The simplicity of early UNIX
- The minimalism of early Linux
- Modern systems programming practices with Rust

