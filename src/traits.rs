use std::{char, net::{Ipv4Addr, Ipv6Addr}};

/// Types that have *successor* and *predecessor* operations.
///
/// Reimplementation of std's Step trait, becasue Step is unstable.
/// The implementation is simplified, only required features are used
pub trait Step: Clone + Ord + Sized {
    /// Get the successor of `start` and check for overflow
    fn forward_checked(start: Self) -> Option<Self>;
    /// Get the successor of `start` panic if overflow is detected
    fn forward(start: Self) -> Self {
        Step::forward_checked(start)
            .expect("overflow in `Step::backward`")
    }

    /// Get the predecessor of `start` and check for underflow
    fn backward_checked(start: Self) -> Option<Self>;
    /// Get the predecessor of `start` panic if underflow is detected
    fn backward(start: Self) -> Self {
        Step::backward_checked(start)
            .expect("underflow in `Step::backward`")
    }
}

macro_rules! impl_step {
    [$($t: ty)*] => {$(
        impl Step for $t {
            #[inline]
            fn forward_checked(start: Self) -> Option<Self> {
                start.checked_add(1)
            }

            #[inline]
            fn backward_checked(start: Self) -> Option<Self> {
                start.checked_sub(1)
            }
        }
    )*};
}

impl_step![i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize];

impl Step for char {
    #[inline]
    fn forward_checked(start: char) -> Option<char> {
        const MAX_CHAR: u32 = char::MAX as u32;
        let res = match start as u32 {
            0xD7FF => 0xE000,
            MAX_CHAR => { return None },
            s => Step::forward_checked(s)?
        };
        // SAFETY: res is a valid unicode scalar
        // (below 0x110000 and not in 0xD800..0xE000)
        let ch = unsafe { char::from_u32_unchecked(res) };
        Some(ch)
    }

    #[inline]
    fn backward_checked(start: char) -> Option<char> {
        let res = match start as u32 {
            0xE000 => 0xD7FF,
            s => Step::backward_checked(s)?
        };
        // SAFETY: res is a valid unicode scalar
        // (below 0x110000 and not in 0xD800..0xE000)
        let ch = unsafe { char::from_u32_unchecked(res) };
        Some(ch)
    }
}

impl Step for Ipv4Addr {
    #[inline]
    fn forward_checked(start: Ipv4Addr) -> Option<Self> {
        u32::forward_checked(start.to_bits()).map(Ipv4Addr::from_bits)
    }

    #[inline]
    fn backward_checked(start: Ipv4Addr) -> Option<Self> {
        u32::backward_checked(start.to_bits()).map(Ipv4Addr::from_bits)
    }
}

impl Step for Ipv6Addr {
    #[inline]
    fn forward_checked(start: Ipv6Addr) -> Option<Self> {
        u128::forward_checked(start.to_bits()).map(Ipv6Addr::from_bits)
    }

    #[inline]
    fn backward_checked(start: Ipv6Addr) -> Option<Self> {
        u128::backward_checked(start.to_bits()).map(Ipv6Addr::from_bits)
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
