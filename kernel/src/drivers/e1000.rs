// Intel E1000 Network Card Driver (82540EM)
//
// This driver supports the Intel E1000 (82540EM) Gigabit Ethernet controller,
// which is commonly emulated by QEMU and VirtualBox.

use crate::arch::port::{inl, outl};
use core::sync::atomic::{AtomicBool, Ordering};

/// E1000 device I/O base address (to be set during initialization)
static mut IO_BASE: u16 = 0;

/// Whether the E1000 device has been initialized
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// MAC address of the network interface
static mut MAC_ADDRESS: [u8; 6] = [0; 6];

// E1000 Register offsets
const REG_CTRL: u32 = 0x0000;      // Device Control
const REG_STATUS: u32 = 0x0008;    // Device Status
const REG_EEPROM: u32 = 0x0014;    // EEPROM Read
const REG_CTRL_EXT: u32 = 0x0018;  // Extended Device Control
const REG_ICR: u32 = 0x00C0;       // Interrupt Cause Read
const REG_IMS: u32 = 0x00D0;       // Interrupt Mask Set/Read
const REG_RCTL: u32 = 0x0100;      // Receive Control
const REG_TCTL: u32 = 0x0400;      // Transmit Control
const REG_RDBAL: u32 = 0x2800;     // RX Descriptor Base Low
const REG_RDBAH: u32 = 0x2804;     // RX Descriptor Base High
const REG_RDLEN: u32 = 0x2808;     // RX Descriptor Length
const REG_RDH: u32 = 0x2810;       // RX Descriptor Head
const REG_RDT: u32 = 0x2818;       // RX Descriptor Tail
const REG_TDBAL: u32 = 0x3800;     // TX Descriptor Base Low
const REG_TDBAH: u32 = 0x3804;     // TX Descriptor Base High
const REG_TDLEN: u32 = 0x3808;     // TX Descriptor Length
const REG_TDH: u32 = 0x3810;       // TX Descriptor Head
const REG_TDT: u32 = 0x3818;       // TX Descriptor Tail
const REG_MTA: u32 = 0x5200;       // Multicast Table Array

// Control Register Bits
const CTRL_RST: u32 = 1 << 26;     // Device Reset
const CTRL_SLU: u32 = 1 << 6;      // Set Link Up

// EEPROM Register Bits
const EEPROM_START: u32 = 1 << 0;  // Start Read
const EEPROM_DONE: u32 = 1 << 4;   // Read Done

// Receive Control Register Bits
const RCTL_EN: u32 = 1 << 1;       // Receiver Enable
const RCTL_SBP: u32 = 1 << 2;      // Store Bad Packets
const RCTL_UPE: u32 = 1 << 3;      // Unicast Promiscuous Enabled
const RCTL_MPE: u32 = 1 << 4;      // Multicast Promiscuous Enabled
const RCTL_BAM: u32 = 1 << 15;     // Broadcast Accept Mode
const RCTL_BSIZE_8192: u32 = 2 << 16; // Buffer Size 8192
const RCTL_SECRC: u32 = 1 << 26;   // Strip Ethernet CRC

// Transmit Control Register Bits
const TCTL_EN: u32 = 1 << 1;       // Transmit Enable
const TCTL_PSP: u32 = 1 << 3;      // Pad Short Packets

// Descriptor status bits
const DESC_STATUS_DD: u8 = 1 << 0;  // Descriptor Done
const DESC_STATUS_EOP: u8 = 1 << 1; // End of Packet

/// Receive Descriptor
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct RxDescriptor {
    addr: u64,       // Buffer address
    length: u16,     // Length of data
    checksum: u16,   // Packet checksum
    status: u8,      // Descriptor status
    errors: u8,      // Descriptor errors
    special: u16,    // VLAN info
}

/// Transmit Descriptor
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct TxDescriptor {
    addr: u64,       // Buffer address
    length: u16,     // Length of data
    cso: u8,         // Checksum offset
    cmd: u8,         // Descriptor command
    status: u8,      // Descriptor status
    css: u8,         // Checksum start
    special: u16,    // VLAN info
}

// Descriptor command bits
const CMD_EOP: u8 = 1 << 0;  // End of Packet
const CMD_RS: u8 = 1 << 3;   // Report Status

/// Number of receive descriptors
const RX_DESC_COUNT: usize = 32;

/// Number of transmit descriptors
const TX_DESC_COUNT: usize = 16;

/// Size of each receive buffer (8KB)
const RX_BUFFER_SIZE: usize = 8192;

