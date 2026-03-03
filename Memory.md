# m5rOS Memory Management

## Overview

m5rOS uses a three-tier memory management system:
1. **Physical Frame Allocator** - Manages physical RAM as 4 KiB frames
2. **Virtual Memory Manager** - Provides process isolation via paging
3. **Kernel Heap Allocator** - Dynamic memory allocation in kernel

## Physical Memory

### Memory Map

At boot, the UEFI bootloader provides a memory map describing physical RAM:

```
Physical Memory Layout (Example):

0x0000000000000000  ┌──────────────────────┐
                    │  Real Mode IVT       │  0-640 KB
0x000000000009FC00  ├──────────────────────┤
                    │  EBDA                │
0x00000000000A0000  ├──────────────────────┤
                    │  VGA/BIOS ROM        │  640 KB - 1 MB
0x0000000000100000  ├──────────────────────┤
                    │  Kernel Image        │  1 MB+
                    │  (loaded here)       │
                    ├──────────────────────┤
                    │  Available RAM       │  ← Managed by frame allocator
                    │                      │
                    │  (varies by system)  │
                    │                      │
                    └──────────────────────┘
```

### Frame Allocator

**Implementation**: Bitmap-based allocator

**Frame Size**: 4 KiB (4096 bytes)

**Capacity**: Up to 4 GB (1M frames) in Phase 1

#### Data Structures

```rust
// Each bit represents one frame
static FRAME_BITMAP: Mutex<[u64; MAX_FRAMES / 64]>;

// Optimization: next frame to check
static NEXT_FREE_FRAME: AtomicUsize;

// Statistics
static TOTAL_FRAMES: AtomicUsize;
static ALLOCATED_FRAMES: AtomicUsize;
```

#### Allocation Algorithm

1. Check NEXT_FREE_FRAME hint
2. Scan bitmap from hint position
3. Find first zero bit (free frame)
4. Set bit to 1 (allocated)
5. Update NEXT_FREE_FRAME hint
6. Return Frame struct

**Time Complexity**: O(n) worst case, O(1) average with hint

#### Deallocation

1. Locate frame in bitmap
2. Clear bit to 0 (free)
3. Update hint if appropriate
4. Zero frame contents (security)

#### Memory Zeroing

For security, all frames are zeroed:
- **On allocation**: Prevents information leakage between processes
- **On deallocation**: Clears sensitive data

This is enforced by the allocator, not left to callers.

## Virtual Memory

### Paging Structure

x86_64 uses 4-level paging:

```
Virtual Address (64-bit):
┌────────┬─────┬─────┬─────┬─────┬──────────┐
│ Sign   │ PML4│ PDPT│ PD  │ PT  │ Offset   │
│ Extend │     │     │     │     │          │
└────────┴─────┴─────┴─────┴─────┴──────────┘
  16 bits  9     9     9     9     12 bits
  (unused) bits bits  bits  bits

PML4  = Page Map Level 4 (512 entries)
PDPT  = Page Directory Pointer Table (512 entries)
PD    = Page Directory (512 entries)
PT    = Page Table (512 entries)
Offset = Offset within 4 KiB page (4096 bytes)
```

### Page Table Entry Format

```
63────────────────────────────────────12─11──────9─8─7─6─5─4──3──2──1─0
│      Physical Frame Address          │ Avail │G│S│A│D│PCD│PWT│U│W│P│
└──────────────────────────────────────┴───────┴─┴─┴─┴─┴───┴───┴─┴─┴─┘

P   = Present (1 = in memory)
W   = Writable (1 = read/write, 0 = read-only)
U   = User (1 = user accessible, 0 = kernel only)
PWT = Page Write-Through (cache control)
PCD = Page Cache Disable
A   = Accessed (set by CPU when page is accessed)
D   = Dirty (set by CPU when page is written)
S   = Page Size (1 = large page, 0 = 4 KiB)
G   = Global (not flushed on CR3 reload)
```

### Address Space Layout

#### Kernel Space (Higher Half)

```
0xFFFFFFFF80000000 ┌─────────────────────┐
                   │ Kernel .text        │  ← Code (read-only, executable)
                   ├─────────────────────┤
                   │ Kernel .rodata      │  ← Constants (read-only)
                   ├─────────────────────┤
                   │ Kernel .data        │  ← Initialized data (read-write)
                   ├─────────────────────┤
                   │ Kernel .bss         │  ← Uninitialized data
                   ├─────────────────────┤
                   │ Kernel Heap         │  ← Dynamic allocation
                   │  (grows upward)     │
                   ├─────────────────────┤
                   │ Physical Memory Map │  ← Direct map of physical RAM
                   │  (future)           │
                   ├─────────────────────┤
                   │ MMIO Devices        │  ← Memory-mapped I/O
                   └─────────────────────┘
```

#### User Space (Lower Half)

```
0x0000000000000000 ┌─────────────────────┐
                   │ NULL Page (unmapped)│  ← Catch null pointer dereferences
0x0000000000001000 ├─────────────────────┤
                   │ .text               │  ← Program code
                   ├─────────────────────┤
                   │ .rodata             │  ← Read-only data
                   ├─────────────────────┤
                   │ .data               │  ← Initialized data
                   ├─────────────────────┤
                   │ .bss                │  ← Uninitialized data
                   ├─────────────────────┤
                   │ Heap (grows up)     │  ← malloc/free
                   │        ↓            │
                   │                     │
                   │     (unmapped)      │  ← Guard pages
                   │                     │
                   │        ↑            │
                   │ Stack (grows down)  │  ← Local variables, call frames
0x00007FFFFFFFFFFF └─────────────────────┘
```

