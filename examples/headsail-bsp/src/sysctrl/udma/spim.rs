use core::marker::PhantomData;

use super::{Disabled, Enabled};
use crate::pac;

pub const SPI_CMD_CFG: u32 = 0x00000000;
pub const SPI_CMD_SOT: u32 = 0x10000000;
pub const SPI_CMD_EOT: u32 = 0x90000000;
pub const SPI_CMD_SEND_CMD_BASE: u32 = 0x20070000;
pub const SPI_CMD_DUMMY: u32 = 0x400F0000;
pub const SPI_CMD_RX_CHECK: u32 = 0xB0200000;
pub const SPI_CMD_RX_DATA: u32 = 0x74000000;
pub const SPI_CMD_TX_DATA: u32 = 0x64000000;
pub const SPI_CMD_SETUP_UCA: u32 = 0xD0000000;
pub const SPI_CMD_SETUP_UCS: u32 = 0xE0000000;

/// Obtain an instance by calling [Udma::split]
pub struct UdmaSpim<'u, UdmaPeriphState>(
    pub(crate) &'u pac::sysctrl::Udma,
    pub(crate) PhantomData<UdmaPeriphState>,
);

impl<'u> UdmaSpim<'u, Disabled> {
    #[inline]
    pub fn enable(self) -> UdmaSpim<'u, Enabled> {
        let spim = &self.0;

        // Turn on the clock gates for SPIM
        spim.ctrl_cfg_cg().modify(|_r, w| w.cg_spim().set_bit());

        UdmaSpim::<Enabled>(self.0, PhantomData)
    }
}

