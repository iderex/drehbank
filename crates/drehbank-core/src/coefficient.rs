//! What a coefficient is, and the whole of what the algebra may ask of one.
//!
//! `docs/decisions/0002-coefficients.md` fixes the surface: addition,
//! subtraction, multiplication, negation, zero, one, and construction from a
//! small integer. Seven operations and no eighth. Division is deliberately
//! absent, because it appears in one place in the package and the three
//! coefficient types behave differently there, and ordering is deliberately
//! absent, because a comparison on an interval is partial and a shared total
//! order would have to lie for one of the three.
//!
//! The width of this trait is what the generic instantiation cost is paid for,
//! so a later operation added here is a change to that document first.

use core::fmt;

/// The arithmetic the series of [`crate::series`] is written against.
///
/// A type implementing this is a commutative ring with unit as far as the
/// series arithmetic is concerned, and every algebraic property the series
/// carries is inherited from that. Machine floating point is not such a ring,
/// because rounding is neither associative nor distributive, and that is a fact
/// about the type rather than about this trait: the properties hold exactly for
/// a coefficient whose arithmetic is exact, and hold for `f64` only where no
/// operation rounds. The suite says which of the two it is running under rather
/// than asserting the stronger claim everywhere.
///
/// [`Clone`] is here because a graded array of coefficients is built and sliced
/// rather than moved through, and [`fmt::Debug`] is here so a refusal can name
/// the coefficient it refused. Neither adds an arithmetic operation.
pub trait Coefficient: Clone + fmt::Debug {
    /// The additive identity.
    fn zero() -> Self;

    /// The multiplicative identity.
    fn one() -> Self;

    /// The coefficient a small integer denotes.
    ///
    /// This is what the factorial denominators of a Lie series and the integer
    /// weights of a bracket are built from, and it is narrow on purpose: an
    /// `i32` is what those quantities are, and a wider one would invite a
    /// caller to route a coefficient through it.
    fn from_small_integer(value: i32) -> Self;

    /// `self + other`.
    fn add(&self, other: &Self) -> Self;

    /// `self - other`.
    fn subtract(&self, other: &Self) -> Self;

    /// `self * other`.
    fn multiply(&self, other: &Self) -> Self;

    /// `-self`.
    fn negate(&self) -> Self;
}

/// IEEE 754 binary64, the default of 0002 and the only type the throughput path
/// uses.
///
/// Every operation is the machine one. Nothing here rounds outward, nothing
/// here is exact, and the sentence this type licenses is the one 0002 writes
/// for it: the coefficients are what the stated algorithm produces in binary64
/// in the order 0009 fixes, and nothing at all about how far any of them is
/// from the exact value.
impl Coefficient for f64 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn from_small_integer(value: i32) -> Self {
        // Every `i32` is exactly representable in binary64, so this conversion
        // is the one case in this file where nothing is lost.
        f64::from(value)
    }

    fn add(&self, other: &Self) -> Self {
        self + other
    }

    fn subtract(&self, other: &Self) -> Self {
        self - other
    }

    fn multiply(&self, other: &Self) -> Self {
        self * other
    }

    fn negate(&self) -> Self {
        -self
    }
}
