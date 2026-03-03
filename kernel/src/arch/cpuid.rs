// CPU identification and feature detection using CPUID instruction
//
// Provides safe wrappers around the CPUID instruction to detect:
// - CPU vendor and model
// - Supported features (SSE, AVX, etc.)
// - Cache information
// - Topology information

use core::arch::asm;

/// Result of a CPUID query
#[derive(Debug, Clone, Copy)]
pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

/// Execute CPUID instruction
///
/// # Safety
/// CPUID is always safe to execute on x86_64
#[inline]
pub fn cpuid(leaf: u32, subleaf: u32) -> CpuidResult {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;

    // SAFETY: CPUID is a standard x86_64 instruction that cannot cause undefined behavior
    // We work around the rbx restriction by saving/restoring it manually
    unsafe {
        asm!(
            "push rbx",        // Save rbx (LLVM uses it internally)
            "cpuid",           // Execute CPUID
            "mov {0:e}, ebx",  // Move ebx result to output
            "pop rbx",         // Restore rbx
            out(reg) ebx,
            inout("eax") leaf => eax,
            inout("ecx") subleaf => ecx,
            out("edx") edx,
            options(nostack, preserves_flags)
        );
    }

    CpuidResult { eax, ebx, ecx, edx }
}

/// CPU vendor identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    Intel,
    AMD,
    Unknown,
}

/// Get CPU vendor string
pub fn get_vendor() -> CpuVendor {
    let result = cpuid(0, 0);

    // Vendor string is in EBX, EDX, ECX (in that order)
    let vendor_bytes = [
        (result.ebx & 0xFF) as u8,
        ((result.ebx >> 8) & 0xFF) as u8,
        ((result.ebx >> 16) & 0xFF) as u8,
        ((result.ebx >> 24) & 0xFF) as u8,
        (result.edx & 0xFF) as u8,
        ((result.edx >> 8) & 0xFF) as u8,
        ((result.edx >> 16) & 0xFF) as u8,
        ((result.edx >> 24) & 0xFF) as u8,
        (result.ecx & 0xFF) as u8,
        ((result.ecx >> 8) & 0xFF) as u8,
        ((result.ecx >> 16) & 0xFF) as u8,
        ((result.ecx >> 24) & 0xFF) as u8,
    ];

    if &vendor_bytes[..12] == b"GenuineIntel" {
        CpuVendor::Intel
    } else if &vendor_bytes[..12] == b"AuthenticAMD" {
        CpuVendor::AMD
    } else {
        CpuVendor::Unknown
    }
}

/// CPU feature flags from CPUID leaf 1, EDX
pub struct CpuFeatures {
    bits: u32,
}

impl CpuFeatures {
    /// Get CPU features from CPUID
    pub fn detect() -> Self {
        let result = cpuid(1, 0);
        CpuFeatures { bits: result.edx }
    }

    /// Check if FPU is present
    pub fn has_fpu(&self) -> bool {
        self.bits & (1 << 0) != 0
    }

    /// Check if PSE (Page Size Extension) is supported
    #[allow(dead_code)]
    pub fn has_pse(&self) -> bool {
        self.bits & (1 << 3) != 0
    }

    /// Check if TSC (Time Stamp Counter) is supported
    pub fn has_tsc(&self) -> bool {
        self.bits & (1 << 4) != 0
    }

    /// Check if MSR (Model Specific Registers) are supported
    pub fn has_msr(&self) -> bool {
        self.bits & (1 << 5) != 0
    }

    /// Check if PAE (Physical Address Extension) is supported
    pub fn has_pae(&self) -> bool {
        self.bits & (1 << 6) != 0
    }

    /// Check if APIC is present
    pub fn has_apic(&self) -> bool {
        self.bits & (1 << 9) != 0
    }

    /// Check if SYSENTER/SYSEXIT is supported
    #[allow(dead_code)]
    pub fn has_sep(&self) -> bool {
        self.bits & (1 << 11) != 0
    }

    /// Check if PGE (Page Global Enable) is supported
    #[allow(dead_code)]
    pub fn has_pge(&self) -> bool {
        self.bits & (1 << 13) != 0
    }

    /// Check if PAT (Page Attribute Table) is supported
    #[allow(dead_code)]
    pub fn has_pat(&self) -> bool {
        self.bits & (1 << 16) != 0
    }

    /// Check if MMX is supported
    #[allow(dead_code)]
    pub fn has_mmx(&self) -> bool {
        self.bits & (1 << 23) != 0
    }

    /// Check if SSE is supported
    pub fn has_sse(&self) -> bool {
        self.bits & (1 << 25) != 0
    }

    /// Check if SSE2 is supported
    pub fn has_sse2(&self) -> bool {
        self.bits & (1 << 26) != 0
    }
}

/// Extended CPU features from CPUID leaf 1, ECX
pub struct ExtendedCpuFeatures {
    bits: u32,
}

impl ExtendedCpuFeatures {
    /// Get extended CPU features from CPUID
    pub fn detect() -> Self {
        let result = cpuid(1, 0);
        ExtendedCpuFeatures { bits: result.ecx }
    }

    /// Check if SSE3 is supported
    pub fn has_sse3(&self) -> bool {
        self.bits & (1 << 0) != 0
    }

    /// Check if SSSE3 is supported
    pub fn has_ssse3(&self) -> bool {
        self.bits & (1 << 9) != 0
    }

    /// Check if SSE4.1 is supported
    pub fn has_sse4_1(&self) -> bool {
        self.bits & (1 << 19) != 0
    }

    /// Check if SSE4.2 is supported
    pub fn has_sse4_2(&self) -> bool {
        self.bits & (1 << 20) != 0
    }

    /// Check if AVX is supported
    pub fn has_avx(&self) -> bool {
        self.bits & (1 << 28) != 0
    }

    /// Check if x2APIC is supported
    #[allow(dead_code)]
    pub fn has_x2apic(&self) -> bool {
        self.bits & (1 << 21) != 0
    }
}

/// Print CPU information to serial
pub fn print_cpu_info() {
    use crate::drivers::serial;

    let vendor = get_vendor();
    let features = CpuFeatures::detect();
    let ext_features = ExtendedCpuFeatures::detect();

    serial::write_str("CPU Info:\n");

    match vendor {
        CpuVendor::Intel => serial::write_str("  Vendor: Intel\n"),
        CpuVendor::AMD => serial::write_str("  Vendor: AMD\n"),
        CpuVendor::Unknown => serial::write_str("  Vendor: Unknown\n"),
    }

    serial::write_str("  Features: ");
    if features.has_fpu() { serial::write_str("FPU "); }
    if features.has_tsc() { serial::write_str("TSC "); }
    if features.has_msr() { serial::write_str("MSR "); }
    if features.has_pae() { serial::write_str("PAE "); }
    if features.has_apic() { serial::write_str("APIC "); }
    if features.has_sse() { serial::write_str("SSE "); }
    if features.has_sse2() { serial::write_str("SSE2 "); }
    if ext_features.has_sse3() { serial::write_str("SSE3 "); }
    if ext_features.has_ssse3() { serial::write_str("SSSE3 "); }
    if ext_features.has_sse4_1() { serial::write_str("SSE4.1 "); }
    if ext_features.has_sse4_2() { serial::write_str("SSE4.2 "); }
    if ext_features.has_avx() { serial::write_str("AVX "); }
    serial::write_str("\n");
}
