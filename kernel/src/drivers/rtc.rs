// RTC (Real-Time Clock) driver for CMOS/RTC chip
//
// Reads date and time from the CMOS chip

use crate::arch::port::{inb, outb};

/// CMOS/RTC ports
const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

/// CMOS registers
const RTC_SECONDS: u8 = 0x00;
const RTC_MINUTES: u8 = 0x02;
const RTC_HOURS: u8 = 0x04;
const RTC_DAY: u8 = 0x07;
const RTC_MONTH: u8 = 0x08;
const RTC_YEAR: u8 = 0x09;
const RTC_STATUS_A: u8 = 0x0A;
const RTC_STATUS_B: u8 = 0x0B;

/// Date and time structure
#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

impl DateTime {
    /// Format as a date string (YYYY-MM-DD)
    pub fn format_date(&self) -> [u8; 10] {
        let mut buf = [0u8; 10];

        // Year (4 digits)
        buf[0] = b'0' + ((self.year / 1000) % 10) as u8;
        buf[1] = b'0' + ((self.year / 100) % 10) as u8;
        buf[2] = b'0' + ((self.year / 10) % 10) as u8;
        buf[3] = b'0' + (self.year % 10) as u8;
        buf[4] = b'-';

        // Month (2 digits)
        buf[5] = b'0' + (self.month / 10);
        buf[6] = b'0' + (self.month % 10);
        buf[7] = b'-';

        // Day (2 digits)
        buf[8] = b'0' + (self.day / 10);
        buf[9] = b'0' + (self.day % 10);

        buf
    }

    /// Format as a time string (HH:MM:SS)
    pub fn format_time(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];

        // Hour (2 digits)
        buf[0] = b'0' + (self.hour / 10);
        buf[1] = b'0' + (self.hour % 10);
        buf[2] = b':';

        // Minute (2 digits)
        buf[3] = b'0' + (self.minute / 10);
        buf[4] = b'0' + (self.minute % 10);
        buf[5] = b':';

        // Second (2 digits)
        buf[6] = b'0' + (self.second / 10);
        buf[7] = b'0' + (self.second % 10);

        buf
    }
}

/// Read a CMOS register
///
/// # Safety
/// Must ensure NMI bit is handled properly
unsafe fn read_cmos(reg: u8) -> u8 {
    // Disable NMI by setting bit 7
    outb(CMOS_ADDRESS, 0x80 | reg);
    inb(CMOS_DATA)
}

/// Check if RTC is updating
///
/// # Safety
/// Reads from CMOS ports
unsafe fn is_updating() -> bool {
    outb(CMOS_ADDRESS, 0x80 | RTC_STATUS_A);
    (inb(CMOS_DATA) & 0x80) != 0
}

/// Convert BCD to binary
fn bcd_to_binary(bcd: u8) -> u8 {
    ((bcd >> 4) * 10) + (bcd & 0x0F)
}

/// Read current date and time from RTC
///
/// # Safety
/// Reads from CMOS hardware ports
pub unsafe fn read_rtc() -> DateTime {
    // Wait for any update to complete
    while is_updating() {}

    // Read all values
    let mut second = read_cmos(RTC_SECONDS);
    let mut minute = read_cmos(RTC_MINUTES);
    let mut hour = read_cmos(RTC_HOURS);
    let mut day = read_cmos(RTC_DAY);
    let mut month = read_cmos(RTC_MONTH);
    let mut year = read_cmos(RTC_YEAR);

    // Read status register B to check format
    let status_b = read_cmos(RTC_STATUS_B);
    let is_24_hour = (status_b & 0x02) != 0;
    let is_binary = (status_b & 0x04) != 0;

    // Convert BCD to binary if needed
    if !is_binary {
        second = bcd_to_binary(second);
        minute = bcd_to_binary(minute);
        hour = bcd_to_binary(hour);
        day = bcd_to_binary(day);
        month = bcd_to_binary(month);
        year = bcd_to_binary(year);
    }

    // Convert to 24-hour format if needed
    if !is_24_hour && (hour & 0x80) != 0 {
        hour = ((hour & 0x7F) + 12) % 24;
    }

    // Assume 21st century for now (2000 + year)
    let full_year = 2000 + year as u16;

    DateTime {
        second,
        minute,
        hour,
        day,
        month,
        year: full_year,
    }
}

/// Get day of week name
pub fn day_of_week_name(year: u16, month: u8, day: u8) -> &'static str {
    // Zeller's congruence algorithm
    let mut y = year as i32;
    let mut m = month as i32;

    if m < 3 {
        m += 12;
        y -= 1;
    }

    let q = day as i32;
    let k = y % 100;
    let j = y / 100;

    let h = (q + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    let dow = (h + 5) % 7; // Adjust to Monday = 0

    match dow {
        0 => "Monday",
        1 => "Tuesday",
        2 => "Wednesday",
        3 => "Thursday",
        4 => "Friday",
        5 => "Saturday",
        6 => "Sunday",
        _ => "Unknown",
    }
}

/// Get month name
pub fn month_name(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}
