// m5edit - A vi-like text editor for m5rOS
//
// Implements a minimal text editor with vi-like keybindings

use crate::drivers::vga;
use crate::drivers::keyboard;

/// Editor modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorMode {
    Normal,
    Insert,
    Command,
}

/// Text buffer for editor
pub struct TextBuffer {
    lines: [Line; MAX_LINES],
    line_count: usize,
}

/// Maximum number of lines
const MAX_LINES: usize = 100;
/// Maximum characters per line
const MAX_LINE_LENGTH: usize = 80;

/// Single line of text
#[derive(Clone, Copy)]
struct Line {
    chars: [u8; MAX_LINE_LENGTH],
    length: usize,
}

impl Line {
    const fn new() -> Self {
        Line {
            chars: [0; MAX_LINE_LENGTH],
            length: 0,
        }
    }

    fn push(&mut self, c: u8) -> bool {
        if self.length < MAX_LINE_LENGTH {
            self.chars[self.length] = c;
            self.length += 1;
            true
        } else {
            false
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.length > 0 {
            self.length -= 1;
            Some(self.chars[self.length])
        } else {
            None
        }
    }

    fn insert(&mut self, pos: usize, c: u8) -> bool {
        if self.length >= MAX_LINE_LENGTH || pos > self.length {
            return false;
        }

        // Shift characters right
        for i in (pos..self.length).rev() {
            self.chars[i + 1] = self.chars[i];
        }

        self.chars[pos] = c;
        self.length += 1;
        true
    }

    fn delete(&mut self, pos: usize) -> bool {
        if pos >= self.length {
            return false;
        }

        // Shift characters left
        for i in pos..self.length - 1 {
            self.chars[i] = self.chars[i + 1];
        }

        self.length -= 1;
        true
    }

    fn as_str(&self) -> &str {
        // SAFETY: We only store ASCII characters
        unsafe { core::str::from_utf8_unchecked(&self.chars[..self.length]) }
    }
}

impl TextBuffer {
    pub const fn new() -> Self {
        TextBuffer {
            lines: [Line::new(); MAX_LINES],
            line_count: 1, // Start with one empty line
        }
    }

    fn insert_char(&mut self, line: usize, col: usize, c: u8) -> bool {
        if line >= self.line_count {
            return false;
        }
        self.lines[line].insert(col, c)
    }

    fn delete_char(&mut self, line: usize, col: usize) -> bool {
        if line >= self.line_count {
            return false;
        }
        self.lines[line].delete(col)
    }

    fn insert_newline(&mut self, line: usize, col: usize) -> bool {
        if self.line_count >= MAX_LINES {
            return false;
        }

        // Split current line at cursor position
        let mut new_line = Line::new();

        // Copy characters after cursor to new line
        let current_line = &mut self.lines[line];
        for i in col..current_line.length {
            new_line.push(current_line.chars[i]);
        }
        current_line.length = col;

        // Shift lines down
        for i in (line + 1..self.line_count).rev() {
            self.lines[i + 1] = self.lines[i];
        }

        // Insert new line
        self.lines[line + 1] = new_line;
        self.line_count += 1;

        true
    }

