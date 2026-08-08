//! The series core.
//!
//! This crate holds the mathematics: the monomial index and the degree tables,
//! the truncated series and its arithmetic, the Poisson bracket, the Lie
//! transforms, the resonance handling and the remainder estimates.
//!
//! It is empty today. Issue #16 lays out the workspace and pins the toolchain
//! and nothing else, so that the first mathematics lands in a tree whose shape
//! has already been argued. The monomial index is issue #28.
//!
//! Nothing in this crate may depend on the command line, so that the boundary
//! the dependency check applies to is the one the workspace declares.

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

/// A deliberate defect, held on this branch and never merged, so that the
/// `Build` check can be shown refusing the thing it names. See #18.
///
/// The guard below is always true, because an index into a flat coefficient
/// buffer is unsigned and cannot be below zero. It is the shape somebody writes
/// after changing a signed offset to an index and carrying the old bound with
/// it, and it reads as a range check while checking nothing. The gate compiles
/// with warnings as errors, so this stops the build.
pub fn index_is_in_range(index: usize) -> bool {
    index >= 0
}
