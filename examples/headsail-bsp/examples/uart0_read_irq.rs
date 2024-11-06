//! Reads bytes over UART using an interrupt
//!
//! Assumes test is run on hart 0 with no other cores interfering.
#![no_std]
#![no_main]

use headsail_bsp::{
    apb_uart::{ApbUart0, UartInterrupt},
    riscv::InterruptNumber,
    rt::entry,
    sprint, sprintln, Interrupt, Priority, PLIC,
};

static mut UART: Option<ApbUart0> = None;

#[entry]
fn main() -> ! {
    let (soc_freq, baud) = (30_000_000, 115_200);
    let mut uart = ApbUart0::init(soc_freq, baud);

    // Raise an interrupt when a byte is available
    uart.listen(UartInterrupt::OnData);

    // Share UART to the interrupt
    let _ = unsafe { UART.insert(uart) };

    unsafe {
        // Enable machine external interrupts (such as UART0)
        riscv::register::mie::set_mext();

        // Set UART0 priority to max
        PLIC::priorities().set_priority(Interrupt::Uart0, Priority::P7);

        // Enable UART0 at context 0
        PLIC::ctx0().enables().enable(Interrupt::Uart0);

        // Enable interrupts globally
        riscv::interrupt::enable();
    };

    sprintln!("Input a character to raise an interrupt");

    loop {
        riscv::asm::wfi();
    }
}

#[export_name = "MachineExternal"]
fn receive_byte() {
    sprintln!("enter receive_byte");

    let mip = riscv::register::mip::read();
    sprintln!("mip: {:#x}", mip.bits());

    // Claim interrupt by reading interrupt ID from claim register
    if let Some(id) = PLIC::ctx0().claim().claim::<Interrupt>() {
        sprintln!("claim: {}", id.number());

        unsafe { UART.as_mut() }.map(|uart| {
            let byte = uart.getc();
            sprintln!("read byte: {}", byte);
        });

        sprintln!("complete: {}", id.number());
        PLIC::ctx0().claim().complete(id);
    }
    sprintln!("exit receive_byte");
}
