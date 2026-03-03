// Command parser and handler
//
// Handles keyboard input and executes commands

use crate::drivers::vga;
use crate::sysinfo;
use crate::stats;
use crate::drivers::rtc;
use crate::fs;
use crate::editor;
use crate::net;

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
        "ifconfig" => cmd_ifconfig(rest),
        "ping" => cmd_ping(rest),
        "arp" => cmd_arp(),
        "netinit" => cmd_netinit(rest),
        "lspci" => cmd_lspci(),
        "gpuinfo" => cmd_gpuinfo(),
        "wifiinfo" => cmd_wifiinfo(),
        "wifiscan" => cmd_wifiscan(),
        "btinfo" => cmd_btinfo(),
        "btscan" => cmd_btscan(),
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
    vga::write_str("  install-m5ros - Install m5rOS to disk\n\n");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Hardware:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("  lspci    - List all PCI devices\n");
    vga::write_str("  gpuinfo  - Display integrated graphics info\n\n");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Network:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("  netinit  - Initialize network card\n");
    vga::write_str("  ifconfig - Configure network interface\n");
    vga::write_str("  ping     - Send ICMP echo request\n");
    vga::write_str("  arp      - Display ARP cache\n\n");

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Wireless:\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("  wifiinfo - Display Wi-Fi adapter information\n");
    vga::write_str("  wifiscan - Scan for Wi-Fi networks\n");
    vga::write_str("  btinfo   - Display Bluetooth adapter info\n");
    vga::write_str("  btscan   - Scan for Bluetooth devices\n");
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

/// Initialize network card
fn cmd_netinit(args: &str) {
    vga::write_str("\n");
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Network Initialization\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("======================\n\n");

    // Parse I/O base from args, default to 0xC000 (QEMU E1000)
    let io_base: u16 = if !args.is_empty() {
        // Try to parse hex or decimal
        if args.starts_with("0x") {
            u16::from_str_radix(&args[2..], 16).unwrap_or(0xC000)
        } else {
            args.parse().unwrap_or(0xC000)
        }
    } else {
        0xC000 // Default QEMU E1000 I/O base
    };

    vga::write_str("Initializing E1000 at I/O base 0x");
    crate::util::write_hex_u16(io_base);
    vga::write_str("...\n");

    unsafe {
        match crate::drivers::e1000::init(io_base) {
            Ok(_) => {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::write_str("SUCCESS: ");
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::write_str("Network card initialized\n");

                let mac = crate::drivers::e1000::get_mac_address();
                vga::write_str("MAC Address: ");
                for i in 0..6 {
                    crate::util::write_hex_u8(mac[i]);
                    if i < 5 {
                        vga::write_str(":");
                    }
                }
                vga::write_str("\n");

                let link_up = crate::drivers::e1000::is_link_up();
                vga::write_str("Link Status: ");
                if link_up {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::write_str("UP\n");
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::write_str("DOWN\n");
                }
                vga::set_color(vga::Color::White, vga::Color::Black);
            }
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::write_str("ERROR: ");
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::write_str(e);
                vga::write_str("\n");
            }
        }
    }

    vga::write_str("\n");
}

