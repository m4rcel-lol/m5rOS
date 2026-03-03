// Physical frame allocator using bitmap-based allocation
//
// This allocator manages physical memory frames (4 KiB pages) using a bitmap
// where each bit represents whether a frame is free (0) or allocated (1).

use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

/// Size of a physical frame (4 KiB)
pub const FRAME_SIZE: usize = 4096;

/// Maximum number of frames we can manage (supports up to 4GB of RAM)
const MAX_FRAMES: usize = 1024 * 1024; // 1M frames = 4GB

/// Bitmap for tracking frame allocation
/// Each bit represents one 4KB frame
static FRAME_BITMAP: Mutex<[u64; MAX_FRAMES / 64]> = Mutex::new([0; MAX_FRAMES / 64]);

/// Next frame to check when allocating (optimization to avoid scanning from start)
static NEXT_FREE_FRAME: AtomicUsize = AtomicUsize::new(0);

/// Total number of frames available
static TOTAL_FRAMES: AtomicUsize = AtomicUsize::new(0);

/// Number of frames currently allocated
static ALLOCATED_FRAMES: AtomicUsize = AtomicUsize::new(0);

/// Represents a physical frame
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    /// Get the physical address of this frame
    pub fn start_address(&self) -> usize {
        self.number * FRAME_SIZE
    }

    /// Create a frame containing the given physical address
    pub fn containing_address(addr: usize) -> Self {
        Frame {
            number: addr / FRAME_SIZE,
        }
    }
}

/// Initialize the frame allocator
///
/// # Arguments
/// * `memory_start` - Start of usable physical memory
/// * `memory_end` - End of usable physical memory
///
/// # Safety
/// Must only be called once during kernel initialization
/// The memory range must be valid and not currently in use
pub unsafe fn init(memory_start: usize, memory_end: usize) {
    let frame_count = (memory_end - memory_start) / FRAME_SIZE;
    let frame_count = frame_count.min(MAX_FRAMES);

    TOTAL_FRAMES.store(frame_count, Ordering::Relaxed);

    // Mark all frames as free initially
    let mut bitmap = FRAME_BITMAP.lock();
    for i in 0..bitmap.len() {
        bitmap[i] = 0;
    }
}

/// Allocate a single physical frame
///
/// Returns None if no frames are available
pub fn allocate_frame() -> Option<Frame> {
    let mut bitmap = FRAME_BITMAP.lock();
    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let start = NEXT_FREE_FRAME.load(Ordering::Relaxed);

    // Search for a free frame starting from the hint
    for offset in 0..total {
        let frame_num = (start + offset) % total;
        let word_index = frame_num / 64;
        let bit_index = frame_num % 64;

        if word_index >= bitmap.len() {
            break;
        }

        let mask = 1u64 << bit_index;
        if bitmap[word_index] & mask == 0 {
            // Frame is free, allocate it
            bitmap[word_index] |= mask;
            ALLOCATED_FRAMES.fetch_add(1, Ordering::Relaxed);
            NEXT_FREE_FRAME.store((frame_num + 1) % total, Ordering::Relaxed);

            return Some(Frame { number: frame_num });
        }
    }

    None
}

/// Deallocate a physical frame
///
/// # Safety
/// The frame must have been previously allocated and not already freed
/// The frame must not be in use
pub unsafe fn deallocate_frame(frame: Frame) {
    let mut bitmap = FRAME_BITMAP.lock();
    let word_index = frame.number / 64;
    let bit_index = frame.number % 64;

    if word_index >= bitmap.len() {
        return;
    }

    let mask = 1u64 << bit_index;
    if bitmap[word_index] & mask != 0 {
        // Frame was allocated, free it
        bitmap[word_index] &= !mask;
        ALLOCATED_FRAMES.fetch_sub(1, Ordering::Relaxed);

        // Update hint if this frame comes before current hint
        let current_hint = NEXT_FREE_FRAME.load(Ordering::Relaxed);
        if frame.number < current_hint {
            NEXT_FREE_FRAME.store(frame.number, Ordering::Relaxed);
        }
    }
}

/// Get statistics about frame allocation
pub fn stats() -> (usize, usize, usize) {
    let total = TOTAL_FRAMES.load(Ordering::Relaxed);
    let allocated = ALLOCATED_FRAMES.load(Ordering::Relaxed);
    let free = total.saturating_sub(allocated);
    (total, allocated, free)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_allocation() {
        // This would require a test harness to run properly
        // Placeholder for when we implement kernel tests
    }
}
