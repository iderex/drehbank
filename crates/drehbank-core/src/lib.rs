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

    /// A deliberate defect, held on this branch and never merged, so that the
    /// `Test` check can be shown red while `Build` stays green. See #18.
    ///
    /// It compiles cleanly and warns about nothing, which is the point: a
    /// failing assertion is not a compile problem and the two checks have to
    /// say different things about it.
    #[test]
    fn a_deliberately_failing_assertion() {
        let terms_at_degree_two_in_two_variables = 3;
        assert_eq!(
            terms_at_degree_two_in_two_variables, 4,
            "held on a proof branch for #18; the real dimension table is #28"
        );
    }
}
