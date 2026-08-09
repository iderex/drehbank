//! The properties of the resonance lattice.
//!
//! `docs/decisions/0007-resonance-module.md` states three of them in the form
//! a test takes, and issue #39 adds a fourth and a count. They are here in that
//! order.
//!
//! Two of them are checked against something other than the crate.
//! Membership is checked against a rank comparison over the rationals, which is
//! a different algorithm reaching the same answer because the module is
//! saturated. The retained count is checked against an enumeration of exponent
//! pairs that never touches the monomial index the crate counts through. Both
//! live in `support::lattice_fixture` and are written there rather than
//! borrowed, because a check that borrows what it is checking proves nothing.

mod support;

use drehbank_core::error::Error;
use drehbank_core::resonance::ResonanceModule;
use proptest::prelude::*;
use proptest::test_runner::TestError;

use support::lattice_fixture::{
    Declaration, declarations, lies_in_the_rational_span, retained_by_enumeration,
};

/// How many cases each property runs.
///
/// Not the `CASES` the index properties use, because the generator here is
/// narrower: two to four degrees of freedom and at most three relations of
/// entries between minus four and four. The whole space is walked many times
/// over at this count, and the six properties together take well under a second
/// of the suite, which is what decided the number rather than a guess at the
/// cost.
const CASES: u32 = 1024;

fn refuse(error: Error) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn over_declarations<F>(body: F) -> Result<(), TestError<Declaration>>
where
    F: Fn(Declaration) -> Result<(), TestCaseError>,
{
    support::fixed_seed_runner_with(CASES).run(&declarations(), body)
}

/// Canonicalisation is a function of the lattice, not of the declaration.
///
/// The rewriting adds multiples of one relation to another and appends scaled
/// copies, neither of which moves the rational span, so the two declarations
/// name one lattice and have to produce one basis. This is the property the
/// whole design rests on: without it, two users who wrote the same physics down
/// differently get results that cannot be compared.
#[test]
fn a_rewritten_declaration_names_the_same_lattice() {
    over_declarations(|declaration| {
        let first =
            ResonanceModule::declare(declaration.freedoms, &declaration.rows).map_err(refuse)?;
        let second = ResonanceModule::declare(declaration.freedoms, &declaration.rewritten())
            .map_err(refuse)?;
        prop_assert_eq!(first.basis(), second.basis());
        Ok(())
    })
    .expect("canonicalisation is a function of the lattice");
}

/// Declaring the canonical basis again returns it unchanged.
#[test]
fn canonicalisation_is_idempotent() {
    over_declarations(|declaration| {
        let once =
            ResonanceModule::declare(declaration.freedoms, &declaration.rows).map_err(refuse)?;
        let twice = ResonanceModule::declare(declaration.freedoms, once.basis()).map_err(refuse)?;
        prop_assert_eq!(once, twice);
        Ok(())
    })
    .expect("canonicalisation is idempotent");
}

/// Adding a relation the module already implies leaves the module alone.
///
/// The relation added is an integer combination of the canonical basis, which
/// is the case a user meets when they write down a consequence of what they
/// already declared. The trivial module has no basis to combine, so it is
/// skipped, and the property below covers what it says about membership.
#[test]
fn declaring_an_implied_relation_changes_nothing() {
    over_declarations(|declaration| {
        let module =
            ResonanceModule::declare(declaration.freedoms, &declaration.rows).map_err(refuse)?;
        if module.dimension() == 0 {
            return Ok(());
        }
        let mut implied = vec![0i64; declaration.freedoms];
        for (weight, row) in [2i64, -1, 3].iter().cycle().zip(module.basis().iter()) {
            for (slot, entry) in implied.iter_mut().zip(row.iter()) {
                *slot += weight * entry;
            }
        }
        let mut widened = declaration.rows.clone();
        widened.push(implied);
        let again = ResonanceModule::declare(declaration.freedoms, &widened).map_err(refuse)?;
        prop_assert_eq!(module, again);
        Ok(())
    })
    .expect("an implied relation adds nothing");
}

/// The module is saturated: `c k` is a member exactly when `k` is.
#[test]
fn membership_is_saturated() {
    over_declarations(|declaration| {
        let module =
            ResonanceModule::declare(declaration.freedoms, &declaration.rows).map_err(refuse)?;
        for multi_index in small_multi_indices(declaration.freedoms) {
            let plain = module.contains(&multi_index).map_err(refuse)?;
            for factor in [2i64, 3, 5] {
                let scaled: Vec<i64> = multi_index.iter().map(|entry| factor * entry).collect();
                prop_assert_eq!(
                    module.contains(&scaled).map_err(refuse)?,
                    plain,
                    "a lattice that is saturated cannot tell {:?} from {:?}",
                    multi_index,
                    scaled
                );
            }
        }
        Ok(())
    })
    .expect("the module is saturated");
}

/// Membership agrees with a rank comparison over the rationals.
///
/// The crate reduces against the canonical basis. The fixture asks whether the
/// rank of the declared rows moves when the vector is added to them. They are
/// different computations and they answer the same question, because the module
/// is the saturation of the declared lattice and the saturation is exactly the
/// integer points of the rational span.
#[test]
fn membership_agrees_with_the_rational_span() {
    over_declarations(|declaration| {
        let module =
            ResonanceModule::declare(declaration.freedoms, &declaration.rows).map_err(refuse)?;
        for multi_index in small_multi_indices(declaration.freedoms) {
            prop_assert_eq!(
                module.contains(&multi_index).map_err(refuse)?,
                lies_in_the_rational_span(&declaration.rows, &multi_index),
                "the two answers disagree about {:?} against {:?}",
                multi_index,
                declaration.rows
            );
        }
        Ok(())
    })
    .expect("membership is membership of the rational span");
}

/// The retained count agrees with an enumeration that never touches the
/// monomial index.
#[test]
fn the_retained_count_agrees_with_a_direct_enumeration() {
    over_declarations(|declaration| {
        let module =
            ResonanceModule::declare(declaration.freedoms, &declaration.rows).map_err(refuse)?;
        // Four is as far as an enumeration over every exponent pair is cheap
        // at four degrees of freedom, and it is above the degree where the
        // first resonant terms appear.
        let counted = module.retained_per_degree(4).map_err(refuse)?;
        for (degree, count) in counted.iter().enumerate() {
            let enumerated = retained_by_enumeration(declaration.freedoms, degree as u32, &|k| {
                module.contains(k).expect("the width is the module's own")
            });
            prop_assert_eq!(
                *count,
                enumerated,
                "the counts disagree at degree {}",
                degree
            );
        }
        Ok(())
    })
    .expect("the count is a count of the monomials the storage holds");
}

/// Every multi-index with entries in a small range, which is the set the
/// membership properties are checked over.
///
/// Exhaustive rather than sampled. The set is small at these widths, and a
/// membership defect that only shows on one lattice point is exactly what a
/// sample misses.
fn small_multi_indices(freedoms: usize) -> Vec<Vec<i64>> {
    let mut all = vec![Vec::new()];
    for _ in 0..freedoms {
        let mut wider = Vec::new();
        for prefix in &all {
            for entry in -2i64..=2 {
                let mut one = prefix.clone();
                one.push(entry);
                wider.push(one);
            }
        }
        all = wider;
    }
    all
}
