// Allowed for extra clarity in certain cases
#![allow(clippy::identity_op)]

pub const EXT_ACCESS_BIT: usize = match () {
    #[cfg(feature = "hpc")]
    () => 1 << 32,
    #[cfg(not(feature = "hpc"))]
    () => 0,
};

mod uart {
    use super::EXT_ACCESS_BIT;

    /// Separation between each UART register
    ///
    /// NS16550 specifies a byte-spaced UART API, and this is indeed implemented
    /// by the VP. However, the ASIC implements the PULP APB UART which comes
    /// with word-spaced (32-bit) registers.
    const REG_SEP: usize = match () {
        #[cfg(feature = "vp")]
        () => 1,
        #[cfg(not(feature = "vp"))]
        () => 4,
    };

    /// Location of PULP APB UART #0
    pub const UART0_ADDR: usize = 0xFFF00000 | EXT_ACCESS_BIT;
    /// Location of PULP APB UART #1
    pub const UART1_ADDR: usize = 0xFFF01000 | EXT_ACCESS_BIT;

    /// Receiver Buffer Register (RBR) / Transmitter Holding Register (THR) /
    /// Divisor Latch LSB (DLL)
    ///
    /// - `LCR[7] == 0`: RBR and THR are accessible
    /// - `LCR[7] == 1`: DLL is accessible
    ///
    /// ## Receiver Buffer Register (read-only)
    ///
    /// Returns the next character buffered on UART and clears the register.
    ///
    /// ## Transmitter Holding Register (write-only)
    ///
    /// Sets the byte to be transmitted over UART.
    ///
    /// ## Divisor latch LSB (read-write)
    ///
    /// Reads or writes the 8 LSBs of the divisor that --- together with clock
    /// frequency --- determines resultant BAUD rate.
    pub const UART_RBR_THR_DLL_OFS: usize = 0;

    /// Interrupt Enable Register (IER) / Divisor Latch MSB (DLM)
    ///
    /// - `LCR[7] == 0`: IER is accessible
    /// - `LCR[7] == 1`: DLM is accessible
    ///
    /// ## Interrupt Enable Register (read-write, `[0:2]`)
    ///
    /// - `[0]`: Interrupt is raised when...
    ///     - (fifo disabled) received data is available
    ///     - (fifo enabled) trigger level has been reached (sa.
    ///       [UART_IIR_FCR_OFS])
    ///     - character timeout has been reached
    /// - `[1]`: Interrupt is raised when [UART_RBR_THR_DLL_OFS] is empty
    /// - `[2]`: Interrupt is raised on Overrun error, parity error, framing
    ///   error or break interrupt
    ///
    /// ## Divisor latch LSB (read-write)
    ///
    /// Reads or writes the 8 MSBs of the divisor that --- together with clock
    /// frequency --- determines resultant BAUD rate.
    pub const UART_IER_DLM_OFS: usize = 1 * REG_SEP;

    /// Interrupt Identification Register (IIR) / FIFO Control Register (FCR)
    ///
    /// ## Interrupt Identification Register (read-only)
    ///
    /// ## FIFO Control Register (write-only)
    ///
    /// - `[1]`: Clear the RX FIFO
    /// - `[2]`: Clear the TX FIFO
    /// - `[6:7]`: Set the trigger level
    ///     - `0b00`: trigger level is high when there is 1 element in the fifo
    ///     - `0b01`: trigger level is high when there are 4 elements in the fifo
    ///     - `0b10`: trigger level is high when there are 8 elements in the fifo
    ///     - `0b11`: trigger level is high when there are 14 elements in the fifo
    pub const UART_IIR_FCR_OFS: usize = 2 * REG_SEP;

    // FIFO_EN_BIT seems to be undocumented
    pub const UART_FCR_FIFO_EN_BIT: u8 = 0b1;
    pub const UART_FCR_FIFO_RX_RESET_BIT: u8 = 0b1 << 1;
    pub const UART_FCR_FIFO_TX_RESET_BIT: u8 = 0b1 << 2;
    pub const UART_FCR_TRIG_RX_LSB: u8 = 0b1 << 6;
    pub const UART_FCR_TRIG_RX_MSB: u8 = 0b1 << 7;

    /// Line Control Register
    ///
    /// LCR configures the main operation of the uart. It configures the width of the received data,
    /// stop bit, parity, and DLAB bit.
    ///
    /// - `[0:1]`: data configuration bits
    ///     - `0b00`: data is configured to be 5 bits
    ///     - `0b01`: data is configured to be 6 bits
    ///     - `0b10`: data is configured to be 7 bits
    ///     - `0b11`: data is configured to be 8 bits
    /// - `[2]`: stop bit configuration
    ///     - `0b0`: 1 stop bit
    ///     - `0b1`: 1.5 stop bits for 5 bits data word OR 2 stop bits 6, 7 or 8 bits data word
    /// - `[3]`: parity enable bit
    /// - `[7]`: divisor latch access bit (DLAB)
    ///     - `0b0`: RBR, THR and IER accessible
    ///     - `0b1`: DLL and DLM accessible
    pub const UART_LCR_OFS: usize = 3 * REG_SEP;

    /// Divisor Latch Access Bit
    pub const UART_LCR_DLAB_BIT: u8 = 0b1 << 7;

    /// Line Status Register
    ///
    /// - `[0]`: RX FIFO data valid
    /// - `[1]`: *not used*
    /// - `[2]`: parity error from the RX FIFO
    /// - `[3]`: *not used*
    /// - `[4]`: *not used*
    /// - `[5]`: the TX FIFO is empty
    /// - `[6]`: shift register and TX FIFO are empty
    pub const UART_LSR_OFS: usize = 5 * REG_SEP;

    pub const UART_LSR_RX_FIFO_VALID: u8 = 0b1;
    pub const UART_LSR_TX_FIFO_EMPTY_BIT: u8 = 1 << 5;

    // The following registers are not used by either PULP APB UART implemented
    // on Headsail:
    /*
    pub const UART_MCR: usize = 4 * REG_SEP;
    pub const UART_MODEM_STATUS_OFFSET: usize = 6 * REG_SEP;
    pub const UART_SCRATCH_OFFSET: usize = 7 * REG_SEP;
    */
}
pub use self::uart::*;

// HPC's timers
pub const TIMER0_ADDR: usize = 0x5_0000;
pub const TIMER1_ADDR: usize = 0x5_0010;
pub const TIMER2_ADDR: usize = 0x5_0020;
pub const TIMER3_ADDR: usize = 0x5_0030;

// Base addres for SDRAM configuration registers
pub const SDRAM_CONFIG_ADDR: usize = 0xFFD0_0000;
