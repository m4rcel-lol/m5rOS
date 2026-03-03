// ATA PIO disk driver
//
// Basic ATA Programmed I/O driver for IDE hard drives

use crate::arch::port::{inb, outb, insw};

/// ATA primary bus I/O ports
const ATA_PRIMARY_DATA: u16 = 0x1F0;
const ATA_PRIMARY_ERROR: u16 = 0x1F1;
const ATA_PRIMARY_SECTOR_COUNT: u16 = 0x1F2;
const ATA_PRIMARY_LBA_LOW: u16 = 0x1F3;
const ATA_PRIMARY_LBA_MID: u16 = 0x1F4;
const ATA_PRIMARY_LBA_HIGH: u16 = 0x1F5;
const ATA_PRIMARY_DRIVE_SELECT: u16 = 0x1F6;
const ATA_PRIMARY_STATUS: u16 = 0x1F7;
const ATA_PRIMARY_COMMAND: u16 = 0x1F7;

/// ATA status register bits
const ATA_STATUS_BSY: u8 = 0x80; // Busy
const ATA_STATUS_DRDY: u8 = 0x40; // Drive ready
const ATA_STATUS_DRQ: u8 = 0x08; // Data request ready
const ATA_STATUS_ERR: u8 = 0x01; // Error

/// ATA commands
const ATA_CMD_READ_SECTORS: u8 = 0x20;
const ATA_CMD_WRITE_SECTORS: u8 = 0x30;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

/// ATA drive information
#[derive(Debug, Clone, Copy)]
pub struct AtaDriveInfo {
    pub exists: bool,
    pub sectors: u64,
    pub model: [u8; 40],
}

impl AtaDriveInfo {
    pub const fn new() -> Self {
        AtaDriveInfo {
            exists: false,
            sectors: 0,
            model: [0; 40],
        }
    }
}

/// Wait for ATA drive to be ready
///
/// # Safety
/// Must ensure drive exists and is properly initialized
unsafe fn wait_for_ready() -> Result<(), &'static str> {
    for _ in 0..100000 {
        let status = inb(ATA_PRIMARY_STATUS);
        if (status & ATA_STATUS_BSY) == 0 && (status & ATA_STATUS_DRDY) != 0 {
            return Ok(());
        }
    }
    Err("ATA timeout waiting for ready")
}

/// Wait for ATA drive to be ready for data transfer
///
/// # Safety
/// Must ensure drive exists and is properly initialized
unsafe fn wait_for_drq() -> Result<(), &'static str> {
    for _ in 0..100000 {
        let status = inb(ATA_PRIMARY_STATUS);
        if (status & ATA_STATUS_BSY) == 0 {
            if (status & ATA_STATUS_DRQ) != 0 {
                return Ok(());
            }
            if (status & ATA_STATUS_ERR) != 0 {
                return Err("ATA error during data transfer");
            }
        }
    }
    Err("ATA timeout waiting for DRQ")
}

/// Identify ATA drive
///
/// # Safety
/// Performs hardware I/O operations
pub unsafe fn identify_drive() -> Result<AtaDriveInfo, &'static str> {
    let mut info = AtaDriveInfo::new();

    // Select drive 0
    outb(ATA_PRIMARY_DRIVE_SELECT, 0xA0);

    // Set sector count and LBA to 0
    outb(ATA_PRIMARY_SECTOR_COUNT, 0);
    outb(ATA_PRIMARY_LBA_LOW, 0);
    outb(ATA_PRIMARY_LBA_MID, 0);
    outb(ATA_PRIMARY_LBA_HIGH, 0);

    // Send IDENTIFY command
    outb(ATA_PRIMARY_COMMAND, ATA_CMD_IDENTIFY);

    // Check if drive exists
    let status = inb(ATA_PRIMARY_STATUS);
    if status == 0 {
        return Err("No ATA drive present");
    }

    // Wait for BSY to clear
    wait_for_ready()?;

    // Check LBA mid and high - should be 0 for ATA
    let lba_mid = inb(ATA_PRIMARY_LBA_MID);
    let lba_high = inb(ATA_PRIMARY_LBA_HIGH);
    if lba_mid != 0 || lba_high != 0 {
        return Err("Not an ATA device");
    }

    // Wait for DRQ
    wait_for_drq()?;

    // Read 256 words (512 bytes) of identification data
    let mut buffer = [0u16; 256];
    for i in 0..256 {
        buffer[i] = insw(ATA_PRIMARY_DATA);
    }

    info.exists = true;

    // Extract model string (words 27-46)
    for i in 0..20 {
        let word = buffer[27 + i];
        info.model[i * 2] = (word >> 8) as u8;
        info.model[i * 2 + 1] = (word & 0xFF) as u8;
    }

    // Extract sector count (words 60-61 for 28-bit LBA)
    info.sectors = (buffer[60] as u64) | ((buffer[61] as u64) << 16);

    Ok(info)
}

