#![allow(clippy::type_complexity)]

use std::iter::Peekable;
use std::marker::PhantomData;
use std::cmp::{Ordering, Ordering::{Less, Equal, Greater}};

pub trait IteratorExtensions: Iterator {
    fn merge_by<B, F>(self, b: B, f: F) -> MergeBy<Self::Item, Self, B::IntoIter, F>
    where
        Self: Sized,
        B: IntoIterator<Item = Self::Item>,
        F: FnMut(&Self::Item, &Self::Item) -> Ordering;

    fn merge<B>(self, b: B) -> MergeBy<Self::Item, Self, B::IntoIter, fn(&Self::Item, &Self::Item) -> Ordering>
    where
        Self: Sized,
        B: IntoIterator<Item = Self::Item>,
        Self::Item: Ord;
}

impl<T> IteratorExtensions for T
where
    T: Iterator
{
    fn merge_by<B, F>(self, b: B, f: F) -> MergeBy<Self::Item, Self, B::IntoIter, F>
    where
        Self: Sized,
        B: IntoIterator<Item = Self::Item>,
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
    {
        MergeBy::new(self, b.into_iter(), f)
    }

    fn merge<B>(self, b: B) -> MergeBy<Self::Item, Self, B::IntoIter, fn(&Self::Item, &Self::Item) -> Ordering>
    where
        Self: Sized,
        B: IntoIterator<Item = Self::Item>,
        Self::Item: Ord,
    {
        MergeBy::new(self, b.into_iter(), Ord::cmp)
    }
}


/// Iterates in order over two ordererd iterators.
#[derive(Clone)]
pub struct MergeBy<T, A, B, F>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
    F: FnMut(&T, &T) -> Ordering,
{
    a: Peekable<A>,
    b: Peekable<B>,
    f: F,
    item_type: PhantomData<T>,
}

impl<T, A, B, F> MergeBy<T, A, B, F>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
    F: FnMut(&T, &T) -> Ordering,
{
    fn new(a: A, b: B, f: F) -> Self {
        Self {
            a: a.peekable(),
            b: b.peekable(),
            f,
            item_type: PhantomData,
        }
    }
}

impl<T, A, B, F> Iterator for MergeBy<T, A, B, F>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
    F: FnMut(&T, &T) -> Ordering,
{
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        match (self.a.peek(), self.b.peek()) {
            (None, _) => {
                self.b.next()
            },
            (_, None) => {
                self.a.next()
            },
            (Some(a), Some(b)) => {
                match (self.f)(a, b) {
                    Less    => { self.a.next() },
                    Equal   => { self.a.next(); self.next() }
                    Greater => { self.b.next() },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IteratorExtensions;

    #[test]
    fn merge_by_test() {
        let a = vec![1, 3, 5, 7, 9];
        let b = vec![2, 4, 6, 8];
        assert_eq!(a.into_iter().merge(b).collect::<Vec<_>>(), &[1,2,3,4,5,6,7,8,9]);
    }
}