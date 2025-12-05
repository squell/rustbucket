#![allow(dead_code)]
//! A LinkedList sandbox

fn main() {
    let mut list: LinkedList<_> = (0..5).collect();
    println!("sum: {}", list.reduce(|x: u64, y| x + y));

    let mut cursor = list.iter_mut();
    while let Some(val) = cursor.value() {
        println!("{}", val);
        if *val == 3 {
            for elem in cursor.cut() {
                println!("CUT: {}", elem);
            }
            cursor.next();
        } else {
            cursor.next();
        }
    }
    println!("---");
    for elem in list.iter_mut() {
        println!("{}", elem);
    }
}

struct LinkedList<'a, T>(*mut Node<'a, T>);

impl<T> LinkedList<'_, T> {
    fn new() -> Self {
        LinkedList(std::ptr::null_mut())
    }

    fn singleton(value: T) -> Self {
        let node = Box::into_raw(Box::new(Node {
            value,
            rest: std::ptr::null_mut(),
        }));

        LinkedList(node)
    }

    fn pop(&mut self) -> Option<T> {
        let mut node: Box<Node<T>> = if self.0.is_null() {
            return None;
        } else {
            unsafe { Box::from_raw(self.0) }
        };
        self.0 = node.rest;
        node.rest = std::ptr::null_mut();

        Some(node.value)
    }
}

struct Node<'a, T> {
    value: T,
    rest: *mut Node<'a, T>,
}

impl<T> FromIterator<T> for LinkedList<'_, T> {
    fn from_iter<C: IntoIterator<Item = T>>(source: C) -> Self {
        let mut built = LinkedList::new();
        let mut last = &mut built.0;
        for value in source.into_iter() {
            let new = LinkedList::singleton(value);
            *last = new.0;
            last = unsafe { &mut (*new.0).rest };
            std::mem::forget(new)
        }

        built
    }
}

impl<'a, T: 'a> LinkedList<'a, T> {
    fn reduce<B: Default>(&self, mut op: impl FnMut(B, &T) -> B) -> B {
        let mut acc = Default::default();
        for elem in self.iter() {
            acc = op(acc, elem)
        }

        acc
    }
}

impl<T> Drop for LinkedList<'_, T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

struct Iter<'a, T: 'a> {
    ptr: *const Node<'a, T>,
}

struct IntoIter<'a, T> {
    list: LinkedList<'a, T>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.ptr.is_null() {
            let node = unsafe { &*self.ptr };
            let value = &node.value;
            self.ptr = node.rest;

            Some(value)
        } else {
            None
        }
    }
}

impl<T> Iterator for IntoIter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop()
    }
}

struct Cursor<'b, 'a, T> {
    referent: &'b mut *mut Node<'a, T>,
}

impl<'a, T> Cursor<'_, 'a, T> {
    fn value(&self) -> Option<&T> {
        if !self.referent.is_null() {
            let node = unsafe { &**self.referent };
            Some(&node.value)
        } else {
            None
        }
    }

    fn value_mut(&mut self) -> Option<&mut T> {
        if !self.referent.is_null() {
            let node = unsafe { &mut **self.referent };
            Some(&mut node.value)
        } else {
            None
        }
    }

    fn delete(&mut self) -> LinkedList<'a, T> {
        let deleted_node;
        *self.referent = if !self.referent.is_null() {
            deleted_node = LinkedList(*self.referent);
            unsafe { (**self.referent).rest }
        } else {
            panic!("attempt at null pointer dereference")
        };

        unsafe { (*deleted_node.0).rest = std::ptr::null_mut() };
        deleted_node
    }

    fn insert_before(&mut self, value: T) {
        let new_node = LinkedList::singleton(value);
        unsafe { (*new_node.0).rest = *self.referent };
        *self.referent = new_node.0;

        std::mem::forget(new_node);
        self.next();
    }

    fn insert_after(&mut self, value: T) {
        let new_node = LinkedList::singleton(value);
        let next = if !self.referent.is_null() {
            unsafe { &mut (**self.referent).rest }
        } else {
            panic!("attemt at null pointer dereference")
        };
        unsafe { (*new_node.0).rest = *next };
        *next = new_node.0;
        std::mem::forget(new_node);
    }

    fn cut(&mut self) -> LinkedList<'a, T> {
        let result = LinkedList(*self.referent);
        *self.referent = std::ptr::null_mut();

        result
    }
}

impl<'a, T> Iterator for Cursor<'a, '_, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = if !self.referent.is_null() {
            unsafe { &mut **self.referent }
        } else {
            return None;
        };
        self.referent = &mut node.rest;

        Some(&mut node.value)
    }
}

