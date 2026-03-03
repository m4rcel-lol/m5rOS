// Global Descriptor Table (GDT)
//
// The GDT defines memory segments for x86_64 protected mode.
// While x86_64 uses paging for memory protection, the GDT is still required
// for switching between kernel and user mode, and for the TSS.

use core::arch::asm;

/// GDT Entry
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    /// Create a null GDT entry
    const fn null() -> Self {
        GdtEntry {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }

    /// Create a code segment entry
    const fn code_segment() -> Self {
        GdtEntry {
            limit_low: 0xFFFF,
            base_low: 0,
            base_middle: 0,
            access: 0x9A, // Present, Ring 0, Code, Executable, Readable
            granularity: 0xAF, // 64-bit, 4KB granularity
            base_high: 0,
        }
    }

    /// Create a data segment entry
    const fn data_segment() -> Self {
        GdtEntry {
            limit_low: 0xFFFF,
            base_low: 0,
            base_middle: 0,
            access: 0x92, // Present, Ring 0, Data, Writable
            granularity: 0xCF, // 32-bit, 4KB granularity
            base_high: 0,
        }
    }

    /// Create a user code segment entry (Ring 3)
    const fn user_code_segment() -> Self {
        GdtEntry {
            limit_low: 0xFFFF,
            base_low: 0,
            base_middle: 0,
            access: 0xFA, // Present, Ring 3, Code, Executable, Readable
            granularity: 0xAF, // 64-bit, 4KB granularity
            base_high: 0,
        }
    }

    /// Create a user data segment entry (Ring 3)
    const fn user_data_segment() -> Self {
        GdtEntry {
            limit_low: 0xFFFF,
            base_low: 0,
            base_middle: 0,
            access: 0xF2, // Present, Ring 3, Data, Writable
            granularity: 0xCF, // 32-bit, 4KB granularity
            base_high: 0,
        }
    }
}

/// GDT Pointer structure for LGDT instruction
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

/// The Global Descriptor Table
#[repr(C, align(16))]
struct Gdt {
    entries: [GdtEntry; 5],
}

impl Gdt {
    /// Create a new GDT with standard segments
    const fn new() -> Self {
        Gdt {
            entries: [
                GdtEntry::null(),           // 0x00: Null segment
                GdtEntry::code_segment(),   // 0x08: Kernel code segment
                GdtEntry::data_segment(),   // 0x10: Kernel data segment
                GdtEntry::user_data_segment(), // 0x18: User data segment
                GdtEntry::user_code_segment(), // 0x20: User code segment
            ],
        }
    }

    /// Get a pointer to this GDT suitable for loading
    fn pointer(&self) -> GdtPointer {
        GdtPointer {
            limit: (core::mem::size_of::<Self>() - 1) as u16,
            base: self as *const _ as u64,
        }
    }

    /// Load this GDT and update segment registers
    ///
    /// # Safety
    /// Must only be called once with a valid GDT
    unsafe fn load(&'static self) {
        let ptr = self.pointer();

        // Load the GDT
        // SAFETY: Caller ensures GDT is valid and will remain valid
        asm!(
            "lgdt [{}]",
            in(reg) &ptr,
            options(readonly, nostack, preserves_flags)
        );

        // Update segment registers
        // SAFETY: We've just loaded a valid GDT with these segments
        asm!(
            // Set data segments (DS, ES, FS, GS, SS) to kernel data segment (0x10)
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",

            // Update code segment with a far return
            // Push kernel code segment selector (0x08)
            "push 0x08",
            // Push return address (next instruction)
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            // Far return loads CS with the pushed segment selector
            "retfq",
            "2:",

            tmp = out(reg) _,
            options(preserves_flags)
        );
    }
}

/// Global GDT instance
static GDT: Gdt = Gdt::new();

/// Initialize the GDT
///
/// # Safety
/// Must only be called once during kernel initialization
pub unsafe fn init() {
    GDT.load();
}

/// Segment selectors
#[allow(dead_code)]
pub mod selector {
    /// Kernel code segment selector (Ring 0)
    pub const KERNEL_CODE: u16 = 0x08;

    /// Kernel data segment selector (Ring 0)
    pub const KERNEL_DATA: u16 = 0x10;

    /// User data segment selector (Ring 3)
    pub const USER_DATA: u16 = 0x18 | 0x03; // RPL = 3

    /// User code segment selector (Ring 3)
    pub const USER_CODE: u16 = 0x20 | 0x03; // RPL = 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdt_size() {
        // Verify GDT structure sizes
        assert_eq!(core::mem::size_of::<GdtEntry>(), 8);
        assert_eq!(core::mem::size_of::<GdtPointer>(), 10);
    }
}