/// Receive descriptors (statically allocated)
static mut RX_DESCS: [RxDescriptor; RX_DESC_COUNT] = [RxDescriptor {
    addr: 0,
    length: 0,
    checksum: 0,
    status: 0,
    errors: 0,
    special: 0,
}; RX_DESC_COUNT];

/// Transmit descriptors (statically allocated)
static mut TX_DESCS: [TxDescriptor; TX_DESC_COUNT] = [TxDescriptor {
    addr: 0,
    length: 0,
    cso: 0,
    cmd: 0,
    status: 0,
    css: 0,
    special: 0,
}; TX_DESC_COUNT];

/// Receive buffers (statically allocated - 32 * 8KB = 256KB)
static mut RX_BUFFERS: [[u8; RX_BUFFER_SIZE]; RX_DESC_COUNT] = [[0; RX_BUFFER_SIZE]; RX_DESC_COUNT];

/// Transmit buffers (statically allocated - 16 * 8KB = 128KB)
static mut TX_BUFFERS: [[u8; RX_BUFFER_SIZE]; TX_DESC_COUNT] = [[0; RX_BUFFER_SIZE]; TX_DESC_COUNT];

/// Current RX tail pointer
static mut RX_TAIL: usize = 0;

/// Current TX tail pointer
static mut TX_TAIL: usize = 0;

/// Read a 32-bit value from E1000 register
unsafe fn read_reg(reg: u32) -> u32 {
    inl(IO_BASE + (reg as u16))
}

/// Write a 32-bit value to E1000 register
unsafe fn write_reg(reg: u32, value: u32) {
    outl(IO_BASE + (reg as u16), value);
}

/// Read EEPROM word
unsafe fn read_eeprom(addr: u8) -> u16 {
    // Start EEPROM read
    write_reg(REG_EEPROM, EEPROM_START | ((addr as u32) << 8));

    // Wait for read to complete
    let mut timeout = 1000;
    while timeout > 0 {
        let value = read_reg(REG_EEPROM);
        if value & EEPROM_DONE != 0 {
            return ((value >> 16) & 0xFFFF) as u16;
        }
        timeout -= 1;
    }

    0xFFFF // Timeout
}

/// Initialize the E1000 network card
///
/// # Safety
/// This function must be called only once during kernel initialization
pub unsafe fn init(io_base: u16) -> Result<(), &'static str> {
    use crate::drivers::serial;

    IO_BASE = io_base;

    serial::write_str("E1000: Initializing at I/O base 0x");
    crate::util::write_hex_u16(io_base);
    serial::write_str("\n");

    // Reset the device
    write_reg(REG_CTRL, read_reg(REG_CTRL) | CTRL_RST);

    // Wait for reset to complete (small delay)
    for _ in 0..1000 {
        core::arch::asm!("pause");
    }

    // Read MAC address from EEPROM
    let mac_word0 = read_eeprom(0);
    let mac_word1 = read_eeprom(1);
    let mac_word2 = read_eeprom(2);

    MAC_ADDRESS[0] = (mac_word0 & 0xFF) as u8;
    MAC_ADDRESS[1] = ((mac_word0 >> 8) & 0xFF) as u8;
    MAC_ADDRESS[2] = (mac_word1 & 0xFF) as u8;
    MAC_ADDRESS[3] = ((mac_word1 >> 8) & 0xFF) as u8;
    MAC_ADDRESS[4] = (mac_word2 & 0xFF) as u8;
    MAC_ADDRESS[5] = ((mac_word2 >> 8) & 0xFF) as u8;

    serial::write_str("E1000: MAC address: ");
    for i in 0..6 {
        crate::util::write_hex_u8(MAC_ADDRESS[i]);
        if i < 5 {
            serial::write_str(":");
        }
    }
    serial::write_str("\n");

    // Clear multicast table array
    for i in 0..128 {
        write_reg(REG_MTA + (i * 4), 0);
    }

    // Initialize receive descriptors
    for i in 0..RX_DESC_COUNT {
        RX_DESCS[i].addr = RX_BUFFERS[i].as_ptr() as u64;
        RX_DESCS[i].status = 0;
    }

    // Initialize transmit descriptors
    for i in 0..TX_DESC_COUNT {
        TX_DESCS[i].addr = TX_BUFFERS[i].as_ptr() as u64;
        TX_DESCS[i].status = DESC_STATUS_DD; // Mark as done initially
        TX_DESCS[i].cmd = 0;
    }

    // Set up receive descriptor ring
    let rx_desc_addr = RX_DESCS.as_ptr() as u64;
    write_reg(REG_RDBAL, (rx_desc_addr & 0xFFFFFFFF) as u32);
    write_reg(REG_RDBAH, ((rx_desc_addr >> 32) & 0xFFFFFFFF) as u32);
    write_reg(REG_RDLEN, (RX_DESC_COUNT * 16) as u32);
    write_reg(REG_RDH, 0);
    write_reg(REG_RDT, (RX_DESC_COUNT - 1) as u32);

    RX_TAIL = RX_DESC_COUNT - 1;

    // Set up transmit descriptor ring
    let tx_desc_addr = TX_DESCS.as_ptr() as u64;
    write_reg(REG_TDBAL, (tx_desc_addr & 0xFFFFFFFF) as u32);
    write_reg(REG_TDBAH, ((tx_desc_addr >> 32) & 0xFFFFFFFF) as u32);
    write_reg(REG_TDLEN, (TX_DESC_COUNT * 16) as u32);
    write_reg(REG_TDH, 0);
    write_reg(REG_TDT, 0);

    TX_TAIL = 0;

    // Enable receiver
    write_reg(REG_RCTL, RCTL_EN | RCTL_SBP | RCTL_UPE | RCTL_MPE |
              RCTL_BAM | RCTL_BSIZE_8192 | RCTL_SECRC);

    // Enable transmitter
    write_reg(REG_TCTL, TCTL_EN | TCTL_PSP | (0x10 << 4)); // Collision threshold = 16

    // Set link up
    write_reg(REG_CTRL, read_reg(REG_CTRL) | CTRL_SLU);

    INITIALIZED.store(true, Ordering::Release);

    serial::write_str("E1000: Initialized successfully\n");

    Ok(())
}

