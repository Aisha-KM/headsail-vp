#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use headsail_bsp::{init_heap, rt::entry, sprintln};

#[entry]
fn main() -> ! {
    sprintln!("Hello sprintln!");
    // SAFETY: `init_heap` must be called once only
    unsafe { init_heap() };
    let v = vec![1, 2, 3];
    for x in v {
        sprintln!("{:?}", x);
    }
    loop {}
}
