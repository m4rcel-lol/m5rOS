// IPv4 protocol implementation

/// IPv4 packet structure
pub struct Ipv4Packet<'a> {
    pub version: u8,
    pub ihl: u8,
    pub dscp: u8,
    pub ecn: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags: u8,
    pub fragment_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub source_ip: [u8; 4],
    pub dest_ip: [u8; 4],
    pub payload: &'a [u8],
}

/// IP protocols
pub const PROTOCOL_ICMP: u8 = 1;
pub const PROTOCOL_TCP: u8 = 6;
pub const PROTOCOL_UDP: u8 = 17;

/// Parse an IPv4 packet
pub fn parse_packet(data: &[u8]) -> Option<Ipv4Packet> {
    if data.len() < 20 {
        return None;
    }

    let version = (data[0] >> 4) & 0x0F;
    let ihl = data[0] & 0x0F;

    if version != 4 {
        return None; // Not IPv4
    }

    let header_len = (ihl as usize) * 4;
    if data.len() < header_len {
        return None;
    }

    let dscp = (data[1] >> 2) & 0x3F;
    let ecn = data[1] & 0x03;
    let total_length = u16::from_be_bytes([data[2], data[3]]);
    let identification = u16::from_be_bytes([data[4], data[5]]);
    let flags = (data[6] >> 5) & 0x07;
    let fragment_offset = u16::from_be_bytes([data[6] & 0x1F, data[7]]);
    let ttl = data[8];
    let protocol = data[9];
    let checksum = u16::from_be_bytes([data[10], data[11]]);

    let mut source_ip = [0u8; 4];
    let mut dest_ip = [0u8; 4];

    source_ip.copy_from_slice(&data[12..16]);
    dest_ip.copy_from_slice(&data[16..20]);

    let payload = &data[header_len..];

    Some(Ipv4Packet {
        version,
        ihl,
        dscp,
        ecn,
        total_length,
        identification,
        flags,
        fragment_offset,
        ttl,
        protocol,
        checksum,
        source_ip,
        dest_ip,
        payload,
    })
}

/// Build an IPv4 packet
pub fn build_packet(
    dest_ip: [u8; 4],
    protocol: u8,
    payload: &[u8],
    buffer: &mut [u8],
) -> usize {
    let our_ip = crate::net::get_config().ip_addr;

    let total_length = 20 + payload.len();
    if buffer.len() < total_length {
        return 0;
    }

    // Version and IHL
    buffer[0] = (4 << 4) | 5; // Version 4, IHL 5 (20 bytes)

    // DSCP and ECN
    buffer[1] = 0;

    // Total length
    buffer[2..4].copy_from_slice(&(total_length as u16).to_be_bytes());

    // Identification
    buffer[4..6].copy_from_slice(&0u16.to_be_bytes());

    // Flags and fragment offset
    buffer[6] = 0x40; // Don't fragment
    buffer[7] = 0;

    // TTL
    buffer[8] = 64;

    // Protocol
    buffer[9] = protocol;

    // Checksum (set to 0 for calculation)
    buffer[10] = 0;
    buffer[11] = 0;

    // Source IP
    buffer[12..16].copy_from_slice(&our_ip);

    // Destination IP
    buffer[16..20].copy_from_slice(&dest_ip);

    // Calculate checksum
    let checksum = calculate_checksum(&buffer[..20]);
    buffer[10..12].copy_from_slice(&checksum.to_be_bytes());

    // Copy payload
    buffer[20..20 + payload.len()].copy_from_slice(payload);

    total_length
}

/// Calculate IPv4 checksum
pub fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // Sum all 16-bit words
    for i in (0..data.len()).step_by(2) {
        if i + 1 < data.len() {
            let word = u16::from_be_bytes([data[i], data[i + 1]]);
            sum += word as u32;
        } else {
            // Odd number of bytes, pad with 0
            sum += (data[i] as u32) << 8;
        }
    }

    // Add carry bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    // One's complement
    !sum as u16
}

/// Compare IP addresses
pub fn ip_equals(a: &[u8; 4], b: &[u8; 4]) -> bool {
    a[0] == b[0] && a[1] == b[1] && a[2] == b[2] && a[3] == b[3]
}
