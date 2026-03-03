// String formatting utilities without alloc
//
// Provides simple string formatting for kernel output

/// A simple string builder that writes to a fixed buffer
pub struct StringWriter<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> StringWriter<'a> {
    /// Create a new string writer
    pub fn new(buffer: &'a mut [u8]) -> Self {
        StringWriter {
            buffer,
            position: 0,
        }
    }

    /// Write a string slice
    pub fn write_str(&mut self, s: &str) -> Result<(), ()> {
        let bytes = s.as_bytes();
        if self.position + bytes.len() > self.buffer.len() {
            return Err(());
        }

        self.buffer[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.position += bytes.len();
        Ok(())
    }

    /// Write a single character
    pub fn write_char(&mut self, c: char) -> Result<(), ()> {
        if c.is_ascii() {
            if self.position >= self.buffer.len() {
                return Err(());
            }
            self.buffer[self.position] = c as u8;
            self.position += 1;
            Ok(())
        } else {
            // For non-ASCII, encode as UTF-8
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            self.write_str(s)
        }
    }

    /// Get the written content as a string slice
    pub fn as_str(&self) -> &str {
        // SAFETY: We only write valid UTF-8
        unsafe { core::str::from_utf8_unchecked(&self.buffer[..self.position]) }
    }

    /// Get the number of bytes written
    pub fn len(&self) -> usize {
        self.position
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.position == 0
    }
}

/// Format a decimal number into a buffer
pub fn format_decimal(value: u64, buffer: &mut [u8]) -> &str {
    if value == 0 {
        buffer[0] = b'0';
        // SAFETY: We just wrote ASCII '0'
        return unsafe { core::str::from_utf8_unchecked(&buffer[..1]) };
    }

    let mut num = value;
    let mut pos = 0;
    let mut temp = [0u8; 20]; // u64 max is 20 digits

    while num > 0 {
        temp[pos] = b'0' + (num % 10) as u8;
        num /= 10;
        pos += 1;
    }

    // Reverse into output buffer
    for i in 0..pos {
        buffer[i] = temp[pos - 1 - i];
    }

    // SAFETY: We just wrote ASCII digits
    unsafe { core::str::from_utf8_unchecked(&buffer[..pos]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_writer() {
        let mut buf = [0u8; 64];
        let mut writer = StringWriter::new(&mut buf);
        writer.write_str("Hello").unwrap();
        writer.write_char(' ').unwrap();
        writer.write_str("World").unwrap();
        assert_eq!(writer.as_str(), "Hello World");
    }

    #[test]
    fn test_format_decimal() {
        let mut buf = [0u8; 20];
        assert_eq!(format_decimal(0, &mut buf), "0");
        assert_eq!(format_decimal(42, &mut buf), "42");
        assert_eq!(format_decimal(1234567890, &mut buf), "1234567890");
    }
}
