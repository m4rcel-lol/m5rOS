// Interrupt handling utilities
//
// Provides safe wrappers for interrupt control and utility functions

use core::arch::asm;

/// Check if interrupts are enabled
///
/// # Safety
/// Reading RFLAGS is always safe
#[allow(dead_code)]
pub fn are_enabled() -> bool {
    let rflags: u64;
    // SAFETY: Reading RFLAGS doesn't modify system state
    unsafe {
        asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
    }
    (rflags & (1 << 9)) != 0
}

/// Enable interrupts
///
/// # Safety
/// Should only be called when interrupt handlers are properly set up
pub unsafe fn enable() {
    asm!("sti", options(nomem, nostack));
}

/// Disable interrupts
///
/// # Safety
/// Always safe to disable interrupts
pub unsafe fn disable() {
    asm!("cli", options(nomem, nostack));
}

/// Execute a closure with interrupts disabled
///
/// Automatically re-enables interrupts after the closure completes if they were enabled before
#[allow(dead_code)]
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let were_enabled = are_enabled();

    if were_enabled {
        // SAFETY: We're temporarily disabling interrupts and will restore state
        unsafe { disable(); }
    }

    let result = f();

    if were_enabled {
        // SAFETY: We're restoring the previous interrupt state
        unsafe { enable(); }
    }

    result
}

/// Halt the CPU until next interrupt
///
/// # Safety
/// Always safe to halt the CPU
#[inline]
pub unsafe fn hlt() {
    asm!("hlt", options(nomem, nostack, preserves_flags));
}

/// Halt the CPU in a loop (for idle or panic situations)
pub fn halt_loop() -> ! {
    loop {
        // SAFETY: HLT is safe to execute
        unsafe {
            hlt();
        }
    }
}

/// No-op (pause instruction for spin-wait loops)
///
/// # Safety
/// Always safe
#[allow(dead_code)]
#[inline]
pub fn spin_loop_hint() {
    // SAFETY: PAUSE instruction is safe and improves spin-loop performance
    unsafe {
        asm!("pause", options(nomem, nostack, preserves_flags));
    }
}
