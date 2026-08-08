//! The properties of the monomial index, and the proof that they refuse a
//! bijection that is wrong.
//!
//! Two properties, from `docs/decisions/0003-series-representation.md`: an
//! index that survives a round trip through the exponent vector, and distinct
//! indices that stay distinct. Both are written once, over the forward map, so
//! the same property can be run against the real one and against the one in
//! `support` that drops the shift.

mod support;

use drehbank_core::monomial::{IndexError, dimension, exponents_of, index_of};
use proptest::prelude::*;
use proptest::test_runner::TestError;

use support::Forward;

/// The largest case the properties are run at.
///
/// Six variables is the target of the scale milestone and eight is the order it
/// names, so the properties are exercised at the shape the package is for
/// rather than at a shape chosen to be quick. `M(8, 6)` is 1287, so the whole
/// space at the largest case is small enough that the generator reaches every
/// corner of it.
const VARIABLES: std::ops::RangeInclusive<usize> = 1..=6;
const DEGREE: std::ops::RangeInclusive<u32> = 0..=8;

/// A monomial: a width, a degree, and an index inside that degree.
fn a_monomial() -> impl Strategy<Value = (usize, u32, u64)> {
    (VARIABLES, DEGREE).prop_flat_map(|(variables, degree)| {
        let count = dimension(variables, degree).expect("every case here is addressable");
        (Just(variables), Just(degree), 0..count)
    })
}

/// Two different indices at one width and degree.
///
/// The second is drawn from a range one shorter and stepped over the first,
/// rather than drawn freely and rejected when the two collide. Rejection would
/// throw away every case at a degree with a single monomial, and proptest gives
/// up on a strategy that rejects too often, so the property would quietly stop
/// running at exactly the widths it is cheapest to be wrong at.
fn two_monomials() -> impl Strategy<Value = (usize, u32, u64, u64)> {
    (VARIABLES, DEGREE)
        .prop_filter(
            "a degree with one monomial has no distinct pair",
            |&(variables, degree)| {
                dimension(variables, degree).expect("every case here is addressable") >= 2
            },
        )
        .prop_flat_map(|(variables, degree)| {
            let count = dimension(variables, degree).expect("every case here is addressable");
            (Just(variables), Just(degree), 0..count, 0..count - 1)
        })
        .prop_map(|(variables, degree, first, second)| {
            let second = if second >= first { second + 1 } else { second };
            (variables, degree, first, second)
        })
}

fn refuse(error: IndexError) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

/// An index, through the exponent vector and back, is the index it started as.
fn round_trip(forward: Forward) -> Result<(), TestError<(usize, u32, u64)>> {
    support::fixed_seed_runner().run(&a_monomial(), move |(variables, degree, index)| {
        let exponents = exponents_of(index, variables, degree).map_err(refuse)?;
        prop_assert_eq!(forward(&exponents, degree).map_err(refuse)?, index);
        Ok(())
    })
}

/// Two different indices name two different monomials.
fn distinctness(forward: Forward) -> Result<(), TestError<(usize, u32, u64, u64)>> {
    support::fixed_seed_runner().run(
        &two_monomials(),
        move |(variables, degree, first, second)| {
            let left = exponents_of(first, variables, degree).map_err(refuse)?;
            let right = exponents_of(second, variables, degree).map_err(refuse)?;
            prop_assert_ne!(&left, &right);
            prop_assert_ne!(
                forward(&left, degree).map_err(refuse)?,
                forward(&right, degree).map_err(refuse)?
            );
            Ok(())
        },
    )
}

#[test]
fn an_index_survives_the_round_trip_through_its_exponent_vector() {
    round_trip(support::index_of_without_the_shift)
        .expect("the round trip holds for the index this crate ships");
}

#[test]
fn distinct_indices_name_distinct_monomials() {
    distinctness(index_of).expect("the index this crate ships is injective");
}

/// The round trip property, run against the bijection with the shift dropped.
///
/// This is the guard being watched to bite. Both of the tests above pass
/// against code that has never been wrong, which says nothing about whether
/// they could tell if it were. Pointing the same property at
/// `index_of_without_the_shift` answers that, and it fails.
#[test]
fn the_round_trip_refuses_the_bijection_with_the_shift_dropped() {
    let outcome = round_trip(support::index_of_without_the_shift);
    let error =
        outcome.expect_err("the round trip has to refuse a forward map that is not inverse");
    let TestError::Fail(_, (variables, degree, index)) = error else {
        panic!("expected a failing case, got {error:?}");
    };
    println!("refused: {variables} variables, degree {degree}, index {index}");
}

/// The distinctness property, run against the same broken forward map.
///
/// It fails for the reason the shift exists: two monomials rank to one number.
/// The pair it shrinks to is in the fixture directory as
/// `dropped-shift-collision.txt`, so the case outlives this test.
#[test]
fn distinctness_refuses_the_bijection_with_the_shift_dropped() {
    let outcome = distinctness(support::index_of_without_the_shift);
    let error = outcome.expect_err("the shift is what makes the rank injective");
    let TestError::Fail(_, (variables, degree, first, second)) = error else {
        panic!("expected a failing case, got {error:?}");
    };
    println!("refused: {variables} variables, degree {degree}, indices {first} and {second}");
}

/// Every case in the fixture directory still holds.
///
/// A counterexample that fixed a defect and then stopped being tested is a
/// defect waiting to come back. The loader refuses a malformed file rather than
/// skipping it, and this refuses an empty directory, so neither route ends in a
/// green run over nothing.
#[test]
fn every_regression_case_in_the_fixture_directory_holds() {
    let directory = support::fixture_directory();
    let cases = support::load_cases(&directory);
    assert!(
        !cases.is_empty(),
        "no case in {directory:?}; an empty fixture directory passes this test for the wrong reason"
    );

    let mut checked = 0usize;
    for case in &cases {
        for monomial in &case.monomials {
            let exponents = exponents_of(monomial.index, case.variables, case.degree)
                .unwrap_or_else(|error| panic!("{}: {error}", case.name));
            assert_eq!(
                exponents, monomial.exponents,
                "{}: index {} is a different monomial now",
                case.name, monomial.index
            );
            let index = index_of(&monomial.exponents, case.degree)
                .unwrap_or_else(|error| panic!("{}: {error}", case.name));
            assert_eq!(
                index, monomial.index,
                "{}: {:?} has a different index now",
                case.name, monomial.exponents
            );
            checked += 1;
        }
        for (position, monomial) in case.monomials.iter().enumerate() {
            for other in &case.monomials[position + 1..] {
                assert_ne!(
                    monomial.index, other.index,
                    "{}: the same index twice",
                    case.name
                );
                assert_ne!(
                    monomial.exponents, other.exponents,
                    "{}: the same exponent vector twice",
                    case.name
                );
            }
        }
    }
    println!("{} case file(s), {checked} monomial(s)", cases.len());
}
