use crate::{
    mmap::{UART0_ADDR, UART1_ADDR, UART_LSR_RX_FIFO_VALID, UART_RBR_THR_DLL_OFS},
    read_u8, write_u8,
};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// When to raise a UART interrupt
#[derive(Clone)]
#[repr(u8)]
pub enum UartInterrupt {
    /// Interrupt is raised when...
    ///
    /// * (fifo disabled) received data is available
    /// * (fifo enabled) trigger level has been reached (sa. [UART_IIR_FCR_OFS])
    /// * character timeout has been reached error or break interrupt
    OnData = 0b1,
    /// Interrupt is raised when [UART_RBR_THR_DLL_OFS] is empty, i.e., when
    /// character has been consumed by polling.
    OnEmpty = 0b1 << 1,
    /// Interrupt is raised on overrun error, parity error, framing error or
    /// break interrupt
    OnError = 0b1 << 2,
}

/// Relocatable driver for NS16550 UART IP
///
/// The generic represents the base address for the UART. This driver is
/// compatible with both ASIC and the VP. Use
///
/// * `-Fasic` for ASIC implementation
/// * and `-Fvp` for VP implementation.
pub struct ApbUart<const BASE_ADDRESS: usize>;

/// Type alias for APB UART 0
pub type ApbUart0 = ApbUart<UART0_ADDR>;

/// Type alias for APB UART 1
pub type ApbUart1 = ApbUart<UART1_ADDR>;

impl<const BASE_ADDR: usize> ApbUart<BASE_ADDR> {
    /// # Parameters
    ///
    /// * `soc_freq`-  used to calculate BAUD rate together with divisor
    /// * `baud` - target BAUD (sa. UART protocol)
    #[allow(unused_variables)]
    pub fn init(soc_freq: u32, baud: u32) -> Self {
        #[cfg(feature = "asic")]
        {
            use crate::{
                mask_u8,
                mmap::{
                    UART_FCR_FIFO_EN_BIT, UART_FCR_FIFO_RX_RESET_BIT, UART_FCR_FIFO_TX_RESET_BIT,
                    UART_FCR_TRIG_RX_LSB, UART_FCR_TRIG_RX_MSB, UART_IER_DLM_OFS, UART_IIR_FCR_OFS,
                    UART_LCR_DLAB_BIT, UART_LCR_OFS, UART_RBR_THR_DLL_OFS,
                },
                unmask_u8,
            };

            #[repr(u8)]
            pub enum UartLcrDataBits {
                /*
                Bits5 = 0b00,
                Bits6 = 0b01,
                Bits7 = 0b10,
                */
                Bits8 = 0b11,
            }

            const PERIPH_CLK_DIV: u32 = 1;
            let divisor: u32 = soc_freq / PERIPH_CLK_DIV / (baud << 4);

            // Safety: all PULP APB UART registers are 4-byte aligned so no bus can stop us
            unsafe {
                // Enable DLAB (to set baud rate divisor)
                mask_u8(BASE_ADDR + UART_LCR_OFS, UART_LCR_DLAB_BIT);
                // Set low & high byte of divisor
                write_u8(BASE_ADDR + UART_RBR_THR_DLL_OFS, divisor as u8);
                write_u8(BASE_ADDR + UART_IER_DLM_OFS, (divisor >> 8) as u8);
                // Data is 8 bits, one stop bit, no parity
                write_u8(BASE_ADDR + UART_LCR_OFS, UartLcrDataBits::Bits8 as u8);
                // Restore DLAB state
                unmask_u8(BASE_ADDR + UART_LCR_OFS, UART_LCR_DLAB_BIT);

                // Enable FIFO, clear RX & TX, use 14-byte threshold
                write_u8(
                    BASE_ADDR + UART_IIR_FCR_OFS,
                    UART_FCR_FIFO_EN_BIT
                        | UART_FCR_FIFO_RX_RESET_BIT
                        | UART_FCR_FIFO_TX_RESET_BIT
                        | UART_FCR_TRIG_RX_LSB
                        | UART_FCR_TRIG_RX_MSB,
                );
            }
        }

        #[cfg(feature = "panic-apb-uart0")]
        unsafe {
            crate::ufmt_panic::PANIC_UART_IS_INIT = true
        };
        Self {}
    }

    /// # Safety
    ///
    /// Returns a potentially uninitialized instance of APB UART. On ASIC, make
    /// sure to call [ApbUart::init] prior to this call, otherwise the UART
    /// won't behave properly.
    pub const unsafe fn instance() -> Self {
        Self {}
    }

    #[cfg(feature = "asic")]
    #[inline]
    fn is_transmit_empty(&self) -> bool {
        use crate::mmap::{UART_LSR_OFS, UART_LSR_TX_FIFO_EMPTY_BIT};

        // Safety: UART_LSR is 4-byte aligned
        (unsafe { read_u8(BASE_ADDR + UART_LSR_OFS) } & UART_LSR_TX_FIFO_EMPTY_BIT) != 0
    }

    #[inline]
    pub fn write(&mut self, buf: &[u8]) {
        for b in buf {
            self.putc(*b);
        }
    }

    #[inline]
    pub fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes());
    }

    /// Flush this output stream, blocking until all intermediately buffered contents reach their
    /// destination.
    #[inline]
    pub fn flush(&mut self) {
        // Wait for hardware to report completion
        #[cfg(feature = "asic")]
        while !self.is_transmit_empty() {}
    }

    #[inline]
    pub fn putc(&mut self, c: u8) {
        // Wait for hardware to report completion
        #[cfg(feature = "asic")]
        while !self.is_transmit_empty() {}

        // Safety: UART_THR is 4-byte aligned
        unsafe { write_u8(BASE_ADDR + UART_RBR_THR_DLL_OFS, c) };
    }

    #[inline]
    pub fn getc(&mut self) -> u8 {
        // Wait for data to become ready
        while unsafe { read_u8(BASE_ADDR + crate::mmap::UART_LSR_OFS) } & UART_LSR_RX_FIFO_VALID
            == 0
        {}

        // SAFETY: UART0_ADDR is 4-byte aligned
        unsafe { read_u8(BASE_ADDR) }
    }

    #[inline]
    pub fn listen(&mut self, int: UartInterrupt) {
        use crate::mmap::*;

        unsafe {
            // Save LCR for restoration
            let p_lcr = read_u8(BASE_ADDR + UART_LCR_OFS);

            // Deassert `LCR[7]` => IER_DLM is IER
            let lcr = p_lcr & (0b1 << 7);
            write_u8(BASE_ADDR + UART_LCR_OFS, lcr);

            // Set IER
            write_u8(BASE_ADDR + UART_IER_DLM_OFS, int as u8);

            // Restore `LCR`
            write_u8(BASE_ADDR + UART_LCR_OFS, p_lcr);
        }
    }

    #[cfg(feature = "alloc")]
    pub fn read_to_heap(&mut self, bytes: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(bytes);
        for _ in 0..bytes {
            result.push(self.getc())
        }
        result
    }
}

impl<const BASE_ADDR: usize> ufmt_write::uWrite for ApbUart<BASE_ADDR> {
    type Error = core::convert::Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.write_str(s);
        Ok(())
    }
}
