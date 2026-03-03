# m5rOS Architecture

## Overview

m5rOS is a custom operating system designed from first principles, inspired by the simplicity of early UNIX and the minimalism of early Linux. It features a hybrid monolithic kernel written in Rust, with userspace utilities and shell implemented in C.

**Version**: 0.1.0 (Foundation Phase)
**Target Architecture**: x86_64
**Boot Interface**: UEFI

## Design Philosophy

### Core Principles

1. **Memory Safety First**: Leverage Rust's ownership system to prevent entire classes of bugs
2. **Explicit Unsafety**: All unsafe operations clearly documented with SAFETY comments
3. **Original Implementation**: No reuse of Linux, BSD, or GPL kernel code
4. **Minimalism**: Lean, auditable codebase with no unnecessary complexity
5. **Modularity**: Clear subsystem boundaries and well-defined interfaces

### Technology Choices

**Kernel Language: Rust**
- Memory safety without garbage collection
- Ownership and borrow checking eliminate use-after-free, double-free, and data races
- Strong type system enforces invariants at compile time
- Explicit `unsafe` blocks make dangerous operations visible and auditable
- Growing adoption in production kernel development

**Userspace Language: C**
- Minimal runtime overhead and small binary size
- Natural fit for POSIX-inspired utilities
- Well-understood ABI for system call wrappers
- Traditional choice for shells and core utilities

## System Architecture

### Boot Sequence

```
┌─────────────────┐
│  UEFI Firmware  │
└────────┬────────┘
         │
         ▼
┌─────────────────────┐
│ UEFI Bootloader     │  ← Rust implementation
│  - Load kernel ELF  │
│  - Setup paging     │
│  - Pass memory map  │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│   Kernel (m5rOS)    │
│  - Init hardware    │
│  - Setup memory     │
│  - Init interrupts  │
│  - Mount filesystem │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│   init (PID 1)      │
│  - Spawn services   │
│  - Reap zombies     │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│  m5sh (Shell)       │
│  - User interaction │
└─────────────────────┘
```

## Kernel Architecture

### Kernel Type: Hybrid Monolithic

m5rOS uses a hybrid monolithic architecture similar to early UNIX and Linux:
- Single kernel image loaded into memory
- All core services run in kernel space (Ring 0)
- Modular internal structure for maintainability
- Clear subsystem boundaries despite monolithic design

### Memory Layout

```
Virtual Address Space (x86_64):

0x0000000000000000 ┌────────────────────────┐
                   │   User Space           │
                   │   (Ring 3)             │
                   │                        │
                   │   - User code          │
                   │   - User data          │
                   │   - User heap          │
                   │   - User stack         │
0x0000800000000000 ├────────────────────────┤
                   │   Canonical Hole       │
                   │   (Non-addressable)    │
0xFFFF800000000000 ├────────────────────────┤
                   │   Kernel Space         │
                   │   (Ring 0)             │
                   │                        │
0xFFFFFFFF80000000 │   - Kernel .text       │  ← Kernel loaded here
                   │   - Kernel .rodata     │
                   │   - Kernel .data       │
                   │   - Kernel .bss        │
                   │   - Kernel heap        │
                   │   - Kernel stacks      │
0xFFFFFFFFFFFFFFFF └────────────────────────┘
```

### Kernel Subsystems

#### 1. Memory Management

**Physical Memory Allocator**
- Bitmap-based frame allocation
- Manages 4 KiB frames
- Supports up to 4 GB RAM (Phase 1)
- Thread-safe using spin locks
- O(n) allocation with hint optimization

**Virtual Memory Manager** (Planned)
- 4-level paging (PML4 → PDPT → PD → PT)
- Per-process page tables
- Copy-on-write support
- Demand paging
- Memory-mapped files

**Kernel Heap Allocator** (Planned)
- Buddy allocator for variable-size allocations
- Slab allocator for fixed-size objects
- Coalescing to prevent fragmentation
- Lock-free fast path where possible

