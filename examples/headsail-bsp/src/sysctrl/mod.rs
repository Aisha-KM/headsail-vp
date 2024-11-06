//! Abstractions that only exist on SysCtrl
pub mod gpio;
pub mod soc_ctrl;
#[cfg(feature = "pac")]
pub mod udma;

pub mod interrupt;
pub mod mmap;
