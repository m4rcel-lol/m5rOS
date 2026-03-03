#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod arch;
mod drivers;
mod mem;
mod util;
mod fmt;
mod sysinfo;
mod command;
mod stats;

/// Kernel entry point
///
/// Called by the bootloader after setting up paging and loading the kernel
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Initialize serial port for debugging
    // SAFETY: This is called once during kernel initialization
    unsafe {
        drivers::serial::init();
    }

    // Print boot message
    drivers::serial::write_str("m5rOS v0.2.0 - Booting...\n");

    // Print CPU information
    arch::cpuid::print_cpu_info();

    // Initialize VGA text mode
    // SAFETY: This is called once during kernel initialization
    unsafe {
        drivers::vga::init();
    }
    drivers::vga::clear_screen();
    drivers::vga::write_str("m5rOS v0.2.0\n");
    drivers::vga::write_str("=============\n\n");

    drivers::serial::write_str("Initializing GDT...\n");
    drivers::vga::write_str("Initializing GDT...\n");
    // Initialize GDT
    // SAFETY: This is called once during kernel initialization
    unsafe {
        arch::gdt::init();
    }

    drivers::serial::write_str("Initializing IDT...\n");
    drivers::vga::write_str("Initializing IDT...\n");
    // Initialize IDT
    // SAFETY: This is called once during kernel initialization
    unsafe {
        arch::idt::init();
    }

    drivers::serial::write_str("Initializing PIC...\n");
    drivers::vga::write_str("Initializing PIC...\n");
    // Initialize PIC (Programmable Interrupt Controller)
    // SAFETY: This is called once during kernel initialization
    unsafe {
        arch::pic::init();
    }

    drivers::serial::write_str("Initializing PIT...\n");
    drivers::vga::write_str("Initializing PIT...\n");
    // Initialize PIT (Programmable Interval Timer)
    // SAFETY: This is called once during kernel initialization
    unsafe {
        arch::pit::init();
    }

    drivers::serial::write_str("Initializing keyboard...\n");
    drivers::vga::write_str("Initializing keyboard...\n");
    // Initialize keyboard driver
    // SAFETY: This is called once during kernel initialization
    unsafe {
        drivers::keyboard::init();
    }

    // Enable interrupts now that all handlers are set up
    drivers::serial::write_str("Enabling interrupts...\n");
    drivers::vga::write_str("Enabling interrupts...\n");

    // Enable timer and keyboard IRQs
    // SAFETY: Interrupt handlers are set up
    unsafe {
        arch::pic::enable_irq(arch::pic::irq::TIMER);
        arch::pic::enable_irq(arch::pic::irq::KEYBOARD);

        // Enable interrupts globally
        arch::interrupts::enable();
    }

    // Note: Heap initialization requires proper paging setup first
    // This will be enabled once we have a working bootloader and paging

    drivers::serial::write_str("Kernel initialized successfully\n");
    drivers::vga::write_str("Kernel initialized successfully!\n\n");

    drivers::vga::set_color(drivers::vga::Color::LightGreen, drivers::vga::Color::Black);
    drivers::vga::write_str("All systems operational.\n");
    drivers::vga::set_color(drivers::vga::Color::LightCyan, drivers::vga::Color::Black);
    drivers::vga::write_str("Timer and keyboard interrupts enabled.\n");
    drivers::vga::write_str("Type 'help' for available commands or 'fetch' for system info!\n\n");
    drivers::vga::set_color(drivers::vga::Color::White, drivers::vga::Color::Black);

    // Show command prompt
    drivers::vga::write_str("> ");

    // Halt the CPU
    arch::interrupts::halt_loop();
}

/// Kernel panic handler
///
/// Outputs panic information and halts
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts to prevent further issues
    // SAFETY: We're in a panic state, need to stop everything
    unsafe {
        arch::interrupts::disable();
    }

    drivers::serial::write_str("\n!!! KERNEL PANIC !!!\n");
    drivers::vga::set_color(drivers::vga::Color::White, drivers::vga::Color::Red);
    drivers::vga::write_str("\n!!! KERNEL PANIC !!!\n");

    if let Some(location) = info.location() {
        drivers::serial::write_str("Location: ");
        drivers::serial::write_str(location.file());
        drivers::serial::write_str(":");
        // Note: Can't format line number without alloc
        drivers::serial::write_str("\n");

        drivers::vga::write_str("Location: ");
        drivers::vga::write_str(location.file());
        drivers::vga::write_str("\n");
    }

    // Note: info.message() returns a PanicMessage, which we can't easily format without alloc
    drivers::serial::write_str("Message: [panic message]\n");
    drivers::vga::write_str("Message: [panic message]\n");

    // Dump CPU registers
    drivers::serial::write_str("\nRegister dump:\n");
    drivers::vga::write_str("\nRegister dump:\n");

    // SAFETY: Reading registers for debugging during panic
    unsafe {
        let mut rsp: u64;
        let mut rbp: u64;
        let mut cr2: u64;
        let mut cr3: u64;
        let mut rflags: u64;

        core::arch::asm!(
            "mov {}, rsp",
            "mov {}, rbp",
            "mov {}, cr2",
            "mov {}, cr3",
            "pushfq",
            "pop {}",
            out(reg) rsp,
            out(reg) rbp,
            out(reg) cr2,
            out(reg) cr3,
            out(reg) rflags,
        );

        drivers::serial::write_str("  RSP: 0x");
        util::write_hex_u64(rsp);
        drivers::serial::write_str("\n");

        drivers::serial::write_str("  RBP: 0x");
        util::write_hex_u64(rbp);
        drivers::serial::write_str("\n");

        drivers::serial::write_str("  CR2: 0x");
        util::write_hex_u64(cr2);
        drivers::serial::write_str(" (page fault addr)\n");

        drivers::serial::write_str("  CR3: 0x");
        util::write_hex_u64(cr3);
        drivers::serial::write_str(" (page table)\n");

        drivers::serial::write_str("  RFLAGS: 0x");
        util::write_hex_u64(rflags);
        drivers::serial::write_str("\n");

        drivers::vga::write_str("  RSP: 0x");
        let mut buf = [0u8; 16];
        let hex_str = util::format_hex_u64(rsp, &mut buf);
        drivers::vga::write_str(hex_str);
        drivers::vga::write_str("\n");

        drivers::vga::write_str("  CR2: 0x");
        let hex_str = util::format_hex_u64(cr2, &mut buf);
        drivers::vga::write_str(hex_str);
        drivers::vga::write_str("\n");

        drivers::vga::write_str("  CR3: 0x");
        let hex_str = util::format_hex_u64(cr3, &mut buf);
        drivers::vga::write_str(hex_str);
        drivers::vga::write_str("\n");
    }

    drivers::serial::write_str("\nSystem halted\n");
    drivers::vga::write_str("\nSystem halted\n");

    arch::interrupts::halt_loop();
}
