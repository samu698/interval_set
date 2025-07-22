use std::fmt::{Debug, Display};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::traits::{Bounded, Step};

/// An interval over the type `Idx`.
///
/// The intervals must be finite, with a lower and upper bound.
/// The lower bound must be less or equal than the upper bound.
#[derive(Clone)]
pub struct Interval<Idx: Step> {
    lo: Idx,
    hi: Idx
}

impl<Idx: Step> Interval<Idx> {
    /// Create a new interval.
    ///
    /// An interval can be constructed from a range `a..b` using `.into()`
    ///
    /// Panics:
    /// - If the upper bound is greater than the lower bound
    #[inline]
    pub fn new(lo: Idx, hi: Idx) -> Self {
        assert!(lo <= hi, "The left bound of an interval must be less or equal that the right bound");
        Self { lo, hi }
    }

    /// Get the lower bound of the interval
    #[inline]
    pub fn lo(&self) -> &Idx { &self.lo }

    /// Get the upper bound of the interval
    #[inline]
    pub fn hi(&self) -> &Idx { &self.hi }

    /// Returns a lower bound for the number of elements in the interval
    ///
    /// The returned value can be lower than the real number of elements,
    /// use [`Interval::size_exact`] to get the exact size.
    ///
    /// If this value is less than [`usize::MAX`] then the value is always
    /// correct
    pub fn size(&self) -> usize {
        Idx::steps_between(&self.lo, &self.hi).0
            .saturating_add(1)
    }

    /// Returns the number of elements in the interval
    ///
    /// This value is [`None`] when the number of elements is greater than
    /// `usize::MAX` and would overflow `usize`
    pub fn size_exact(&self) -> Option<usize> {
        Idx::steps_between(&self.lo, &self.hi).1
            .and_then(|s| s.checked_add(1))
    }

    /// Computes the hull between of the intervals
    ///
    /// The hull is the interval that contains both intervals.
    pub fn hull(&self, other: &Self) -> Self {
        let (l1, l2) = (&self.lo, &other.lo);
        let (h1, h2) = (&self.hi, &other.hi);
        Self {
            lo: l1.min(l2).clone(),
            hi: h1.max(h2).clone()
        }
    }

    /// Computes the intersection of the intervals
    ///
    /// The intersection is the interval contained in both intervals.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let (l1, l2) = (&self.lo, &other.lo);
        let (h1, h2) = (&self.hi, &other.hi);

        let lo = l1.max(l2);
        let hi = h1.min(h2);
        if lo <= hi {
            Some(Self { lo: lo.clone(), hi: hi.clone() })
        } else {
            None
        }
    }

    /// Computes the difference of the intervals
    ///
    /// This function returns a tuple containing the left and the right part
    /// of the difference. The resulting values are `None` when the interval for
    /// that region would be empty
    ///
    /// The left interval contains values *smaller* than the subtrahend while the
    /// right interval contains values *bigger* than the subtrahend.
    ///
    /// Example:
    /// ```
    /// a..................A
    ///       b......B
    /// --------------------
    /// l....L        r....R
    /// ```
    pub fn difference(&self, other: &Self) -> (Option<Self>, Option<Self>) {
        let (a_lo, a_hi) = (&self.lo, &self.hi);
        let (b_lo, b_hi) = (&other.lo, &other.hi);

        let left = if a_lo < b_lo {
            let hi = a_hi.clone().min(Idx::backward(b_lo));
            Some(Self::new(a_lo.clone(), hi))
        } else {
            None
        };

        let right = if a_hi > b_hi {
            let lo = a_lo.clone().max(Idx::forward(b_hi));
            Some(Self::new(lo, a_hi.clone()))
        } else {
            None
        };

        (left, right)
    }

    /// Checks if the interval overlaps another interval
    pub fn overlaps(&self, other: &Self) -> bool {
        self.hi >= other.lo && other.hi >= self.lo
    }
}

impl<Idx> Interval<Idx>
    where Idx: Bounded + Step
{
    /// Get the interval the spans all possible values of the type
    ///
    /// The index type must be [`Bounded`] to use this operation
    pub fn full() -> Self {
        Self::new(Idx::MIN, Idx::MAX)
    }
}

impl<Idx> Debug for Interval<Idx>
    where Idx: Debug + Step
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:?}, {:?})", self.lo, self.hi)
    }
}

impl<Idx> Display for Interval<Idx>
    where Idx: Display + Step
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.lo != self.hi {
            write!(f, "{}..={}", self.lo, self.hi)
        } else {
            write!(f, "{}", self.lo)
        }
    }
}

impl<Idx> Copy for Interval<Idx> where Idx: Copy + Step {}

impl<Idx: Step> From<Idx> for Interval<Idx> {
    fn from(value: Idx) -> Self {
        Self::new(value.clone(), value)
    }
}

impl<Idx: Step> From<&Idx> for Interval<Idx> {
    fn from(value: &Idx) -> Self {
        Self::new(value.clone(), value.clone())
    }
}

impl<Idx: Step> From<Range<Idx>> for Interval<Idx> {
    #[inline]
    fn from(value: Range<Idx>) -> Self {
        let hi = Idx::backward(&value.end);
        Self::new(value.start, hi)
    }
}

impl<Idx: Step> From<RangeInclusive<Idx>> for Interval<Idx> {
    #[inline]
    fn from(value: RangeInclusive<Idx>) -> Self {
        let (lo, hi) = value.into_inner();
        Self::new(lo, hi)
    }
}

impl<Idx> From<RangeTo<Idx>> for Interval<Idx>
    where Idx: Bounded + Step
{
    #[inline]
    fn from(value: RangeTo<Idx>) -> Self {
        let hi = Idx::backward(&value.end);
        Self::new(Idx::MIN, hi)
    }
}

impl<Idx> From<RangeToInclusive<Idx>> for Interval<Idx>
    where Idx: Bounded + Step
{
    #[inline]
    fn from(value: RangeToInclusive<Idx>) -> Self {
        Self::new(Idx::MIN, value.end)
    }
}

impl<Idx> From<RangeFrom<Idx>> for Interval<Idx>
    where Idx: Bounded + Step
{
    #[inline]
    fn from(value: RangeFrom<Idx>) -> Self {
        Self::new(value.start, Idx::MAX)
    }
}

impl<Idx> From<RangeFull> for Interval<Idx>
    where Idx: Bounded + Step
{
    #[inline]
    fn from(_: RangeFull) -> Self {
        Self::new(Idx::MIN, Idx::MAX)
    }
}
