// Framebuffer graphics driver
//
// Provides pixel-based graphics output for VESA/UEFI GOP framebuffers

use core::ptr;
use spin::Mutex;

/// Pixel format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGB format (Red, Green, Blue)
    RGB,
    /// BGR format (Blue, Green, Red)
    BGR,
}

/// Framebuffer information
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// Base address of the framebuffer
    pub addr: usize,
    /// Width in pixels
    pub width: usize,
    /// Height in pixels
    pub height: usize,
    /// Bytes per pixel
    pub bytes_per_pixel: usize,
    /// Stride (bytes per line)
    pub stride: usize,
    /// Pixel format
    pub format: PixelFormat,
}

/// Global framebuffer state
static FRAMEBUFFER: Mutex<Option<FramebufferInfo>> = Mutex::new(None);

/// Initialize framebuffer with given parameters
///
/// # Safety
/// Must be called once during kernel initialization
/// The framebuffer address must be valid and accessible
pub unsafe fn init(info: FramebufferInfo) {
    *FRAMEBUFFER.lock() = Some(info);
}

/// Check if framebuffer is available
pub fn is_available() -> bool {
    FRAMEBUFFER.lock().is_some()
}

/// Get framebuffer info
pub fn info() -> Option<FramebufferInfo> {
    *FRAMEBUFFER.lock()
}

/// RGB color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    // Common colors
    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const RED: Color = Color::new(255, 0, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);
    pub const YELLOW: Color = Color::new(255, 255, 0);
    pub const CYAN: Color = Color::new(0, 255, 255);
    pub const MAGENTA: Color = Color::new(255, 0, 255);
    pub const GRAY: Color = Color::new(128, 128, 128);
    pub const LIGHT_GRAY: Color = Color::new(192, 192, 192);
    pub const DARK_GRAY: Color = Color::new(64, 64, 64);
    pub const ORANGE: Color = Color::new(255, 165, 0);
    pub const PURPLE: Color = Color::new(128, 0, 128);
}

/// Draw a pixel at (x, y) with the given color
pub fn draw_pixel(x: usize, y: usize, color: Color) {
    let fb_lock = FRAMEBUFFER.lock();
    if let Some(fb) = *fb_lock {
        if x >= fb.width || y >= fb.height {
            return;
        }

        let pixel_offset = y * fb.stride + x * fb.bytes_per_pixel;
        let pixel_addr = fb.addr + pixel_offset;

        // SAFETY: We checked bounds and framebuffer is properly initialized
        unsafe {
            match fb.format {
                PixelFormat::RGB => {
                    ptr::write_volatile((pixel_addr) as *mut u8, color.r);
                    ptr::write_volatile((pixel_addr + 1) as *mut u8, color.g);
                    ptr::write_volatile((pixel_addr + 2) as *mut u8, color.b);
                }
                PixelFormat::BGR => {
                    ptr::write_volatile((pixel_addr) as *mut u8, color.b);
                    ptr::write_volatile((pixel_addr + 1) as *mut u8, color.g);
                    ptr::write_volatile((pixel_addr + 2) as *mut u8, color.r);
                }
            }
        }
    }
}

/// Fill a rectangle with a color
pub fn fill_rect(x: usize, y: usize, width: usize, height: usize, color: Color) {
    for dy in 0..height {
        for dx in 0..width {
            draw_pixel(x + dx, y + dy, color);
        }
    }
}

/// Clear the entire screen with a color
pub fn clear_screen(color: Color) {
    let fb_lock = FRAMEBUFFER.lock();
    if let Some(fb) = *fb_lock {
        drop(fb_lock); // Release lock before drawing
        fill_rect(0, 0, fb.width, fb.height, color);
    }
}

/// Draw a horizontal line
pub fn draw_hline(x: usize, y: usize, width: usize, color: Color) {
    for i in 0..width {
        draw_pixel(x + i, y, color);
    }
}

/// Draw a vertical line
pub fn draw_vline(x: usize, y: usize, height: usize, color: Color) {
    for i in 0..height {
        draw_pixel(x, y + i, color);
    }
}

/// Draw a rectangle outline
pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: Color) {
    draw_hline(x, y, width, color);
    draw_hline(x, y + height - 1, width, color);
    draw_vline(x, y, height, color);
    draw_vline(x + width - 1, y, height, color);
}

/// Simple 8x8 font for ASCII characters (basic bitmap font)
/// This is a very simple font - each character is 8x8 pixels
/// Each byte represents a row, with bits indicating pixels
const FONT_8X8: [[u8; 8]; 128] = include!("font_8x8.inc");

/// Draw a character at (x, y)
pub fn draw_char(x: usize, y: usize, c: char, fg: Color, bg: Color) {
    let c_idx = (c as usize).min(127);
    let glyph = FONT_8X8[c_idx];

    for (dy, &row) in glyph.iter().enumerate() {
        for dx in 0..8 {
            let pixel = (row >> (7 - dx)) & 1;
            let color = if pixel == 1 { fg } else { bg };
            draw_pixel(x + dx, y + dy, color);
        }
    }
}

/// Draw a string at (x, y)
pub fn draw_string(x: usize, y: usize, s: &str, fg: Color, bg: Color) {
    let mut cx = x;
    for c in s.chars() {
        if c == '\n' {
            // Newlines not supported in this simple implementation
            continue;
        }
        draw_char(cx, y, c, fg, bg);
        cx += 8;
    }
}
