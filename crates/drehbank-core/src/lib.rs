//! The series core.
//!
//! This crate holds the mathematics: the monomial index and the degree tables,
//! the truncated series and its arithmetic, the Poisson bracket, the Lie
//! transforms, the resonance handling and the remainder estimates.
//!
//! The monomial index of issue #28 is in [`monomial`], the coefficient
//! abstraction of `docs/decisions/0002-coefficients.md` is in [`coefficient`],
//! the truncated series of issue #29 is in [`series`], and the resonance
//! lattice of issue #39 is in [`resonance`]. The partial derivative and the
//! Poisson bracket of issue #30 are in [`series`] too, beside the arithmetic
//! they are built from. The Lie transforms and the estimates are still to come.
//!
//! Nothing in this crate may depend on the command line, so that the boundary
//! the dependency check applies to is the one the workspace declares.

pub mod coefficient;
pub mod error;
pub mod monomial;
pub mod resonance;
pub mod series;

pub use coefficient::Coefficient;
pub use error::Error;
pub use resonance::ResonanceModule;
pub use series::Series;

#[cfg(test)]
mod tests {
    /// The `[profile.test]` block in the workspace manifest turns overflow
    /// checks on, and this is the thing that says it took. Set
    /// `overflow-checks = false` there and the addition below wraps to zero
    /// instead of panicking, so the test goes red for the reason it names.
    ///
    /// It is written against a `usize` because that is what an index into a
    /// flat coefficient buffer is, and a wrapped index reads the wrong term
    /// rather than stopping.
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn the_test_profile_refuses_an_overflow_in_index_arithmetic() {
        let index = std::hint::black_box(usize::MAX);
        std::hint::black_box(index + 1);
    }
}
