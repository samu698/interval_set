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

impl<Idx: Step> IntervalSet<Idx> {
    /// Returns the empty interval set
    pub fn empty() -> Self {
        Self { intervals: vec![] }
    }

    /// Returns the set the contains a single interval
    pub fn interval(interval: impl Into<Interval<Idx>>) -> Self {
        Self { intervals: vec![ interval.into() ] }
    }

    /// Returns the set the contains a single value
    pub fn single(value: Idx) -> Self {
        Self { intervals: vec![ (value.clone()..=value).into() ] }
    }

    /// Inserts an interval in the set
    pub fn insert(&mut self, interval: impl Into<Interval<Idx>>) {
        // TODO: make this better
        let tmp = Self::interval(interval);
        *self = self.union(&tmp);
    }

    /// Inserts a single value in the set
    pub fn insert_single(&mut self, value: Idx) {
        let tmp = Self::single(value);
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
            if interval.lo() <= &Idx::forward(prev.hi().clone()) {
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
}

impl<Idx> Debug for IntervalSet<Idx>
    where Idx: Ord + Step + Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.intervals)
    }
}

impl<Idx> Display for IntervalSet<Idx>
    where Idx: Ord + Step + Display
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
