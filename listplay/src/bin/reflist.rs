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
        } else {
            cursor.next();
        }
    }
    println!("---");
    for elem in list {
        println!("{}", elem);
    }
}

struct LinkedList<'a, T>(LinkedListInnard<'a, T>);

impl<T> LinkedList<'_, T> {
    fn new() -> Self {
        LinkedList(None)
    }

    fn singleton(value: T) -> Self {
        let node = Box::leak(Box::new(Node { value, rest: None }));

        LinkedList(Some(node))
    }

    fn pop(&mut self) -> Option<T> {
        // have fun with Miri!
        let mut node: Box<Node<T>> = unsafe { Box::from_raw(self.0.take()?) };
        self.0 = node.rest.take();

        Some(node.value)
    }
}

type LinkedListInnard<'a, T> = Option<&'a mut Node<'a, T>>;

struct Node<'a, T> {
    value: T,
    rest: LinkedListInnard<'a, T>,
}

impl<T> FromIterator<T> for LinkedList<'_, T> {
    fn from_iter<C: IntoIterator<Item = T>>(source: C) -> Self {
        let mut built = LinkedList::new();
        let mut last = &mut built.0;
        for value in source.into_iter() {
            *last = LinkedList::singleton(value).0.take();
            last = &mut last.as_mut().unwrap().rest;
        }

        built
    }
}

impl<T> LinkedList<'_, T> {
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

struct Iter<'b, 'a, T> {
    list: Option<&'b Node<'a, T>>,
}

struct IntoIter<'a, T> {
    list: LinkedList<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, '_, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.list?;
        let value = &node.value;
        self.list = node.rest.as_deref();

        Some(value)
    }
}

impl<T> Iterator for IntoIter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop()
    }
}

struct Cursor<'b, 'a, T> {
    referent: Option<&'b mut LinkedListInnard<'a, T>>,
}

impl<'a, T> Cursor<'_, 'a, T> {
    fn value(&self) -> Option<&T> {
        let list = self.referent.as_deref().unwrap();
        Some(&list.as_deref()?.value)
    }

    fn value_mut(&mut self) -> Option<&mut T> {
        let list = self.referent.as_mut().unwrap();
        Some(&mut list.as_mut()?.value)
    }

    fn delete(&mut self) -> LinkedList<'a, T> {
        let list = self.referent.as_mut().unwrap();

        let next = &mut list.as_mut().unwrap().rest.take();
        let deleted_node = LinkedList(list.take());
        **list = next.take();

        deleted_node
    }

    fn insert_before(&mut self, value: T) {
        let list = self.referent.as_mut().unwrap();

        let mut new_node = LinkedList::singleton(value);
        new_node.0.as_mut().unwrap().rest = list.take();
        **list = new_node.0.take();

        self.next();
    }

    fn insert_after(&mut self, value: T) {
        let list = self.referent.as_mut().unwrap();

        let next = &mut list.as_mut().unwrap().rest;
        let mut new_node = LinkedList::singleton(value);
        new_node.0.as_mut().unwrap().rest = next.take();
        *next = new_node.0.take();
    }

    fn cut(&mut self) -> LinkedList<'a, T> {
        let list = self.referent.as_mut().unwrap();
        LinkedList(list.take())
    }
}

impl<'a, T> Iterator for Cursor<'a, '_, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.referent.as_ref().unwrap().is_none() {
            return None;
        }
        let list = self.referent.take()?;
        let node = list.as_mut().unwrap();
        self.referent = Some(&mut node.rest);

        Some(&mut node.value)
    }
}

impl<'a, T> LinkedList<'a, T> {
    fn iter(&self) -> Iter<'_, 'a, T> {
        Iter {
            list: self.0.as_deref(),
        }
    }

    fn iter_mut(&mut self) -> Cursor<'_, 'a, T> {
        Cursor {
            referent: Some(&mut self.0),
        }
    }

    fn reverse(&mut self) {
        let mut cur = self.0.take();
        let mut tail = None;
        while let Some(node) = cur.take() {
            let next = node.rest.take();
            node.rest = tail.take();
            tail = Some(node);
            cur = next;
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