impl<'a, T> LinkedList<'a, T> {
    fn iter(&self) -> Iter<'a, T> {
        Iter { ptr: self.0 }
    }

    fn iter_mut(&mut self) -> Cursor<'_, 'a, T> {
        Cursor {
            referent: &mut self.0,
        }
    }

    fn reverse(&mut self) {
        let mut cur = self.0;
        let mut tail = std::ptr::null_mut();
        while !cur.is_null() {
            let node = unsafe { &mut *cur };
            (node.rest, cur, tail) = (tail, node.rest, cur)
        }
        self.0 = tail;
    }
}

impl<'a, T> IntoIterator for LinkedList<'a, T> {
    type Item = T;
    type IntoIter = IntoIter<'a, T>;
    fn into_iter(self) -> IntoIter<'a, T> {
        IntoIter { list: self }
    }
}

#[cfg(test)]
mod tests {
    use super::LinkedList;

    #[test]
    fn test_iter() {
        let list = ["aap", "noot", "mies"]
            .into_iter()
            .collect::<LinkedList<_>>();

        assert_eq!(
            vec!["aap", "noot", "mies"],
            list.iter().cloned().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_reverse() {
        let mut list = ["wim", "zus", "jet", "schaap"]
            .into_iter()
            .collect::<LinkedList<_>>();

        list.reverse();

        assert_eq!(
            vec!["schaap", "jet", "zus", "wim"],
            list.into_iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_mut() {
        let mut list = (0u8..5u8).collect::<LinkedList<_>>();

        assert_eq!(
            vec![0, 1, 2, 3, 4],
            list.iter().cloned().collect::<Vec<_>>()
        );

        for elem in list.iter_mut() {
            *elem = 5
        }

        assert_eq!(vec![5, 5, 5, 5, 5], list.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_cursor_iter() {
        let mut list = ["aap", "noot", "mies"]
            .into_iter()
            .collect::<LinkedList<_>>();

        assert_eq!(
            vec!["aap", "noot", "mies"],
            list.iter_mut().map(|x| -> &str { x }).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_cursor_delete() {
        for i in 0..=2 {
            let mut vec = vec!["app", "noot", "mies"];
            let mut list = vec.iter().cloned().collect::<LinkedList<_>>();

            let mut cursor = list.iter_mut();
            for _ in 0..i {
                cursor.next();
            }
            cursor.delete();

            vec.remove(i);

            assert_eq!(
                vec,
                list.iter_mut().map(|x| -> &str { x }).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_cursor_insert1() {
        for i in 0..=3 {
            let mut vec = vec!["app", "noot", "mies"];
            let mut list = vec.iter().cloned().collect::<LinkedList<_>>();

            let mut cursor = list.iter_mut();
            for _ in 0..i {
                cursor.next();
            }
            cursor.insert_before("flierp");

            vec.insert(i, "flierp");

            assert_eq!(
                vec,
                list.iter_mut().map(|x| -> &str { x }).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_cursor_insert2() {
        for i in 0..=2 {
            let mut vec = vec!["app", "noot", "mies"];
            let mut list = vec.iter().cloned().collect::<LinkedList<_>>();

            let mut cursor = list.iter_mut();
            for _ in 0..i {
                cursor.next();
            }
            cursor.insert_after("flierp");

            vec.insert(i + 1, "flierp");

            assert_eq!(
                vec,
                list.iter_mut().map(|x| -> &str { x }).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_panic() {
        let vec = ["app", "noot", "mies"];
        let mut list = vec.iter().cloned().collect::<LinkedList<_>>();
        let mut cursor = list.iter_mut();
        for _ in 0..10 {
            println!("{:?}", cursor.next());
        }
    }

    fn static_test_lifetime() {
        fn foo(mut iter: super::Iter<'_, u8>) -> Option<&'_ u8> {
            iter.next()
        }
    }

    fn invalid<'a, 'b>(list: &'b mut LinkedList<'a, ()>) -> super::Cursor<'b, 'a, ()> {
        let mut cursor = list.iter_mut();
        cursor.next();
        cursor.next();
        cursor.next();
        cursor
    }

    #[test]
    #[should_panic]
    fn test_ub_delete() {
        let mut empty = LinkedList::new();
        invalid(&mut empty).delete();
    }

    #[test]
    fn test_ub_insert1() {
        let mut empty = LinkedList::new();
        invalid(&mut empty).insert_before(());
    }

    #[test]
    #[should_panic]
    fn test_ub_insert2() {
        let mut empty = LinkedList::new();
        invalid(&mut empty).insert_after(());
    }

    #[test]
    #[should_panic]
    fn test_ub_value() {
        let mut list = LinkedList::new();
        invalid(&mut list).value().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_ub_value_mut() {
        let mut list = LinkedList::new();
        invalid(&mut list).value_mut().unwrap();
    }
}
