#![no_std]
#![no_main]

use headsail_bsp::{
    pac::{self, Sysctrl},
    ufmt,
};

/// Print the name of the current file, i.e., test name.
///
/// This must be a macro to make sure core::file matches the file this is
/// invoked in.
#[macro_export]
macro_rules! print_example_name {
    () => {
        use $crate::sysctrl_print;
        sysctrl_print(b"[");
        sysctrl_print(core::file!().as_bytes());
        sysctrl_print(b"]\r\n");
    };
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
    () => 200_000,
    // ASIC --release
    #[cfg(all(feature = "asic", not(debug_assertions)))]
    () => 4_000_000,
};

/// Make sure to enable uDMA UART prior to using this function
pub fn sysctrl_print(buf: &[u8]) {
    let sysctrl = Sysctrl::ptr();
    let udma = unsafe { (*sysctrl).udma() };

    udma.uart_tx_saddr()
        .write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
    udma.uart_tx_size()
        .write(|w| unsafe { w.bits(buf.len() as u32) });

    // (3) Dispatch transmission
    udma.uart_tx_cfg().write(
        |w| w.en().set_bit(), // If we want "continuous mode". In continuous mode, uDMA reloads the address and transmits it again
                              //.continous().set_bit()
    );

    // (4) Poll until finished
    while udma.uart_tx_saddr().read().bits() != 0 {}
}

pub struct UdmaUart;

impl UdmaUart {
    pub fn init() {
        let sysctrl = unsafe { pac::Sysctrl::steal() };
        let udma = headsail_bsp::sysctrl::udma::Udma(sysctrl.udma());

        // Set the bit length, enable TX, set clk_div
        let (soc_freq, baud) = (30_000_000, 9600_u32);
        let clk_div: u16 = (soc_freq / baud) as u16;
        let _uart = udma.split().uart.enable(|w| {
            unsafe {
                w
                    // Use this if using parity bit
                    .parity_ena()
                    .bit(false)
                    .bit_length()
                    .bits(0b11)
                    // Stop bit?
                    .stop_bits()
                    .bit(false)
                    .tx_ena()
                    .bit(true)
                    .rx_ena()
                    .bit(true)
                    .clkdiv()
                    .bits(clk_div)
            }
        });
    }
}

impl ufmt::uWrite for UdmaUart {
    type Error = core::convert::Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        sysctrl_print(s.as_bytes());
        Ok(())
    }
}

#[macro_export]
macro_rules! sprint {
    ($s:expr) => {{
        use $crate::UdmaUart;
        ufmt::uwrite!(UdmaUart, $s).unwrap()
    }};
    ($($tt:tt)*) => {{
        use $crate::UdmaUart;
        ufmt::uwrite!(UdmaUart, $($tt)*).unwrap()
    }};
}

#[macro_export]
macro_rules! sprintln {
    () => {{
        use $crate::sprint;
        sprint!("\r\n");
    }};
    // IMPORTANT use `tt` fragments instead of `expr` fragments (i.e. `$($exprs:expr),*`)
    ($($tt:tt)*) => {{
        use $crate::sprint;
        sprint!($($tt)*);
        sprint!("\r\n");
    }};
}
