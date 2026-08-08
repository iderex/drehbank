//! The series core.
//!
//! This crate holds the mathematics: the monomial index and the degree tables,
//! the truncated series and its arithmetic, the Poisson bracket, the Lie
//! transforms, the resonance handling and the remainder estimates.
//!
//! The monomial index of issue #28 is in [`monomial`], and it is the first of
//! those. Everything else in the list is still to come.
//!
//! Nothing in this crate may depend on the command line, so that the boundary
//! the dependency check applies to is the one the workspace declares.

pub mod monomial;

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
