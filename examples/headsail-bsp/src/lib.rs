//! A light-weight memory map based board support package for Headsail.
#![no_std]

// Pick an optional PAC based on the target CPU. Some drivers may depend on it.
#[cfg(feature = "hpc-pac")]
pub use headsail_hpc_pac as pac;
#[cfg(feature = "sysctrl-pac")]
pub use headsail_sysctrl_pac as pac;

// CPU specific modules
#[cfg(feature = "hpc")]
mod hpc;
#[cfg(feature = "sysctrl")]
pub mod sysctrl;
#[cfg(feature = "hpc")]
pub use hpc::*;

#[cfg(feature = "alloc")]
pub mod alloc;
#[cfg(feature = "alloc")]
pub use alloc::init_heap;

// Timer implementation is somewhat different on hardware and VP. We pick and
// re-export the correct one here.
#[cfg(not(feature = "vp"))]
mod apb_timer;
#[cfg(feature = "vp")]
mod timer_unit;
pub mod timer {
    /*!
     * Timer module for Headsail. When running on the Renode
     * Virtual Prototype, the "vp" feature should be enabled.
     * All operations are Read-Modify-Write.
     *
     * HOW TO USE THIS DRIVER:
     * In order to use any of the four timers that come with
     * Headsail HPC SubSystem, use the respective Timer{0..3}
     * type alias provided and the functions associated with it.
     */
    #[cfg(not(feature = "vp"))]
    pub use crate::apb_timer::*;
    #[cfg(feature = "vp")]
    pub use crate::timer_unit::*;
}

// Print-implementation specific modules
#[cfg(feature = "sprint-apb-uart0")]
pub mod sprintln;
#[cfg(any(feature = "panic-apb-uart0", feature = "panic-sysctrl-uart"))]
mod ufmt_panic;
pub use ufmt;

pub mod apb_uart;
pub mod mmap;
mod mmio;
pub mod sdram;
pub mod tb;

pub use mmio::*;
pub use riscv;
#[cfg(feature = "rt")]
pub use riscv_rt as rt;
