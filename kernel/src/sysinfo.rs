// System information module
//
// Provides functions to retrieve system information for display

use crate::arch;
use crate::mem;

/// System information structure
#[derive(Debug)]
pub struct SystemInfo {
    pub os_name: &'static str,
    pub os_version: &'static str,
    pub kernel_version: &'static str,
    pub uptime_seconds: u64,
    pub cpu_vendor: &'static str,
    pub cpu_features: CpuFeatures,
    pub memory_total: usize,
    pub memory_allocated: usize,
    pub memory_free: usize,
}

/// CPU feature flags
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    pub fpu: bool,
    pub tsc: bool,
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub sse4_1: bool,
    pub sse4_2: bool,
    pub avx: bool,
    pub avx2: bool,
}

impl CpuFeatures {
    /// Get a string representation of supported features
    pub fn as_string<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        let mut offset = 0;
        let features = [
            ("FPU", self.fpu),
            ("TSC", self.tsc),
            ("SSE", self.sse),
            ("SSE2", self.sse2),
            ("SSE3", self.sse3),
            ("SSE4.1", self.sse4_1),
            ("SSE4.2", self.sse4_2),
            ("AVX", self.avx),
            ("AVX2", self.avx2),
        ];

        for (_i, (name, supported)) in features.iter().enumerate() {
            if *supported {
                if offset > 0 && offset < buf.len() {
                    buf[offset] = b' ';
                    offset += 1;
                }
                let name_bytes = name.as_bytes();
                let copy_len = (buf.len() - offset).min(name_bytes.len());
                buf[offset..offset + copy_len].copy_from_slice(&name_bytes[..copy_len]);
                offset += copy_len;
            }
        }

        // SAFETY: We only wrote ASCII characters
        unsafe { core::str::from_utf8_unchecked(&buf[..offset]) }
    }
}

/// Get current system information
pub fn get_system_info() -> SystemInfo {
    // Get CPU information
    let cpu_vendor = arch::cpuid::get_vendor_string();
    let cpu_info = arch::cpuid::get_feature_info();

    let cpu_features = CpuFeatures {
        fpu: cpu_info.edx & (1 << 0) != 0,
        tsc: cpu_info.edx & (1 << 4) != 0,
        sse: cpu_info.edx & (1 << 25) != 0,
        sse2: cpu_info.edx & (1 << 26) != 0,
        sse3: cpu_info.ecx & (1 << 0) != 0,
        sse4_1: cpu_info.ecx & (1 << 19) != 0,
        sse4_2: cpu_info.ecx & (1 << 20) != 0,
        avx: cpu_info.ecx & (1 << 28) != 0,
        avx2: false, // Would need to check extended features
    };

    // Get memory statistics
    let (total_frames, allocated_frames, free_frames) = mem::frame_allocator::stats();
    let memory_total = total_frames * mem::frame_allocator::FRAME_SIZE;
    let memory_allocated = allocated_frames * mem::frame_allocator::FRAME_SIZE;
    let memory_free = free_frames * mem::frame_allocator::FRAME_SIZE;

    // Get uptime from PIT
    let uptime_seconds = arch::pit::uptime_seconds();

    SystemInfo {
        os_name: "m5rOS",
        os_version: "0.2.0",
        kernel_version: "0.2.0-dev",
        uptime_seconds,
        cpu_vendor,
        cpu_features,
        memory_total,
        memory_allocated,
        memory_free,
    }
}

/// Format size in human-readable format
pub fn format_size<'a>(bytes: usize, buf: &'a mut [u8]) -> &'a str {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    let (value, unit) = if bytes >= GB {
        (bytes / GB, "GB")
    } else if bytes >= MB {
        (bytes / MB, "MB")
    } else if bytes >= KB {
        (bytes / KB, "KB")
    } else {
        (bytes, "B")
    };

    // Simple integer to string conversion
    let mut temp = value;
    let mut digits = [0u8; 20];
    let mut digit_count = 0;

    if temp == 0 {
        digits[0] = b'0';
        digit_count = 1;
    } else {
        while temp > 0 {
            digits[digit_count] = b'0' + (temp % 10) as u8;
            temp /= 10;
            digit_count += 1;
        }
    }

    // Reverse digits into buffer
    let mut offset = 0;
    for i in (0..digit_count).rev() {
        if offset < buf.len() {
            buf[offset] = digits[i];
            offset += 1;
        }
    }

    // Add unit
    if offset < buf.len() {
        buf[offset] = b' ';
        offset += 1;
    }
    for &b in unit.as_bytes() {
        if offset < buf.len() {
            buf[offset] = b;
            offset += 1;
        }
    }

    // SAFETY: We only wrote ASCII characters
    unsafe { core::str::from_utf8_unchecked(&buf[..offset]) }
}

/// Format uptime in human-readable format (HH:MM:SS)
pub fn format_uptime<'a>(seconds: u64, buf: &'a mut [u8]) -> &'a str {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut offset = 0;

    // Format hours
    if hours > 0 {
        let h_tens = (hours / 10) as u8;
        let h_ones = (hours % 10) as u8;
        if offset < buf.len() { buf[offset] = b'0' + h_tens; offset += 1; }
        if offset < buf.len() { buf[offset] = b'0' + h_ones; offset += 1; }
        if offset < buf.len() { buf[offset] = b'h'; offset += 1; }
        if offset < buf.len() { buf[offset] = b' '; offset += 1; }
    }

    // Format minutes
    let m_tens = (minutes / 10) as u8;
    let m_ones = (minutes % 10) as u8;
    if offset < buf.len() { buf[offset] = b'0' + m_tens; offset += 1; }
    if offset < buf.len() { buf[offset] = b'0' + m_ones; offset += 1; }
    if offset < buf.len() { buf[offset] = b'm'; offset += 1; }
    if offset < buf.len() { buf[offset] = b' '; offset += 1; }

    // Format seconds
    let s_tens = (secs / 10) as u8;
    let s_ones = (secs % 10) as u8;
    if offset < buf.len() { buf[offset] = b'0' + s_tens; offset += 1; }
    if offset < buf.len() { buf[offset] = b'0' + s_ones; offset += 1; }
    if offset < buf.len() { buf[offset] = b's'; offset += 1; }

    // SAFETY: We only wrote ASCII characters
    unsafe { core::str::from_utf8_unchecked(&buf[..offset]) }
}
