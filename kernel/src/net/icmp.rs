// ICMP (Internet Control Message Protocol) implementation

use crate::drivers::e1000;
use crate::net::{ethernet, ipv4, arp};
use crate::drivers::serial;

/// ICMP packet structure
pub struct IcmpPacket<'a> {
    pub icmp_type: u8,
    pub code: u8,
    pub checksum: u16,
    pub rest_of_header: [u8; 4],
    pub payload: &'a [u8],
}

/// ICMP types
pub const ICMP_ECHO_REPLY: u8 = 0;
pub const ICMP_ECHO_REQUEST: u8 = 8;

/// Parse an ICMP packet
pub fn parse_packet(data: &[u8]) -> Option<IcmpPacket> {
    if data.len() < 8 {
        return None;
    }

    let icmp_type = data[0];
    let code = data[1];
    let checksum = u16::from_be_bytes([data[2], data[3]]);

    let mut rest_of_header = [0u8; 4];
    rest_of_header.copy_from_slice(&data[4..8]);

    let payload = &data[8..];

    Some(IcmpPacket {
        icmp_type,
        code,
        checksum,
        rest_of_header,
        payload,
    })
}

/// Handle an ICMP packet
pub fn handle_packet(ip_packet: &ipv4::Ipv4Packet, icmp_packet: &IcmpPacket) {
    match icmp_packet.icmp_type {
        ICMP_ECHO_REQUEST => {
            // Respond to ping request
            serial::write_str("ICMP: Received ping request from ");
            for i in 0..4 {
                crate::util::write_hex_u8(ip_packet.source_ip[i]);
                if i < 3 {
                    serial::write_str(".");
                }
            }
            serial::write_str("\n");

            send_echo_reply(
                ip_packet.source_ip,
                icmp_packet.rest_of_header,
                icmp_packet.payload,
            );
        }
        ICMP_ECHO_REPLY => {
            // Received ping reply
            serial::write_str("ICMP: Received ping reply from ");
            for i in 0..4 {
                crate::util::write_hex_u8(ip_packet.source_ip[i]);
                if i < 3 {
                    serial::write_str(".");
                }
            }
            serial::write_str("\n");
        }
        _ => {
            // Other ICMP types, ignore for now
        }
    }
}

/// Send an ICMP echo reply
fn send_echo_reply(dest_ip: [u8; 4], rest_of_header: [u8; 4], payload: &[u8]) {
    let mut icmp_buffer = [0u8; 1500];

    // Type (Echo Reply)
    icmp_buffer[0] = ICMP_ECHO_REPLY;

    // Code
    icmp_buffer[1] = 0;

    // Checksum (calculate later)
    icmp_buffer[2] = 0;
    icmp_buffer[3] = 0;

    // Rest of header (echo the identifier and sequence)
    icmp_buffer[4..8].copy_from_slice(&rest_of_header);

    // Payload
    let payload_len = core::cmp::min(payload.len(), 1500 - 8);
    icmp_buffer[8..8 + payload_len].copy_from_slice(&payload[..payload_len]);

    let icmp_len = 8 + payload_len;

    // Calculate checksum
    let checksum = ipv4::calculate_checksum(&icmp_buffer[..icmp_len]);
    icmp_buffer[2..4].copy_from_slice(&checksum.to_be_bytes());

    // Build IPv4 packet
    let mut ip_buffer = [0u8; 1520];
    let ip_len = ipv4::build_packet(dest_ip, ipv4::PROTOCOL_ICMP, &icmp_buffer[..icmp_len], &mut ip_buffer);

    if ip_len == 0 {
        return;
    }

    // Lookup destination MAC or send ARP request
    let dest_mac = match arp::lookup_mac(dest_ip) {
        Some(mac) => mac,
        None => {
            // Send ARP request and drop this packet
            arp::send_arp_request(dest_ip);
            return;
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
    let _ = e1000::send_packet(&frame_buffer[..frame_len]);
}

/// Send an ICMP echo request (ping)
pub fn send_echo_request(dest_ip: [u8; 4], identifier: u16, sequence: u16) {
    let mut icmp_buffer = [0u8; 64];

    // Type (Echo Request)
    icmp_buffer[0] = ICMP_ECHO_REQUEST;

    // Code
    icmp_buffer[1] = 0;

    // Checksum (calculate later)
    icmp_buffer[2] = 0;
    icmp_buffer[3] = 0;

    // Identifier
    icmp_buffer[4..6].copy_from_slice(&identifier.to_be_bytes());

    // Sequence
    icmp_buffer[6..8].copy_from_slice(&sequence.to_be_bytes());

    // Payload (simple pattern)
    for i in 8..64 {
        icmp_buffer[i] = (i as u8) & 0xFF;
    }

    // Calculate checksum
    let checksum = ipv4::calculate_checksum(&icmp_buffer[..64]);
    icmp_buffer[2..4].copy_from_slice(&checksum.to_be_bytes());

    // Build IPv4 packet
    let mut ip_buffer = [0u8; 84];
    let ip_len = ipv4::build_packet(dest_ip, ipv4::PROTOCOL_ICMP, &icmp_buffer[..64], &mut ip_buffer);

    if ip_len == 0 {
        return;
    }

    // Lookup destination MAC or send ARP request
    let dest_mac = match arp::lookup_mac(dest_ip) {
        Some(mac) => mac,
        None => {
            // Send ARP request first
            arp::send_arp_request(dest_ip);
            return;
        }
    };

    // Build Ethernet frame
    let mut frame_buffer = [0u8; 100];
    let our_mac = e1000::get_mac_address();
    let frame_len = ethernet::build_frame(
        dest_mac,
        our_mac,
        ethernet::ETHERTYPE_IPV4,
        &ip_buffer[..ip_len],
        &mut frame_buffer,
    );

    // Send the frame
    let _ = e1000::send_packet(&frame_buffer[..frame_len]);
}
