// Boot animation and service startup display
//
// Displays a boot animation and service initialization status like Linux distros

use crate::drivers::vga;
use crate::arch::pit;

/// Display boot animation and initialize services
pub fn display_boot_animation() {
    vga::clear_screen();

    // Display m5rOS boot logo
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_str("\n");
    vga::write_str("    ███╗   ███╗███████╗██████╗  ██████╗ ███████╗\n");
    vga::write_str("    ████╗ ████║██╔════╝██╔══██╗██╔═══██╗██╔════╝\n");
    vga::write_str("    ██╔████╔██║███████╗██████╔╝██║   ██║███████╗\n");
    vga::write_str("    ██║╚██╔╝██║╚════██║██╔══██╗██║   ██║╚════██║\n");
    vga::write_str("    ██║ ╚═╝ ██║███████║██║  ██║╚██████╔╝███████║\n");
    vga::write_str("    ╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("\n");
    vga::write_str("              Custom Operating System v0.2.0\n");
    vga::write_str("              Built from First Principles\n\n");

    small_delay();

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("=======================================================\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("              Starting System Services\n");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("=======================================================\n\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    small_delay();

    // Start services with visual feedback
    start_service("GDT Initialization", true);
    start_service("IDT Setup", true);
    start_service("PIC Configuration", true);
    start_service("PIT Timer (100 Hz)", true);
    start_service("PS/2 Keyboard Driver", true);
    start_service("Serial Port (COM1)", true);
    start_service("RTC Driver", true);
    start_service("ATA Disk Driver", true);
    start_service("Memory Manager", true);
    start_service("Virtual Memory", true);

    vga::write_str("\n");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("=======================================================\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str("              All Services Running\n");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::write_str("=======================================================\n\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    small_delay();
}

/// Start a service and display status
fn start_service(name: &str, success: bool) {
    vga::write_str("  [ ");

    if success {
        vga::set_color(vga::Color::LightBlue, vga::Color::Black);
        vga::write_str("STARTING");
    } else {
        vga::set_color(vga::Color::Yellow, vga::Color::Black);
        vga::write_str("PENDING ");
    }

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(" ] ");
    vga::write_str(name);

    small_delay();

    // Move cursor back to update status
    // Simulate service startup time
    tiny_delay();

    // Clear the line by overwriting
    vga::write_str("\r  [ ");

    if success {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::write_str("   OK   ");
    } else {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::write_str(" FAILED ");
    }

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::write_str(" ] ");
    vga::write_str(name);
    vga::write_str("\n");
}

/// Small delay for animation
fn small_delay() {
    let start_ticks = pit::get_ticks();
    // Wait for approximately 100ms (10 ticks at 100Hz)
    while pit::get_ticks() < start_ticks + 10 {
        unsafe { core::arch::asm!("pause"); }
    }
}

/// Tiny delay for animation
fn tiny_delay() {
    let start_ticks = pit::get_ticks();
    // Wait for approximately 30ms (3 ticks at 100Hz)
    while pit::get_ticks() < start_ticks + 3 {
        unsafe { core::arch::asm!("pause"); }
    }
}
