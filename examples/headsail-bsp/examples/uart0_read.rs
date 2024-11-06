#![no_std]
#![no_main]

extern crate alloc;
use headsail_bsp::{apb_uart::ApbUart0, init_heap, rt::entry, sprint, sprintln};

#[entry]
fn main() -> ! {
    let (soc_freq, baud) = (30_000_000, 115_200);
    let mut uart = ApbUart0::init(soc_freq, baud);

    sprintln!("Connect to APB UART 0 with: screen /tmp/uart0");
    // SAFETY: `init_heap` must be called once only
    unsafe { init_heap() };
    loop {
        let res = uart.read_to_heap(8);
        for x in res {
            if x != 0 {
                sprint!("{:x} ", x)
            }
        }
    }
}
