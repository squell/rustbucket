pub trait Bits {
    fn new() -> Self;
    fn append(&self, _: bool) -> Self;
}

impl Bits for String {
    fn new() -> Self {
        String::new()
    }

    fn append(&self, b: bool) -> Self {
        self.clone() + (if b {"1"} else {"0"})
    }
}

#[derive(Debug,Clone,Copy)]
pub struct RealBits(usize, u8);

impl Bits for RealBits {
    fn new() -> Self {
        RealBits(0,0)
    }

    fn append(&self, b: bool) -> Self {
        let RealBits(i, siz) = self;
        RealBits(i<<1 | (b as usize), siz+1)
    }

}

impl RealBits {
    pub fn from_u8(n: u8) -> Self {
        RealBits(n as usize, 8)
    }
}

impl Iterator for RealBits {
    type Item = bool;
    fn next(&mut self) -> Option<bool> {
        let &mut RealBits(value, mut siz) = self;
        if siz != 0 {
            siz -= 1;
            self.1 = siz;
            Some((value & (1<<siz)) != 0)
        } else {
            None
        }
    }
}