#### 2. Process Management (Planned)

**Task Control Block (TCB)**
```rust
struct Task {
    pid: usize,
    state: TaskState,
    page_table: PhysAddr,
    kernel_stack: VirtAddr,
    user_stack: VirtAddr,
    context: Context,
    parent: Option<usize>,
    children: Vec<usize>,
}
```

**Scheduler**
- Round-robin scheduling (Phase 1)
- Preemptive multitasking
- Priority-based scheduling (Phase 2)
- SMP-aware design (future)

**Context Switching**
- Inline assembly for register save/restore
- Safe Rust wrapper for context switch function
- Per-CPU kernel stacks
- Fast system call path

#### 3. Interrupt & Exception Handling

**Interrupt Descriptor Table (IDT)**
- 256 entries covering all x86_64 interrupt vectors
- Separate handlers for each exception type
- Debug/stack trace on unhandled exceptions

**Programmable Interrupt Controller (PIC)**
- 8259 PIC support (Phase 1)
- APIC/IO-APIC support (Phase 2)
- Interrupt masking and prioritization

**Timer (PIT/APIC)**
- Programmable Interval Timer for scheduling
- Configurable tick rate (typically 100 Hz)
- High-resolution timers (future)

#### 4. Device Drivers

**Current Drivers:**
- 16550 UART Serial Port (COM1) - for debugging
- VGA Text Mode (80×25) - planned
- PS/2 Keyboard - planned
- ATA PIO Disk - planned

**Driver Model:**
- Each driver exposes standard interface
- Registration at boot time
- Interrupt-driven I/O where possible
- DMA support (future)

#### 5. Filesystem

**m5fs - Custom Filesystem**
- Extent-based block allocation
- Inodes for file metadata
- Directory entries with name→inode mapping
- Block size: 4 KiB
- No journaling (Phase 1)

**VFS Layer**
- Abstract filesystem interface
- Mount point management
- Path resolution
- File descriptor table

#### 6. System Call Interface (Planned)

**System Call Mechanism:**
- Uses `syscall` instruction (x86_64)
- Arguments passed in registers
- Number in RAX, args in RDI, RSI, RDX, R10, R8, R9
- Return value in RAX
- Error codes as negative returns

**Core System Calls:**
```
read    - Read from file descriptor
write   - Write to file descriptor
open    - Open file
close   - Close file descriptor
fork    - Create child process
exec    - Execute program
exit    - Terminate process
wait    - Wait for child process
getpid  - Get process ID
kill    - Send signal to process
```

## Security Model

### Privilege Levels

- **Ring 0 (Kernel)**: Full hardware access, kernel code only
- **Ring 3 (User)**: Limited access, all user processes

### Capability-Based Security (Planned)

- No traditional root user
- Capabilities grant specific privileges
- Principle of least privilege
- Capabilities inherited across fork, dropped on exec

### Memory Protection

- NX (No Execute) bit enforced
- DEP (Data Execution Prevention)
- ASLR-ready memory layout
- Stack canaries in kernel
- Guard pages for stack overflow detection

### Safety Guarantees

1. **Rust Safety**: Most kernel code is safe Rust
2. **Unsafe Audit**: All unsafe blocks documented
3. **No Uninitialized Memory**: Memory zeroed on allocation
4. **Bounds Checking**: Array access validated
5. **Type Safety**: Strong typing prevents misuse

## Userspace Architecture

### Process Model

- Unix-like process tree
- PID 1 is init
- fork/exec model for process creation
- Copy-on-write for efficient forking
- Zombie reaping by parent or init

### Address Space Layout

