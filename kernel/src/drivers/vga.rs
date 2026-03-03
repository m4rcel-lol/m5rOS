// VGA text mode driver (80x25)
//
// Provides a simple interface for writing colored text to the VGA buffer
// at physical address 0xB8000 in text mode 3 (80x25, 16 colors)

use core::fmt;
use spin::Mutex;

/// VGA buffer physical address
const VGA_BUFFER: usize = 0xB8000;

/// VGA text mode dimensions
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// VGA color codes
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A color code combining foreground and background colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Create a new color code
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character with color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// VGA buffer as a 2D array of screen characters
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// VGA writer
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Create a new VGA writer
    ///
    /// # Safety
    /// Must only be called once to create a single global writer
    /// The VGA buffer address must be valid
    unsafe fn new() -> Self {
        Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            // SAFETY: VGA buffer at 0xB8000 is a valid memory-mapped region
            buffer: &mut *(VGA_BUFFER as *mut Buffer),
        }
    }

    /// Write a single byte to the VGA buffer
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.column_position += 1;
            }
        }
    }

    /// Write a string to the VGA buffer
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not printable, print a box character instead
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Move to a new line, scrolling if necessary
    fn new_line(&mut self) {
        // Scroll all lines up by one using unsafe copy for better performance
        // SAFETY: The source and destination don't overlap, and both are valid memory
        unsafe {
            let src = self.buffer.chars[1].as_ptr();
            let dst = self.buffer.chars[0].as_mut_ptr();
            let count = BUFFER_WIDTH * (BUFFER_HEIGHT - 1);
            core::ptr::copy(src, dst, count);
        }
        // Clear the last line
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clear a row
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col] = blank;
        }
    }

    /// Clear the entire screen
    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
    }

    /// Set foreground and background colors
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }
}

// Implement fmt::Write to allow using write! and writeln! macros
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Global VGA writer protected by a mutex
static WRITER: Mutex<Option<Writer>> = Mutex::new(None);

/// Initialize the VGA driver
///
/// # Safety
/// Must only be called once during kernel initialization
pub unsafe fn init() {
    let writer = Writer::new();
    *WRITER.lock() = Some(writer);
}

/// Write a string to the VGA buffer
pub fn write_str(s: &str) {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.write_string(s);
    }
}

/// Clear the VGA screen
pub fn clear_screen() {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.clear_screen();
    }
}

/// Set VGA colors
pub fn set_color(foreground: Color, background: Color) {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.set_color(foreground, background);
    }
}

/// Macro for printing to VGA
#[macro_export]
macro_rules! vga_print {
    ($($arg:tt)*) => {
        $crate::drivers::vga::_print(format_args!($($arg)*))
    };
}

/// Macro for printing to VGA with newline
#[macro_export]
macro_rules! vga_println {
    () => ($crate::vga_print!("\n"));
    ($($arg:tt)*) => ($crate::vga_print!("{}\n", format_args!($($arg)*)));
}

/// Internal print function for macros
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.write_fmt(args).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vga_write() {
        // This would need a test harness to run properly
    }
}
