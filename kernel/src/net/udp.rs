// UDP (User Datagram Protocol) implementation

/// UDP packet structure
pub struct UdpPacket<'a> {
    pub source_port: u16,
    pub dest_port: u16,
    pub length: u16,
    pub checksum: u16,
    pub payload: &'a [u8],
}

/// Parse a UDP packet
pub fn parse_packet(data: &[u8]) -> Option<UdpPacket> {
    if data.len() < 8 {
        return None;
    }

    let source_port = u16::from_be_bytes([data[0], data[1]]);
    let dest_port = u16::from_be_bytes([data[2], data[3]]);
    let length = u16::from_be_bytes([data[4], data[5]]);
    let checksum = u16::from_be_bytes([data[6], data[7]]);

    let payload = &data[8..];

    Some(UdpPacket {
        source_port,
        dest_port,
        length,
        checksum,
        payload,
    })
}

/// Handle a UDP packet
pub fn handle_packet(ip_packet: &crate::net::ipv4::Ipv4Packet, udp_packet: &UdpPacket) {
    use crate::drivers::serial;

    // For now, just log UDP packets
    serial::write_str("UDP: Received packet from ");
    for i in 0..4 {
        crate::util::write_hex_u8(ip_packet.source_ip[i]);
        if i < 3 {
            serial::write_str(".");
        }
    }
    serial::write_str(":");
    crate::util::write_hex_u16(udp_packet.source_port);
    serial::write_str(" to port ");
    crate::util::write_hex_u16(udp_packet.dest_port);
    serial::write_str("\n");

    // Future: Add port listeners and dispatch to handlers
}

/// Build a UDP packet
pub fn build_packet(
    source_port: u16,
    dest_port: u16,
    payload: &[u8],
    buffer: &mut [u8],
) -> usize {
    let length = 8 + payload.len();
    if buffer.len() < length {
        return 0;
    }

    // Source port
    buffer[0..2].copy_from_slice(&source_port.to_be_bytes());

    // Destination port
    buffer[2..4].copy_from_slice(&dest_port.to_be_bytes());

    // Length
    buffer[4..6].copy_from_slice(&(length as u16).to_be_bytes());

    // Checksum (set to 0 for now - optional in IPv4)
    buffer[6] = 0;
    buffer[7] = 0;

    // Payload
    buffer[8..8 + payload.len()].copy_from_slice(payload);

    length
}

/// Send a UDP packet
pub fn send_packet(
    dest_ip: [u8; 4],
    source_port: u16,
    dest_port: u16,
    payload: &[u8],
) -> Result<(), &'static str> {
    use crate::drivers::e1000;
    use crate::net::{ethernet, ipv4, arp};

    let mut udp_buffer = [0u8; 1500];
    let udp_len = build_packet(source_port, dest_port, payload, &mut udp_buffer);

    if udp_len == 0 {
        return Err("UDP packet too large");
    }

    // Build IPv4 packet
    let mut ip_buffer = [0u8; 1520];
    let ip_len = ipv4::build_packet(dest_ip, ipv4::PROTOCOL_UDP, &udp_buffer[..udp_len], &mut ip_buffer);

    if ip_len == 0 {
        return Err("Failed to build IP packet");
    }

    // Lookup destination MAC
    let dest_mac = match arp::lookup_mac(dest_ip) {
        Some(mac) => mac,
        None => {
            // Send ARP request and return error
            arp::send_arp_request(dest_ip);
            return Err("Destination MAC unknown, ARP request sent");
        }
    };

    // Build Ethernet frame
    let mut frame_buffer = [0u8; 1536];
    let our_mac = e1000::get_mac_address();
    let frame_len = ethernet::build_frame(
        dest_mac,
        our_mac,
        ethernet::ETHERTYPE_IPV4,
        &ip_buffer[..ip_len],
        &mut frame_buffer,
    );

    // Send the frame
    e1000::send_packet(&frame_buffer[..frame_len])
}
