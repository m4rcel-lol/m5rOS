// Paging and virtual memory management for x86_64
//
// Implements 4-level page tables (PML4 -> PDPT -> PD -> PT) for virtual memory.
// Each level has 512 entries, providing 48-bit virtual address space.

use core::ops::{Index, IndexMut};
use bitflags::bitflags;

/// Page table entry flags
bitflags! {
    /// Page table entry flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PageTableFlags: u64 {
        /// Page is present in memory
        const PRESENT = 1 << 0;
        /// Page is writable
        const WRITABLE = 1 << 1;
        /// Page is accessible from user mode
        const USER_ACCESSIBLE = 1 << 2;
        /// Write-through caching
        const WRITE_THROUGH = 1 << 3;
        /// Cache disabled
        const NO_CACHE = 1 << 4;
        /// Page has been accessed
        const ACCESSED = 1 << 5;
        /// Page has been written to (dirty)
        const DIRTY = 1 << 6;
        /// Huge page (2MB or 1GB)
        const HUGE_PAGE = 1 << 7;
        /// Page is global (not flushed from TLB on context switch)
        const GLOBAL = 1 << 8;
        /// Disable execution (NX bit)
        const NO_EXECUTE = 1 << 63;
    }
}

/// A page table entry
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    /// Create a new unused entry
    pub const fn new() -> Self {
        PageTableEntry { entry: 0 }
    }

    /// Check if entry is unused
    pub fn is_unused(&self) -> bool {
        self.entry == 0
    }

    /// Set entry to unused
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    /// Get the flags of this entry
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.entry)
    }

    /// Get the physical frame this entry points to
    pub fn frame(&self) -> Option<usize> {
        if self.flags().contains(PageTableFlags::PRESENT) {
            Some((self.entry & 0x000f_ffff_ffff_f000) as usize)
        } else {
            None
        }
    }

    /// Set the entry to map a physical frame
    pub fn set_frame(&mut self, frame: usize, flags: PageTableFlags) {
        assert_eq!(frame & 0xfff, 0, "Frame address must be 4KB aligned");
        self.entry = (frame as u64) | flags.bits();
    }

    /// Set the flags for this entry
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        let frame = self.entry & 0x000f_ffff_ffff_f000;
        self.entry = frame | flags.bits();
    }
}

/// A page table with 512 entries
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Create a new empty page table
    pub const fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); 512],
        }
    }

    /// Clear all entries in the page table
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

/// Virtual address in x86_64 (48-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    /// Create a new virtual address
    pub fn new(addr: u64) -> Self {
        // Sign-extend the 48-bit address
        let sign_extended = if addr & (1 << 47) != 0 {
            addr | 0xffff_0000_0000_0000
        } else {
            addr & 0x0000_ffff_ffff_ffff
        };
        VirtAddr(sign_extended)
    }

    /// Create from a pointer
    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Self::new(ptr as u64)
    }

    /// Convert to a pointer
    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Convert to a mutable pointer
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    /// Get the page offset (bits 0-11)
    pub fn page_offset(self) -> u64 {
        self.0 & 0xfff
    }

    /// Get the PML4 index (bits 39-47)
    pub fn p4_index(self) -> usize {
        ((self.0 >> 39) & 0x1ff) as usize
    }

    /// Get the PDPT index (bits 30-38)
    pub fn p3_index(self) -> usize {
        ((self.0 >> 30) & 0x1ff) as usize
    }

    /// Get the PD index (bits 21-29)
    pub fn p2_index(self) -> usize {
        ((self.0 >> 21) & 0x1ff) as usize
    }

    /// Get the PT index (bits 12-20)
    pub fn p1_index(self) -> usize {
        ((self.0 >> 12) & 0x1ff) as usize
    }

    /// Align down to page boundary
    pub fn align_down(self) -> Self {
        VirtAddr(self.0 & !0xfff)
    }

    /// Align up to page boundary
    pub fn align_up(self) -> Self {
        VirtAddr((self.0 + 0xfff) & !0xfff)
    }

    /// Check if address is page-aligned
    pub fn is_aligned(self) -> bool {
        self.page_offset() == 0
    }
}

/// Physical address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    /// Create a new physical address
    pub fn new(addr: u64) -> Self {
        PhysAddr(addr & 0x000f_ffff_ffff_ffff) // 52-bit physical address
    }

    /// Get the page offset (bits 0-11)
    pub fn page_offset(self) -> u64 {
        self.0 & 0xfff
    }

    /// Align down to page boundary
    pub fn align_down(self) -> Self {
        PhysAddr(self.0 & !0xfff)
    }

    /// Align up to page boundary
    pub fn align_up(self) -> Self {
        PhysAddr((self.0 + 0xfff) & !0xfff)
    }

    /// Check if address is page-aligned
    pub fn is_aligned(self) -> bool {
        self.page_offset() == 0
    }
}

/// Page size (4 KiB)
pub const PAGE_SIZE: usize = 4096;

/// Get the current CR3 register value (physical address of PML4)
///
/// # Safety
/// Reading CR3 is safe
pub unsafe fn get_cr3() -> u64 {
    let value: u64;
    core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

/// Set the CR3 register (load a new page table)
///
/// # Safety
/// The caller must ensure the page table is valid and properly set up
pub unsafe fn set_cr3(pml4_addr: u64) {
    core::arch::asm!("mov cr3, {}", in(reg) pml4_addr, options(nostack, preserves_flags));
}

/// Flush a single TLB entry
///
/// # Safety
/// Flushing TLB entries is safe but may impact performance
pub unsafe fn flush_tlb(addr: VirtAddr) {
    core::arch::asm!("invlpg [{}]", in(reg) addr.0, options(nostack, preserves_flags));
}

/// Flush all TLB entries by reloading CR3
///
/// # Safety
/// This is safe but expensive
pub unsafe fn flush_tlb_all() {
    let cr3 = get_cr3();
    set_cr3(cr3);
}

/// Identity map a region of physical memory
///
/// Maps virtual addresses 1:1 to physical addresses
///
/// # Safety
/// The caller must ensure:
/// - The page table is valid
/// - The physical memory region is valid
/// - The region does not overlap with existing mappings
pub unsafe fn identity_map_region(
    pml4: &mut PageTable,
    start: PhysAddr,
    size: usize,
    flags: PageTableFlags,
) {
    let start_page = start.align_down().0 as usize;
    let end_page = start.align_up().0 as usize + size;

    for addr in (start_page..end_page).step_by(PAGE_SIZE) {
        let virt = VirtAddr::new(addr as u64);
        let phys = PhysAddr::new(addr as u64);

        // This is a simplified version - full implementation would need
        // to allocate intermediate page tables as needed
        let p4_index = virt.p4_index();

        if pml4[p4_index].is_unused() {
            // Would need to allocate PDPT here
            // For now, this is a stub
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virt_addr_indices() {
        let addr = VirtAddr::new(0x0000_7fff_ffff_ffff);
        assert_eq!(addr.p4_index(), 255);
        assert_eq!(addr.p3_index(), 511);
        assert_eq!(addr.p2_index(), 511);
        assert_eq!(addr.p1_index(), 511);
        assert_eq!(addr.page_offset(), 0xfff);
    }

    #[test]
    fn test_addr_alignment() {
        let addr = VirtAddr::new(0x1234);
        assert!(!addr.is_aligned());
        assert_eq!(addr.align_down().0, 0x1000);
        assert_eq!(addr.align_up().0, 0x2000);
    }
}
