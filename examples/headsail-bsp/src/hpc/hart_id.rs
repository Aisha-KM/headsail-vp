use riscv_pac::{result::Error, HartIdNumber};

/// HPC has 4 HARTs
#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HartId {
    H0 = 0,
    H1 = 1,
    H2 = 2,
    H3 = 3,
}

unsafe impl HartIdNumber for HartId {
    const MAX_HART_ID_NUMBER: usize = 3;

    #[inline]
    fn number(self) -> usize {
        self as _
    }

    #[inline]
    fn from_number(number: usize) -> Result<Self, Error> {
        match number {
            0 => Ok(HartId::H0),
            1 => Ok(HartId::H1),
            2 => Ok(HartId::H2),
            3 => Ok(HartId::H3),
            _ => Err(Error::IndexOutOfBounds {
                index: number,
                min: 0,
                max: Self::MAX_HART_ID_NUMBER,
            }),
        }
    }
}
