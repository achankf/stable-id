use crate::CastUsize;

impl CastUsize for u8 {
    fn to(self) -> usize {
        self as usize
    }

    fn from(val: usize) -> Self {
        assert!(val < Self::max_value().into());

        val as Self
    }
}

impl CastUsize for u16 {
    fn to(self) -> usize {
        self as usize
    }

    fn from(val: usize) -> Self {
        assert!(val < Self::max_value().into());

        val as Self
    }
}

impl CastUsize for u32 {
    fn to(self) -> usize {
        self as usize
    }

    fn from(val: usize) -> Self {
        assert!(val < Self::max_value() as usize);

        val as Self
    }
}

impl CastUsize for usize {
    fn to(self) -> usize {
        self
    }

    fn from(val: usize) -> Self {
        val
    }
}