### Page Fault Handling

When CPU accesses unmapped or protected page:

1. CPU triggers Page Fault exception (interrupt 14)
2. Error code pushed on stack indicates:
   - Present: page not present vs. protection violation
   - Write: was it a write access?
   - User: did it occur in user mode?
3. CR2 register contains faulting address
4. Kernel page fault handler:
   - Check if address is valid
   - If valid: page in from disk (demand paging)
   - If invalid: terminate process (SIGSEGV)

### Copy-on-Write (COW)

Used for efficient fork():

1. Fork creates new process
2. Page tables copied, but physical pages shared
3. All pages marked read-only in both processes
4. When either process writes:
   - Page fault occurs
   - Allocate new physical frame
   - Copy page contents
   - Map new frame with write permissions
   - Resume execution

This defers copying until actually needed.

## Kernel Heap

### Heap Allocator Design

m5rOS uses a **buddy allocator** for variable-size allocations:

#### Buddy System

- Maintains free lists for power-of-2 sizes (e.g., 16B, 32B, 64B, ..., 4KB)
- Splits larger blocks when needed
- Merges adjacent free blocks (buddies) when freed
- Efficient: O(log n) allocation and deallocation

#### Allocation

```rust
fn allocate(size: usize) -> *mut u8 {
    // 1. Round size up to next power of 2
    // 2. Find free block of that size (or split larger block)
    // 3. Mark as allocated
    // 4. Return pointer
}
```

#### Deallocation

```rust
fn deallocate(ptr: *mut u8) {
    // 1. Find block metadata
    // 2. Mark as free
    // 3. Try to merge with buddy
    // 4. If merged, try to merge next level up
}
```

### Slab Allocator (Future)

For fixed-size objects (e.g., TCB, file descriptors):

- Pre-allocates pages of objects
- No fragmentation
- Very fast allocation/deallocation
- Cache-friendly (objects same size)

## Memory Safety

### Rust Guarantees

- **Ownership**: Prevents use-after-free
- **Borrowing**: Prevents data races
- **Lifetimes**: Ensures references are valid
- **No null**: Option<T> instead of null pointers

### Additional Protections

1. **NX (No Execute)**: Code pages not writable, data pages not executable
2. **ASLR**: Randomize addresses to prevent attacks
3. **Stack Canaries**: Detect stack buffer overflows
4. **Guard Pages**: Catch stack overflows
5. **Bounds Checking**: Validate array access

### Unsafe Blocks

When unsafe is required:

```rust
// SAFETY: Physical address is valid and not aliased
// We have exclusive access during initialization
unsafe {
    let frame = Frame::containing_address(addr);
    // ... operate on frame
}
```

Every unsafe block must have a SAFETY comment explaining:
- Why unsafe is needed
- What invariants are upheld
- Why the operation is safe in this context

## Memory Statistics

### Tracking

```rust
pub struct MemoryStats {
    pub total_frames: usize,
    pub allocated_frames: usize,
    pub free_frames: usize,
    pub kernel_heap_used: usize,
    pub kernel_heap_free: usize,
}
```

### Monitoring

Exposed via `/proc/meminfo` (future):

```
MemTotal:        262144 kB
MemFree:         245632 kB
MemAvailable:    245000 kB
Buffers:          2048 kB
Cached:           4096 kB
SwapTotal:           0 kB
SwapFree:            0 kB
```

## Performance Considerations

### Optimization Strategies

1. **Lazy Allocation**: Don't allocate until actually needed
2. **Bulk Operations**: Allocate multiple frames at once
3. **Per-CPU Caches**: Reduce lock contention
4. **Huge Pages**: Use 2 MB or 1 GB pages where appropriate
5. **Page Reclaim**: Free unused pages under memory pressure

### Benchmarks (Target)

| Operation | Target | Current |
|-----------|--------|---------|
| Frame allocation | < 100 ns | Not measured |
| Frame deallocation | < 100 ns | Not measured |
| Heap allocation (small) | < 200 ns | Not measured |
| Page table walk | < 50 ns | Not measured |
| TLB shootdown | < 1 μs | Not implemented |

## Future Enhancements

1. **Swapping**: Move pages to disk when RAM is low
2. **NUMA Support**: Optimize for non-uniform memory access
3. **Huge Pages**: 2 MB and 1 GB page support
4. **Memory Compression**: Compress unused pages
5. **Page Cache**: Cache file contents in memory
6. **Shared Memory**: Allow processes to share pages
7. **Memory Mapping**: mmap() for files and devices

## Debugging

### Tools

- **Memory dumps**: Display page tables, frame allocator state
- **Leak detection**: Track allocations and ensure all are freed
- **Guard bytes**: Detect overruns and underruns
- **Statistics**: Monitor allocation patterns

### Common Issues

- **Double free**: Freeing same frame twice
- **Use after free**: Accessing frame after freeing
- **Memory leak**: Failing to free allocated frames
- **Fragmentation**: Inability to allocate despite free memory

## References

- Intel® 64 and IA-32 Architectures Software Developer's Manual, Volume 3A: System Programming Guide
- AMD64 Architecture Programmer's Manual, Volume 2: System Programming
- "Understanding the Linux Virtual Memory Manager" by Mel Gorman
- "The Buddy Memory Allocation Algorithm" by Donald Knuth
