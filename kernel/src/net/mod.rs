// Network stack modules

pub mod ethernet;
pub mod arp;
pub mod ipv4;
pub mod icmp;
pub mod udp;

use crate::drivers::e1000;

/// Network interface configuration
static mut NET_CONFIG: NetConfig = NetConfig {
    ip_addr: [0, 0, 0, 0],
    subnet_mask: [0, 0, 0, 0],
    gateway: [0, 0, 0, 0],
    configured: false,
};

/// Network configuration
pub struct NetConfig {
    pub ip_addr: [u8; 4],
    pub subnet_mask: [u8; 4],
    pub gateway: [u8; 4],
    pub configured: bool,
}

/// Initialize network stack
pub fn init() {
    use crate::drivers::serial;
    serial::write_str("Network: Initializing stack\n");

    // Initialize ARP cache
    arp::init();

    serial::write_str("Network: Stack initialized\n");
}

/// Configure network interface
pub fn configure(ip: [u8; 4], netmask: [u8; 4], gateway: [u8; 4]) {
    unsafe {
        NET_CONFIG.ip_addr = ip;
        NET_CONFIG.subnet_mask = netmask;
        NET_CONFIG.gateway = gateway;
        NET_CONFIG.configured = true;
    }
}

/// Get network configuration
pub fn get_config() -> &'static NetConfig {
    unsafe { &NET_CONFIG }
}

/// Process incoming packets
pub fn process_packets() {
    while let Some(packet) = e1000::receive_packet() {
        // Parse Ethernet frame
        if let Some(eth_frame) = ethernet::parse_frame(packet) {
            match eth_frame.ethertype {
                ethernet::ETHERTYPE_ARP => {
                    // Process ARP packet
                    if let Some(arp_packet) = arp::parse_packet(eth_frame.payload) {
                        arp::handle_packet(&arp_packet);
                    }
                }
                ethernet::ETHERTYPE_IPV4 => {
                    // Process IPv4 packet
                    if let Some(ipv4_packet) = ipv4::parse_packet(eth_frame.payload) {
                        match ipv4_packet.protocol {
                            ipv4::PROTOCOL_ICMP => {
                                // Process ICMP packet
                                if let Some(icmp_packet) = icmp::parse_packet(ipv4_packet.payload) {
                                    icmp::handle_packet(&ipv4_packet, &icmp_packet);
                                }
                            }
                            ipv4::PROTOCOL_UDP => {
                                // Process UDP packet
                                if let Some(udp_packet) = udp::parse_packet(ipv4_packet.payload) {
                                    udp::handle_packet(&ipv4_packet, &udp_packet);
                                }
                            }
                            _ => {
                                // Unknown protocol, ignore
                            }
                        }
                    }
                }
                _ => {
                    // Unknown ethertype, ignore
                }
            }
        }
    }
}
