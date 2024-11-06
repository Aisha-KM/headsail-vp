#![no_std]
#![no_main]

use headsail_bsp::{rt::entry, sprintln};
use hello_dla::{dla_read, dla_write};

#[entry]
fn main() -> ! {
    sprintln!("Hello world!");
    dla_write("Hello DLA");
    let mut buf: [u8; 9] = [0; 9];
    dla_read(&mut buf, 9, 0);
    #[allow(clippy::empty_loop)]
    loop {}
}
