#![no_std]

mod mmap;

use core::ptr;
use mmap::{DLA0_ADDR, UART0_ADDR};

pub fn uart_write(s: &str) {
    for b in s.as_bytes() {
        unsafe { ptr::write_volatile(UART0_ADDR as *mut u8, *b) };
    }
}

pub fn dla_write(s: &str) {
    for b in s.as_bytes() {
        unsafe { ptr::write_volatile(DLA0_ADDR as *mut u8, *b) };
    }
}

pub fn dla_read(buf: &mut [u8], len: usize, offset: usize) {
    buf.iter_mut()
        .take(len)
        .enumerate()
        .for_each(|(i, byte)| unsafe {
            *byte = ptr::read_volatile((DLA0_ADDR + offset + i) as *const u8);
        });
}

/// Experimentally found value for number of nops HPC is capable of executing per second.
///
/// * ASIC values are obtained with 30 MHz reference clocks.
/// * VP values are obtained with the default performance of a Renode CPU at 100 MIPS
pub const NOPS_PER_SEC: usize = match () {
    // VP
    #[cfg(all(not(feature = "asic"), debug_assertions))]
    () => 750_000,
    // VP --release
    #[cfg(all(not(feature = "asic"), not(debug_assertions)))]
    () => 30_000_000,
    // ASIC
    #[cfg(all(feature = "asic", debug_assertions))]
    () => 6_000,
    // ASIC --release
    #[cfg(all(feature = "asic", not(debug_assertions)))]
    () => 120_000,
};