/// Check if the E1000 driver is initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}

/// Get the MAC address
pub fn get_mac_address() -> [u8; 6] {
    unsafe { MAC_ADDRESS }
}

/// Receive a packet
///
/// Returns the packet data and length if a packet is available
pub fn receive_packet() -> Option<&'static [u8]> {
    if !is_initialized() {
        return None;
    }

    unsafe {
        let rx_head = read_reg(REG_RDH) as usize;
        let rx_next = (RX_TAIL + 1) % RX_DESC_COUNT;

        if rx_next == rx_head {
            // No packets available
            return None;
        }

        let desc = &RX_DESCS[rx_next];

        // Check if descriptor has data
        if desc.status & DESC_STATUS_DD == 0 {
            return None;
        }

        let length = desc.length as usize;
        if length > 0 && length <= RX_BUFFER_SIZE {
            let packet = &RX_BUFFERS[rx_next][..length];

            // Mark descriptor as available again
            RX_DESCS[rx_next].status = 0;

            // Update tail pointer
            RX_TAIL = rx_next;
            write_reg(REG_RDT, RX_TAIL as u32);

            return Some(packet);
        }

        None
    }
}

/// Send a packet
///
/// Returns Ok(()) if the packet was queued successfully
pub fn send_packet(data: &[u8]) -> Result<(), &'static str> {
    if !is_initialized() {
        return Err("E1000 not initialized");
    }

    if data.len() == 0 || data.len() > RX_BUFFER_SIZE {
        return Err("Invalid packet size");
    }

    unsafe {
        let tx_head = read_reg(REG_TDH) as usize;
        let tx_next = (TX_TAIL + 1) % TX_DESC_COUNT;

        // Check if queue is full
        if tx_next == tx_head {
            return Err("Transmit queue full");
        }

        // Copy data to buffer
        let buffer = &mut TX_BUFFERS[TX_TAIL];
        buffer[..data.len()].copy_from_slice(data);

        // Set up descriptor
        TX_DESCS[TX_TAIL].length = data.len() as u16;
        TX_DESCS[TX_TAIL].cmd = CMD_EOP | CMD_RS;
        TX_DESCS[TX_TAIL].status = 0;

        // Update tail pointer
        TX_TAIL = tx_next;
        write_reg(REG_TDT, TX_TAIL as u32);

        Ok(())
    }
}

/// Get device status
pub fn get_status() -> u32 {
    if !is_initialized() {
        return 0;
    }

    unsafe { read_reg(REG_STATUS) }
}

/// Check if link is up
pub fn is_link_up() -> bool {
    if !is_initialized() {
        return false;
    }

    let status = get_status();
    status & (1 << 1) != 0  // Link up bit
}
