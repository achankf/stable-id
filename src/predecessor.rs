use crate::Predecessor;

impl Predecessor for u8 {
    fn prev(self) -> Self {
        assert!(self > 0);
        self - 1
    }
}

impl Predecessor for u16 {
    fn prev(self) -> Self {
        assert!(self > 0);
        self - 1
    }
}

impl Predecessor for u32 {
    fn prev(self) -> Self {
        assert!(self > 0);
        self - 1
    }
}

impl Predecessor for usize {
    fn prev(self) -> Self {
        assert!(self > 0);
        self - 1
    }
}
