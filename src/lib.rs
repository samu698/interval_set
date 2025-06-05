#![deny(missing_docs)]

//! Implementaion of a [`IntervalSet`] that allows to store a set of minimized
//! intervals.
//!
//! Look at [`IntervalSet`] for the full documentation of the data structure.
//!
//! Intervals are represented using the [`Interval`] type
//!
//! Types using this data structure require a notion of successor and
//! predecessor and so, the trait [`Step`] needs to be implemented.

mod traits;
pub use traits::{Step, Bounded};

mod interval;
pub use interval::Interval;

mod set;
pub use set::IntervalSet;

