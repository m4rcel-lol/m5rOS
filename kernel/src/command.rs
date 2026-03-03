// Command parser and handler
//
// Handles keyboard input and executes commands

use crate::drivers::vga;
use crate::sysinfo;

/// Maximum command buffer size
const CMD_BUF_SIZE: usize = 256;

/// Command buffer state
pub struct CommandBuffer {
    buffer: [u8; CMD_BUF_SIZE],
    length: usize,
}

impl CommandBuffer {
    /// Create a new empty command buffer
    pub const fn new() -> Self {
        CommandBuffer {
            buffer: [0; CMD_BUF_SIZE],
            length: 0,
        }
    }

    /// Add a character to the buffer
    pub fn push(&mut self, c: u8) {
        if self.length < CMD_BUF_SIZE {
            self.buffer[self.length] = c;
            self.length += 1;
        }
    }

    /// Remove the last character from the buffer
    pub fn pop(&mut self) -> Option<u8> {
        if self.length > 0 {
            self.length -= 1;
            Some(self.buffer[self.length])
        } else {
            None
        }
    }

    /// Get the current command as a string
    pub fn as_str(&self) -> &str {
        // SAFETY: We only push ASCII characters
        unsafe { core::str::from_utf8_unchecked(&self.buffer[..self.length]) }
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.length = 0;
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

/// Parse and execute a command
pub fn execute_command(cmd: &str) {
    let trimmed = cmd.trim();

    if trimmed.is_empty() {
        return;
    }

    // Split command into words
    let mut words = trimmed.split_whitespace();
    let command = words.next().unwrap_or("");

    match command {
        "fetch" => cmd_fetch(),
        "help" => cmd_help(),
        "clear" | "cls" => cmd_clear(),
        "uptime" => cmd_uptime(),
        "meminfo" => cmd_meminfo(),
        "cpuinfo" => cmd_cpuinfo(),
        _ => {
            vga::write_str("Unknown command: ");
            vga::write_str(command);
            vga::write_str("\nType 'help' for available commands.\n");
        }
    }
}

/// Display system information (fastfetch-like)
fn cmd_fetch() {
    vga::write_str("\n");

    // ASCII art logo for m5rOS
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("    __  ___      ____  ____  _____\n");
    vga::write_str("   /  |/  /___  / __ \\/ __ \\/ ___/\n");
    vga::write_str("  / /|_/ / __ \\/ /_/ / / / /\\__ \\ \n");
    vga::write_str(" / /  / / /_/ / _, _/ /_/ /___/ / \n");
    vga::write_str("/_/  /_/\\____/_/ |_|\\____//____/  \n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("\n");

    // Get system information
    let info = sysinfo::get_system_info();

    // Display information with labels
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("OS:           ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(info.os_name);
    vga::write_str(" ");
    vga::write_str(info.os_version);
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Kernel:       ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(info.kernel_version);
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Uptime:       ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    let mut uptime_buf = [0u8; 32];
    let uptime_str = sysinfo::format_uptime(info.uptime_seconds, &mut uptime_buf);
    vga::write_str(uptime_str);
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("CPU:          ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(info.cpu_vendor);
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("CPU Features: ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    let mut features_buf = [0u8; 128];
    let features_str = info.cpu_features.as_string(&mut features_buf);
    vga::write_str(features_str);
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Memory:       ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    let mut mem_buf = [0u8; 32];
    let mem_str = sysinfo::format_size(info.memory_allocated, &mut mem_buf);
    vga::write_str(mem_str);
    vga::write_str(" / ");
    let total_str = sysinfo::format_size(info.memory_total, &mut mem_buf);
    vga::write_str(total_str);
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Memory Free:  ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    let free_str = sysinfo::format_size(info.memory_free, &mut mem_buf);
    vga::write_str(free_str);
    vga::write_str("\n");

    // Color bar
    vga::write_str("\n");
    vga::set_color(vga::Color::Black, vga::Color::Black);
    vga::write_str("   ");
    vga::set_color(vga::Color::Red, vga::Color::Red);
    vga::write_str("   ");
    vga::set_color(vga::Color::Green, vga::Color::Green);
    vga::write_str("   ");
    vga::set_color(vga::Color::Yellow, vga::Color::Yellow);
    vga::write_str("   ");
    vga::set_color(vga::Color::Blue, vga::Color::Blue);
    vga::write_str("   ");
    vga::set_color(vga::Color::Magenta, vga::Color::Magenta);
    vga::write_str("   ");
    vga::set_color(vga::Color::Cyan, vga::Color::Cyan);
    vga::write_str("   ");
    vga::set_color(vga::Color::White, vga::Color::White);
    vga::write_str("   ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("\n\n");
}

/// Display help information
fn cmd_help() {
    vga::write_str("\nAvailable commands:\n");
    vga::write_str("  fetch    - Display system information\n");
    vga::write_str("  help     - Display this help message\n");
    vga::write_str("  clear    - Clear the screen\n");
    vga::write_str("  uptime   - Display system uptime\n");
    vga::write_str("  meminfo  - Display memory information\n");
    vga::write_str("  cpuinfo  - Display CPU information\n");
    vga::write_str("\n");
}

/// Clear the screen
fn cmd_clear() {
    vga::clear_screen();
}

/// Display system uptime
fn cmd_uptime() {
    let info = sysinfo::get_system_info();
    let mut uptime_buf = [0u8; 32];
    let uptime_str = sysinfo::format_uptime(info.uptime_seconds, &mut uptime_buf);

    vga::write_str("Uptime: ");
    vga::write_str(uptime_str);
    vga::write_str("\n");
}

/// Display memory information
fn cmd_meminfo() {
    let info = sysinfo::get_system_info();
    let mut mem_buf = [0u8; 32];

    vga::write_str("\nMemory Information:\n");

    vga::write_str("  Total:     ");
    let total_str = sysinfo::format_size(info.memory_total, &mut mem_buf);
    vga::write_str(total_str);
    vga::write_str("\n");

    vga::write_str("  Allocated: ");
    let alloc_str = sysinfo::format_size(info.memory_allocated, &mut mem_buf);
    vga::write_str(alloc_str);
    vga::write_str("\n");

    vga::write_str("  Free:      ");
    let free_str = sysinfo::format_size(info.memory_free, &mut mem_buf);
    vga::write_str(free_str);
    vga::write_str("\n\n");
}

/// Display CPU information
fn cmd_cpuinfo() {
    let info = sysinfo::get_system_info();

    vga::write_str("\nCPU Information:\n");

    vga::write_str("  Vendor:   ");
    vga::write_str(info.cpu_vendor);
    vga::write_str("\n");

    vga::write_str("  Features: ");
    let mut features_buf = [0u8; 128];
    let features_str = info.cpu_features.as_string(&mut features_buf);
    vga::write_str(features_str);
    vga::write_str("\n\n");
}
