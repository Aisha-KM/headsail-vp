use core::marker::PhantomData;

use super::{Disabled, Enabled};
use crate::pac;

/// Obtain an instance by calling [Udma::split]
pub struct UdmaUart<'u, UdmaPeriphState>(
    pub(crate) &'u pac::sysctrl::Udma,
    pub(crate) PhantomData<UdmaPeriphState>,
);

type UartSetupW = pac::sysctrl::udma::uart_setup::W;

impl<'u> UdmaUart<'u, Disabled> {
    #[inline]
    pub fn enable<F>(self, setup_spec: F) -> UdmaUart<'u, Enabled>
    where
        F: FnOnce(&mut UartSetupW) -> &mut UartSetupW,
    {
        let udma = &self.0;

        // Turn on the clock gates for UART
        udma.ctrl_cfg_cg().modify(|_r, w| w.cg_uart().set_bit());

        // Setup UART
        udma.uart_setup().write(|w| unsafe { w.bits(0) });
        udma.uart_setup().write(setup_spec);

        #[cfg(feature = "panic-sysctrl-uart")]
        unsafe {
            crate::ufmt_panic::PANIC_UART_IS_INIT = true
        };

        UdmaUart::<Enabled>(self.0, PhantomData)
    }
}

impl<'u> UdmaUart<'u, Enabled> {
    #[inline]
    pub fn disable(self) -> UdmaUart<'u, Disabled> {
        self.0.ctrl_cfg_cg().modify(|_r, w| w.cg_uart().clear_bit());
        UdmaUart::<Disabled>(self.0, PhantomData)
    }

    /// # Safety
    ///
    /// This will not configure the UART in any way.
    #[inline]
    pub unsafe fn steal(udma: &'static pac::sysctrl::Udma) -> Self {
        Self(udma, PhantomData)
    }

    #[inline]
    pub fn write(&mut self, buf: &[u8]) {
        let udma = &self.0;

        // Write buffer location & len
        udma.uart_tx_saddr()
            .write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
        udma.uart_tx_size()
            .write(|w| unsafe { w.bits(buf.len() as u32) });

        // Dispatch transmission
        udma.uart_tx_cfg().write(
            |w| w.en().set_bit(), // If we want "continuous mode". In continuous mode, uDMA reloads the address and transmits it again
                                  //.continous().set_bit()
        );

        // Poll until finished (prevents `buf` leakage)
        while udma.uart_tx_saddr().read().bits() != 0 {}
    }

    #[inline]
    pub fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes());
    }
}

impl<'a> ufmt_write::uWrite for UdmaUart<'a, Enabled> {
    type Error = core::convert::Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.write(s.as_bytes());
        Ok(())
    }
}
