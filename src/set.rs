use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::iter::Peekable;

use crate::traits::Bounded;
use crate::{Interval, Step};

/// Datatype for storing a set of intervals.
///
/// The number of intervals are minimized, intervals are merged when possible:
/// when intervals are overlapping or touching (one end of an interval is
/// the successor/predecessor of the one end of the other interval)
#[derive(Clone)]
pub struct IntervalSet<Idx: Step> {
    intervals: Vec<Interval<Idx>>
}

/// Macro helper for initializing an [`IntervalSet`]
#[macro_export]
macro_rules! iset {
    [] => { IntervalSet::empty() };
    [$first:expr $(, $int:expr)* $(,)?] => {{
        let mut __set = $crate::IntervalSet::interval($first);
        $(__set.insert($int);)*;
        __set
    }};
}

impl<Idx: Step> IntervalSet<Idx> {
    /// Returns the empty interval set
    pub fn empty() -> Self {
        Self { intervals: vec![] }
    }

    /// Returns the set the contains a single interval
    pub fn interval(interval: impl Into<Interval<Idx>>) -> Self {
        Self { intervals: vec![ interval.into() ] }
    }

    /// Return the number of intervals contained in the set
    ///
    /// To get the number of elements in the set use [`IntervalSet::size`] or
    /// [`IntervalSet::size_exact`]
    pub fn intervals(&self) -> usize {
        self.intervals.len()
    }

    /// Returns a lower bound for the number of elements in the set
    ///
    /// The returned value can be lower than the real number of elements,
    /// use [`IntervalSet::size_exact`] to get the exact size.
    ///
    /// If this value is less than [`usize::MAX`] then the value is always
    /// correct
    pub fn size(&self) -> usize {
        let mut size = 0usize;
        for interval in self.iter() {
            size = match size.checked_add(interval.size()) {
                Some(sum) => sum,
                None => return usize::MAX,
            };
        }
        size
    }

    /// Returns the number of elements in the set
    ///
    /// This value is [`None`] when the number of elements is greater than
    /// `usize::MAX` and would overflow `usize`
    pub fn size_exact(&self) -> Option<usize> {
        let mut size = 0usize;
        for interval in self.iter() {
            size = match size.checked_add(interval.size_exact()?) {
                Some(sum) => sum,
                None => return None,
            };
        }
        Some(size)
    }

    /// Inserts an interval in the set
    pub fn insert(&mut self, interval: impl Into<Interval<Idx>>) {
        // TODO: make this better
        let tmp = Self::interval(interval);
        *self = self.union(&tmp);
    }

    /// Performs the union between two sets
    pub fn union(&self, other: &Self) -> Self {
        let mut result = vec![];

        let mut iter = MergeIter::new(
            self.iter(), other.iter(),
            |l, r| l.lo().cmp(r.lo())
        );

        let mut prev = match iter.next() {
            Some(p) => p.clone(),
            None => return Self { intervals: result }
        };
        for interval in iter {
            if interval.lo() <= &Idx::forward(prev.hi()) {
                prev = prev.hull(interval);
            } else {
                result.push(prev);
                prev = interval.clone();
            }
        }
        result.push(prev);

        Self { intervals: result }
    }

    /// Performs the intersection between two sets
    pub fn intersection(&self, other: &Self) -> Self {
        let mut result = vec![];

        let mut iter = MergeIter::new(
            self.iter(), other.iter(),
            |l, r| l.lo().cmp(r.lo())
        );

        let mut prev = match iter.next() {
            Some(p) => p.clone(),
            None => return Self { intervals: result }
        };
        for interval in iter {
            if interval.lo() <= prev.hi() {
                if let Some(intersection) = prev.intersection(interval) {
                    result.push(intersection);
                }

                if interval.hi() > prev.hi() {
                    prev = interval.clone();
                }
            } else {
                prev = interval.clone();
            }
        }

        Self { intervals: result }
    }

