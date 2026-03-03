// 16550 UART serial driver for debugging and logging

use crate::arch::port::{inb, outb};
use spin::Mutex;

/// COM1 serial port base address
const COM1: u16 = 0x3F8;

/// Serial port writer
pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    /// Initialize the serial port
    ///
    /// # Safety
    /// Must only be called once during kernel initialization
    pub unsafe fn init(base: u16) -> Self {
        // Disable interrupts
        outb(base + 1, 0x00);
        // Enable DLAB to set baud rate
        outb(base + 3, 0x80);
        // Set divisor to 3 (38400 baud)
        outb(base + 0, 0x03);
        outb(base + 1, 0x00);
        // 8 bits, no parity, one stop bit
        outb(base + 3, 0x03);
        // Enable FIFO, clear them, with 14-byte threshold
        outb(base + 2, 0xC7);
        // Enable IRQs, set RTS/DSR
        outb(base + 4, 0x0B);

        SerialPort { base }
    }

    /// Check if transmit FIFO is empty
    fn is_transmit_empty(&self) -> bool {
        // SAFETY: Reading from COM1+5 (line status register) is safe
        unsafe { inb(self.base + 5) & 0x20 != 0 }
    }

    /// Write a byte to the serial port
    pub fn write_byte(&mut self, byte: u8) {
        // Wait for transmit buffer to be empty
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        // SAFETY: Writing to COM1 data port is safe
        unsafe {
            outb(self.base, byte);
        }
    }

    /// Write a string to the serial port
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
}

// Global serial port
static SERIAL: Mutex<Option<SerialPort>> = Mutex::new(None);

/// Initialize the global serial port
///
/// # Safety
/// Must only be called once during kernel initialization
pub unsafe fn init() {
    let serial = SerialPort::init(COM1);
    *SERIAL.lock() = Some(serial);
}

/// Write to the serial port
pub fn write_str(s: &str) {
    if let Some(ref mut serial) = *SERIAL.lock() {
        serial.write_str(s);
    }
}
