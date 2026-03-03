// Interrupt Descriptor Table (IDT)
//
// The IDT defines handlers for CPU exceptions and hardware interrupts.
// Each entry points to an interrupt handler function.

use core::arch::{asm, naked_asm};
use bitflags::bitflags;

/// IDT Entry (Interrupt Gate)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,    // Offset bits 0..15
    selector: u16,      // Code segment selector
    ist: u8,            // Interrupt Stack Table offset
    type_attr: u8,      // Type and attributes
    offset_mid: u16,    // Offset bits 16..31
    offset_high: u32,   // Offset bits 32..63
    reserved: u32,      // Reserved (must be zero)
}

bitflags! {
    /// IDT entry attributes
    struct IdtFlags: u8 {
        const PRESENT = 1 << 7;
        const DPL_RING0 = 0 << 5;
        const DPL_RING3 = 3 << 5;
        const GATE_INTERRUPT = 0x0E;
        const GATE_TRAP = 0x0F;
    }
}

impl IdtEntry {
    /// Create a null IDT entry
    const fn null() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    /// Create an IDT entry for an interrupt handler
    fn new(handler: unsafe extern "C" fn(), selector: u16) -> Self {
        let handler_addr = handler as u64;

        IdtEntry {
            offset_low: (handler_addr & 0xFFFF) as u16,
            selector,
            ist: 0,
            type_attr: (IdtFlags::PRESENT | IdtFlags::DPL_RING0 | IdtFlags::GATE_INTERRUPT).bits(),
            offset_mid: ((handler_addr >> 16) & 0xFFFF) as u16,
            offset_high: ((handler_addr >> 32) & 0xFFFFFFFF) as u32,
            reserved: 0,
        }
    }
}

/// IDT Pointer structure for LIDT instruction
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

/// The Interrupt Descriptor Table
#[repr(C, align(16))]
struct Idt {
    entries: [IdtEntry; 256],
}

impl Idt {
    /// Create a new IDT with all null entries
    const fn new() -> Self {
        Idt {
            entries: [IdtEntry::null(); 256],
        }
    }

    /// Set an IDT entry
    fn set_handler(&mut self, index: usize, handler: unsafe extern "C" fn()) {
        // Use kernel code segment selector (0x08)
        self.entries[index] = IdtEntry::new(handler, 0x08);
    }

    /// Get a pointer to this IDT suitable for loading
    fn pointer(&self) -> IdtPointer {
        IdtPointer {
            limit: (core::mem::size_of::<Self>() - 1) as u16,
            base: self as *const _ as u64,
        }
    }

    /// Load this IDT
    ///
    /// # Safety
    /// Must only be called once with a valid IDT
    unsafe fn load(&'static self) {
        let ptr = self.pointer();

        // Load the IDT
        // SAFETY: Caller ensures IDT is valid and will remain valid
        asm!(
            "lidt [{}]",
            in(reg) &ptr,
            options(readonly, nostack, preserves_flags)
        );
    }
}

/// Global IDT instance
static mut IDT: Idt = Idt::new();

// Exception handler stubs
// These are simple handlers that just hang for now
// In a real OS, these would save state and call Rust handlers

macro_rules! exception_handler_no_error {
    ($name:ident, $num:expr) => {
        // SAFETY: naked function required to avoid Rust prologue/epilogue
        // which would corrupt the exception stack frame
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            naked_asm!(
                "push 0",           // Push dummy error code
                "push {}",          // Push exception number
                "jmp exception_common",
                const $num,
            );
        }
    };
}

macro_rules! exception_handler_with_error {
    ($name:ident, $num:expr) => {
        // SAFETY: naked function required to avoid Rust prologue/epilogue
        // which would corrupt the exception stack frame
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            naked_asm!(
                // CPU already pushed error code
                "push {}",          // Push exception number
                "jmp exception_common",
                const $num,
            );
        }
    };
}

// Define exception handlers
exception_handler_no_error!(divide_error_handler, 0);
exception_handler_no_error!(debug_handler, 1);
exception_handler_no_error!(nmi_handler, 2);
exception_handler_no_error!(breakpoint_handler, 3);
exception_handler_no_error!(overflow_handler, 4);
exception_handler_no_error!(bound_range_handler, 5);
exception_handler_no_error!(invalid_opcode_handler, 6);
exception_handler_no_error!(device_not_available_handler, 7);
exception_handler_with_error!(double_fault_handler, 8);
exception_handler_no_error!(coprocessor_segment_handler, 9);
exception_handler_with_error!(invalid_tss_handler, 10);
exception_handler_with_error!(segment_not_present_handler, 11);
exception_handler_with_error!(stack_segment_handler, 12);
exception_handler_with_error!(general_protection_handler, 13);
exception_handler_with_error!(page_fault_handler, 14);
exception_handler_no_error!(x87_fpu_handler, 16);
exception_handler_with_error!(alignment_check_handler, 17);
exception_handler_no_error!(machine_check_handler, 18);
exception_handler_no_error!(simd_exception_handler, 19);
exception_handler_no_error!(virtualization_handler, 20);

/// Common exception handler that all exceptions jump to
// SAFETY: naked function required to properly save/restore all registers
// without interference from Rust-generated code
#[unsafe(naked)]
unsafe extern "C" fn exception_common() {
    naked_asm!(
        // Save all registers
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Call the Rust exception handler
        // RDI = exception number (already on stack)
        // RSI = error code (already on stack)
        "mov rdi, [rsp + 15*8]",  // Exception number
        "mov rsi, [rsp + 16*8]",  // Error code
        "call exception_handler_rust",

        // Restore registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",

        // Remove error code and exception number from stack
        "add rsp, 16",

        // Return from interrupt
        "iretq",
    );
}