    fn delete_line(&mut self, line: usize) -> bool {
        if line >= self.line_count || self.line_count == 1 {
            return false;
        }

        // Shift lines up
        for i in line..self.line_count - 1 {
            self.lines[i] = self.lines[i + 1];
        }

        self.line_count -= 1;
        self.lines[self.line_count] = Line::new();

        true
    }
}

/// Text editor state
pub struct Editor {
    buffer: TextBuffer,
    cursor_line: usize,
    cursor_col: usize,
    mode: EditorMode,
    command_buf: [u8; 64],
    command_len: usize,
    filename: [u8; 64],
    filename_len: usize,
    modified: bool,
    status_message: [u8; 80],
    status_len: usize,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            buffer: TextBuffer::new(),
            cursor_line: 0,
            cursor_col: 0,
            mode: EditorMode::Normal,
            command_buf: [0; 64],
            command_len: 0,
            filename: [0; 64],
            filename_len: 0,
            modified: false,
            status_message: [0; 80],
            status_len: 0,
        }
    }

    /// Open a file (placeholder - needs filesystem)
    pub fn open(&mut self, filename: &str) {
        // Store filename
        self.filename_len = 0;
        for &b in filename.as_bytes() {
            if self.filename_len < 64 {
                self.filename[self.filename_len] = b;
                self.filename_len += 1;
            }
        }

        self.set_status("File opened (filesystem not yet implemented)");
    }

    /// Save file (placeholder - needs filesystem)
    pub fn save(&mut self) {
        if self.filename_len == 0 {
            self.set_status("No filename specified");
            return;
        }

        // TODO: Implement actual file saving when filesystem is ready
        self.modified = false;
        self.set_status("File saved (placeholder - filesystem pending)");
    }

    /// Set status message
    fn set_status(&mut self, msg: &str) {
        self.status_len = 0;
        for &b in msg.as_bytes() {
            if self.status_len < 80 {
                self.status_message[self.status_len] = b;
                self.status_len += 1;
            }
        }
    }

    /// Draw the editor screen
    pub fn draw(&self) {
        vga::clear_screen();

        // Draw buffer content
        for i in 0..self.buffer.line_count.min(23) {
            vga::write_str(self.buffer.lines[i].as_str());
            vga::write_str("\n");
        }

        // Draw status line (line 24)
        vga::set_color(vga::Color::Black, vga::Color::LightGray);
        vga::write_str(" ");

        match self.mode {
            EditorMode::Normal => vga::write_str("-- NORMAL --"),
            EditorMode::Insert => vga::write_str("-- INSERT --"),
            EditorMode::Command => vga::write_str("-- COMMAND --"),
        }

        if self.modified {
            vga::write_str(" [+]");
        }

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::write_str("\n");

        // Draw command line or status (line 25)
        if self.mode == EditorMode::Command {
            vga::write_str(":");
            if let Ok(cmd) = core::str::from_utf8(&self.command_buf[..self.command_len]) {
                vga::write_str(cmd);
            }
        } else if self.status_len > 0 {
            if let Ok(status) = core::str::from_utf8(&self.status_message[..self.status_len]) {
                vga::write_str(status);
            }
        }
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: u8) -> bool {
        match self.mode {
            EditorMode::Normal => self.handle_normal_key(key),
            EditorMode::Insert => self.handle_insert_key(key),
            EditorMode::Command => self.handle_command_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: u8) -> bool {
        match key {
            b'i' => {
                // Enter insert mode
                self.mode = EditorMode::Insert;
                self.set_status("");
            }
            b'a' => {
                // Enter insert mode after cursor
                self.cursor_col += 1;
                self.mode = EditorMode::Insert;
                self.set_status("");
            }
            b':' => {
                // Enter command mode
                self.mode = EditorMode::Command;
                self.command_len = 0;
            }
            b'h' => {
                // Move left
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            b'l' => {
                // Move right
                let line_len = self.buffer.lines[self.cursor_line].length;
                if self.cursor_col < line_len {
                    self.cursor_col += 1;
                }
            }
            b'j' => {
                // Move down
                if self.cursor_line < self.buffer.line_count - 1 {
                    self.cursor_line += 1;
                    let line_len = self.buffer.lines[self.cursor_line].length;
                    if self.cursor_col > line_len {
                        self.cursor_col = line_len;
                    }
                }
            }
            b'k' => {
                // Move up
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    let line_len = self.buffer.lines[self.cursor_line].length;
                    if self.cursor_col > line_len {
                        self.cursor_col = line_len;
                    }
                }
            }
            b'x' => {
                // Delete character under cursor
                self.buffer.delete_char(self.cursor_line, self.cursor_col);
                self.modified = true;
            }
            b'd' => {
                // Delete line (simplified - normally would be 'dd')
                self.buffer.delete_line(self.cursor_line);
                self.modified = true;
            }
            b'q' => {
                // Quit
                return true; // Exit editor
            }
            _ => {}
        }
        false
    }

    fn handle_insert_key(&mut self, key: u8) -> bool {
        match key {
            27 => {
                // ESC - return to normal mode
                self.mode = EditorMode::Normal;
            }
            b'\n' => {
                // Enter - insert newline
                self.buffer.insert_newline(self.cursor_line, self.cursor_col);
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.modified = true;
            }
            8 | 127 => {
                // Backspace
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                    self.buffer.delete_char(self.cursor_line, self.cursor_col);
                    self.modified = true;
                }
            }
            c if c >= 32 && c < 127 => {
                // Printable character
                if self.buffer.insert_char(self.cursor_line, self.cursor_col, c) {
                    self.cursor_col += 1;
                    self.modified = true;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_command_key(&mut self, key: u8) -> bool {
        match key {
            27 => {
                // ESC - cancel command
                self.mode = EditorMode::Normal;
                self.command_len = 0;
            }
            b'\n' => {
                // Execute command
                let should_quit = self.execute_command();
                self.mode = EditorMode::Normal;
                self.command_len = 0;
                return should_quit;
            }
            8 | 127 => {
                // Backspace
                if self.command_len > 0 {
                    self.command_len -= 1;
                }
            }
            c if c >= 32 && c < 127 && self.command_len < 64 => {
                // Add to command buffer
                self.command_buf[self.command_len] = c;
                self.command_len += 1;
            }
            _ => {}
        }
        false
    }

    fn execute_command(&mut self) -> bool {
        if self.command_len == 0 {
            return false;
        }

        let cmd = &self.command_buf[..self.command_len];

        // Parse command
        match cmd {
            [b'q'] => {
                // Quit
                if self.modified {
                    self.set_status("File modified! Use :q! to force quit or :wq to save and quit");
                    return false;
                }
                return true;
            }
            [b'q', b'!'] => {
                // Force quit
                return true;
            }
            [b'w'] => {
                // Save
                self.save();
            }
            [b'w', b'q'] => {
                // Save and quit
                self.save();
                return true;
            }
            _ => {
                self.set_status("Unknown command");
            }
        }

        false
    }
}

/// Run the editor
pub fn run_editor(filename: Option<&str>) {
    let mut editor = Editor::new();

    if let Some(name) = filename {
        editor.open(name);
    }

    editor.set_status("m5edit - Type :q to quit, i for insert mode");
    editor.draw();

    // Note: In a real implementation, this would integrate with the keyboard driver
    // to receive input. For now, this is a framework that can be called from the
    // command system.
}
