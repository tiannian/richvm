use core::panic;

pub enum U3 {
    N0,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
}

impl From<u8> for U3 {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::N0,
            1 => Self::N1,
            2 => Self::N2,
            3 => Self::N3,
            4 => Self::N4,
            5 => Self::N5,
            6 => Self::N6,
            7 => Self::N7,
            _ => panic!("Please check code error"),
        }
    }
}