/// Rust exception handler
#[no_mangle]
extern "C" fn exception_handler_rust(exception_num: u64, _error_code: u64) {
    use crate::drivers::serial;

    serial::write_str("\n!!! EXCEPTION !!!\n");

    // Print exception information
    let exception_name = match exception_num {
        0 => "Division Error",
        1 => "Debug",
        2 => "Non-Maskable Interrupt",
        3 => "Breakpoint",
        4 => "Overflow",
        5 => "Bound Range Exceeded",
        6 => "Invalid Opcode",
        7 => "Device Not Available",
        8 => "Double Fault",
        10 => "Invalid TSS",
        11 => "Segment Not Present",
        12 => "Stack-Segment Fault",
        13 => "General Protection Fault",
        14 => "Page Fault",
        16 => "x87 FPU Error",
        17 => "Alignment Check",
        18 => "Machine Check",
        19 => "SIMD Exception",
        20 => "Virtualization Exception",
        _ => "Unknown Exception",
    };

    serial::write_str("Exception: ");
    serial::write_str(exception_name);
    serial::write_str("\n");

    // For page faults, print CR2 (faulting address)
    if exception_num == 14 {
        // SAFETY: Reading CR2 is safe during a page fault handler
        let cr2: u64 = unsafe {
            let cr2: u64;
            asm!("mov {}, cr2", out(reg) cr2);
            cr2
        };
        serial::write_str("Faulting address: ");
        let mut buf = [0u8; 18];
        let hex_str = crate::util::format_hex_u64(cr2, &mut buf);
        serial::write_str(hex_str);
        serial::write_str("\n");
    }

    // Hang the system
    serial::write_str("System halted\n");
    crate::arch::interrupts::halt_loop();
}

// Hardware interrupt handlers
// These are called by hardware devices (timer, keyboard, etc.)

/// Timer interrupt handler (IRQ0)
// SAFETY: naked function required to avoid Rust prologue/epilogue
#[unsafe(naked)]
unsafe extern "C" fn timer_interrupt_handler() {
    naked_asm!(
        // Save all registers
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Call the Rust timer handler
        "call timer_handler_rust",

        // Restore registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",

        // Return from interrupt
        "iretq",
    );
}

/// Keyboard interrupt handler (IRQ1)
// SAFETY: naked function required to avoid Rust prologue/epilogue
#[unsafe(naked)]
unsafe extern "C" fn keyboard_interrupt_handler() {
    naked_asm!(
        // Save all registers
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Call the Rust keyboard handler
        "call keyboard_handler_rust",

        // Restore registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",

        // Return from interrupt
        "iretq",
    );
}

/// Rust timer interrupt handler
#[no_mangle]
extern "C" fn timer_handler_rust() {
    use crate::arch::{pic, pit};

    // Increment tick counter
    pit::tick();

    // Send EOI to PIC
    // SAFETY: Called once at end of interrupt handler
    unsafe {
        pic::send_eoi(pic::irq::TIMER);
    }
}

/// Rust keyboard interrupt handler
#[no_mangle]
extern "C" fn keyboard_handler_rust() {
    use crate::arch::{pic, port};

    // Read scancode from keyboard
    // SAFETY: Reading from keyboard port is safe
    let scancode = unsafe { port::inb(0x60) };

    // Handle the scancode (will be implemented in keyboard driver)
    crate::drivers::keyboard::handle_scancode(scancode);

    // Send EOI to PIC
    // SAFETY: Called once at end of interrupt handler
    unsafe {
        pic::send_eoi(pic::irq::KEYBOARD);
    }
}

/// Initialize the IDT
///
/// # Safety
/// Must only be called once during kernel initialization
pub unsafe fn init() {
    // Set up exception handlers
    IDT.set_handler(0, divide_error_handler);
    IDT.set_handler(1, debug_handler);
    IDT.set_handler(2, nmi_handler);
    IDT.set_handler(3, breakpoint_handler);
    IDT.set_handler(4, overflow_handler);
    IDT.set_handler(5, bound_range_handler);
    IDT.set_handler(6, invalid_opcode_handler);
    IDT.set_handler(7, device_not_available_handler);
    IDT.set_handler(8, double_fault_handler);
    IDT.set_handler(9, coprocessor_segment_handler);
    IDT.set_handler(10, invalid_tss_handler);
    IDT.set_handler(11, segment_not_present_handler);
    IDT.set_handler(12, stack_segment_handler);
    IDT.set_handler(13, general_protection_handler);
    IDT.set_handler(14, page_fault_handler);
    IDT.set_handler(16, x87_fpu_handler);
    IDT.set_handler(17, alignment_check_handler);
    IDT.set_handler(18, machine_check_handler);
    IDT.set_handler(19, simd_exception_handler);
    IDT.set_handler(20, virtualization_handler);

    // Set up hardware interrupt handlers (IRQs remapped to 32-47)
    IDT.set_handler(32, timer_interrupt_handler);  // IRQ0: Timer
    IDT.set_handler(33, keyboard_interrupt_handler);  // IRQ1: Keyboard

    // Load the IDT
    IDT.load();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idt_size() {
        // Verify IDT structure sizes
        assert_eq!(core::mem::size_of::<IdtEntry>(), 16);
        assert_eq!(core::mem::size_of::<IdtPointer>(), 10);
    }
}