    /// Computes the difference between the two sets
    ///
    /// The result is the set containing all elements in `self` but not in
    /// `other`
    pub fn difference(&self, other: &Self) -> Self {
        let mut result = vec![];

        let mut a_iter = self.iter();
        let mut b_iter = other.iter();

        let mut a_int = a_iter.next();
        let mut b_int = b_iter.next();

        let (mut right, mut left);
        while let (Some(a), Some(b)) = (a_int, b_int) {
            (left, right) = a.difference(b);
            if let Some(left) = left { result.push(left); }
            a_int = match right {
                Some(ref r) => {
                    b_int = b_iter.next();
                    Some(r)
                }
                None => a_iter.next()
            };
        }

        if let Some(a) = a_int { result.push(a.clone()); }
        result.extend(a_iter.cloned());

        Self { intervals: result }
    }

    /// Returns the iterator over all the intervals in the set
    pub fn iter(&self) -> std::slice::Iter<'_, Interval<Idx>> {
        self.intervals.iter()
    }
}

impl<Idx> IntervalSet<Idx>
    where Idx: Bounded + Step
{
    /// Returns the set containing all possible values of the type
    ///
    /// This operation requires the the index is [`Bounded`]
    pub fn full() -> Self {
        Self { intervals: vec![Interval::full()] }
    }

    /// Takes the complement of the set, retuning the set that contains the
    /// elements not in the current set
    ///
    /// This operation requires the the index is [`Bounded`]
    pub fn complement(&self) -> Self {
        Self::full().difference(self)
    }
}

impl<Idx> Debug for IntervalSet<Idx>
    where Idx: Debug + Step
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.intervals)
    }
}

impl<Idx> Display for IntervalSet<Idx>
    where Idx: Display + Step
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(")?;
        let mut iter = self.iter();
        if let Some(first) = iter.next() {
            write!(f, "{first}")?;
            for interval in iter {
                write!(f, " U {interval}")?;
            }
        }
        write!(f, ")")
    }
}

impl<Idx: Step> IntoIterator for IntervalSet<Idx> {
    type Item = Interval<Idx>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.intervals.into_iter()
    }
}

struct MergeIter<'a, Idx, Lhs, Rhs, F> where
    Idx: Ord + Step + 'a,
    Lhs: Iterator<Item = &'a Interval<Idx>>,
    Rhs: Iterator<Item = &'a Interval<Idx>>,
    F: FnMut(&Interval<Idx>, &Interval<Idx>) -> Ordering,
{
    lhs: Peekable<Lhs>,
    rhs: Peekable<Rhs>,
    f: F
}

impl<'a, Idx, Lhs, Rhs, F> Iterator for MergeIter<'a, Idx, Lhs, Rhs, F> where
    Idx: Ord + Step + 'a,
    Lhs: Iterator<Item = &'a Interval<Idx>>,
    Rhs: Iterator<Item = &'a Interval<Idx>>,
    F: FnMut(&Interval<Idx>, &Interval<Idx>) -> Ordering,
{
    type Item = &'a Interval<Idx>;
    fn next(&mut self) -> Option<&'a Interval<Idx>> {
        let f = &mut self.f;
        match (self.lhs.peek(), self.rhs.peek()) {
            (Some(lhs), Some(rhs)) if f(lhs, rhs).is_le() => self.lhs.next(),
            (Some(_), Some(_)) => self.rhs.next(),
            (Some(_), None) => self.lhs.next(),
            (None, Some(_)) => self.rhs.next(),
            (None, None) => None,
        }
    }
}

impl<'a, Idx, Lhs, Rhs, F> MergeIter<'a, Idx, Lhs, Rhs, F> where
    Idx: Ord + Step + 'a,
    Lhs: Iterator<Item = &'a Interval<Idx>>,
    Rhs: Iterator<Item = &'a Interval<Idx>>,
    F: FnMut(&Interval<Idx>, &Interval<Idx>) -> Ordering,
{
    fn new(lhs: Lhs, rhs: Rhs, f: F) -> Self {
        Self {
            lhs: lhs.peekable(),
            rhs: rhs.peekable(),
            f
        }
    }
}
