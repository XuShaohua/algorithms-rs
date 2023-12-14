// Copyright (c) 2023 Xu Shaohua <shaohua@biofan.org>. All rights reserved.
// Use of this source is governed by General Public License that can be
// found in the LICENSE file.

use std::cell::RefCell;
use std::rc::Rc;

type NodePtr<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct Node<T> {
    value: T,
    next: NodePtr<T>,
    previous: NodePtr<T>,
}

pub struct DoublyLinkedList<T> {
    length: usize,
    head: NodePtr<T>,
    tail: NodePtr<T>,
}

pub struct ListIterator<'a T> {
    current: NodePtr<'a T>,
}

impl<T> Node<T> {
    #[must_use]
    pub fn new(value: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            value,
            next: None,
            previous: None,
        }))
    }
}

impl<T> DoublyLinkedList<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            length: 0,
            head: None,
            tail: None,
        }
    }

    pub fn append(&mut self, value: T) {
        let tail = Node::new(value);
        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(tail.clone());
                // Set previous pointer.
                tail.borrow_mut().previous = Some(old_tail);
            }
            None => self.head = Some(tail.clone()),
        }
        self.tail = Some(tail);
        self.length += 1;
    }

    /// # Panics
    /// Raise error if failed to extract node.
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|head: Rc<RefCell<Node<T>>>| {
            if let Some(next) = head.borrow_mut().next.take() {
                // Reset previous pointer.
                next.borrow_mut().previous = None;
                self.head = Some(next);
            } else {
                self.tail.take();
            }
            self.length -= 1;
            let node: Option<RefCell<Node<T>>> = Rc::try_unwrap(head).ok();
            node.expect("").into_inner().value
        })
    }
}

impl<T> ListIterator<T> {
    #[must_use]
    pub const fn new(started_at: NodePtr<T>) -> Self {
        Self {
            current: started_at,
        }
    }
}

impl<T> Iterator for ListIterator<T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = &self.current;
        let mut result = None;
        self.current = current.as_ref().and_then(|current| {
            let current = current.borrow();
            result = Some(&current.value);
            current.next.clone()
        });

        result
    }
}

impl<T> DoubleEndedIterator for ListIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let current = &self.current;
        let mut result = None;
        self.current = current.as_ref().and_then(|current| {
            let current = current.borrow();
            result = Some(current.value.clone());
            current.previous.clone()
        });
        result
    }
}
