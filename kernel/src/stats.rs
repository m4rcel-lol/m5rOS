// Kernel statistics module
//
// Tracks various kernel statistics like interrupt counts, exceptions, etc.

use core::sync::atomic::{AtomicU64, Ordering};

/// IRQ statistics counters
pub struct IrqStats {
    pub timer: AtomicU64,
    pub keyboard: AtomicU64,
    pub total: AtomicU64,
}

impl IrqStats {
    pub const fn new() -> Self {
        IrqStats {
            timer: AtomicU64::new(0),
            keyboard: AtomicU64::new(0),
            total: AtomicU64::new(0),
        }
    }

    pub fn record_timer(&self) {
        self.timer.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_keyboard(&self) {
        self.keyboard.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_timer(&self) -> u64 {
        self.timer.load(Ordering::Relaxed)
    }

    pub fn get_keyboard(&self) -> u64 {
        self.keyboard.load(Ordering::Relaxed)
    }

    pub fn get_total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }
}

/// Exception statistics counters
pub struct ExceptionStats {
    pub divide_error: AtomicU64,
    pub page_fault: AtomicU64,
    pub general_protection: AtomicU64,
    pub total: AtomicU64,
}

impl ExceptionStats {
    pub const fn new() -> Self {
        ExceptionStats {
            divide_error: AtomicU64::new(0),
            page_fault: AtomicU64::new(0),
            general_protection: AtomicU64::new(0),
            total: AtomicU64::new(0),
        }
    }

    pub fn record_divide_error(&self) {
        self.divide_error.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_page_fault(&self) {
        self.page_fault.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_general_protection(&self) {
        self.general_protection.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_other(&self) {
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_divide_error(&self) -> u64 {
        self.divide_error.load(Ordering::Relaxed)
    }

    pub fn get_page_fault(&self) -> u64 {
        self.page_fault.load(Ordering::Relaxed)
    }

    pub fn get_general_protection(&self) -> u64 {
        self.general_protection.load(Ordering::Relaxed)
    }

    pub fn get_total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }
}

/// Global IRQ statistics
pub static IRQ_STATS: IrqStats = IrqStats::new();

/// Global exception statistics
pub static EXCEPTION_STATS: ExceptionStats = ExceptionStats::new();
