// Ethernet frame handling

/// Ethernet frame structure
pub struct EthernetFrame<'a> {
    pub dest_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,
    pub payload: &'a [u8],
}

/// Ethernet protocol types
pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16 = 0x0806;
pub const ETHERTYPE_IPV6: u16 = 0x86DD;

/// Parse an Ethernet frame
pub fn parse_frame(data: &[u8]) -> Option<EthernetFrame> {
    if data.len() < 14 {
        return None;
    }

    let mut dest_mac = [0u8; 6];
    let mut src_mac = [0u8; 6];

    dest_mac.copy_from_slice(&data[0..6]);
    src_mac.copy_from_slice(&data[6..12]);

    let ethertype = u16::from_be_bytes([data[12], data[13]]);

    Some(EthernetFrame {
        dest_mac,
        src_mac,
        ethertype,
        payload: &data[14..],
    })
}

/// Build an Ethernet frame
pub fn build_frame(dest_mac: [u8; 6], src_mac: [u8; 6], ethertype: u16, payload: &[u8], buffer: &mut [u8]) -> usize {
    if buffer.len() < 14 + payload.len() {
        return 0;
    }

    // Copy destination MAC
    buffer[0..6].copy_from_slice(&dest_mac);

    // Copy source MAC
    buffer[6..12].copy_from_slice(&src_mac);

    // Set ethertype
    buffer[12..14].copy_from_slice(&ethertype.to_be_bytes());

    // Copy payload
    buffer[14..14 + payload.len()].copy_from_slice(payload);

    14 + payload.len()
}

/// Broadcast MAC address
pub const BROADCAST_MAC: [u8; 6] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

/// Compare MAC addresses
pub fn mac_equals(a: &[u8; 6], b: &[u8; 6]) -> bool {
    a[0] == b[0] && a[1] == b[1] && a[2] == b[2] &&
    a[3] == b[3] && a[4] == b[4] && a[5] == b[5]
}