impl<'u> UdmaSpim<'u, Enabled> {
    #[inline]
    pub fn disable(self) -> UdmaSpim<'u, Disabled> {
        self.0.ctrl_cfg_cg().modify(|_r, w| w.cg_spim().clear_bit());
        UdmaSpim::<Disabled>(self.0, PhantomData)
    }

    #[inline]
    pub fn enqueue_tx(&mut self, buf: &[u8]) {
        let spim = &self.0;

        // Write buffer location & len
        spim.spim_tx_saddr()
            .write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
        spim.spim_tx_size()
            .write(|w| unsafe { w.bits(buf.len() as u32) });

        // Dispatch transmission
        spim.spim_tx_cfg().write(
            |w| w.en().set_bit(), // If we want "continuous mode". In continuous mode, uDMA reloads the address and transmits it again
                                  //.continous().set_bit()
        );

        // Poll until finished (prevents `buf` leakage)
        while spim.spim_tx_saddr().read().bits() != 0 {}
    }
    pub fn enqueue_rx(&mut self, buf: &[u8]) {
        let spim = &self.0;

        // Write buffer location & len
        spim.spim_rx_saddr()
            .write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
        spim.spim_rx_size()
            .write(|w| unsafe { w.bits(buf.len() as u32) });

        // Dispatch transmission
        spim.spim_rx_cfg().write(
            |w| w.en().set_bit(), // If we want "continuous mode". In continuous mode, uDMA reloads the address and transmits it again
                                  //.continous().set_bit()
        );

        // Poll until finished (prevents `buf` leakage)
        while spim.spim_rx_saddr().read().bits() != 0 {}
    }

    pub fn enqueue_cmd(&mut self, buf: &[u8]) {
        let spim = &self.0;

        // Write buffer location & len
        spim.spim_cmd_saddr()
            .write(|w| unsafe { w.bits(buf.as_ptr() as u32) });
        spim.spim_cmd_size()
            .write(|w| unsafe { w.bits(buf.len() as u32) });

        // Dispatch transmission
        spim.spim_cmd_cfg().write(
            |w| w.en().set_bit(), // If we want "continuous mode". In continuous mode, uDMA reloads the address and transmits it again
                                  //.continous().set_bit()
        );

        // Poll until finished (prevents `buf` leakage)
        while spim.spim_cmd_saddr().read().bits() != 0 {}
    }

    /// This function sends SOT (Start Of Transmission) command.
    pub fn sot(&mut self) {
        let sot_cmd: [u8; 4] = SPI_CMD_SOT.to_ne_bytes();
        self.enqueue_cmd(&sot_cmd);
    }

    /// This function sends EOT (End Of Transmission) command .
    pub fn eot(&mut self) {
        let eot_cmd: [u8; 4] = (SPI_CMD_EOT).to_ne_bytes();
        self.enqueue_cmd(&eot_cmd);
    }

    /// This function sends EOT (End Of Transmission) command but keeps the cs asserted.
    pub fn eot_keep_cs(&mut self) {
        let eot_cmd: [u8; 4] = (SPI_CMD_EOT | 0x03).to_ne_bytes();
        self.enqueue_cmd(&eot_cmd);
    }

    /// This function sends one dummy byte (0xFF), it should be flixable so that the
    /// user can easily choose the number of repetition without using a for loop.
    /// the usage for now is:
    ///
    /// # Examples
    ///
    /// ```
    ///   for _i in 0..10 {
    ///    spim.sot();
    ///   spim.send_dummy();
    /// }
    /// ```
    pub fn send_dummy(&mut self) {
        let mut buffer: [u8; 4] = [0; 4];
        let cmd_cmd: [u8; 4] = (SPI_CMD_SEND_CMD_BASE | 0xFF).to_ne_bytes();

        buffer[0..4].copy_from_slice(&cmd_cmd[0..4]);
        self.enqueue_cmd(&buffer);
    }

    /// This function send data out.
    /// Use this funtion to transfere data via spi to for example SD card.
    ///
    /// # Examples
    ///
    /// ```
    ///   let data: [u8; 2] = [0x01,0x02];
    ///   spim.sot();
    ///   spim.send(&data);
    ///   spim.eot();
    ///
    /// ```
    pub fn send(&mut self, data: &[u8]) {
        let mut cmd_data: [u8; 12] = [0; 12];

        cmd_data[0..4].copy_from_slice(
            &(SPI_CMD_SETUP_UCA | (data.as_ptr() as u32 & 0x0000FFFF)).to_ne_bytes(),
        );
        cmd_data[4..8]
            .copy_from_slice(&(SPI_CMD_SETUP_UCS | (data.len() - 2) as u32).to_ne_bytes()); // 4 byte but change this to depend on data i.e:((data.len() - 2) as u32)
        cmd_data[8..12].copy_from_slice(
            &(SPI_CMD_TX_DATA | (data.len() - 1) as u32 | (7 << 16)).to_ne_bytes(),
        );

        self.enqueue_cmd(&cmd_data);

        // Block until TX buffer is ready for new transfer
        while self.0.spim_tx_cfg().read().pending().bit() {}
        self.enqueue_tx(data);
    }

    /// This function receives data.
    /// Use this funtion to recive data via spi from for example SD card.
    ///
    /// # Examples
    ///
    /// ```
    ///   let data: [u8; 2] = [0;2];
    ///   spim.sot();
    ///   spim.receive(&data);
    ///   spim.eot();
    ///
    /// ```
    pub fn receive(&mut self, data: &[u8]) {
        let mut cmd_data: [u8; 12] = [0; 12];

        cmd_data[0..4].copy_from_slice(
            &(SPI_CMD_SETUP_UCA | (data.as_ptr() as u32 & 0x0000FFFF)).to_ne_bytes(),
        );
        cmd_data[4..8]
            .copy_from_slice(&(SPI_CMD_SETUP_UCS | (data.len() - 2) as u32).to_ne_bytes());
        cmd_data[8..12].copy_from_slice(
            &(SPI_CMD_RX_DATA | (data.len() - 1) as u32 | (7 << 16)).to_ne_bytes(),
        );

        self.enqueue_cmd(&cmd_data);

        // Block until RX buffer is ready for new transfer
        while self.0.spim_rx_cfg().read().pending().bit() {}
        self.enqueue_rx(data);
    }
}
