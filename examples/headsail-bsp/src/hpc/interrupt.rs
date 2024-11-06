use crate::HartId;
use riscv::{ExternalInterruptNumber, InterruptNumber};
use riscv_pac::{result::Error, PriorityNumber};

/// HPC PLIC Interrupt Mappings
///
/// Mappings retrieved from:
///
///  <https://gitlab.tuni.fi/soc-hub/headsail/hw/headsail/-/blob/main/doc/interrupts.md?ref_type=heads#hpc-irq-index-specification>
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Interrupt {
    /* 0 reserved, local IRQ */
    /* [1..=8] APB Timer IRQs ("HPC internal"), local IRQ N/A */
    /// DMA0 (ext. IRQ 0)
    Dma0 = 9,
    /// DMA1 (ext. IRQ 1)
    Dma1 = 10,
    /// UART0 (ext. IRQ 2)
    Uart0 = 11,
    /// UART1 (ext. IRQ 3)
    Uart1 = 12,
    /// SPIM0 [0] (ext. IRQ 4)
    Spim0_0 = 13,
    /// SPIM0 [1] (ext. IRQ 5)
    Spim0_1 = 14,
    /// SPIM1 [0] (ext. IRQ 6)
    Spim1_0 = 15,
    /// SPIM1 [1] (ext. IRQ 7)
    Spim1_1 = 16,
    /// I2C  (ext. IRQ 8)
    I2c = 17,
    /// GPIO (ext. IRQ 9)
    Gpio = 18,
    /// Software 0 (ext. IRQ 10)
    Soft0 = 19,
    /// Software 1 (ext. IRQ 11)
    Soft1 = 20,
    /// Software 2 (ext. IRQ 12)
    Soft2 = 21,
    /// Software 3 (ext. IRQ 13)
    Soft3 = 22,
    /// C2C serial (ext. IRQ 14)
    C2cSerial = 23,
    /// C2C parallel (ext. IRQ 15)
    C2cParallel = 24,
    /// DLA (ext. IRQ 16)
    Dla = 25,
    /// Ethernet [0] (ext. IRQ 17)
    Ethernet0 = 26,
    /// Ethernet [1] (ext. IRQ 18)
    Ethernet1 = 27,
}

unsafe impl InterruptNumber for Interrupt {
    const MAX_INTERRUPT_NUMBER: usize = 27;

    fn number(self) -> usize {
        self as usize
    }

    fn from_number(value: usize) -> riscv::result::Result<Self> {
        match value {
            x if (9..=Self::MAX_INTERRUPT_NUMBER).contains(&x) => {
                Ok(unsafe { core::mem::transmute::<usize, Interrupt>(x) })
            }
            _ => Err(riscv::result::Error::IndexOutOfBounds {
                index: value,
                min: 9,
                max: Self::MAX_INTERRUPT_NUMBER,
            }),
        }
    }
}

unsafe impl ExternalInterruptNumber for Interrupt {}

// HPC-SS specifies that priorities go up to 7
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Priority {
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
    P4 = 4,
    P5 = 5,
    P6 = 6,
    P7 = 7,
}

unsafe impl PriorityNumber for Priority {
    const MAX_PRIORITY_NUMBER: usize = 7;

    #[inline]
    fn number(self) -> usize {
        self as _
    }

    #[inline]
    fn from_number(number: usize) -> Result<Self, Error> {
        match number {
            0 => Ok(Priority::P0),
            1 => Ok(Priority::P1),
            2 => Ok(Priority::P2),
            3 => Ok(Priority::P3),
            4 => Ok(Priority::P4),
            5 => Ok(Priority::P5),
            6 => Ok(Priority::P6),
            7 => Ok(Priority::P7),
            _ => Err(Error::IndexOutOfBounds {
                index: number,
                min: 0,
                max: Self::MAX_PRIORITY_NUMBER,
            }),
        }
    }
}

riscv_peripheral::clint_codegen!(
    base 0x60000,
    freq 32_768,
    mtimecmps [
        mtimecmp0 = (HartId::H0, "`H0`"),
        mtimecmp1 = (HartId::H1, "`H1`"),
        mtimecmp2 = (HartId::H2, "`H2`"),
        mtimecmp3 = (HartId::H3, "`H3`")
    ],
    msips [
        msip0 = (HartId::H0, "`H0`"),
        msip1 = (HartId::H1, "`H1`"),
        msip2 = (HartId::H2, "`H2`"),
        msip3 = (HartId::H3, "`H3`")
    ],
);

// ???: ASIC developer beware, this PLIC implementation works for VP only
//
// Some addresses were tightened to save space for Headsail's PLIC. That means that the ASIC will be
// different from the sim, and that this `riscv_peripheral` provided driver won't work as-is on
// ASIC, while it works perfectly well with the sim.
//
// TODO: fork riscv_peripheral and space out the PLIC codegen to match with ASIC. Then pick the
// right implementation conditionally.
#[cfg(feature = "vp")]
riscv_peripheral::plic_codegen!(
    base 0x80000,
    ctxs [
        ctx0 = (HartId::H0, "`H0M`"),
        ctx1 = (HartId::H0, "`H0S`"),
        ctx2 = (HartId::H1, "`H1M`"),
        ctx3 = (HartId::H1, "`H1S`"),
        ctx4 = (HartId::H2, "`H2M`"),
        ctx5 = (HartId::H2, "`H2S`"),
        ctx6 = (HartId::H3, "`H3M`"),
        ctx7 = (HartId::H3, "`H3S`")
    ],
);
