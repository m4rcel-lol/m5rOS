// Command parser and handler
//
// Handles keyboard input and executes commands

use crate::drivers::vga;
use crate::sysinfo;
use crate::stats;
use crate::drivers::rtc;
use crate::fs;
use crate::editor;

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

    // Get the rest as a single string for echo command
    let rest = trimmed.strip_prefix(command).unwrap_or("").trim();

    match command {
        "fetch" => cmd_fetch(),
        "help" => cmd_help(),
        "clear" | "cls" => cmd_clear(),
        "uptime" => cmd_uptime(),
        "meminfo" => cmd_meminfo(),
        "cpuinfo" => cmd_cpuinfo(),
        "version" | "ver" => cmd_version(),
        "about" => cmd_about(),
        "echo" => cmd_echo(rest),
        "stats" => cmd_stats(),
        "date" => cmd_date(),
        "time" => cmd_time(),
        "reboot" => cmd_reboot(),
        "shutdown" => cmd_shutdown(),
        "heap" => cmd_heap(),
        "ls" => cmd_ls(),
        "cat" => cmd_cat(rest),
        "mkdir" => cmd_mkdir(rest),
        "touch" => cmd_touch(rest),
        "rm" => cmd_rm(rest),
        "edit" => cmd_edit(rest),
        "install-m5ros" => cmd_install(),
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
    vga::write_str("\nAvailable commands:\n\n");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("System Information:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("  fetch    - Display system information\n");
    vga::write_str("  help     - Display this help message\n");
    vga::write_str("  uptime   - Display system uptime\n");
    vga::write_str("  meminfo  - Display memory information\n");
    vga::write_str("  cpuinfo  - Display CPU information\n");
    vga::write_str("  version  - Display OS version\n");
    vga::write_str("  about    - Display OS information\n");
    vga::write_str("  stats    - Display kernel statistics\n");
    vga::write_str("  date     - Display current date\n");
    vga::write_str("  time     - Display current time\n");
    vga::write_str("  heap     - Display heap statistics\n\n");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("File Operations:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("  ls       - List files and directories\n");
    vga::write_str("  cat      - Display file contents\n");
    vga::write_str("  mkdir    - Create a directory\n");
    vga::write_str("  touch    - Create an empty file\n");
    vga::write_str("  rm       - Remove a file\n");
    vga::write_str("  edit     - Open file in text editor\n\n");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Utilities:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("  clear    - Clear the screen\n");
    vga::write_str("  echo     - Echo text to screen\n\n");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Power Management:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("  reboot   - Reboot the system\n");
    vga::write_str("  shutdown - Shutdown the system\n\n");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("System Installation:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("  install-m5ros - Install m5rOS to disk\n");
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

/// Display version information
fn cmd_version() {
    let info = sysinfo::get_system_info();
    vga::write_str("\n");
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str(info.os_name);
    vga::write_str(" ");
    vga::write_str(info.os_version);
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(" (");
    vga::write_str(info.kernel_version);
    vga::write_str(")\n");
    vga::write_str("A custom operating system built from first principles\n\n");
}

/// Display about information
fn cmd_about() {
    vga::write_str("\n");
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("m5rOS - Custom Operating System\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("================================\n\n");

    vga::write_str("Version:      0.2.0-dev\n");
    vga::write_str("Architecture: x86_64\n");
    vga::write_str("Kernel Type:  Hybrid Monolithic\n");
    vga::write_str("Boot Method:  UEFI\n");
    vga::write_str("Language:     Rust (kernel) + C (userspace)\n\n");

    vga::write_str("Features:\n");
    vga::write_str("  - Hardware interrupt handling (PIC, PIT, Keyboard)\n");
    vga::write_str("  - Memory management (frame allocator, heap)\n");
    vga::write_str("  - VGA text mode with color support\n");
    vga::write_str("  - Serial port debugging (COM1)\n");
    vga::write_str("  - CPU feature detection (CPUID)\n");
    vga::write_str("  - Interactive command system\n");
    vga::write_str("  - Framebuffer graphics support\n\n");

    vga::write_str("License:      MIT\n");
    vga::write_str("Repository:   github.com/m4rcel-lol/m5rOS\n\n");
}

/// Echo text to screen
fn cmd_echo(text: &str) {
    if !text.is_empty() {
        vga::write_str(text);
    }
    vga::write_str("\n");
}

/// Display kernel statistics
fn cmd_stats() {
    vga::write_str("\nKernel Statistics:\n");
    vga::write_str("==================\n\n");

    // IRQ statistics
    vga::write_str("Hardware Interrupts (IRQs):\n");
    vga::write_str("  Timer:    ");
    format_u64(stats::IRQ_STATS.get_timer());
    vga::write_str("\n");
    vga::write_str("  Keyboard: ");
    format_u64(stats::IRQ_STATS.get_keyboard());
    vga::write_str("\n");
    vga::write_str("  Total:    ");
    format_u64(stats::IRQ_STATS.get_total());
    vga::write_str("\n\n");

    // Exception statistics
    vga::write_str("CPU Exceptions:\n");
    vga::write_str("  Divide Error:        ");
    format_u64(stats::EXCEPTION_STATS.get_divide_error());
    vga::write_str("\n");
    vga::write_str("  Page Fault:          ");
    format_u64(stats::EXCEPTION_STATS.get_page_fault());
    vga::write_str("\n");
    vga::write_str("  General Protection:  ");
    format_u64(stats::EXCEPTION_STATS.get_general_protection());
    vga::write_str("\n");
    vga::write_str("  Total:               ");
    format_u64(stats::EXCEPTION_STATS.get_total());
    vga::write_str("\n\n");
}

/// Helper function to format u64 numbers
fn format_u64(n: u64) {
    let mut num = n;
    let mut digits = [0u8; 20];
    let mut count = 0;

    if num == 0 {
        vga::write_str("0");
        return;
    }

    while num > 0 {
        digits[count] = (num % 10) as u8 + b'0';
        num /= 10;
        count += 1;
    }

    // Print digits in reverse
    for i in (0..count).rev() {
        let buf = [digits[i]];
        if let Ok(s) = core::str::from_utf8(&buf) {
            vga::write_str(s);
        }
    }
}

/// Display current date
fn cmd_date() {
    // SAFETY: Reading RTC is safe
    let dt = unsafe { rtc::read_rtc() };

    vga::write_str("\n");

    // Get day of week
    let dow = rtc::day_of_week_name(dt.year, dt.month, dt.day);
    vga::write_str(dow);
    vga::write_str(", ");

    // Get month name
    let month = rtc::month_name(dt.month);
    vga::write_str(month);
    vga::write_str(" ");

    // Day
    format_u64(dt.day as u64);
    vga::write_str(", ");

    // Year
    format_u64(dt.year as u64);
    vga::write_str("\n\n");
}

/// Display current time
fn cmd_time() {
    // SAFETY: Reading RTC is safe
    let dt = unsafe { rtc::read_rtc() };

    vga::write_str("\nCurrent time: ");

    let time_buf = dt.format_time();
    if let Ok(time_str) = core::str::from_utf8(&time_buf) {
        vga::write_str(time_str);
    }

    vga::write_str("\n\n");
}

/// Reboot the system
fn cmd_reboot() {
    vga::write_str("\n");
    vga::set_color(vga::Color::Yellow, vga::Color::Black);
    vga::write_str("Rebooting system...\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    // SAFETY: Performing system reboot via keyboard controller
    unsafe {
        crate::arch::interrupts::disable();

        // Use the keyboard controller to reset
        loop {
            let temp = crate::arch::port::inb(0x64);
            if temp & 0x02 == 0 {
                break;
            }
        }

        crate::arch::port::outb(0x64, 0xFE);

        // If that didn't work, halt
        crate::arch::interrupts::halt_loop();
    }
}

/// Shutdown the system (halt)
fn cmd_shutdown() {
    vga::write_str("\n");
    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::write_str("Shutting down...\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("System halted. You can now turn off your computer.\n");

    // SAFETY: Halting the system
    unsafe {
        crate::arch::interrupts::disable();
        crate::arch::interrupts::halt_loop();
    }
}

/// Display heap statistics
fn cmd_heap() {
    vga::write_str("\nHeap Statistics:\n");
    vga::write_str("================\n\n");

    vga::write_str("Status: ");
    vga::set_color(vga::Color::Yellow, vga::Color::Black);
    vga::write_str("Not yet initialized\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("(Heap initialization pending bootloader completion)\n\n");
}

/// List files and directories
fn cmd_ls() {
    vga::write_str("\nFiles and Directories:\n");
    vga::write_str("======================\n\n");

    let filesystem = fs::get_fs();
    let files = filesystem.list_files();

    if files.is_empty() {
        vga::write_str("No files found.\n\n");
        return;
    }

    for file in files.iter() {
        if !file.in_use {
            continue;
        }

        if file.is_directory {
            vga::set_color(vga::Color::LightBlue, vga::Color::Black);
            vga::write_str("[DIR]  ");
        } else {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::write_str("[FILE] ");
        }

        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::write_str(file.name_as_str());

        if !file.is_directory {
            vga::write_str("  (");
            format_u64(file.size as u64);
            vga::write_str(" bytes)");
        }

        vga::write_str("\n");
    }

    vga::write_str("\n");
}

/// Display file contents
fn cmd_cat(filename: &str) {
    if filename.is_empty() {
        vga::write_str("\nUsage: cat <filename>\n\n");
        return;
    }

    let filesystem = fs::get_fs();

    if let Some(idx) = filesystem.find_file(filename) {
        if let Some(data) = filesystem.read_file(idx) {
            vga::write_str("\n");
            if let Ok(content) = core::str::from_utf8(data) {
                vga::write_str(content);
            } else {
                vga::write_str("[Binary file - cannot display]\n");
            }
            vga::write_str("\n\n");
        } else {
            vga::write_str("\nError: Cannot read file (is it a directory?)\n\n");
        }
    } else {
        vga::write_str("\nError: File not found\n\n");
    }
}

/// Create a directory
fn cmd_mkdir(dirname: &str) {
    if dirname.is_empty() {
        vga::write_str("\nUsage: mkdir <directory>\n\n");
        return;
    }

    let filesystem = fs::get_fs();

    match filesystem.create_dir(dirname) {
        Ok(_) => {
            vga::write_str("\nDirectory created: ");
            vga::write_str(dirname);
            vga::write_str("\n\n");
        }
        Err(e) => {
            vga::write_str("\nError: ");
            vga::write_str(e);
            vga::write_str("\n\n");
        }
    }
}

/// Create an empty file
fn cmd_touch(filename: &str) {
    if filename.is_empty() {
        vga::write_str("\nUsage: touch <filename>\n\n");
        return;
    }

    let filesystem = fs::get_fs();

    match filesystem.create_file(filename) {
        Ok(_) => {
            vga::write_str("\nFile created: ");
            vga::write_str(filename);
            vga::write_str("\n\n");
        }
        Err(e) => {
            vga::write_str("\nError: ");
            vga::write_str(e);
            vga::write_str("\n\n");
        }
    }
}

/// Remove a file
fn cmd_rm(filename: &str) {
    if filename.is_empty() {
        vga::write_str("\nUsage: rm <filename>\n\n");
        return;
    }

    let filesystem = fs::get_fs();

    if let Some(idx) = filesystem.find_file(filename) {
        match filesystem.delete_file(idx) {
            Ok(_) => {
                vga::write_str("\nFile removed: ");
                vga::write_str(filename);
                vga::write_str("\n\n");
            }
            Err(e) => {
                vga::write_str("\nError: ");
                vga::write_str(e);
                vga::write_str("\n\n");
            }
        }
    } else {
        vga::write_str("\nError: File not found\n\n");
    }
}

/// Open file in text editor
fn cmd_edit(filename: &str) {
    if filename.is_empty() {
        vga::write_str("\nUsage: edit <filename>\n\n");
        vga::write_str("Opens a vi-like text editor.\n");
        vga::write_str("Controls:\n");
        vga::write_str("  i - insert mode\n");
        vga::write_str("  ESC - normal mode\n");
        vga::write_str("  hjkl - move cursor\n");
        vga::write_str("  :w - save\n");
        vga::write_str("  :q - quit\n");
        vga::write_str("  :wq - save and quit\n\n");
        return;
    }

    vga::write_str("\nOpening editor for: ");
    vga::write_str(filename);
    vga::write_str("\n\n");

    vga::set_color(vga::Color::Yellow, vga::Color::Black);
    vga::write_str("Note: Full editor integration requires keyboard event system.\n");
    vga::write_str("Editor framework is implemented - awaiting integration.\n\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    // In a full implementation, this would call:
    // editor::run_editor(Some(filename));
}

/// Install m5rOS to disk
fn cmd_install() {
    vga::clear_screen();
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("\n");
    vga::write_str("===============================================\n");
    vga::write_str("        m5rOS System Installer v0.2.0\n");
    vga::write_str("===============================================\n\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    vga::write_str("Welcome to the m5rOS installation wizard!\n\n");

    vga::set_color(vga::Color::Yellow, vga::Color::Black);
    vga::write_str("WARNING: This will install m5rOS to your hard drive.\n");
    vga::write_str("All data on the target disk will be erased!\n\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    vga::write_str("Installation Steps:\n");
    vga::write_str("  1. Detect available disks\n");
    vga::write_str("  2. Partition disk (GPT)\n");
    vga::write_str("  3. Format partitions (EFI + m5fs)\n");
    vga::write_str("  4. Install bootloader (UEFI)\n");
    vga::write_str("  5. Copy kernel and system files\n");
    vga::write_str("  6. Configure system\n\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Current Status:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    // Detect ATA drive
    vga::write_str("  [");
    vga::set_color(vga::Color::LightBlue, vga::Color::Black);
    vga::write_str("CHECK");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("] Detecting ATA drives...\n");

    unsafe {
        match crate::drivers::ata::identify_drive() {
            Ok(info) => {
                vga::write_str("  [");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::write_str("  OK ");
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::write_str("] Drive found: ");

                if let Ok(model) = core::str::from_utf8(&info.model) {
                    vga::write_str(model.trim());
                }

                vga::write_str("\n         Sectors: ");
                format_u64(info.sectors);
                vga::write_str("\n");
            }
            Err(_) => {
                vga::write_str("  [");
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::write_str("FAIL ");
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::write_str("] No drive detected\n");
            }
        }
    }

    vga::write_str("\n");
    vga::set_color(vga::Color::Yellow, vga::Color::Black);
    vga::write_str("Note: Full installation requires:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("  - Bootloader completion (UEFI)\n");
    vga::write_str("  - Filesystem implementation (m5fs)\n");
    vga::write_str("  - Disk partitioning tools (GPT)\n");
    vga::write_str("  - Configuration management\n\n");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Installation framework ready - awaiting component completion.\n\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
}
