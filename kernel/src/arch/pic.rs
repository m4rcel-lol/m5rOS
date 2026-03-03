// 8259A Programmable Interrupt Controller (PIC) driver
//
// The 8259A PIC handles hardware interrupts from devices like the keyboard,
// timer, and disk drives. Modern systems use two PICs in cascade (master and slave).

use crate::arch::port::{inb, outb};
use spin::Mutex;

/// Master PIC command port
const PIC1_COMMAND: u16 = 0x20;
/// Master PIC data port
const PIC1_DATA: u16 = 0x21;
/// Slave PIC command port
const PIC2_COMMAND: u16 = 0xA0;
/// Slave PIC data port
const PIC2_DATA: u16 = 0xA1;

/// End of Interrupt command
const PIC_EOI: u8 = 0x20;

/// ICW1: Initialize command
const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;

/// ICW4: 8086 mode
const ICW4_8086: u8 = 0x01;

/// PIC interrupt offset
/// Hardware interrupts (IRQs) are remapped to interrupt vectors 32-47
/// to avoid conflicts with CPU exceptions (0-31)
pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

/// Programmable Interrupt Controller manager
pub struct Pic {
    offset1: u8,
    offset2: u8,
}

impl Pic {
    /// Create a new PIC configuration
    const fn new(offset1: u8, offset2: u8) -> Self {
        Pic { offset1, offset2 }
    }

    /// Initialize both PICs
    ///
    /// # Safety
    /// Must only be called once during kernel initialization
    pub unsafe fn init(&self) {
        // Save current masks
        let mask1 = inb(PIC1_DATA);
        let mask2 = inb(PIC2_DATA);

        // Start initialization sequence
        outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();

        // Set vector offsets
        outb(PIC1_DATA, self.offset1);
        io_wait();
        outb(PIC2_DATA, self.offset2);
        io_wait();

        // Configure cascading (slave PIC at IRQ2)
        outb(PIC1_DATA, 4); // Tell master PIC that slave is at IRQ2
        io_wait();
        outb(PIC2_DATA, 2); // Tell slave PIC its cascade identity
        io_wait();

        // Set 8086 mode
        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        // Restore saved masks
        outb(PIC1_DATA, mask1);
        outb(PIC2_DATA, mask2);
    }

    /// Disable both PICs by masking all interrupts
    ///
    /// # Safety
    /// May affect system behavior if interrupts are needed
    pub unsafe fn disable(&self) {
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }

    /// Send End of Interrupt signal
    ///
    /// Must be called at the end of an interrupt handler to signal
    /// that the interrupt has been processed
    ///
    /// # Safety
    /// Must be called exactly once per interrupt
    pub unsafe fn send_eoi(&self, irq: u8) {
        // If IRQ came from slave PIC, send EOI to both PICs
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }
        // Always send EOI to master PIC
        outb(PIC1_COMMAND, PIC_EOI);
    }

    /// Set interrupt mask for a specific IRQ
    ///
    /// # Safety
    /// Modifies hardware state
    pub unsafe fn set_mask(&self, irq: u8) {
        let port = if irq < 8 {
            PIC1_DATA
        } else {
            PIC2_DATA
        };
        let irq = irq % 8;
        let value = inb(port) | (1 << irq);
        outb(port, value);
    }

    /// Clear interrupt mask for a specific IRQ (enable it)
    ///
    /// # Safety
    /// Modifies hardware state
    pub unsafe fn clear_mask(&self, irq: u8) {
        let port = if irq < 8 {
            PIC1_DATA
        } else {
            PIC2_DATA
        };
        let irq = irq % 8;
        let value = inb(port) & !(1 << irq);
        outb(port, value);
    }

    /// Get the interrupt vector for an IRQ number
    pub fn irq_vector(&self, irq: u8) -> u8 {
        if irq < 8 {
            self.offset1 + irq
        } else {
            self.offset2 + (irq - 8)
        }
    }
}

/// Wait for an I/O operation to complete
/// Uses port 0x80 (unused POST diagnostic port) for a short delay
unsafe fn io_wait() {
    outb(0x80, 0);
}

/// Global PIC instance
static PIC: Mutex<Pic> = Mutex::new(Pic::new(PIC1_OFFSET, PIC2_OFFSET));

/// Initialize the PIC
///
/// # Safety
/// Must only be called once during kernel initialization
pub unsafe fn init() {
    PIC.lock().init();

    // Disable all interrupts initially
    PIC.lock().disable();
}

/// Send End of Interrupt signal for the given IRQ
///
/// # Safety
/// Must be called exactly once at the end of an interrupt handler
pub unsafe fn send_eoi(irq: u8) {
    PIC.lock().send_eoi(irq);
}

/// Enable a specific IRQ
///
/// # Safety
/// The interrupt handler for this IRQ must be properly set up
pub unsafe fn enable_irq(irq: u8) {
    PIC.lock().clear_mask(irq);
}

/// Disable a specific IRQ
///
/// # Safety
/// Modifies hardware state
pub unsafe fn disable_irq(irq: u8) {
    PIC.lock().set_mask(irq);
}

/// Common IRQ numbers
#[allow(dead_code)]
pub mod irq {
    /// Programmable Interval Timer
    pub const TIMER: u8 = 0;
    /// Keyboard
    pub const KEYBOARD: u8 = 1;
    /// Cascade for slave PIC
    pub const CASCADE: u8 = 2;
    /// Serial port 2
    pub const COM2: u8 = 3;
    /// Serial port 1
    pub const COM1: u8 = 4;
    /// Parallel port 2
    pub const LPT2: u8 = 5;
    /// Floppy disk
    pub const FLOPPY: u8 = 6;
    /// Parallel port 1 (or spurious)
    pub const LPT1: u8 = 7;
    /// Real-time clock
    pub const RTC: u8 = 8;
    /// Free for peripherals
    pub const FREE1: u8 = 9;
    /// Free for peripherals
    pub const FREE2: u8 = 10;
    /// Free for peripherals
    pub const FREE3: u8 = 11;
    /// PS/2 mouse
    pub const MOUSE: u8 = 12;
    /// Floating point unit
    pub const FPU: u8 = 13;
    /// Primary ATA hard disk
    pub const PRIMARY_ATA: u8 = 14;
    /// Secondary ATA hard disk
    pub const SECONDARY_ATA: u8 = 15;
}
