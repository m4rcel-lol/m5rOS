# Hardware Support in m5rOS

## Overview

m5rOS includes support for modern hardware through PCI device enumeration and device-specific drivers.

## Supported Hardware

### PCI Bus Enumeration
- Full PCI configuration space access
- Device enumeration on bus 0
- Vendor and device ID detection
- BAR (Base Address Register) reading
- Bus mastering and memory space enablement

### Integrated Graphics
m5rOS supports Intel and AMD integrated graphics processors:

**Intel Graphics:**
- HD Graphics 2000-5600 series
- HD Graphics 520-630 series
- UHD Graphics 630 series
- And many more Intel iGPU models

**AMD Graphics:**
- Radeon HD 8000 series
- Radeon R5/R7 Graphics
- Radeon Vega 3/8 Graphics

The integrated graphics driver provides:
- Automatic PCI device detection
- Framebuffer address and size detection
- Device identification and description
- Memory space and bus mastering enablement

### Wireless Networking

#### Realtek RTL8852BE Wi-Fi 6
- **Standard:** 802.11a/b/g/n/ac/ax (Wi-Fi 6)
- **Configuration:** 2x2 MIMO
- **Maximum Throughput:** 1201 Mbps
- **Features:**
  - Network scanning (skeleton implementation)
  - Connection management (skeleton implementation)
  - Signal strength monitoring
  - State tracking

### Bluetooth

#### Bluetooth 5.3 (RTL8852BE)
- **Version:** Bluetooth 5.3
- **Features:**
  - Bluetooth Low Energy (BLE)
  - Classic Bluetooth
  - Dual mode operation
- **Supported Profiles:** A2DP, HFP, HSP, HID, GATT, GAP
- **Capabilities:**
  - Device scanning (skeleton implementation)
  - Pairing support (skeleton implementation)
  - RSSI monitoring
  - State tracking

## Commands

### Hardware Information
```bash
lspci       # List all PCI devices on the system
gpuinfo     # Display integrated GPU information
```

### Wireless Commands
```bash
wifiinfo    # Display Wi-Fi adapter information and status
wifiscan    # Scan for available Wi-Fi networks
btinfo      # Display Bluetooth adapter information
btscan      # Scan for nearby Bluetooth devices
```

## Implementation Status

### Fully Implemented
- ✅ PCI bus enumeration
- ✅ PCI device detection
- ✅ Integrated graphics detection
- ✅ Wi-Fi adapter detection
- ✅ Bluetooth adapter detection

### Skeleton Implementation
- ⚠️ Wi-Fi network scanning (requires firmware)
- ⚠️ Wi-Fi connection management (requires firmware)
- ⚠️ Bluetooth device scanning (requires firmware and HCI)
- ⚠️ Bluetooth pairing (requires firmware and HCI)
- ⚠️ Data transmission over Wi-Fi/Bluetooth

## Technical Details

### PCI Configuration
The PCI subsystem (`kernel/src/arch/pci.rs`) provides:
- Configuration space access via I/O ports 0xCF8 and 0xCFC
- Device enumeration with multi-function device support
- BAR parsing for memory-mapped and I/O resources
- Vendor/device name lookup tables

### Driver Architecture
All drivers follow a consistent pattern:
1. PCI device detection via vendor/device ID
2. BAR enumeration for resource allocation
3. Device initialization with memory space and bus mastering
4. State management via static atomic flags
5. Command interface for user interaction

### Integrated Graphics Driver
Location: `kernel/src/drivers/igpu.rs`

Detects GPUs by scanning for PCI class 0x03 (Display Controller) with Intel or AMD vendor IDs. Provides device information and framebuffer details.

### Wi-Fi Driver
Location: `kernel/src/drivers/wifi.rs`

Skeleton driver for Realtek RTL8852BE. Implements device detection and state tracking. Full functionality requires:
- Firmware loading from disk
- Hardware register programming
- DMA ring setup
- Interrupt handling
- Integration with network stack

### Bluetooth Driver
Location: `kernel/src/drivers/bluetooth.rs`

Skeleton driver for Bluetooth functionality on RTL8852BE. Requires:
- HCI (Host Controller Interface) implementation
- Firmware loading
- Coexistence management with Wi-Fi
- Profile implementation

## Future Enhancements

### Short Term
- [ ] Load firmware from filesystem
- [ ] Implement hardware register programming
- [ ] Add DMA support
- [ ] Implement interrupt handlers for Wi-Fi/Bluetooth

### Long Term
- [ ] WPA/WPA2/WPA3 authentication
- [ ] Full TCP/IP stack integration with Wi-Fi
- [ ] Bluetooth profile implementations
- [ ] Power management (sleep states)
- [ ] Multiple PCI bus scanning
- [ ] PCIe support
- [ ] MSI/MSI-X interrupt support

## Hardware Requirements

To use the Wi-Fi and Bluetooth features:
- Realtek RTL8852BE wireless card
- Compatible firmware files (not included)
- Sufficient memory for DMA buffers

To use integrated graphics:
- Intel HD/UHD Graphics or AMD Radeon/Vega integrated GPU
- UEFI/BIOS must set up initial framebuffer

## Notes

1. **Firmware Files**: Wi-Fi and Bluetooth drivers require vendor firmware which is not included in m5rOS. These would need to be loaded from the filesystem during initialization.

2. **Hardware Testing**: These drivers are skeleton implementations and have not been fully tested on real hardware. They provide the framework for future development.

3. **Network Stack**: Wi-Fi integration with the existing network stack (Ethernet, ARP, IPv4, ICMP, UDP) is not yet implemented.

4. **Security**: WPA encryption and secure Bluetooth pairing are not yet implemented.

## Contributing

To contribute hardware support:
1. Add new device IDs to the PCI device tables
2. Implement device-specific initialization
3. Follow the existing driver patterns
4. Test on real hardware when possible
5. Document hardware requirements and limitations
