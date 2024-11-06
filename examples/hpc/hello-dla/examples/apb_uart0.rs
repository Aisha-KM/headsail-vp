//! Print over TLP UART (PULP APB UART) #0 using HAL
//!
//! | Date              | Status     | Changes   |
//! | :-                | :-:        | :-        |
//! | 2024-10-24        | *Works*    |           |
#![no_std]
#![no_main]

use core::arch::asm;

use headsail_bsp::{apb_uart::ApbUart0, rt::entry, unmask_u32};
use hello_dla::NOPS_PER_SEC;

#[entry]
fn main() -> ! {
    // Disable GPIO behavior for UART pins
    const PAD_CONF_UART0_TX: usize = 0x1_fff0_7064;
    unmask_u32(PAD_CONF_UART0_TX, (0b1 << 5) | (0b1 << 10));

    let (soc_freq, baud) = (30_000_000, 9600);
    let mut uart = ApbUart0::init(soc_freq, baud);

    loop {
        uart.write(b"Hello TLP UART #0!\r\n");

        for _ in 0..NOPS_PER_SEC {
            unsafe { asm!("nop") };
        }
    }
}
