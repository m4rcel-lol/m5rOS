// ARP (Address Resolution Protocol) implementation

use crate::drivers::e1000;
use crate::net::ethernet;

/// ARP packet structure
pub struct ArpPacket<'a> {
    pub hardware_type: u16,
    pub protocol_type: u16,
    pub hardware_size: u8,
    pub protocol_size: u8,
    pub opcode: u16,
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
    pub _marker: core::marker::PhantomData<&'a ()>,
}

/// ARP opcodes
pub const ARP_REQUEST: u16 = 1;
pub const ARP_REPLY: u16 = 2;

/// Hardware types
pub const HW_TYPE_ETHERNET: u16 = 1;

/// ARP cache entry
#[derive(Clone, Copy)]
pub struct ArpCacheEntry {
    ip: [u8; 4],
    mac: [u8; 6],
    valid: bool,
}

/// ARP cache (static allocation for no_std)
static mut ARP_CACHE: [ArpCacheEntry; 16] = [ArpCacheEntry {
    ip: [0, 0, 0, 0],
    mac: [0, 0, 0, 0, 0, 0],
    valid: false,
}; 16];

/// Initialize ARP cache
pub fn init() {
    // Cache is already initialized with zeros
}

/// Parse an ARP packet
pub fn parse_packet(data: &[u8]) -> Option<ArpPacket> {
    if data.len() < 28 {
        return None;
    }

    let hardware_type = u16::from_be_bytes([data[0], data[1]]);
    let protocol_type = u16::from_be_bytes([data[2], data[3]]);
    let hardware_size = data[4];
    let protocol_size = data[5];
    let opcode = u16::from_be_bytes([data[6], data[7]]);

    let mut sender_mac = [0u8; 6];
    let mut sender_ip = [0u8; 4];
    let mut target_mac = [0u8; 6];
    let mut target_ip = [0u8; 4];

    sender_mac.copy_from_slice(&data[8..14]);
    sender_ip.copy_from_slice(&data[14..18]);
    target_mac.copy_from_slice(&data[18..24]);
    target_ip.copy_from_slice(&data[24..28]);

    Some(ArpPacket {
        hardware_type,
        protocol_type,
        hardware_size,
        protocol_size,
        opcode,
        sender_mac,
        sender_ip,
        target_mac,
        target_ip,
        _marker: core::marker::PhantomData,
    })
}

/// Handle an ARP packet
pub fn handle_packet(packet: &ArpPacket) {
    // Update ARP cache with sender information
    add_to_cache(packet.sender_ip, packet.sender_mac);

    if packet.opcode == ARP_REQUEST {
        // Check if the request is for our IP
        let our_ip = crate::net::get_config().ip_addr;
        if ip_equals(&packet.target_ip, &our_ip) {
            // Send ARP reply
            send_arp_reply(packet.sender_mac, packet.sender_ip);
        }
    }
}

/// Send an ARP reply
fn send_arp_reply(dest_mac: [u8; 6], dest_ip: [u8; 4]) {
    let our_mac = e1000::get_mac_address();
    let our_ip = crate::net::get_config().ip_addr;

    let mut arp_buffer = [0u8; 28];

    // Hardware type (Ethernet)
    arp_buffer[0..2].copy_from_slice(&HW_TYPE_ETHERNET.to_be_bytes());

    // Protocol type (IPv4)
    arp_buffer[2..4].copy_from_slice(&ethernet::ETHERTYPE_IPV4.to_be_bytes());

    // Hardware size
    arp_buffer[4] = 6;

    // Protocol size
    arp_buffer[5] = 4;

    // Opcode (Reply)
    arp_buffer[6..8].copy_from_slice(&ARP_REPLY.to_be_bytes());

    // Sender MAC (us)
    arp_buffer[8..14].copy_from_slice(&our_mac);

    // Sender IP (us)
    arp_buffer[14..18].copy_from_slice(&our_ip);

    // Target MAC
    arp_buffer[18..24].copy_from_slice(&dest_mac);

    // Target IP
    arp_buffer[24..28].copy_from_slice(&dest_ip);

    // Build Ethernet frame
    let mut frame_buffer = [0u8; 64];
    let frame_len = ethernet::build_frame(
        dest_mac,
        our_mac,
        ethernet::ETHERTYPE_ARP,
        &arp_buffer,
        &mut frame_buffer,
    );

    // Send the frame
    let _ = e1000::send_packet(&frame_buffer[..frame_len]);
}

/// Send an ARP request
pub fn send_arp_request(target_ip: [u8; 4]) {
    let our_mac = e1000::get_mac_address();
    let our_ip = crate::net::get_config().ip_addr;

    let mut arp_buffer = [0u8; 28];

    // Hardware type (Ethernet)
    arp_buffer[0..2].copy_from_slice(&HW_TYPE_ETHERNET.to_be_bytes());

    // Protocol type (IPv4)
    arp_buffer[2..4].copy_from_slice(&ethernet::ETHERTYPE_IPV4.to_be_bytes());

    // Hardware size
    arp_buffer[4] = 6;

    // Protocol size
    arp_buffer[5] = 4;

    // Opcode (Request)
    arp_buffer[6..8].copy_from_slice(&ARP_REQUEST.to_be_bytes());

    // Sender MAC (us)
    arp_buffer[8..14].copy_from_slice(&our_mac);

    // Sender IP (us)
    arp_buffer[14..18].copy_from_slice(&our_ip);

    // Target MAC (unknown, set to 0)
    arp_buffer[18..24].copy_from_slice(&[0, 0, 0, 0, 0, 0]);

    // Target IP
    arp_buffer[24..28].copy_from_slice(&target_ip);

    // Build Ethernet frame (broadcast)
    let mut frame_buffer = [0u8; 64];
    let frame_len = ethernet::build_frame(
        ethernet::BROADCAST_MAC,
        our_mac,
        ethernet::ETHERTYPE_ARP,
        &arp_buffer,
        &mut frame_buffer,
    );

    // Send the frame
    let _ = e1000::send_packet(&frame_buffer[..frame_len]);
}

/// Add an entry to the ARP cache
fn add_to_cache(ip: [u8; 4], mac: [u8; 6]) {
    unsafe {
        // Try to find existing entry or empty slot
        for entry in ARP_CACHE.iter_mut() {
            if !entry.valid || ip_equals(&entry.ip, &ip) {
                entry.ip = ip;
                entry.mac = mac;
                entry.valid = true;
                return;
            }
        }

        // Cache is full, replace first entry (simple FIFO)
        ARP_CACHE[0].ip = ip;
        ARP_CACHE[0].mac = mac;
        ARP_CACHE[0].valid = true;
    }
}

/// Lookup MAC address in ARP cache
pub fn lookup_mac(ip: [u8; 4]) -> Option<[u8; 6]> {
    unsafe {
        for entry in ARP_CACHE.iter() {
            if entry.valid && ip_equals(&entry.ip, &ip) {
                return Some(entry.mac);
            }
        }
    }
    None
}

/// Compare IP addresses
fn ip_equals(a: &[u8; 4], b: &[u8; 4]) -> bool {
    a[0] == b[0] && a[1] == b[1] && a[2] == b[2] && a[3] == b[3]
}

/// Get ARP cache entries for display
pub fn get_cache_entries() -> &'static [ArpCacheEntry; 16] {
    unsafe { &ARP_CACHE }
}

impl ArpCacheEntry {
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_ip(&self) -> [u8; 4] {
        self.ip
    }

    pub fn get_mac(&self) -> [u8; 6] {
        self.mac
    }
}
