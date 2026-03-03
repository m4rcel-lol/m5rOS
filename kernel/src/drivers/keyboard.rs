// PS/2 keyboard driver
//
// Handles PS/2 keyboard input by reading scancodes and translating them
// to key events. Supports US QWERTY layout.

use crate::drivers::vga;
use spin::Mutex;

/// Keyboard data port
const KEYBOARD_DATA: u16 = 0x60;

/// Scan code set 1 (used by most PS/2 keyboards)
/// Maps scancodes to ASCII characters
static SCANCODE_SET1: &[u8] = &[
    0, 27, // Escape
    b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0',
    b'-', b'=', b'\x08', // Backspace
    b'\t', // Tab
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p',
    b'[', b']', b'\n', // Enter
    0, // Control
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l',
    b';', b'\'', b'`',
    0, // Left shift
    b'\\',
    b'z', b'x', b'c', b'v', b'b', b'n', b'm',
    b',', b'.', b'/',
    0, // Right shift
    b'*',
    0, // Alt
    b' ', // Space
];

/// Shifted characters (when shift is held)
static SCANCODE_SET1_SHIFT: &[u8] = &[
    0, 27, // Escape
    b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')',
    b'_', b'+', b'\x08', // Backspace
    b'\t', // Tab
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P',
    b'{', b'}', b'\n', // Enter
    0, // Control
    b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L',
    b':', b'"', b'~',
    0, // Left shift
    b'|',
    b'Z', b'X', b'C', b'V', b'B', b'N', b'M',
    b'<', b'>', b'?',
    0, // Right shift
    b'*',
    0, // Alt
    b' ', // Space
];

/// Keyboard state
struct KeyboardState {
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    caps_lock: bool,
}

impl KeyboardState {
    const fn new() -> Self {
        KeyboardState {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            caps_lock: false,
        }
    }
}

/// Global keyboard state
static KEYBOARD_STATE: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

/// Handle a scancode from the keyboard
///
/// Called by the keyboard interrupt handler
pub fn handle_scancode(scancode: u8) {
    let mut state = KEYBOARD_STATE.lock();

    // Check if this is a key release (bit 7 set)
    let is_release = scancode & 0x80 != 0;
    let scancode = scancode & 0x7F; // Remove release bit

    // Handle special keys
    match scancode {
        0x2A | 0x36 => {
            // Left shift (0x2A) or Right shift (0x36)
            state.shift_pressed = !is_release;
            return;
        }
        0x1D => {
            // Control
            state.ctrl_pressed = !is_release;
            return;
        }
        0x38 => {
            // Alt
            state.alt_pressed = !is_release;
            return;
        }
        0x3A => {
            // Caps Lock (toggle on press only)
            if !is_release {
                state.caps_lock = !state.caps_lock;
            }
            return;
        }
        _ => {}
    }

    // Only process key presses (not releases) for printable characters
    if is_release {
        return;
    }

    // Get the character for this scancode
    let ascii = if let Some(&ch) = if state.shift_pressed {
        SCANCODE_SET1_SHIFT.get(scancode as usize)
    } else {
        SCANCODE_SET1.get(scancode as usize)
    } {
        ch
    } else {
        return; // Unknown scancode
    };

    // Don't print control characters (except newline, backspace, tab)
    if ascii == 0 {
        return;
    }

    // Apply caps lock to letters
    let ascii = if state.caps_lock && ascii.is_ascii_alphabetic() {
        if state.shift_pressed {
            // Shift inverts caps lock for letters
            ascii.to_ascii_lowercase()
        } else {
            ascii.to_ascii_uppercase()
        }
    } else {
        ascii
    };

    // Handle special control combinations
    if state.ctrl_pressed {
        match ascii {
            b'c' | b'C' => {
                // Ctrl+C - could be used for interrupting processes later
                vga::write_str("^C");
                return;
            }
            b'l' | b'L' => {
                // Ctrl+L - clear screen
                vga::clear_screen();
                return;
            }
            _ => {}
        }
    }

    // Echo the character to VGA
    let mut buf = [0u8; 1];
    buf[0] = ascii;
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::write_str(s);
    }
}

/// Initialize the keyboard driver
///
/// # Safety
/// Must only be called once during kernel initialization
pub unsafe fn init() {
    // PS/2 keyboard is initialized by BIOS, we just need to enable IRQ1
    // The PIC and interrupt handler setup is done elsewhere
}
