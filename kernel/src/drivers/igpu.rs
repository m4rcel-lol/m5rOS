// Integrated Graphics Driver
// Supports Intel HD Graphics and AMD APU integrated graphics

use crate::arch::pci::{self, PciDevice, PCI_CLASS_DISPLAY, PCI_SUBCLASS_VGA, VENDOR_INTEL, VENDOR_AMD};
use core::sync::atomic::{AtomicBool, Ordering};

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static mut IGPU_DEVICE: Option<PciDevice> = None;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IgpuVendor {
    Intel,
    AMD,
    Unknown,
}

#[derive(Debug)]
pub struct IgpuInfo {
    pub vendor: IgpuVendor,
    pub device_id: u16,
    pub vendor_id: u16,
    pub framebuffer_addr: u64,
    pub framebuffer_size: u32,
}

/// Initialize the integrated graphics driver
pub unsafe fn init() -> Result<(), &'static str> {
    if INITIALIZED.load(Ordering::Acquire) {
        return Err("Integrated graphics already initialized");
    }

    // Find display controller
    let devices = pci::find_devices_by_class(PCI_CLASS_DISPLAY, Some(PCI_SUBCLASS_VGA));

    // Look for Intel or AMD integrated graphics
    for device_opt in &devices {
        if let Some(device) = device_opt {
            if device.vendor_id == VENDOR_INTEL || device.vendor_id == VENDOR_AMD {
                IGPU_DEVICE = Some(*device);

                // Enable memory space and bus mastering
                device.enable_memory_space();
                device.enable_bus_mastering();

                INITIALIZED.store(true, Ordering::Release);
                return Ok(());
            }
        }
    }

    Err("No integrated graphics device found")
}

/// Check if the integrated graphics driver is initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}

/// Get information about the detected integrated GPU
pub fn get_info() -> Option<IgpuInfo> {
    if !is_initialized() {
        return None;
    }

    unsafe {
        IGPU_DEVICE.as_ref().map(|device| {
            let vendor = match device.vendor_id {
                VENDOR_INTEL => IgpuVendor::Intel,
                VENDOR_AMD => IgpuVendor::AMD,
                _ => IgpuVendor::Unknown,
            };

            // Get BAR0 for framebuffer
            let (fb_addr, fb_size, _prefetchable) = device.get_bar(0).unwrap_or((0, 0, false));

            IgpuInfo {
                vendor,
                device_id: device.device_id,
                vendor_id: device.vendor_id,
                framebuffer_addr: fb_addr,
                framebuffer_size: fb_size,
            }
        })
    }
}

/// Get the PCI device for direct access
pub fn get_device() -> Option<PciDevice> {
    unsafe { IGPU_DEVICE }
}

/// Get the vendor of the integrated GPU
pub fn get_vendor() -> Option<IgpuVendor> {
    get_info().map(|info| info.vendor)
}

/// Get vendor name as string
pub fn get_vendor_name() -> &'static str {
    match get_vendor() {
        Some(IgpuVendor::Intel) => "Intel",
        Some(IgpuVendor::AMD) => "AMD",
        Some(IgpuVendor::Unknown) => "Unknown",
        None => "Not initialized",
    }
}

/// Get a human-readable description of the GPU
pub fn get_description() -> &'static str {
    if !is_initialized() {
        return "No integrated graphics";
    }

    if let Some(info) = get_info() {
        match info.vendor {
            IgpuVendor::Intel => match info.device_id {
                // Intel HD Graphics (common device IDs)
                0x0102 => "Intel HD Graphics 2000",
                0x0112 => "Intel HD Graphics 3000",
                0x0122 => "Intel HD Graphics 3000",
                0x0152 => "Intel HD Graphics 2500",
                0x0162 => "Intel HD Graphics 4000",
                0x0166 => "Intel HD Graphics 4000",
                0x0412 => "Intel HD Graphics 4600",
                0x0416 => "Intel HD Graphics 4600",
                0x041E => "Intel HD Graphics 4400",
                0x0A16 => "Intel HD Graphics",
                0x0A26 => "Intel HD Graphics 5000",
                0x1612 => "Intel HD Graphics 5600",
                0x1616 => "Intel HD Graphics 5500",
                0x161E => "Intel HD Graphics 5300",
                0x191B => "Intel HD Graphics 530",
                0x1916 => "Intel HD Graphics 520",
                0x1912 => "Intel HD Graphics 530",
                0x5916 => "Intel HD Graphics 620",
                0x5912 => "Intel HD Graphics 630",
                0x3E92 => "Intel UHD Graphics 630",
                0x3E9B => "Intel UHD Graphics 630",
                0x9BC8 => "Intel UHD Graphics 630",
                _ => "Intel Integrated Graphics",
            },
            IgpuVendor::AMD => match info.device_id {
                // AMD APU Graphics (common device IDs)
                0x9830 => "AMD Radeon HD 8400",
                0x9831 => "AMD Radeon HD 8400E",
                0x9832 => "AMD Radeon HD 8330",
                0x9833 => "AMD Radeon HD 8330E",
                0x9850 => "AMD Radeon R7 Graphics",
                0x9851 => "AMD Radeon R7 Graphics",
                0x9852 => "AMD Radeon R5 Graphics",
                0x9853 => "AMD Radeon R5 Graphics",
                0x15D8 => "AMD Radeon Vega Graphics",
                0x15DD => "AMD Radeon Vega 8 Graphics",
                0x1636 => "AMD Radeon Vega 3 Graphics",
                0x164C => "AMD Radeon Vega 3 Graphics",
                _ => "AMD Integrated Graphics",
            },
            IgpuVendor::Unknown => "Unknown Integrated Graphics",
        }
    } else {
        "Unknown"
    }
}

/// Set display mode (placeholder for future implementation)
pub fn set_mode(_width: u32, _height: u32, _bpp: u32) -> Result<(), &'static str> {
    if !is_initialized() {
        return Err("Integrated graphics not initialized");
    }

    // Mode setting would require complex GPU-specific programming
    // For now, we rely on the bootloader setting up the framebuffer
    Err("Display mode setting not yet implemented")
}
