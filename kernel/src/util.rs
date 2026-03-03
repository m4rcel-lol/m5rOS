// Utility functions and helpers
//
// Common utility functions used throughout the kernel

/// Format a number as hexadecimal string
///
/// Writes the hex representation to a buffer and returns the slice
pub fn format_hex_u64(value: u64, buffer: &mut [u8; 16]) -> &str {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

    for i in 0..16 {
        let nibble = ((value >> (60 - i * 4)) & 0xF) as usize;
        buffer[i] = HEX_CHARS[nibble];
    }

    // SAFETY: We just filled the buffer with valid ASCII
    unsafe { core::str::from_utf8_unchecked(&buffer[..16]) }
}

/// Write a hexadecimal u64 to serial output
///
/// Used by panic handler
pub fn write_hex_u64(value: u64) {
    use crate::drivers::serial;

    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

    for i in 0..16 {
        let nibble = ((value >> (60 - i * 4)) & 0xF) as usize;
        let ch = HEX_CHARS[nibble] as char;
        let buf = [ch as u8];
        if let Ok(s) = core::str::from_utf8(&buf) {
            serial::write_str(s);
        }
    }
}

/// Format a 32-bit number as hexadecimal string
pub fn format_hex_u32(value: u32, buffer: &mut [u8; 10]) -> &str {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

    buffer[0] = b'0';
    buffer[1] = b'x';

    for i in 0..8 {
        let nibble = ((value >> (28 - i * 4)) & 0xF) as usize;
        buffer[2 + i] = HEX_CHARS[nibble];
    }

    // SAFETY: We just filled the buffer with valid ASCII
    unsafe { core::str::from_utf8_unchecked(&buffer[..10]) }
}

/// Format a 16-bit number as hexadecimal string
pub fn format_hex_u16(value: u16, buffer: &mut [u8; 6]) -> &str {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

    buffer[0] = b'0';
    buffer[1] = b'x';

    for i in 0..4 {
        let nibble = ((value >> (12 - i * 4)) & 0xF) as usize;
        buffer[2 + i] = HEX_CHARS[nibble];
    }

    // SAFETY: We just filled the buffer with valid ASCII
    unsafe { core::str::from_utf8_unchecked(&buffer[..6]) }
}

/// Format an 8-bit number as hexadecimal string
pub fn format_hex_u8(value: u8, buffer: &mut [u8; 4]) -> &str {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

    buffer[0] = b'0';
    buffer[1] = b'x';
    buffer[2] = HEX_CHARS[(value >> 4) as usize];
    buffer[3] = HEX_CHARS[(value & 0xF) as usize];

    // SAFETY: We just filled the buffer with valid ASCII
    unsafe { core::str::from_utf8_unchecked(&buffer[..4]) }
}

/// Memory copy (like memcpy)
///
/// # Safety
/// - src must be valid for reads of count bytes
/// - dst must be valid for writes of count bytes
/// - The memory regions must not overlap
pub unsafe fn memcpy(dst: *mut u8, src: *const u8, count: usize) {
    core::ptr::copy_nonoverlapping(src, dst, count);
}

/// Memory move (like memmove)
///
/// # Safety
/// - src must be valid for reads of count bytes
/// - dst must be valid for writes of count bytes
pub unsafe fn memmove(dst: *mut u8, src: *const u8, count: usize) {
    core::ptr::copy(src, dst, count);
}

/// Memory set (like memset)
///
/// # Safety
/// - dst must be valid for writes of count bytes
pub unsafe fn memset(dst: *mut u8, value: u8, count: usize) {
    core::ptr::write_bytes(dst, value, count);
}

/// Memory compare (like memcmp)
///
/// # Safety
/// - a must be valid for reads of count bytes
/// - b must be valid for reads of count bytes
pub unsafe fn memcmp(a: *const u8, b: *const u8, count: usize) -> i32 {
    for i in 0..count {
        let byte_a = *a.add(i);
        let byte_b = *b.add(i);
        if byte_a != byte_b {
            return byte_a as i32 - byte_b as i32;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_formatting() {
        let mut buf = [0u8; 18];
        assert_eq!(format_hex_u64(0xDEADBEEF, &mut buf), "0x00000000DEADBEEF");

        let mut buf = [0u8; 10];
        assert_eq!(format_hex_u32(0xCAFEBABE, &mut buf), "0xCAFEBABE");

        let mut buf = [0u8; 6];
        assert_eq!(format_hex_u16(0xABCD, &mut buf), "0xABCD");

        let mut buf = [0u8; 4];
        assert_eq!(format_hex_u8(0xFF, &mut buf), "0xFF");
    }
}
