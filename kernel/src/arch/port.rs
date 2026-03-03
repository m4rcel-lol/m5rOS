// x86_64 port I/O operations

use core::arch::asm;

/// Read a byte from an I/O port
///
/// # Safety
/// This function is unsafe because it performs raw I/O operations
/// Caller must ensure the port is valid and reading from it is safe
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: Caller guarantees port is valid
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

/// Write a byte to an I/O port
///
/// # Safety
/// This function is unsafe because it performs raw I/O operations
/// Caller must ensure the port is valid and writing to it is safe
#[inline]
pub unsafe fn outb(port: u16, value: u8) {
    // SAFETY: Caller guarantees port is valid
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

/// Read a word from an I/O port
///
/// # Safety
/// This function is unsafe because it performs raw I/O operations
#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    // SAFETY: Caller guarantees port is valid
    asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack));
    value
}

/// Write a word to an I/O port
///
/// # Safety
/// This function is unsafe because it performs raw I/O operations
#[inline]
pub unsafe fn outw(port: u16, value: u16) {
    // SAFETY: Caller guarantees port is valid
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack));
}

/// Read a double word (32-bit) from an I/O port
///
/// # Safety
/// This function is unsafe because it performs raw I/O operations
/// Caller must ensure the port is valid and reading from it is safe
#[inline]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    // SAFETY: Caller guarantees port is valid
    asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack));
    value
}

/// Write a double word (32-bit) to an I/O port
///
/// # Safety
/// This function is unsafe because it performs raw I/O operations
/// Caller must ensure the port is valid and writing to it is safe
#[inline]
pub unsafe fn outl(port: u16, value: u32) {
    // SAFETY: Caller guarantees port is valid
    asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack));
}
