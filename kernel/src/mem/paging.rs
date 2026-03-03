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

/// Page table mapper for managing virtual memory mappings
pub struct PageTableMapper {
    pml4: &'static mut PageTable,
}

impl PageTableMapper {
    /// Create a new page table mapper
    ///
    /// # Safety
    /// The caller must ensure the PML4 pointer is valid and not aliased
    pub unsafe fn new(pml4_addr: usize) -> Self {
        PageTableMapper {
            pml4: &mut *(pml4_addr as *mut PageTable),
        }
    }

    /// Map a virtual page to a physical frame
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The physical frame is valid and available
    /// - The virtual address is not already mapped
    pub unsafe fn map_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: PageTableFlags,
        frame_allocator: &dyn Fn() -> Option<usize>,
    ) -> Result<(), MapError> {
        assert!(virt.is_aligned(), "Virtual address must be page-aligned");
        assert!(phys.is_aligned(), "Physical address must be page-aligned");

        let p4_index = virt.p4_index();
        let p3_index = virt.p3_index();
        let p2_index = virt.p2_index();
        let p1_index = virt.p1_index();

        // Get or create PDPT (Page Directory Pointer Table)
        let p3 = self.get_or_create_table(p4_index, frame_allocator)?;

        // Get or create PD (Page Directory)
        let p2 = Self::get_or_create_next_table(p3, p3_index, frame_allocator)?;

        // Get or create PT (Page Table)
        let p1 = Self::get_or_create_next_table(p2, p2_index, frame_allocator)?;

        // Map the page
        if !p1[p1_index].is_unused() {
            return Err(MapError::PageAlreadyMapped);
        }

        p1[p1_index].set_frame(phys.0 as usize, flags | PageTableFlags::PRESENT);
        flush_tlb(virt);

        Ok(())
    }

    /// Unmap a virtual page
    ///
    /// # Safety
    /// The caller must ensure the page is safe to unmap
    pub unsafe fn unmap_page(&mut self, virt: VirtAddr) -> Result<PhysAddr, MapError> {
        assert!(virt.is_aligned(), "Virtual address must be page-aligned");

        let p1 = self.walk_page_tables(virt)?;
        let p1_index = virt.p1_index();

        if p1[p1_index].is_unused() {
            return Err(MapError::PageNotMapped);
        }

        let phys = PhysAddr::new(p1[p1_index].frame().unwrap() as u64);
        p1[p1_index].set_unused();
        flush_tlb(virt);

        Ok(phys)
    }

    /// Translate a virtual address to a physical address
    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        let p1 = self.walk_page_tables(virt).ok()?;
        let p1_index = virt.p1_index();

        if let Some(frame) = p1[p1_index].frame() {
            Some(PhysAddr::new((frame as u64) + virt.page_offset()))
        } else {
            None
        }
    }

    /// Walk the page tables to find the PT (Page Table)
    fn walk_page_tables(&self, virt: VirtAddr) -> Result<&mut PageTable, MapError> {
        let p4_index = virt.p4_index();
        let p3_index = virt.p3_index();
        let p2_index = virt.p2_index();

        if self.pml4[p4_index].is_unused() {
            return Err(MapError::PageNotMapped);
        }

        let p3 = unsafe {
            &mut *(self.pml4[p4_index].frame().unwrap() as *mut PageTable)
        };

        if p3[p3_index].is_unused() {
            return Err(MapError::PageNotMapped);
        }

        let p2 = unsafe {
            &mut *(p3[p3_index].frame().unwrap() as *mut PageTable)
        };

        if p2[p2_index].is_unused() {
            return Err(MapError::PageNotMapped);
        }

        let p1 = unsafe {
            &mut *(p2[p2_index].frame().unwrap() as *mut PageTable)
        };

        Ok(p1)
    }

    /// Get or create a next level page table
    fn get_or_create_next_table(
        table: &mut PageTable,
        index: usize,
        frame_allocator: &dyn Fn() -> Option<usize>,
    ) -> Result<&'static mut PageTable, MapError> {
        if table[index].is_unused() {
            let frame = frame_allocator().ok_or(MapError::OutOfMemory)?;
            table[index].set_frame(
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );

            // Zero out the new table
            let new_table = unsafe { &mut *(frame as *mut PageTable) };
            new_table.zero();
            Ok(new_table)
        } else {
            Ok(unsafe { &mut *(table[index].frame().unwrap() as *mut PageTable) })
        }
    }

    /// Get or create PDPT from PML4
    fn get_or_create_table(
        &mut self,
        p4_index: usize,
        frame_allocator: &dyn Fn() -> Option<usize>,
    ) -> Result<&'static mut PageTable, MapError> {
        if self.pml4[p4_index].is_unused() {
            let frame = frame_allocator().ok_or(MapError::OutOfMemory)?;
            self.pml4[p4_index].set_frame(
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );

            // Zero out the new table
            let new_table = unsafe { &mut *(frame as *mut PageTable) };
            new_table.zero();
            Ok(new_table)
        } else {
            Ok(unsafe { &mut *(self.pml4[p4_index].frame().unwrap() as *mut PageTable) })
        }
    }
}

/// Errors that can occur during page mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapError {
    /// Page is already mapped
    PageAlreadyMapped,
    /// Page is not mapped
    PageNotMapped,
    /// Out of physical memory
    OutOfMemory,
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
    mapper: &mut PageTableMapper,
    start: PhysAddr,
    size: usize,
    flags: PageTableFlags,
    frame_allocator: &dyn Fn() -> Option<usize>,
) -> Result<(), MapError> {
    let start_page = start.align_down();
    let end_addr = start.0 + size as u64;
    let end_page = PhysAddr::new(end_addr).align_up();

    let mut addr = start_page.0;
    while addr < end_page.0 {
        let virt = VirtAddr::new(addr);
        let phys = PhysAddr::new(addr);
        mapper.map_page(virt, phys, flags, frame_allocator)?;
        addr += PAGE_SIZE as u64;
    }

    Ok(())
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
