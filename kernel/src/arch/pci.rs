// PCI Configuration Space Access
// Provides PCI bus enumeration and device detection for m5rOS

use crate::arch::port::{inl, outl};

// PCI Configuration Space Ports
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

// PCI Configuration Space Offsets
const PCI_VENDOR_ID: u8 = 0x00;
const PCI_DEVICE_ID: u8 = 0x02;
const PCI_COMMAND: u8 = 0x04;
const PCI_STATUS: u8 = 0x06;
const PCI_REVISION_ID: u8 = 0x08;
const PCI_PROG_IF: u8 = 0x09;
const PCI_SUBCLASS: u8 = 0x0A;
const PCI_CLASS_CODE: u8 = 0x0B;
const PCI_HEADER_TYPE: u8 = 0x0E;
const PCI_BAR0: u8 = 0x10;
const PCI_BAR1: u8 = 0x14;
const PCI_BAR2: u8 = 0x18;
const PCI_BAR3: u8 = 0x1C;
const PCI_BAR4: u8 = 0x20;
const PCI_BAR5: u8 = 0x24;

// PCI Class Codes
pub const PCI_CLASS_DISPLAY: u8 = 0x03;
pub const PCI_CLASS_NETWORK: u8 = 0x02;
pub const PCI_CLASS_BRIDGE: u8 = 0x06;

// PCI Subclass Codes
pub const PCI_SUBCLASS_VGA: u8 = 0x00;
pub const PCI_SUBCLASS_ETHERNET: u8 = 0x00;
pub const PCI_SUBCLASS_WIRELESS: u8 = 0x80;

// Known Vendor IDs
pub const VENDOR_INTEL: u16 = 0x8086;
pub const VENDOR_AMD: u16 = 0x1022;
pub const VENDOR_REALTEK: u16 = 0x10EC;

// Known Device IDs
pub const DEVICE_E1000: u16 = 0x100E;
pub const DEVICE_RTL8852BE: u16 = 0xB852;

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision_id: u8,
    pub header_type: u8,
}

impl PciDevice {
    pub fn read_config_u32(&self, offset: u8) -> u32 {
        read_config_u32(self.bus, self.slot, self.function, offset)
    }

    pub fn read_config_u16(&self, offset: u8) -> u16 {
        let value = self.read_config_u32(offset & 0xFC);
        ((value >> ((offset & 2) * 8)) & 0xFFFF) as u16
    }

    pub fn read_config_u8(&self, offset: u8) -> u8 {
        let value = self.read_config_u32(offset & 0xFC);
        ((value >> ((offset & 3) * 8)) & 0xFF) as u8
    }

    pub fn write_config_u32(&self, offset: u8, value: u32) {
        write_config_u32(self.bus, self.slot, self.function, offset, value);
    }

    pub fn get_bar(&self, bar_index: u8) -> Option<(u64, u32, bool)> {
        if bar_index > 5 {
            return None;
        }

        let offset = PCI_BAR0 + (bar_index * 4);
        let bar_value = self.read_config_u32(offset);

        if bar_value == 0 {
            return None;
        }

        let is_mmio = (bar_value & 0x1) == 0;

        if is_mmio {
            // Memory BAR
            let bar_type = (bar_value >> 1) & 0x3;
            let prefetchable = (bar_value & 0x8) != 0;
            let base_addr = (bar_value & 0xFFFFFFF0) as u64;

            // Get size by writing all 1s and reading back
            self.write_config_u32(offset, 0xFFFFFFFF);
            let size_mask = self.read_config_u32(offset);
            self.write_config_u32(offset, bar_value);

            let size = !(size_mask & 0xFFFFFFF0).wrapping_add(1);

            if bar_type == 2 {
                // 64-bit BAR
                let bar_high = self.read_config_u32(offset + 4);
                let base_addr = base_addr | ((bar_high as u64) << 32);
                Some((base_addr, size, prefetchable))
            } else {
                Some((base_addr, size, prefetchable))
            }
        } else {
            // I/O BAR
            let base_addr = (bar_value & 0xFFFFFFFC) as u64;

            self.write_config_u32(offset, 0xFFFFFFFF);
            let size_mask = self.read_config_u32(offset);
            self.write_config_u32(offset, bar_value);

            let size = !(size_mask & 0xFFFFFFFC).wrapping_add(1);
            Some((base_addr, size, false))
        }
    }

    pub fn enable_bus_mastering(&self) {
        let mut command = self.read_config_u16(PCI_COMMAND);
        command |= 0x4; // Bus Master Enable
        let full_value = self.read_config_u32(PCI_COMMAND & 0xFC);
        let new_value = (full_value & 0xFFFF0000) | (command as u32);
        self.write_config_u32(PCI_COMMAND & 0xFC, new_value);
    }

    pub fn enable_memory_space(&self) {
        let mut command = self.read_config_u16(PCI_COMMAND);
        command |= 0x2; // Memory Space Enable
        let full_value = self.read_config_u32(PCI_COMMAND & 0xFC);
        let new_value = (full_value & 0xFFFF0000) | (command as u32);
        self.write_config_u32(PCI_COMMAND & 0xFC, new_value);
    }

