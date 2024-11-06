use riscv::InterruptNumber;

#[cfg(feature = "sysctrl-rt")]
#[export_name = "_setup_interrupts"]
fn setup_interrupt_vector() {
    use riscv::register::mtvec;

    // Set the trap vector
    unsafe {
        extern "C" {
            fn _trap_vector();
        }

        // Set all the trap vectors for good measure
        let bits = _trap_vector as usize;
        mtvec::write(bits, mtvec::TrapMode::Vectored);
    }
}

// The vector table
//
// Do the ESP trick and route all interrupts to the direct dispatcher.
//
// N.b. vectors length must be exactly 0x80
#[cfg(feature = "sysctrl-rt")]
core::arch::global_asm!(
    "
.section .vectors, \"ax\"
    .global _trap_vector
    // Trap vector base address must always be aligned on a 4-byte boundary
    .align 4
_trap_vector:
    j _start_trap
    .rept 31
    j _start_trap // 1..31
    .endr
"
);

/// SysCtrl External Interrupt Mappings
///
/// Mappings retrieved from:
///
/// <https://gitlab.tuni.fi/soc-hub/headsail/hw/headsail/-/blob/main/doc/interrupts.md?ref_type=heads#sysctrl-external-irq-index-specification>
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Interrupt {
    /// Vector idx 0
    Dma0 = 13,
    /// Vector idx 1
    Dma1 = 21,
    /// Vector idx 17
    Ethernet0 = 22,
    /// Vector idx 18
    Ethernet1 = 23,
    /// Vector idx 16
    Dla = 24,
    /// External 0
    ///
    /// Can be mapped by external IRQ router to one of: UART0, UART1, SPIM0, SPIM1, I2C, GPIO, SW IRQ
    Ext0 = 25,
    /// External 1
    ///
    /// Can be mapped by external IRQ router to one of: UART0, UART1, SPIM0, SPIM1, I2C, GPIO, SW IRQ
    Ext1 = 27,
    /// External 2
    ///
    /// Can be mapped by external IRQ router to one of: UART0, UART1, SPIM0, SPIM1, I2C, GPIO, SW IRQ
    Ext2 = 28,
}

unsafe impl InterruptNumber for Interrupt {
    const MAX_INTERRUPT_NUMBER: usize = 28;

    fn number(self) -> usize {
        self as usize
    }

    fn from_number(value: usize) -> riscv::result::Result<Self> {
        match value {
            13 => Ok(Interrupt::Dma0),
            21 => Ok(Interrupt::Dma1),
            22 => Ok(Interrupt::Ethernet0),
            23 => Ok(Interrupt::Ethernet1),
            24 => Ok(Interrupt::Dla),
            25 => Ok(Interrupt::Ext0),
            27 => Ok(Interrupt::Ext1),
            28 => Ok(Interrupt::Ext2),
            _ => Err(riscv::result::Error::InvalidVariant(value)),
        }
    }
}
