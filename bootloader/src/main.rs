#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// UEFI bootloader entry point
///
/// This bootloader is responsible for:
/// 1. Loading the m5rOS kernel ELF from disk
/// 2. Setting up 4-level x86_64 paging
/// 3. Transferring control to the kernel
#[no_mangle]
pub extern "efiapi" fn efi_main(_image_handle: *mut core::ffi::c_void, _system_table: *mut core::ffi::c_void) -> usize {
    // Phase 2 implementation will go here
    // For now, this is a stub that will be implemented in Phase 2
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
