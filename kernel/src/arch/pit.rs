// Programmable Interval Timer (PIT/8253) driver
//
// The PIT generates periodic timer interrupts that are used for:
// - Task scheduling
// - Timekeeping
// - System uptime tracking

use crate::arch::port::outb;
use core::sync::atomic::{AtomicU64, Ordering};

/// PIT channel 0 data port (used for IRQ0 timer)
const PIT_CHANNEL0: u16 = 0x40;
/// PIT command port
const PIT_COMMAND: u16 = 0x43;

/// PIT base frequency (1.193182 MHz)
const PIT_FREQUENCY: u32 = 1193182;

/// Target frequency for timer interrupts (100 Hz = 10ms interval)
pub const TIMER_FREQUENCY: u32 = 100;

/// Global tick counter
static TICKS: AtomicU64 = AtomicU64::new(0);

/// Initialize the PIT
///
/// Sets up the PIT to generate interrupts at TIMER_FREQUENCY Hz
///
/// # Safety
/// Must only be called once during kernel initialization
pub unsafe fn init() {
    // Calculate the divisor for the desired frequency
    let divisor = (PIT_FREQUENCY / TIMER_FREQUENCY) as u16;

    // Command byte:
    // - Channel 0
    // - Access mode: lobyte/hibyte
    // - Operating mode 3: square wave generator
    // - Binary mode (not BCD)
    outb(PIT_COMMAND, 0x36);

    // Send divisor (low byte then high byte)
    outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
    outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);
}

/// Timer interrupt handler
///
/// Called by the interrupt handler when a timer interrupt occurs
pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Get the current tick count
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Get elapsed time in milliseconds since boot
pub fn uptime_ms() -> u64 {
    // Each tick is 1000/TIMER_FREQUENCY milliseconds
    ticks() * (1000 / TIMER_FREQUENCY as u64)
}

/// Get elapsed time in seconds since boot
pub fn uptime_secs() -> u64 {
    uptime_ms() / 1000
}

/// Sleep for approximately the given number of milliseconds
///
/// Note: This is a busy-wait sleep and not very accurate
pub fn sleep_ms(ms: u64) {
    let target = uptime_ms() + ms;
    while uptime_ms() < target {
        core::hint::spin_loop();
    }
}
