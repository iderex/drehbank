//! An exact coefficient, held here as a fixture.
//!
//! The algebraic properties of a truncated series are inherited from the
//! coefficient: a series over a commutative ring is a commutative ring, and a
//! series over machine floating point is not, because rounding is neither
//! associative nor distributive. So a property like "multiplication is
//! associative" is a statement about the series arithmetic only when the
//! coefficient underneath it is exact, and running it in `f64` alone would be
//! measuring the rounding rather than the convolution.
//!
//! This is the exact coefficient the properties in `tests/series.rs` run
//! against. It is the integers, which is the smallest thing that makes the
//! seven operations of `docs/decisions/0002-coefficients.md` exact, and that is
//! enough because division is not one of the seven.
//!
//! **It is a fixture and not a fourth implementation.** 0002 names three
//! implementations the package ships, and the exact one of those three is the
//! arbitrary precision rational, whose implementation is an open entry in issue
//! #2 and is issue #31's to instantiate. Nothing here is reachable from the
//! library, nothing here is published, and this file goes away the day #31
//! lands the real one.

use drehbank_core::coefficient::Coefficient;

/// An integer coefficient, exact within its width.
///
/// `i128` with checked arithmetic, so a generator that walked past the width
/// stops with the operation named rather than wrapping into a plausible wrong
/// answer. A fixture that wraps is worse than no fixture: it would make an
/// exact property fail for a reason that has nothing to do with the series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exact(pub i128);

impl Coefficient for Exact {
    fn zero() -> Self {
        Exact(0)
    }

    fn one() -> Self {
        Exact(1)
    }

    fn from_small_integer(value: i32) -> Self {
        Exact(i128::from(value))
    }

    fn add(&self, other: &Self) -> Self {
        Exact(
            self.0
                .checked_add(other.0)
                .expect("the fixture ranges keep every sum inside i128"),
        )
    }

    fn subtract(&self, other: &Self) -> Self {
        Exact(
            self.0
                .checked_sub(other.0)
                .expect("the fixture ranges keep every difference inside i128"),
        )
    }

    fn multiply(&self, other: &Self) -> Self {
        Exact(
            self.0
                .checked_mul(other.0)
                .expect("the fixture ranges keep every product inside i128"),
        )
    }

    fn negate(&self) -> Self {
        Exact(
            self.0
                .checked_neg()
                .expect("the fixture ranges keep every negation inside i128"),
        )
    }
}
