// Kernel heap allocator
//
// Implements a linked-list allocator for the kernel heap.
// This allocator maintains a linked list of free memory blocks and can properly
// deallocate memory, making it much more efficient than a bump allocator.

use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr::{self, null_mut};
use spin::Mutex;

/// Kernel heap start address (in higher half)
const HEAP_START: usize = 0xFFFFFFFF_C0000000;

/// Kernel heap size (16 MB)
const HEAP_SIZE: usize = 16 * 1024 * 1024;

/// Minimum allocation size (includes header)
const MIN_ALLOC_SIZE: usize = mem::size_of::<Node>();

/// Free list node
struct Node {
    size: usize,
    next: Option<&'static mut Node>,
}

impl Node {
    const fn new(size: usize) -> Self {
        Node { size, next: None }
    }
}

/// Linked-list allocator
pub struct LinkedListAllocator {
    head: Option<&'static mut Node>,
}

impl LinkedListAllocator {
    /// Create a new linked-list allocator
    pub const fn new() -> Self {
        LinkedListAllocator { head: None }
    }

    /// Initialize the allocator with a heap region
    ///
    /// # Safety
    /// The heap region must be properly initialized and mapped
    /// This must only be called once
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    /// Add a free region to the free list
    ///
    /// # Safety
    /// The region must be valid and unused
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // Ensure region is large enough to hold a Node
        assert!(size >= mem::size_of::<Node>());
        assert!(addr % mem::align_of::<Node>() == 0);

        // Create new node
        let mut node = Node::new(size);
        node.next = self.head.take();

        // Write node to memory
        let node_ptr = addr as *mut Node;
        node_ptr.write(node);
        self.head = Some(&mut *node_ptr);
    }

    /// Find a suitable region and remove it from the list
    ///
    /// Returns (region address, region size, allocation size)
    fn find_region(&mut self, size: usize, align: usize) -> Option<(usize, usize, usize)> {
        let mut current: *mut Option<&'static mut Node> = &mut self.head;

        // SAFETY: We control access to the list through &mut self
        unsafe {
            while let Some(node) = (*current).as_mut() {
                // Calculate aligned start address
                let node_addr = node as *const _ as *const u8 as usize;
                let alloc_start = align_up(node_addr, align);
                let alloc_end = alloc_start.checked_add(size)?;

                let region_start = node_addr;
                let region_end = region_start + node.size;

                if alloc_end <= region_end {
                    // Found suitable region - extract data before modifying list
                    let region_size = node.size;

                    // Remove node from list
                    let next = node.next.take();
                    *current = next;

                    return Some((region_start, region_size, size));
                }

                // Move to next node
                current = &mut node.next as *mut Option<&'static mut Node>;
            }
        }

        None
    }

    /// Allocate memory
    ///
    /// # Safety
    /// The heap region must be properly initialized and mapped
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // Ensure minimum size
        let size = layout.size().max(MIN_ALLOC_SIZE);
        let align = layout.align();

        if let Some((region_start, region_size, alloc_size)) = self.find_region(size, align) {
            let alloc_start = align_up(region_start, align);
            let alloc_end = alloc_start + alloc_size;

            // Add excess at beginning back to free list
            let excess_start = region_start;
            let excess_end = alloc_start;
            if excess_end > excess_start {
                let excess_size = excess_end - excess_start;
                if excess_size >= MIN_ALLOC_SIZE {
                    self.add_free_region(excess_start, excess_size);
                }
            }

            // Add excess at end back to free list
            let excess_start = alloc_end;
            let excess_end = region_start + region_size;
            if excess_end > excess_start {
                let excess_size = excess_end - excess_start;
                if excess_size >= MIN_ALLOC_SIZE {
                    self.add_free_region(excess_start, excess_size);
                }
            }

            alloc_start as *mut u8
        } else {
            null_mut()
        }
    }

    /// Deallocate memory
    ///
    /// # Safety
    /// The pointer must have been allocated by this allocator
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(MIN_ALLOC_SIZE);
        self.add_free_region(ptr as usize, size);
    }
}

/// Global allocator instance
static ALLOCATOR: Mutex<LinkedListAllocator> = Mutex::new(LinkedListAllocator::new());

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
    ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
}

/// Get heap statistics
pub fn stats() -> (usize, usize) {
    let allocator = ALLOCATOR.lock();
    let mut free = 0;
    let mut current = &allocator.head;

    while let Some(ref node) = current {
        free += node.size;
        current = &node.next;
    }

    let used = HEAP_SIZE - free;
    (used, free)
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