    pub fn enable_io_space(&self) {
        let mut command = self.read_config_u16(PCI_COMMAND);
        command |= 0x1; // I/O Space Enable
        let full_value = self.read_config_u32(PCI_COMMAND & 0xFC);
        let new_value = (full_value & 0xFFFF0000) | (command as u32);
        self.write_config_u32(PCI_COMMAND & 0xFC, new_value);
    }
}

/// Read a 32-bit value from PCI configuration space
pub fn read_config_u32(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = 0x80000000
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        outl(PCI_CONFIG_ADDRESS, address);
        inl(PCI_CONFIG_DATA)
    }
}

/// Write a 32-bit value to PCI configuration space
pub fn write_config_u32(bus: u8, slot: u8, function: u8, offset: u8, value: u32) {
    let address: u32 = 0x80000000
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        outl(PCI_CONFIG_ADDRESS, address);
        outl(PCI_CONFIG_DATA, value);
    }
}

/// Check if a PCI device exists at the given location
fn probe_device(bus: u8, slot: u8, function: u8) -> Option<PciDevice> {
    let vendor_device = read_config_u32(bus, slot, function, PCI_VENDOR_ID);
    let vendor_id = (vendor_device & 0xFFFF) as u16;
    let device_id = ((vendor_device >> 16) & 0xFFFF) as u16;

    // 0xFFFF indicates no device
    if vendor_id == 0xFFFF {
        return None;
    }

    let class_info = read_config_u32(bus, slot, function, PCI_REVISION_ID);
    let revision_id = (class_info & 0xFF) as u8;
    let prog_if = ((class_info >> 8) & 0xFF) as u8;
    let subclass = ((class_info >> 16) & 0xFF) as u8;
    let class_code = ((class_info >> 24) & 0xFF) as u8;

    let header_type = read_config_u32(bus, slot, function, PCI_HEADER_TYPE & 0xFC);
    let header_type = ((header_type >> ((PCI_HEADER_TYPE & 3) * 8)) & 0xFF) as u8;

    Some(PciDevice {
        bus,
        slot,
        function,
        vendor_id,
        device_id,
        class_code,
        subclass,
        prog_if,
        revision_id,
        header_type: header_type & 0x7F,
    })
}

/// Enumerate all PCI devices on the system
/// Returns up to 32 devices
pub fn enumerate_devices() -> [Option<PciDevice>; 32] {
    let mut devices = [None; 32];
    let mut count = 0;

    // Scan bus 0 (we don't scan multiple buses for simplicity)
    for slot in 0..32 {
        if count >= 32 {
            break;
        }

        // Check function 0 first
        if let Some(device) = probe_device(0, slot, 0) {
            devices[count] = Some(device);
            count += 1;

            // If bit 7 of header type is set, it's a multi-function device
            if (device.header_type & 0x80) != 0 {
                // Scan functions 1-7
                for function in 1..8 {
                    if count >= 32 {
                        break;
                    }
                    if let Some(func_device) = probe_device(0, slot, function) {
                        devices[count] = Some(func_device);
                        count += 1;
                    }
                }
            }
        }
    }

    devices
}

/// Find a device by vendor and device ID
pub fn find_device(vendor_id: u16, device_id: u16) -> Option<PciDevice> {
    let devices = enumerate_devices();
    for device in devices.iter().filter_map(|d| *d) {
        if device.vendor_id == vendor_id && device.device_id == device_id {
            return Some(device);
        }
    }
    None
}

/// Find devices by class code
/// Returns up to 16 matching devices
pub fn find_devices_by_class(class_code: u8, subclass: Option<u8>) -> [Option<PciDevice>; 16] {
    let mut result = [None; 16];
    let mut count = 0;

    let devices = enumerate_devices();
    for device in devices.iter().filter_map(|d| *d) {
        if count >= 16 {
            break;
        }
        if device.class_code == class_code
            && (subclass.is_none() || Some(device.subclass) == subclass)
        {
            result[count] = Some(device);
            count += 1;
        }
    }

    result
}

pub fn get_vendor_name(vendor_id: u16) -> &'static str {
    match vendor_id {
        VENDOR_INTEL => "Intel Corporation",
        VENDOR_AMD => "Advanced Micro Devices",
        VENDOR_REALTEK => "Realtek Semiconductor",
        0x1234 => "QEMU",
        0x1AF4 => "Red Hat (VirtIO)",
        0x15AD => "VMware",
        0x1B36 => "Red Hat (QEMU)",
        _ => "Unknown Vendor",
    }
}

pub fn get_class_name(class_code: u8) -> &'static str {
    match class_code {
        0x00 => "Unclassified",
        0x01 => "Mass Storage Controller",
        0x02 => "Network Controller",
        0x03 => "Display Controller",
        0x04 => "Multimedia Controller",
        0x05 => "Memory Controller",
        0x06 => "Bridge Device",
        0x07 => "Communication Controller",
        0x08 => "Generic System Peripheral",
        0x09 => "Input Device",
        0x0A => "Docking Station",
        0x0B => "Processor",
        0x0C => "Serial Bus Controller",
        0x0D => "Wireless Controller",
        0x0E => "Intelligent I/O Controller",
        0x0F => "Satellite Communication Controller",
        0x10 => "Encryption/Decryption Controller",
        0x11 => "Data Acquisition Controller",
        _ => "Unknown Class",
    }
}
