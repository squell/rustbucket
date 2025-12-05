use std::fmt::Debug;

pub trait Alloc<'b,T> {
    fn obtain(&mut self, x: T) -> &'b T;
}

pub struct LeakyPlumber { }

impl<'b,T> Alloc<'b,T> for LeakyPlumber where T: Debug {
    fn obtain(&mut self, x: T) -> &'b T {
        let tmp = Box::new(x);
        Box::leak(tmp)
    }
}

pub struct LocalPlumber<'b,T>(pub &'b mut [T]);

impl<'b,T> Alloc<'b,T> for LocalPlumber<'b,T> {
    fn obtain(&mut self, x: T) -> &'b T {
        let cell;
        (cell,self.0) = std::mem::take(&mut self.0).split_at_mut(1);
        cell[0] = x;
        &cell[0]
    }
}
