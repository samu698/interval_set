use std::{char, net::{Ipv4Addr, Ipv6Addr}};

/// Types that have *successor* and *predecessor* operations.
///
/// Reimplementation of std's Step trait, becasue Step is unstable.
/// The implementation is simplified, only required features are used
pub trait Step: Clone + Ord + Sized {
    /// Returns the number of successor steps required to get from start to end.
    ///
    /// The first element of the result is a lower bound, the second is an exact
    /// bound and is not always available.
    ///
    /// - Returns `(usize::MAX, None)` if the number of steps would overflow `usize`
    /// - Returns `(0, None)` if `end < start`
    /// - Returns `(0, Some(0))` if `end == start`
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>);

    /// Get the successor of `start` and check for overflow
    fn forward_checked(start: &Self) -> Option<Self>;
    /// Get the successor of `start` panic if overflow is detected
    fn forward(start: &Self) -> Self {
        Step::forward_checked(start)
            .expect("overflow in `Step::backward`")
    }

    /// Get the predecessor of `start` and check for underflow
    fn backward_checked(start: &Self) -> Option<Self>;
    /// Get the predecessor of `start` panic if underflow is detected
    fn backward(start: &Self) -> Self {
        Step::backward_checked(start)
            .expect("underflow in `Step::backward`")
    }
}

macro_rules! impl_step_common {
    () => {
        #[inline]
        fn forward_checked(start: &Self) -> Option<Self> {
            start.checked_add(1)
        }

        #[inline]
        fn backward_checked(start: &Self) -> Option<Self> {
            start.checked_sub(1)
        }
    };
}

macro_rules! impl_step {
    {
        narrower than usize: $( [$u_narrower:ty, $i_narrower:ty] ),+;
        wider than usize: $( [$u_wider:ty, $i_wider:ty] ),+;
    } => {
    $(
        impl Step for $u_narrower {
            #[inline]
            fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                if *start <= *end {
                    let steps = (*end - *start) as usize;
                    (steps, Some(steps))
                } else {
                    (0, None)
                }
            }

            impl_step_common!();
        }

        impl Step for $i_narrower {
            #[inline]
            fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                if *start <= *end {
                    let steps = (*end as isize).wrapping_sub(*start as isize) as usize;
                    (steps, Some(steps))
                } else {
                    (0, None)
                }
            }

            impl_step_common!();
        }
    )+

    $(
        impl Step for $u_wider {
            #[inline]
            fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                if *start <= *end {
                    if let Ok(steps) = usize::try_from(*end - *start) {
                        (steps, Some(steps))
                    } else {
                        (usize::MAX, None)
                    }
                } else {
                    (0, None)
                }
            }

            impl_step_common!();
        }

        impl Step for $i_wider {
            #[inline]
            fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                if *start <= *end {
                    match end.checked_sub(*start) {
                        Some(result) => {
                            if let Ok(steps) = usize::try_from(result) {
                                (steps, Some(steps))
                            } else {
                                (usize::MAX, None)
                            }
                        }
                        None => (usize::MAX, None)
                    }
                } else {
                    (0, None)
                }
            }

            impl_step_common!();
        }
    )+
    };
}

#[cfg(target_pointer_width = "64")]
impl_step! {
    narrower than usize: [u8, i8], [u16, i16], [u32, i32], [u64, i64], [usize, isize];
    wider than usize: [u128, i128];
}

#[cfg(target_pointer_width = "32")]
impl_step! {
    narrower than usize: [u8, i8], [u16, i16], [u32, i32], [usize, isize];
    wider than usize: [u64, i64], [u128, i128];
}

#[cfg(target_pointer_width = "16")]
impl_step! {
    narrower than usize: [u8, i8], [u16, i16], [usize, isize];
    wider than usize: [u32, i32], [u64, i64], [u128, i128];
}

impl Step for char {
    #[inline]
    fn steps_between(&start: &Self, &end: &Self) -> (usize, Option<usize>) {
        let start = start as u32;
        let end = end as u32;
        if start <= end {
            let count = end - start;
            let sub = if start < 0xD800 && 0xE000 <= end { 0x800 } else { 0 };
            if let Ok(steps) = usize::try_from(count - sub) {
                (steps, Some(steps))
            } else {
                (usize::MAX, None)
            }
        } else {
            (0, None)
        }
    }

    #[inline]
    fn forward_checked(start: &char) -> Option<char> {
        const MAX_CHAR: u32 = char::MAX as u32;
        let res = match *start as u32 {
            0xD7FF => 0xE000,
            MAX_CHAR => { return None },
            s => Step::forward_checked(&s)?
        };
        // SAFETY: res is a valid unicode scalar
        // (below 0x110000 and not in 0xD800..0xE000)
        let ch = unsafe { char::from_u32_unchecked(res) };
        Some(ch)
    }

    #[inline]
    fn backward_checked(start: &char) -> Option<char> {
        let res = match *start as u32 {
            0xE000 => 0xD7FF,
            s => Step::backward_checked(&s)?
        };
        // SAFETY: res is a valid unicode scalar
        // (below 0x110000 and not in 0xD800..0xE000)
        let ch = unsafe { char::from_u32_unchecked(res) };
        Some(ch)
    }
}

impl Step for Ipv4Addr {
    #[inline]
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        u32::steps_between(&start.to_bits(), &end.to_bits())
    }

    #[inline]
    fn forward_checked(start: &Self) -> Option<Self> {
        u32::forward_checked(&start.to_bits()).map(Self::from_bits)
    }

    #[inline]
    fn backward_checked(start: &Self) -> Option<Self> {
        u32::backward_checked(&start.to_bits()).map(Self::from_bits)
    }
}

impl Step for Ipv6Addr {
    #[inline]
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        u128::steps_between(&start.to_bits(), &end.to_bits())
    }

    #[inline]
    fn forward_checked(start: &Self) -> Option<Self> {
        u128::forward_checked(&start.to_bits()).map(Self::from_bits)
    }

    #[inline]
    fn backward_checked(start: &Self) -> Option<Self> {
        u128::backward_checked(&start.to_bits()).map(Self::from_bits)
    }
}

/// Types that are bounded, that have a minimum and maximum value
pub trait Bounded: Clone + Ord + Sized {
    /// The minimum value for the type
    const MIN: Self;
    /// The maximum value for the type
    const MAX: Self;
}

macro_rules! impl_bounded {
    [$($t: ty)*] => {$(
        impl Bounded for $t {
            const MIN: Self = <$t>::MIN;
            const MAX: Self = <$t>::MAX;
        }
    )*}
}

impl_bounded![i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize char];

impl Bounded for Ipv4Addr {
    const MIN: Self = Ipv4Addr::from_bits(u32::MIN);
    const MAX: Self = Ipv4Addr::from_bits(u32::MAX);
}

impl Bounded for Ipv6Addr {
    const MIN: Self = Ipv6Addr::from_bits(u128::MIN);
    const MAX: Self = Ipv6Addr::from_bits(u128::MAX);
}