/// Read sectors from ATA drive
///
/// # Safety
/// Performs hardware I/O operations
/// Buffer must be large enough to hold sector_count * 512 bytes
pub unsafe fn read_sectors(
    lba: u32,
    sector_count: u8,
    buffer: &mut [u16],
) -> Result<(), &'static str> {
    if buffer.len() < (sector_count as usize * 256) {
        return Err("Buffer too small");
    }

    wait_for_ready()?;

    // Select drive 0 with LBA mode
    outb(ATA_PRIMARY_DRIVE_SELECT, 0xE0 | ((lba >> 24) & 0x0F) as u8);

    // Set sector count and LBA
    outb(ATA_PRIMARY_SECTOR_COUNT, sector_count);
    outb(ATA_PRIMARY_LBA_LOW, (lba & 0xFF) as u8);
    outb(ATA_PRIMARY_LBA_MID, ((lba >> 8) & 0xFF) as u8);
    outb(ATA_PRIMARY_LBA_HIGH, ((lba >> 16) & 0xFF) as u8);

    // Send read command
    outb(ATA_PRIMARY_COMMAND, ATA_CMD_READ_SECTORS);

    // Read each sector
    for sector in 0..sector_count {
        wait_for_drq()?;

        // Read 256 words (512 bytes) per sector
        let offset = (sector as usize) * 256;
        for i in 0..256 {
            buffer[offset + i] = insw(ATA_PRIMARY_DATA);
        }
    }

    Ok(())
}

/// Write sectors to ATA drive
///
/// # Safety
/// Performs hardware I/O operations
/// Buffer must contain at least sector_count * 512 bytes
pub unsafe fn write_sectors(
    lba: u32,
    sector_count: u8,
    buffer: &[u16],
) -> Result<(), &'static str> {
    if buffer.len() < (sector_count as usize * 256) {
        return Err("Buffer too small");
    }

    wait_for_ready()?;

    // Select drive 0 with LBA mode
    outb(ATA_PRIMARY_DRIVE_SELECT, 0xE0 | ((lba >> 24) & 0x0F) as u8);

    // Set sector count and LBA
    outb(ATA_PRIMARY_SECTOR_COUNT, sector_count);
    outb(ATA_PRIMARY_LBA_LOW, (lba & 0xFF) as u8);
    outb(ATA_PRIMARY_LBA_MID, ((lba >> 8) & 0xFF) as u8);
    outb(ATA_PRIMARY_LBA_HIGH, ((lba >> 16) & 0xFF) as u8);

    // Send write command
    outb(ATA_PRIMARY_COMMAND, ATA_CMD_WRITE_SECTORS);

    // Write each sector
    for sector in 0..sector_count {
        wait_for_drq()?;

        // Write 256 words (512 bytes) per sector
        let offset = (sector as usize) * 256;
        for i in 0..256 {
            let word = buffer[offset + i];
            outb(ATA_PRIMARY_DATA, (word & 0xFF) as u8);
            outb(ATA_PRIMARY_DATA, (word >> 8) as u8);
        }
    }

    // Wait for write to complete
    wait_for_ready()?;

    Ok(())
}