/// Configure network interface
fn cmd_ifconfig(args: &str) {
    use crate::drivers::e1000;

    vga::write_str("\n");

    if args.is_empty() {
        // Display current configuration
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::write_str("Network Interface Configuration\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::write_str("================================\n\n");

        if !e1000::is_initialized() {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::write_str("Network card not initialized. Use 'netinit' first.\n\n");
            vga::set_color(vga::Color::White, vga::Color::Black);
            return;
        }

        let mac = e1000::get_mac_address();
        vga::write_str("eth0:\n");
        vga::write_str("  MAC Address: ");
        for i in 0..6 {
            crate::util::write_hex_u8(mac[i]);
            if i < 5 {
                vga::write_str(":");
            }
        }
        vga::write_str("\n");

        let config = net::get_config();
        if config.configured {
            vga::write_str("  IP Address:  ");
            for i in 0..4 {
                format_u8(config.ip_addr[i]);
                if i < 3 {
                    vga::write_str(".");
                }
            }
            vga::write_str("\n");

            vga::write_str("  Netmask:     ");
            for i in 0..4 {
                format_u8(config.subnet_mask[i]);
                if i < 3 {
                    vga::write_str(".");
                }
            }
            vga::write_str("\n");

            vga::write_str("  Gateway:     ");
            for i in 0..4 {
                format_u8(config.gateway[i]);
                if i < 3 {
                    vga::write_str(".");
                }
            }
            vga::write_str("\n");
        } else {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::write_str("  Not configured\n");
            vga::set_color(vga::Color::White, vga::Color::Black);
        }

        let link_up = e1000::is_link_up();
        vga::write_str("  Link:        ");
        if link_up {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::write_str("UP\n");
        } else {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::write_str("DOWN\n");
        }
        vga::set_color(vga::Color::White, vga::Color::Black);
    } else {
        // Parse configuration: ifconfig <ip> <netmask> <gateway>
        let parts: [&str; 3] = {
            let mut iter = args.split_whitespace();
            [
                iter.next().unwrap_or(""),
                iter.next().unwrap_or(""),
                iter.next().unwrap_or(""),
            ]
        };

        if parts[0].is_empty() || parts[1].is_empty() {
            vga::write_str("Usage: ifconfig <ip> <netmask> <gateway>\n");
            vga::write_str("Example: ifconfig 10.0.2.15 255.255.255.0 10.0.2.2\n\n");
            return;
        }

        // Parse IP addresses
        let ip = parse_ip(parts[0]);
        let netmask = parse_ip(parts[1]);
        let gateway = if !parts[2].is_empty() {
            parse_ip(parts[2])
        } else {
            [0, 0, 0, 0]
        };

        if ip[0] == 0 && ip[1] == 0 && ip[2] == 0 && ip[3] == 0 {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::write_str("Error: Invalid IP address\n\n");
            vga::set_color(vga::Color::White, vga::Color::Black);
            return;
        }

        // Configure network
        net::configure(ip, netmask, gateway);

        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::write_str("Network configured successfully\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::write_str("IP:      ");
        for i in 0..4 {
            format_u8(ip[i]);
            if i < 3 {
                vga::write_str(".");
            }
        }
        vga::write_str("\n");
        vga::write_str("Netmask: ");
        for i in 0..4 {
            format_u8(netmask[i]);
            if i < 3 {
                vga::write_str(".");
            }
        }
        vga::write_str("\n");
        if gateway[0] != 0 || gateway[1] != 0 || gateway[2] != 0 || gateway[3] != 0 {
            vga::write_str("Gateway: ");
            for i in 0..4 {
                format_u8(gateway[i]);
                if i < 3 {
                    vga::write_str(".");
                }
            }
            vga::write_str("\n");
        }
    }

    vga::write_str("\n");
}

/// Send ping (ICMP echo request)
fn cmd_ping(args: &str) {
    use crate::drivers::e1000;
    use crate::net::icmp;

    vga::write_str("\n");

    if !e1000::is_initialized() {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::write_str("Network card not initialized. Use 'netinit' first.\n\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
        return;
    }

    let config = net::get_config();
    if !config.configured {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::write_str("Network not configured. Use 'ifconfig' first.\n\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
        return;
    }

    if args.is_empty() {
        vga::write_str("Usage: ping <ip_address>\n");
        vga::write_str("Example: ping 10.0.2.2\n\n");
        return;
    }

    let dest_ip = parse_ip(args);
    if dest_ip[0] == 0 && dest_ip[1] == 0 && dest_ip[2] == 0 && dest_ip[3] == 0 {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::write_str("Error: Invalid IP address\n\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
        return;
    }

    vga::write_str("PING ");
    for i in 0..4 {
        format_u8(dest_ip[i]);
        if i < 3 {
            vga::write_str(".");
        }
    }
    vga::write_str(" 56 bytes of data\n");

    // Send ping
    icmp::send_echo_request(dest_ip, 1, 1);

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Ping request sent\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("Note: Reply processing requires packet polling in main loop\n\n");
}

/// Display ARP cache
fn cmd_arp() {
    use crate::net::arp;

    vga::write_str("\n");
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("ARP Cache\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("=========\n\n");

    let entries = arp::get_cache_entries();
    let mut found = false;

    vga::write_str("IP Address      MAC Address\n");
    vga::write_str("----------      -----------\n");

    for entry in entries.iter() {
        if entry.is_valid() {
            found = true;
            let ip = entry.get_ip();
            let mac = entry.get_mac();

            // Display IP
            for i in 0..4 {
                format_u8(ip[i]);
                if i < 3 {
                    vga::write_str(".");
                }
            }

            // Padding
            vga::write_str("   ");

            // Display MAC
            for i in 0..6 {
                crate::util::write_hex_u8(mac[i]);
                if i < 5 {
                    vga::write_str(":");
                }
            }

            vga::write_str("\n");
        }
    }

    if !found {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::write_str("ARP cache is empty\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
    }

    vga::write_str("\n");
}

/// Parse IP address from string (e.g., "192.168.1.1")
fn parse_ip(s: &str) -> [u8; 4] {
    let mut result = [0u8; 4];
    let mut idx = 0;
    let mut current = 0u8;

    for ch in s.chars() {
        if ch == '.' {
            if idx < 4 {
                result[idx] = current;
                idx += 1;
                current = 0;
            }
        } else if ch >= '0' && ch <= '9' {
            let digit = (ch as u8) - b'0';
            current = current.saturating_mul(10).saturating_add(digit);
        }
    }

    if idx < 4 {
        result[idx] = current;
    }

    result
}

/// Format u8 as decimal
fn format_u8(value: u8) {
    if value >= 100 {
        let hundreds = value / 100;
        let tens = (value % 100) / 10;
        let ones = value % 10;
        let buf = [(b'0' + hundreds), (b'0' + tens), (b'0' + ones)];
        if let Ok(s) = core::str::from_utf8(&buf) {
            vga::write_str(s);
        }
    } else if value >= 10 {
        let tens = value / 10;
        let ones = value % 10;
        let buf = [(b'0' + tens), (b'0' + ones)];
        if let Ok(s) = core::str::from_utf8(&buf) {
            vga::write_str(s);
        }
    } else {
        let buf = [(b'0' + value)];
        if let Ok(s) = core::str::from_utf8(&buf) {
            vga::write_str(s);
        }
    }
}

/// List all PCI devices
fn cmd_lspci() {
    use crate::arch::pci;

    vga::write_str("PCI Devices:\n");
    vga::write_str("-----------\n");

    let devices = pci::enumerate_devices();

    if devices.iter().all(|d| d.is_none()) {
        vga::write_str("No PCI devices found.\n");
        return;
    }

    for device_opt in &devices {
        if let Some(device) = device_opt {
            // Format: Bus:Slot.Func VendorID:DeviceID Class: Description
            vga::write_str("  ");
            format_hex_u8(device.bus);
            vga::write_str(":");
            format_hex_u8(device.slot);
            vga::write_str(".");
            format_u8(device.function);
            vga::write_str(" ");

            format_hex_u16(device.vendor_id);
            vga::write_str(":");
            format_hex_u16(device.device_id);
            vga::write_str(" ");

            vga::write_str(pci::get_class_name(device.class_code));
            vga::write_str(" - ");
            vga::write_str(pci::get_vendor_name(device.vendor_id));
            vga::write_str("\n");
        }
    }

    vga::write_str("\n");
}

/// Display integrated GPU information
fn cmd_gpuinfo() {
    use crate::drivers::igpu;

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Integrated Graphics Information\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("================================\n\n");

    if !igpu::is_initialized() {
        vga::write_str("Attempting to initialize integrated graphics...\n");
        unsafe {
            match igpu::init() {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::write_str("Successfully initialized!\n\n");
                    vga::set_color(vga::Color::White, vga::Color::Black);
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::write_str("Error: ");
                    vga::write_str(e);
                    vga::write_str("\n");
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    return;
                }
            }
        }
    }

    if let Some(info) = igpu::get_info() {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::write_str("Vendor:       ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::write_str(igpu::get_vendor_name());
        vga::write_str("\n");

        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::write_str("Device:       ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::write_str(igpu::get_description());
        vga::write_str("\n");

        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::write_str("Device ID:    ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        format_hex_u16(info.device_id);
        vga::write_str("\n");

        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::write_str("Framebuffer:  ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        format_hex_u64(info.framebuffer_addr);
        vga::write_str(" (");
        format_size(info.framebuffer_size as u64);
        vga::write_str(")\n");
    }

    vga::write_str("\n");
}

/// Display Wi-Fi information
fn cmd_wifiinfo() {
    use crate::drivers::wifi;

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Wi-Fi Information\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("=================\n\n");

    if !wifi::is_initialized() {
        vga::write_str("Attempting to initialize Wi-Fi driver...\n");
        unsafe {
            match wifi::init() {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::write_str("Successfully initialized!\n\n");
                    vga::set_color(vga::Color::White, vga::Color::Black);
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::write_str("Error: ");
                    vga::write_str(e);
                    vga::write_str("\n");
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    return;
                }
            }
        }
    }

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Device:       ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("Realtek RTL8852BE\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Standards:    ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(wifi::get_supported_standards());
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Max Speed:    ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(wifi::get_max_throughput());
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("State:        ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    match wifi::get_state() {
        wifi::WifiState::Uninitialized => vga::write_str("Uninitialized"),
        wifi::WifiState::Initialized => vga::write_str("Initialized"),
        wifi::WifiState::Scanning => vga::write_str("Scanning"),
        wifi::WifiState::Connecting => vga::write_str("Connecting"),
        wifi::WifiState::Connected => vga::write_str("Connected"),
        wifi::WifiState::Disconnected => vga::write_str("Disconnected"),
    }
    vga::write_str("\n");

    if wifi::is_connected() {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::write_str("Signal:       ");
        vga::set_color(vga::Color::White, vga::Color::Black);
        let signal = wifi::get_signal_strength();
        if signal > -50 {
            vga::write_str("Excellent");
        } else if signal > -60 {
            vga::write_str("Good");
        } else if signal > -70 {
            vga::write_str("Fair");
        } else {
            vga::write_str("Weak");
        }
        vga::write_str(" (");
        format_i8(signal);
        vga::write_str(" dBm)\n");
    }

    vga::write_str("\n");
}

/// Scan for Wi-Fi networks
fn cmd_wifiscan() {
    use crate::drivers::wifi;

    vga::write_str("Scanning for Wi-Fi networks...\n");

    if !wifi::is_initialized() {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::write_str("Error: Wi-Fi driver not initialized. Run 'wifiinfo' first.\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
        return;
    }

    match wifi::scan_networks() {
        Ok(networks) => {
            if networks.iter().all(|n| n.is_none()) {
                vga::write_str("No networks found.\n");
                vga::write_str("\n");
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::write_str("Note: Wi-Fi scanning requires firmware and hardware-specific\n");
                vga::write_str("      implementation. This is a driver skeleton.\n");
                vga::set_color(vga::Color::White, vga::Color::Black);
            } else {
                vga::write_str("\nAvailable Networks:\n");
                vga::write_str("-------------------\n");
                for network_opt in &networks {
                    if let Some(network) = network_opt {
                        vga::write_str("  SSID: ");
                        let ssid = core::str::from_utf8(&network.ssid[..network.ssid_len])
                            .unwrap_or("<invalid>");
                        vga::write_str(ssid);
                        vga::write_str(" (Channel ");
                        format_u8(network.channel);
                        vga::write_str(", Signal: ");
                        format_i8(network.signal_strength);
                        vga::write_str(" dBm)\n");
                    }
                }
            }
        }
        Err(e) => {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::write_str("Error: ");
            vga::write_str(e);
            vga::write_str("\n");
            vga::set_color(vga::Color::White, vga::Color::Black);
        }
    }

    vga::write_str("\n");
}

/// Display Bluetooth information
fn cmd_btinfo() {
    use crate::drivers::bluetooth;

    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("Bluetooth Information\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("=====================\n\n");

    if !bluetooth::is_initialized() {
        vga::write_str("Attempting to initialize Bluetooth driver...\n");
        unsafe {
            match bluetooth::init() {
                Ok(_) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::write_str("Successfully initialized!\n\n");
                    vga::set_color(vga::Color::White, vga::Color::Black);
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::write_str("Error: ");
                    vga::write_str(e);
                    vga::write_str("\n");
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    return;
                }
            }
        }
    }

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Device:       ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("Realtek RTL8852BE\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Version:      ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(bluetooth::get_version_string());
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Features:     ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(bluetooth::get_features());
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Profiles:     ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(bluetooth::get_supported_profiles());
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("State:        ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    match bluetooth::get_state() {
        bluetooth::BluetoothState::Uninitialized => vga::write_str("Uninitialized"),
        bluetooth::BluetoothState::Initialized => vga::write_str("Initialized"),
        bluetooth::BluetoothState::Scanning => vga::write_str("Scanning"),
        bluetooth::BluetoothState::Connecting => vga::write_str("Connecting"),
        bluetooth::BluetoothState::Connected => vga::write_str("Connected"),
        bluetooth::BluetoothState::Disconnected => vga::write_str("Disconnected"),
    }
    vga::write_str("\n");

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("Name:         ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(bluetooth::get_device_name());
    vga::write_str("\n");

    vga::write_str("\n");
}

/// Scan for Bluetooth devices
fn cmd_btscan() {
    use crate::drivers::bluetooth;

    vga::write_str("Scanning for Bluetooth devices...\n");

    if !bluetooth::is_initialized() {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::write_str("Error: Bluetooth driver not initialized. Run 'btinfo' first.\n");
        vga::set_color(vga::Color::White, vga::Color::Black);
        return;
    }

    match bluetooth::scan_devices(5000) {
        Ok(devices) => {
            if devices.iter().all(|d| d.is_none()) {
                vga::write_str("No devices found.\n");
                vga::write_str("\n");
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::write_str("Note: Bluetooth scanning requires firmware and HCI\n");
                vga::write_str("      implementation. This is a driver skeleton.\n");
                vga::set_color(vga::Color::White, vga::Color::Black);
            } else {
                vga::write_str("\nDiscovered Devices:\n");
                vga::write_str("-------------------\n");
                for device_opt in &devices {
                    if let Some(device) = device_opt {
                        vga::write_str("  ");
                        let name = core::str::from_utf8(&device.name[..device.name_len])
                            .unwrap_or("<unknown>");
                        vga::write_str(name);
                        vga::write_str(" (");
                        for (i, &byte) in device.address.iter().enumerate() {
                            if i > 0 {
                                vga::write_str(":");
                            }
                            format_hex_u8(byte);
                        }
                        vga::write_str(", RSSI: ");
                        format_i8(device.rssi);
                        vga::write_str(" dBm)\n");
                    }
                }
            }
        }
        Err(e) => {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::write_str("Error: ");
            vga::write_str(e);
            vga::write_str("\n");
            vga::set_color(vga::Color::White, vga::Color::Black);
        }
    }

    vga::write_str("\n");
}

/// Format an i8 value
fn format_i8(value: i8) {
    if value < 0 {
        vga::write_str("-");
        format_u8((-value) as u8);
    } else {
        format_u8(value as u8);
    }
}

/// Format size in bytes
fn format_size(bytes: u64) {
    if bytes >= 1024 * 1024 * 1024 {
        let gb = bytes / (1024 * 1024 * 1024);
        format_u64(gb);
        vga::write_str(" GB");
    } else if bytes >= 1024 * 1024 {
        let mb = bytes / (1024 * 1024);
        format_u64(mb);
        vga::write_str(" MB");
    } else if bytes >= 1024 {
        let kb = bytes / 1024;
        format_u64(kb);
        vga::write_str(" KB");
    } else {
        format_u64(bytes);
        vga::write_str(" B");
    }
}

/// Format u64 as hexadecimal
fn format_hex_u64(value: u64) {
    vga::write_str("0x");
    for i in (0..16).rev() {
        let nibble = ((value >> (i * 4)) & 0xF) as u8;
        let ch = if nibble < 10 {
            b'0' + nibble
        } else {
            b'A' + (nibble - 10)
        };
        let s = [ch];
        if let Ok(s) = core::str::from_utf8(&s) {
            vga::write_str(s);
        }
    }
}

/// Format u16 as hexadecimal
fn format_hex_u16(value: u16) {
    vga::write_str("0x");
    for i in (0..4).rev() {
        let nibble = ((value >> (i * 4)) & 0xF) as u8;
        let ch = if nibble < 10 {
            b'0' + nibble
        } else {
            b'A' + (nibble - 10)
        };
        let s = [ch];
        if let Ok(s) = core::str::from_utf8(&s) {
            vga::write_str(s);
        }
    }
}

/// Format u8 as hexadecimal
fn format_hex_u8(value: u8) {
    vga::write_str("0x");
    for i in (0..2).rev() {
        let nibble = ((value >> (i * 4)) & 0xF) as u8;
        let ch = if nibble < 10 {
            b'0' + nibble
        } else {
            b'A' + (nibble - 10)
        };
        let s = [ch];
        if let Ok(s) = core::str::from_utf8(&s) {
            vga::write_str(s);
        }
    }
}
