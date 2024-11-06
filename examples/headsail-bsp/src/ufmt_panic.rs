//! Set panicking behavior to print into UART

use core::panic::PanicInfo;

use crate::ufmt::uDisplay;
use ufmt::uwrite;

pub(crate) static mut PANIC_UART_IS_INIT: bool = false;

pub(crate) struct PanicInfoWrapper<'a>(pub(crate) &'a PanicInfo<'a>);

impl uDisplay for PanicInfoWrapper<'_> {
    fn fmt<W>(&self, f: &mut crate::ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: crate::ufmt::uWrite + ?Sized,
    {
        let info = self.0;
        if let Some(loc) = info.location() {
            f.write_str("panic at (")?;
            uwrite!(f, "{}:{}", loc.file(), loc.line())?;
            f.write_str(")\n")?;
        }
        if let Some(m) = info.message().as_str() {
            f.write_str(m)?;
        } else {
            // We don't support panic parameters due to code size constraints
            //
            // Printing parameters would make us depend on core::fmt.
            f.write_str("cause lost")?;
        }
        Ok(())
    }
}

#[cfg(feature = "panic-apb-uart0")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if !unsafe { crate::ufmt_panic::PANIC_UART_IS_INIT } {
        // If UART is not already initialized, init with hale mary values
        crate::apb_uart::ApbUart0::init(30_000_000, 115_200);
    }

    ufmt::uwrite!(
        unsafe { crate::apb_uart::ApbUart0::instance() },
        "{}",
        PanicInfoWrapper(info)
    )
    .unwrap();

    loop {}
}

#[cfg(feature = "panic-sysctrl-uart")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let sysctrl = crate::pac::Sysctrl::ptr();
    let udma = unsafe { (*sysctrl).udma() };

    let mut serial = if !unsafe { crate::ufmt_panic::PANIC_UART_IS_INIT } {
        // If UART is not already initialized, init with hale mary values
        let (soc_freq, baud) = (30_000_000, 9600_u32);
        let clk_div: u16 = (soc_freq / baud) as u16;

        let udma = crate::sysctrl::udma::Udma(udma);
        udma.split().uart.enable(|w| {
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
        })
    } else {
        unsafe { crate::sysctrl::udma::UdmaUart::steal(udma) }
    };
    ufmt::uwrite!(serial, "{}", PanicInfoWrapper(info)).unwrap();

    loop {}
}
