use std::fmt::Debug;

pub trait Alloc<'b,T> {
    fn obtain(self, x:T) -> (&'b T, Self);
}

pub struct LeakyPlumber { }

impl<'b,T> Alloc<'b,T> for LeakyPlumber where T: Debug {
    fn obtain(self, x:T) -> (&'b T, Self) {
        let tmp = Box::new(x);
        (Box::leak(tmp), self)
    }
}

pub struct LocalPlumber<'b,T>(pub &'b mut [T]);

impl<'b,T> Alloc<'b,T> for LocalPlumber<'b,T> {
    fn obtain(self, x:T) -> (&'b T, Self) {
        let (cell,rest) = self.0.split_at_mut(1);
        let elem = &mut cell[0];
        *elem = x;
        return (elem, LocalPlumber(rest))
    }
}
