#![no_std]
#![no_main]
// Will need abi_x86_interrupt feature for interrupt handling in Phase 5
// #![feature(abi_x86_interrupt)]

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
    drivers::serial::write_str("Kernel initialized successfully\n");

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
