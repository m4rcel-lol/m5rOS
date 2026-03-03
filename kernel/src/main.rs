#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod arch;
mod drivers;
mod mem;

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
    drivers::serial::write_str("m5rOS v0.1.0 - Booting...\n");

    // Initialize VGA text mode
    // SAFETY: This is called once during kernel initialization
    unsafe {
        drivers::vga::init();
    }
    drivers::vga::clear_screen();
    drivers::vga::write_str("m5rOS v0.1.0\n");
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
        core::arch::asm!("sti");
    }

    // Note: Heap initialization requires proper paging setup first
    // This will be enabled once we have a working bootloader and paging

    drivers::serial::write_str("Kernel initialized successfully\n");
    drivers::vga::write_str("Kernel initialized successfully!\n\n");

    drivers::vga::set_color(drivers::vga::Color::LightGreen, drivers::vga::Color::Black);
    drivers::vga::write_str("All systems operational.\n");
    drivers::vga::set_color(drivers::vga::Color::LightCyan, drivers::vga::Color::Black);
    drivers::vga::write_str("Timer and keyboard interrupts enabled.\n");
    drivers::vga::write_str("Type to test keyboard input!\n\n");
    drivers::vga::set_color(drivers::vga::Color::White, drivers::vga::Color::Black);

    // Halt the CPU
    loop {
        // SAFETY: HLT is safe to execute
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Kernel panic handler
///
/// Outputs register state and stack trace via serial port
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    drivers::serial::write_str("\n!!! KERNEL PANIC !!!\n");
    if let Some(location) = info.location() {
        drivers::serial::write_str("Location: ");
        drivers::serial::write_str(location.file());
        drivers::serial::write_str("\n");
    }

    loop {
        // SAFETY: HLT is safe to execute
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
