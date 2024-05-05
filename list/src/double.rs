// Copyright (c) 2023 Xu Shaohua <shaohua@biofan.org>. All rights reserved.
// Use of this source is governed by General Public License that can be
// found in the LICENSE file.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;
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

pub struct IntoIter<T>(DoublyLinkedList<T>);

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

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new()
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

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.length
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Add a new node to tail of list.
    pub fn push_back(&mut self, value: T) {
        let new_tail = Node::new(value);
        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(new_tail.clone());
                // Set previous pointer.
                new_tail.borrow_mut().previous = Some(old_tail);
            }
            None => self.head = Some(new_tail.clone()),
        }
        self.tail = Some(new_tail);
        self.length += 1;
    }

    /// Add a new node to head of list.
    pub fn push_front(&mut self, value: T) {
        let new_head = Node::new(value);
        match self.head.take() {
            Some(old_head) => {
                old_head.borrow_mut().previous = Some(new_head.clone());
                new_head.borrow_mut().next = Some(old_head);
            }
            None => self.tail = Some(new_head.clone()),
        }
        self.head = Some(new_head);
        self.length += 1;
    }

    /// Remove one node from head of list.
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().and_then(|old_head| {
            if let Some(new_head) = old_head.borrow_mut().next.take() {
                // Reset previous pointer.
                new_head.borrow_mut().previous = None;
                self.head = Some(new_head);
            } else {
                // Reset tail to None if both head and tail points to the same node.
                self.tail.take();
            }
            self.length -= 1;

            // Extract value from head if it has only one strong reference.
            Rc::try_unwrap(old_head)
                .ok()
                .map(|head| head.into_inner().value)
        })
    }

    /// Remove one node from tail of list.
    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().and_then(|old_tail| {
            if let Some(new_tail) = old_tail.borrow_mut().previous.take() {
                // Reset next pointer.
                new_tail.borrow_mut().next = None;
                self.tail = Some(new_tail);
            } else {
                // Reset head to None if both head and tail points to the same node.
                self.head.take();
            }
            self.length -= 1;

            // Extract value from tail if it has only one strong reference.
            Rc::try_unwrap(old_tail)
                .ok()
                .map(|tail| tail.into_inner().value)
        })
    }

    /// Get a reference to the front node, or `None` if the list is empty.
    #[inline]
    #[must_use]
    pub fn front(&self) -> Option<&T> {
        unsafe {
            self.head
                .as_ref()
                .and_then(|node: &Rc<RefCell<Node<T>>>| node.try_borrow_unguarded().ok())
                .map(|node| &node.value)
        }
    }

    /// Get a mutable reference to the front node, or `None` if the list is empty.
    #[inline]
    #[must_use]
    pub fn front_mut(&mut self) -> Option<RefMut<T>> {
        self.head
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.value))
    }

    /// Get a reference to the back node, or `None` if the list is empty.
    #[inline]
    #[must_use]
    pub fn back(&self) -> Option<&T> {
        unsafe {
            self.tail
                .as_ref()
                .and_then(|node| node.try_borrow_unguarded().ok())
                .map(|node| &node.value)
        }
    }

    /// Get a mutable reference to the back node, or `None` if the list is empty.
    #[inline]
    #[must_use]
    pub fn back_mut(&mut self) -> Option<RefMut<T>> {
        self.tail
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.value))
    }
}

impl<T> IntoIterator for DoublyLinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {
            // Empty
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

#[cfg(test)]
mod tests {
    use super::DoublyLinkedList;

    #[test]
    fn test_new() {
        let list = DoublyLinkedList::<i32>::new();
        assert!(list.is_empty());
    }

    #[test]
    fn test_push() {
        let mut list = DoublyLinkedList::new();
        list.push_front(2);
        list.push_front(3);
        list.push_front(5);
        list.push_front(7);
        list.push_front(11);
        assert_eq!(list.len(), 5);
    }

    #[test]
    fn test_pop_front() {
        let mut list = DoublyLinkedList::new();
        list.push_front(3);
        list.push_front(5);
        list.push_front(7);
        assert_eq!(list.pop_front(), Some(7));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(3));
        assert!(list.is_empty());
    }

    #[test]
    fn test_pop_back() {
        let mut list = DoublyLinkedList::new();
        list.push_back(3);
        list.push_back(5);
        list.push_back(7);
        assert_eq!(list.pop_back(), Some(7));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(3));
        assert!(list.is_empty());
    }

    #[test]
    fn test_back() {
        let mut list = DoublyLinkedList::new();
        list.push_back(5);
        list.push_back(7);
        assert_eq!(list.back(), Some(&7));
        assert_eq!(list.front(), Some(&5));
    }

    #[test]
    fn test_back_mut() {
        let mut list = DoublyLinkedList::new();
        list.push_back(5);
        list.push_back(7);
        if let Some(mut value) = list.back_mut() {
            *value = 11;
        }
        assert_eq!(list.back(), Some(&11));
    }

    #[test]
    fn test_drop() {
        let mut list = DoublyLinkedList::new();
        for i in 0..(128 * 200) {
            list.push_front(i);
        }
        drop(list);
    }

    #[test]
    fn test_into_iter() {
        let mut list = DoublyLinkedList::new();
        list.push_front(2);
        list.push_front(3);
        list.push_front(5);
        list.push_front(7);
        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(7));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);
    }
}
