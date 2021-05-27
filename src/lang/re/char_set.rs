use crate::utils::iter::IteratorExtensions;
use std::iter::once;
use std::cmp::{min, max, Ordering::{Less, Greater}};
use std::slice;
use std::ops::RangeInclusive;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharSet {
    intervals: Vec<(u8, u8)>,
}

impl CharSet {
    #[must_use]
    pub fn empty() -> Self {
        Self { intervals: Vec::new() }
    }

    #[must_use]
    pub fn universe() -> Self {
        Self { intervals: vec![(u8::MIN, u8::MAX)] }
    }

    #[must_use]
    pub fn point(value: u8) -> Self {
        Self { intervals: vec![(value, value)] }
    }

    #[must_use]
    pub fn range(from: u8, to: u8) -> Self {
        Self { intervals: vec![if from <= to { (from, to) } else { (to, from) }] }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    #[must_use]
    pub fn is_universe(&self) -> bool {
        if let Some(&interval) = self.intervals.get(0) {
            interval.0 == u8::MIN && interval.1 == u8::MAX
        } else {
            false
        }
    }

    #[must_use]
    pub fn min(&self) -> Option<u8> {
        if let Some(interval) = self.intervals.first() {
            Some(interval.0)
        } else {
            None
        }
    }

    #[must_use]
    pub fn contains(&self, x: u8) -> bool {
        // Standard binary search. Not using binary_search_by() since
        // we don't need the index of containing interval.

        let mut size = self.intervals.len();
        if size == 0 {
            return false;
        }
        let mut base = 0_usize;

        while size > 1 {
            let half = size / 2;
            let mid = base + half;

            let (start, end) = self.intervals[mid];

            if x < start{
                size -= half;
            } else if x > end {
                size -= half;
                base = mid;
            } else {
                return true;
            }
        }

        let (start, end) = self.intervals[base];

        start <= x && x <= end
    }

    #[must_use]
    pub fn complement(&self) -> Self {
        let intervals = match *self.intervals {
            [(low, _), .., (_, high)] => {
                let iter = self.intervals[..self.intervals.len()].iter().zip(&self.intervals[1..]).map(|((_, a), (b, _))| (a + 1, b - 1));
                match (low, high) {
                    (u8::MIN, u8::MAX) => { iter.collect() },
                    (u8::MIN, high)    => { iter.chain(once((high + 1, u8::MAX))).collect() },
                    (low, u8::MAX)     => { once((u8::MIN, low - 1)).chain(iter).collect() },
                    (low, high)        => { once((u8::MIN, low - 1)).chain(iter).chain(once((high + 1, u8::MAX))).collect() },
                }
            },
            [value] => {
                match value {
                    (u8::MIN, u8::MAX) => { Vec::new() },
                    (u8::MIN, high)    => { vec![(high + 1, u8::MAX)] },
                    (low, u8::MAX)     => { vec![(u8::MIN, low - 1)] },
                    (low, high)        => { vec![(u8::MIN, low - 1), (high + 1, u8::MAX)] },
                }
            },
            [] =>  {
                vec![(u8::MIN, u8::MAX)]
            },
        };

        Self { intervals }
    }

    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        let mut intervals = Vec::new();
        
        let mut iter1 = self.intervals.iter();
        let mut iter2 = other.intervals.iter();
        
        let mut a = if let Some(interval) = iter1.next() { *interval } else { return Self { intervals } };
        let mut b = if let Some(interval) = iter2.next() { *interval } else { return Self { intervals } };

        loop {
            let low = max(a.0, b.0);
            let high = min(a.1, b.1);

            if low <= high {
                intervals.push((low, high));
            }

            if a.1 < b.1 {
                a = if let Some(interval) = iter1.next() { *interval } else { return Self { intervals } };
            } else {
                b = if let Some(interval) = iter2.next() { *interval } else { return Self { intervals } };
            }
        }
    }

    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        // TODO: refactor. Using merge_by() results in more comparisons that necessary.

        let iter1 = self.intervals.iter().copied();
        let iter2 = other.intervals.iter().copied();

