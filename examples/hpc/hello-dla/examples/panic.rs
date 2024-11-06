//! Panic, making sure we get developer-observable output
#![no_std]
#![no_main]

use headsail_bsp::rt::entry;

#[entry]
fn main() -> ! {
    panic!("explicit panic");
}
