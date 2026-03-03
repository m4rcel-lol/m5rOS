// Kernel heap allocator
//
// Implements a simple bump allocator for the kernel heap.
// This is a basic allocator that just increments a pointer for each allocation.
// While not efficient for general use, it's simple and works for early kernel development.
// A more sophisticated allocator (buddy or slab) should replace this in the future.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use spin::Mutex;

/// Kernel heap start address (in higher half)
const HEAP_START: usize = 0xFFFFFFFF_C0000000;

/// Kernel heap size (16 MB)
const HEAP_SIZE: usize = 16 * 1024 * 1024;

/// Simple bump allocator
struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Create a new bump allocator
    const fn new() -> Self {
        BumpAllocator {
            heap_start: HEAP_START,
            heap_end: HEAP_START + HEAP_SIZE,
            next: HEAP_START,
            allocations: 0,
        }
    }

    /// Allocate memory
    ///
    /// # Safety
    /// The heap region must be properly initialized and mapped
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let alloc_start = align_up(self.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return null_mut(),
        };

        if alloc_end > self.heap_end {
            // Out of memory
            return null_mut();
        }

        self.next = alloc_end;
        self.allocations += 1;

        alloc_start as *mut u8
    }

    /// Deallocate memory (no-op for bump allocator)
    ///
    /// Note: Bump allocators don't actually free memory until reset
    /// This is a limitation that should be addressed with a better allocator
    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't support deallocation
        // In a real implementation, we'd use a more sophisticated allocator
        self.allocations -= 1;
    }

    /// Get allocation statistics
    pub fn used(&self) -> usize {
        self.next - self.heap_start
    }

    /// Get free space
    pub fn free(&self) -> usize {
        self.heap_end - self.next
    }

    /// Get total allocations count
    pub fn allocations(&self) -> usize {
        self.allocations
    }
}

/// Global allocator instance
static ALLOCATOR: Mutex<BumpAllocator> = Mutex::new(BumpAllocator::new());

/// Global allocator for Rust's alloc crate
struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATOR.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATOR.lock().dealloc(ptr, layout)
    }
}

#[global_allocator]
static KERNEL_ALLOCATOR: KernelAllocator = KernelAllocator;

/// Align upwards
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Initialize the kernel heap
///
/// # Safety
/// Must only be called once during kernel initialization
/// The heap memory region must be properly mapped
pub unsafe fn init() {
    // In a real implementation, we would:
    // 1. Allocate physical frames for the heap
    // 2. Map them to the heap virtual address range
    // 3. Zero out the memory
    //
    // For now, we assume the bootloader has set up identity mapping
    // and the heap region is accessible (this is a simplification)
}

/// Get heap statistics
pub fn stats() -> (usize, usize, usize) {
    let allocator = ALLOCATOR.lock();
    let used = allocator.used();
    let free = allocator.free();
    let allocations = allocator.allocations();
    (used, free, allocations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
    }
}
