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
        let mut temp: &'b mut [T] = &mut [];
        std::mem::swap(&mut self.0, &mut temp);
        let cell;
        (cell,self.0) = temp.split_at_mut(1);
        let elem = &mut cell[0];
        *elem = x;
        return elem;
    }
}