```
User Process Virtual Memory:

0x0000000000000000 ┌────────────────┐
                   │   NULL page    │  (unmapped, causes segfault)
0x0000000000001000 ├────────────────┤
                   │   .text        │  (executable code)
                   ├────────────────┤
                   │   .rodata      │  (read-only data)
                   ├────────────────┤
                   │   .data        │  (initialized data)
                   ├────────────────┤
                   │   .bss         │  (uninitialized data)
                   ├────────────────┤
                   │   Heap ↓       │  (grows upward)
                   │                │
                   │   ↓            │
                   │                │
                   │   (unmapped)   │
                   │                │
                   │   ↑            │
                   │                │
                   │   Stack ↑      │  (grows downward)
0x00007FFFFFFFFFFF └────────────────┘
```

### C Library (libc)

**Minimal Custom Implementation:**
- String functions (memcpy, memset, strlen, strcmp, etc.)
- Memory functions (malloc, free, calloc, realloc)
- I/O functions (printf, scanf, fopen, fread, fwrite)
- System call wrappers (one per syscall)

**No Dependencies:**
- No glibc, musl, or other existing libc
- Implemented specifically for m5rOS
- Only functions actually needed

### Shell (m5sh)

**Features:**
- Interactive command-line interface
- Command parsing (handles quotes, escapes)
- Pipeline support (`cmd1 | cmd2 | cmd3`)
- I/O redirection (`>`, `>>`, `<`)
- Background execution (`&`)
- Environment variables
- Command history (in-memory, persisted)
- Tab completion
- Script execution (`.m5` files)

**Built-in Commands:**
- `cd`, `ls`, `cat`, `echo`, `mkdir`, `rm`
- `ps`, `kill`, `clear`, `help`, `exit`

## Filesystem Layout

```
/
├── bin/          Core user-facing binaries (m5sh, utilities)
├── boot/         Bootloader and kernel image
├── dev/          Device nodes (character/block devices)
├── etc/          System configuration files
├── home/         User home directories
│   └── user/     Individual user directory
├── lib/          Shared libraries and syscall wrappers
├── proc/         Process information (virtual FS, future)
├── sys/          Kernel-exported info (virtual FS, future)
└── var/          Variable data (logs, runtime state)
```

## Communication & IPC (Planned)

- Pipes for process communication
- Signals for asynchronous notifications
- Shared memory (future)
- Message queues (future)
- Unix domain sockets (future)

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Boot time (QEMU) | < 2 seconds | Not measured |
| Idle memory footprint | < 32 MB | Not measured |
| Kernel image size | < 2 MB | ~100 KB |
| Shell startup | < 100 ms | Not implemented |
| Context switch | < 10 μs | Not implemented |

## Future Extensibility

The architecture is designed to support:

1. **SMP (Symmetric Multiprocessing)**
   - Per-CPU data structures
   - CPU-local APIC
   - Lock-free algorithms where possible

2. **Networking Stack**
   - TCP/IP implementation
   - Socket API
   - Network drivers (E1000, virtio-net)

3. **GUI System**
   - Custom compositor (Wayland-inspired)
   - Framebuffer driver
   - Input event handling

4. **Loadable Kernel Modules**
   - Dynamic driver loading
   - Symbol resolution
   - Module dependencies

## Development Phases

**Phase 1: Foundation** (Current)
- Build system, kernel skeleton, basic I/O

**Phase 2: Core Kernel**
- Memory management, interrupts, exceptions

**Phase 3: Process Management**
- Scheduler, context switching, fork/exec

**Phase 4: Filesystem**
- m5fs, VFS, ATA driver

**Phase 5: Userspace**
- libc, init, utilities

**Phase 6: Shell**
- m5sh with full feature set

**Phase 7: Stability**
- Testing, debugging, optimization

**Phase 8+: Extensions**
- Networking, GUI, SMP

## References

- Intel® 64 and IA-32 Architectures Software Developer Manuals
- AMD64 Architecture Programmer's Manual
- The Rust Programming Language Book
- OSDev Wiki (https://wiki.osdev.org/)
- UEFI Specification
