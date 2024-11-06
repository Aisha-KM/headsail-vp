//! Print over TLP UART (PULP APB UART) #0 using HAL
//!
//! | Date              | Status     | Changes   |
//! | :-                | :-:        | :-        |
//! | 2024-09-04        | *Untested* |           |
//! | 2024-10-24        | *Works*    |           |
#![no_std]
#![no_main]

use headsail_bsp::rt::entry;

#[entry]
fn main() -> ! {
    panic!("explicit panic");
}
