#![no_std]
#![no_main]

mod tests;

use bsp::{
    apb_uart::{ApbUart, ApbUart0},
    riscv::asm::wfi,
    rt::entry,
    sprint, sprintln,
    tb::{report_fail, report_pass},
    ufmt::{self, uDebug},
};

struct Error(tests::Error);

impl uDebug for Error {
    fn fmt<W>(&self, f: &mut bsp::ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: bsp::ufmt::uWrite + ?Sized,
    {
        match self.0 {
            tests::Error::ReadValueIsNotResetValue {
                read_val,
                reset_val,
                reg_uid,
                reg_addr,
            } => {
                bsp::ufmt::uwrite!(
                    f,
                    "read value is not reset value: {} != {}, register: {}@{}",
                    read_val,
                    reset_val,
                    reg_uid,
                    reg_addr
                )
            }
        }
    }
}

fn print_test_case_info<const UART_ADDR: usize>(
    uart: &mut ApbUart<UART_ADDR>,
    idx: usize,
    count: usize,
    longest_uid: usize,
    uid: &str,
    addr: usize,
) {
    let spacing = longest_uid - uid.chars().count() + 1;
    sprint!("[{}/{}] Testing {}", idx + 1, count, uid);
    for _ in 0..spacing {
        uart.putc(b' ');
    }
    sprintln!("@0x{:x}", addr);
}

#[entry]
fn main() -> ! {
    // 30 MHz
    let (soc_freq, baud) = (30_000_000, 115_200);
    let mut uart = ApbUart0::init(soc_freq, baud);
    sprintln!("[{}]", core::env!("CARGO_CRATE_NAME"));

    let cases = &tests::TEST_CASES;
    let longest_uid = cases.iter().map(|t| t.uid.chars().count()).max().unwrap();
    let count = cases.len();
    let mut failures = 0;
    for (idx, t) in cases.iter().enumerate() {
        print_test_case_info(&mut uart, idx, count, longest_uid, t.uid, t.addr);
        if let Err(e) = (t.function)() {
            sprintln!("  {:?}", Error(e));
            // Report failure but run to completion
            report_fail();
            failures += 1;
        } else {
            sprintln!("  reset value ok")
        }
    }

    if failures == 0 {
        report_pass();
    } else {
        report_fail();
    }

    loop {
        wfi();
    }
}
