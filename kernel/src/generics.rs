use core::{
    cmp::{Eq, PartialEq},
    fmt,
    fmt::Display,
    iter::Iterator,
    marker::PhantomData,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct Link<T> {
    ptr: usize,
    _marker: PhantomData<T>,
}

impl<T> fmt::Display for Link<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Link({:#012x})", self.ptr)
    }
}

impl<T> PartialEq for Link<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Eq for Link<T> {}

impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Link<T> {}

impl<T> Default for Link<T> {
    fn default() -> Self {
        Self {
            ptr: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> Link<T> {
    pub const fn none() -> Self {
        Self {
            ptr: 0,
            _marker: PhantomData,
        }
    }

    pub const fn some(p: usize) -> Self {
        Self {
            ptr: p,
            _marker: PhantomData,
        }
    }

    pub fn is_none(&self) -> bool {
        self.ptr == 0
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn resolve(&self) -> &T {
        unsafe { core::mem::transmute::<usize, &T>(self.ptr) }
    }
    pub fn resolve_mut(&self) -> &mut T {
        unsafe { core::mem::transmute::<usize, &mut T>(self.ptr) }
    }

    pub fn take(&mut self) -> usize {
        let p = self.ptr;
        self.ptr = 0;
        p
    }
}

pub trait DoublyLinkable {
    type T;
    fn set_prev(&mut self, link: Link<Self::T>);
    fn set_next(&mut self, link: Link<Self::T>);

    fn prev(&self) -> Link<Self::T>;
    fn next(&self) -> Link<Self::T>;
}

pub struct DoublyLinkedList<T> {
    head: Link<T>,
    tail: Link<T>,
    _len: usize,
}

pub struct SinglyLinkedList<T> {
    head: Link<T>,
    _len: usize,
}

impl<T> DoublyLinkedList<T>
where
    T: DoublyLinkable + DoublyLinkable<T = T>,
{
    pub fn new() -> Self {
        Self {
            head: Link::none(),
            tail: Link::none(),
            _len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self._len
    }

    pub fn push_front(&mut self, p: usize) {
        if self.head.is_none() {
            let link: Link<T> = Link::some(p);
            self.head = link;
            self.tail = link;
        } else {
            let link: Link<T> = Link::some(p);

            // update the inserted
            link.resolve_mut().set_next(self.head);

            // update the head
            self.head.resolve_mut().set_prev(link);
            self.head = link;
        }

        self._len = self._len + 1;
    }

    pub fn push_back(&mut self, p: usize) {
        if self.tail.is_none() {
            let link: Link<T> = Link::some(p);
            self.head = link;
            self.tail = link;
        } else {
            let link: Link<T> = Link::some(p);

            // update the inserted
            link.resolve_mut().set_prev(self.tail);

            // update the tail
            self.tail.resolve_mut().set_next(link);
            self.tail = link;
        }

        self._len = self._len + 1;
    }

    pub fn remove(&mut self, p: usize) -> bool {
        let l = Link::some(p);
        let mut removed = Link::none();
        if self.len() == 0 {
        } else if self.head == l && self.len() == 1 {
            removed = self.head;
            self.head = Link::none();
            self.tail = Link::none();
        } else if self.head == l {
            let new_head = self.head.resolve().next();
            new_head.resolve_mut().set_prev(Link::none());
            removed = self.head;
            self.head = new_head;
        } else {
            for c in self.iter() {
                if c == l {
                    if c == self.tail {
                        let new_tail = self.tail.resolve().prev();
                        new_tail.resolve_mut().set_next(Link::none());
                        removed = c;
                        self.tail = new_tail;
                        break;
                    } else {
                        let before_c = c.resolve().prev();
                        let next_c = c.resolve().next();
                        before_c.resolve_mut().set_next(next_c);
                        next_c.resolve_mut().set_prev(before_c);
                        removed = c;
                        break;
                    }
                }
            }
        }

        if removed.is_some() {
            self._len = self._len - 1;
            removed.resolve_mut().set_prev(Link::none());
            removed.resolve_mut().set_next(Link::none());
        }

        removed.is_some()
    }

    pub fn iter(&self) -> DoublyLinkedListIterator<T> {
        DoublyLinkedListIterator::new(self.head)
    }

    pub fn head(&self) -> Link<T> {
        self.head
    }
    pub fn tail(&self) -> Link<T> {
        self.tail
    }
}

pub struct DoublyLinkedListIterator<T> {
    next: Link<T>,
}

impl<T> DoublyLinkedListIterator<T> {
    pub fn new(h: Link<T>) -> Self {
        Self { next: h }
    }
}

impl<T> Iterator for DoublyLinkedListIterator<T>
where
    T: DoublyLinkable + DoublyLinkable<T = T>,
{
    type Item = Link<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_none() {
            None
        } else {
            let t = self.next;
            self.next = self.next.resolve().next();
            Some(t)
        }
    }
}

impl<T> fmt::Display for DoublyLinkedList<T>
where
    T: Display + DoublyLinkable + DoublyLinkable<T = T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for l in self.iter() {
            write!(f, "{} -> ", *l.resolve())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{impl_doubly_linkable, println};
    use test_macros::kernel_test;

    #[derive(Debug)]
    struct TestLinkable {
        v: usize,
        prev_link: Link<Self>,
        next_link: Link<Self>,
    }

    impl_doubly_linkable!(TestLinkable);

    impl fmt::Display for TestLinkable {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{},{},{}", self.prev_link, self.v, self.next_link)
        }
    }

    impl TestLinkable {
        pub fn new(v: usize) -> Self {
            Self {
                v,
                prev_link: Link::none(),
                next_link: Link::none(),
            }
        }
    }

    #[kernel_test]
    fn test_linked_list() {
        {
            let ts = [
                TestLinkable::new(0),
                TestLinkable::new(1),
                TestLinkable::new(2),
                TestLinkable::new(3),
                TestLinkable::new(4),
            ];
            let mut dll: DoublyLinkedList<TestLinkable> = DoublyLinkedList::new();
            for t in ts.iter() {
                dll.push_front(t as *const _ as usize);
            }

            for (i, l) in dll.iter().enumerate() {
                assert_eq!(
                    &ts[ts.len() - 1 - i] as *const _ as usize,
                    l.resolve() as *const _ as usize
                );
            }

            assert!(dll.remove(&ts[2] as *const TestLinkable as usize));
            // println!("after remove 2 =  {}", dll);
            assert!(!dll.remove(&ts[2] as *const TestLinkable as usize));
            assert!(dll.remove(&ts[0] as *const TestLinkable as usize));
            // println!("after remove 0 =  {}", dll);
            assert!(dll.remove(&ts[4] as *const TestLinkable as usize));
            // println!("after remove 4 =  {}", dll);
            let ts2 = [
                TestLinkable::new(5),
                TestLinkable::new(6),
                TestLinkable::new(7),
                TestLinkable::new(8),
                TestLinkable::new(9),
            ];
            for t in ts2.iter() {
                dll.push_front(t as *const _ as usize);
            }
            // println!("dll {}", dll);
        }
        {
            let ts = [
                TestLinkable::new(0),
                TestLinkable::new(1),
                TestLinkable::new(2),
                TestLinkable::new(3),
                TestLinkable::new(4),
            ];
            let mut dll: DoublyLinkedList<TestLinkable> = DoublyLinkedList::new();
            for t in ts.iter() {
                dll.push_back(t as *const _ as usize);
            }

            for (i, l) in dll.iter().enumerate() {
                assert_eq!(
                    &ts[i] as *const _ as usize,
                    l.resolve() as *const _ as usize
                );
            }

            assert!(dll.remove(&ts[2] as *const TestLinkable as usize));
            println!("after remove 2 =  {}", dll);
            assert!(!dll.remove(&ts[2] as *const TestLinkable as usize));
            assert!(dll.remove(&ts[0] as *const TestLinkable as usize));
            println!("after remove 0 =  {}", dll);
            assert!(dll.remove(&ts[4] as *const TestLinkable as usize));
            println!("after remove 4 =  {}", dll);
            let ts2 = [
                TestLinkable::new(5),
                TestLinkable::new(6),
                TestLinkable::new(7),
                TestLinkable::new(8),
                TestLinkable::new(9),
            ];
            for t in ts2.iter() {
                dll.push_front(t as *const _ as usize);
            }
            println!("dll {}", dll);
        }
    }
}