        let mut iter = iter1.merge_by(iter2, |i1, i2| {
            if i1.0 <= i2.0 { Less } else { Greater }
        });

        let mut intervals = Vec::new();

        if let Some(mut interval) = iter.next() {
            for next_interval in iter {
                if interval.1 >= next_interval.0 || next_interval.0 - interval.1 == 1 {
                    interval.1 = max(interval.1, next_interval.1);
                } else {
                    intervals.push(interval);
                    interval = next_interval;
                }
            }

            intervals.push(interval);
        }

        Self { intervals }
    }

    #[must_use]
    pub fn chars(&self) -> Chars {
        Chars::new(self)
    }
}

impl std::fmt::Debug for CharSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if self.intervals.is_empty() {
            f.write_str("{}")
        } else {
            f.write_str("{")?;
            f.write_str(&self.intervals.iter().map(|i| {
                if i.0 == i.1 {
                    format!("{:02x}", i.0)
                } else {
                    format!("[{:02x}..{:02x}]", i.0, i.1)
                }
            }).collect::<Vec<_>>().join(","))?;
            f.write_str("}")
        }
    }
}

pub struct Chars<'a> {
    interval_iter: slice::Iter<'a, (u8, u8)>,
    range: RangeInclusive<u8>,
}

impl<'a> Chars<'a> {
    fn new(set: &'a CharSet) -> Self {
        Self {
            interval_iter: set.intervals.iter(),
            range: RangeInclusive::new(1, 0), // starts range as done
        }
    }
}

impl Iterator for Chars<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().or_else(|| {
            if let Some(&(start, end)) = self.interval_iter.next() {
                self.range = RangeInclusive::new(start, end);
                self.range.next()
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CharSet;

    #[test]
    fn contains() {
        let set1 = CharSet::range(10, 20);
        let set2 = CharSet::range(30, 40);
        let set3 = CharSet::range(50, 60);
        let set4 = CharSet::range(70, 80);
        let set5 = CharSet::range(90, 100);
        let set = set1.union(&set2).union(&set3).union(&set4).union(&set5);

        for x in 0..10    { assert!( !set.contains(x) ); }
        for x in 10..=20  { assert!(  set.contains(x) ); }
        for x in 21..30   { assert!( !set.contains(x) ); }
        for x in 30..=40  { assert!(  set.contains(x) ); }
        for x in 41..50   { assert!( !set.contains(x) ); }
        for x in 50..=60  { assert!(  set.contains(x) ); }
        for x in 61..70   { assert!( !set.contains(x) ); }
        for x in 70..=80  { assert!(  set.contains(x) ); }
        for x in 81..90   { assert!( !set.contains(x) ); }
        for x in 90..=100 { assert!(  set.contains(x) ); }
        for x in 101..110 { assert!( !set.contains(x) ); }
    }

    #[test]
    fn intersection() {
        let set1 = CharSet::range(60, 180);
        let set2 = set1.complement();

        assert_eq!(set2.intervals, &[(0, 59), (181, 255)]);
        assert_eq!(set2.intersection(&set1).intervals, &[]);
        assert_eq!(CharSet::empty().intervals, CharSet::range(0, 255).complement().intervals)
    }

    #[test]
    fn union() {
        let set1 = CharSet::range(60, 180);
        let set2 = CharSet::range(10, 20);
        let set3 = CharSet::range(150, 200);

        let union = set1.union(&set2).union(&set3);

        assert_eq!(union.intervals, &[(10, 20), (60, 200)]);
    }

    #[test]
    fn chars() {
        let set = CharSet::range(1, 3).union(&CharSet::range(5, 7));
        let mut iter = set.chars();
        assert_eq!(iter.next(), Some(1_u8));
        assert_eq!(iter.next(), Some(2_u8));
        assert_eq!(iter.next(), Some(3_u8));
        assert_eq!(iter.next(), Some(5_u8));
        assert_eq!(iter.next(), Some(6_u8));
        assert_eq!(iter.next(), Some(7_u8));
        assert_eq!(iter.next(), None);
    }
}