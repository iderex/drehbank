//! What detection finds, what it refuses, and what a caller has to do before
//! any of it becomes the module in force.
//!
//! The four cases issue #40 asks for are here, and each of them is written
//! against a frequency vector whose relative divisors are exactly representable
//! in binary64, so every comparison below is an equality rather than a
//! tolerance. That is arranged rather than convenient: a case whose expected
//! `rho` is only approximately a number cannot say whether the formula under
//! test is the one 0006 writes or a neighbouring one.
//!
//! # Why the expected numbers are written out
//!
//! `rho(k) = |<k, omega>| / ( |k|_1 * ||omega||_inf )` has three factors and
//! two of them are easy to leave out. A test that only asked whether the right
//! relations came back would pass against a detection missing the `|k|_1`, on
//! any case whose relations all have the same order. So the cases below assert
//! the value of `rho` for the relation they find, and they use a frequency
//! vector on which dropping either factor moves the answer.

mod support;

use drehbank_core::error::Error;
use drehbank_core::resonance::{DETECTION_CEILING, Proposal, Provenance, ResonanceModule};
use drehbank_core::series::Series;

/// The exactly resonant case, and the gap to the next relation.
///
/// `omega = (1, 1, 2)` with `||omega||_inf = 2`. Over `0 < |k|_1 <= 2` there is
/// one exact relation, `(1, -1, 0)`, and the nearest thing to it is `(1, 0, -1)`
/// and `(0, 1, -1)`, both with
///
///     rho = |1 - 2| / ( 2 * 2 ) = 0.25
///
/// every factor of which is a power of two, so the number is exact. A tolerance
/// under that gap finds the exact relation and nothing else.
const RESONANT: [f64; 3] = [1.0, 1.0, 2.0];

/// The same vector with the second frequency moved by one part in `2^20`.
///
/// `1 + 2^-20` is exactly representable, and `1 - (1 + 2^-20)` is exact by
/// Sterbenz, so the divisor of `(1, -1, 0)` is exactly `-2^-20` and its
/// relative divisor is exactly `2^-20 / (2 * 2) = 2^-22`. Nothing here is
/// approximate, which is what lets the case assert the small number rather than
/// assert that it is small.
fn perturbed() -> [f64; 3] {
    [1.0, 1.0 + f64::powi(2.0, -20), 2.0]
}

/// On an exactly resonant vector, detection finds the true relations and
/// nothing else, for a tolerance under the gap.
#[test]
fn an_exact_resonance_is_found_and_nothing_else_is() {
    let proposal = Proposal::detect(&RESONANT, 0.1, 2).expect("the vector and the bound are sound");

    assert_eq!(proposal.relations().len(), 1);
    let found = &proposal.relations()[0];
    assert_eq!(found.relation(), [1, -1, 0]);
    assert_eq!(found.order(), 2);
    assert_eq!(found.divisor(), 0.0);
    assert_eq!(found.relative_divisor(), 0.0);
    assert_eq!(proposal.basis(), [vec![1, -1, 0]]);
    assert_eq!(proposal.tolerance(), 0.1);
    assert_eq!(proposal.order_bound(), 2);
}

/// The gap is where the answer moves, and the comparison is strict.
///
/// At a tolerance of exactly `0.25` the two relations at `rho = 0.25` are still
/// out, because 0007 and 0006 both write the condition as `rho < tau`. One step
/// above and they are in. This is the near miss for both the strictness and the
/// value of `rho`: a detection that dropped the `|k|_1` from the denominator
/// would put `rho((1,0,-1))` at `0.5` and would answer one relation at both
/// tolerances.
#[test]
fn the_tolerance_is_compared_strictly_and_at_the_value_0006_writes() {
    let under = Proposal::detect(&RESONANT, 0.25, 2).expect("the vector and the bound are sound");
    assert_eq!(under.relations().len(), 1);

    let over = Proposal::detect(&RESONANT, 0.26, 2).expect("the vector and the bound are sound");
    let relations: Vec<(&[i64], f64)> = over
        .relations()
        .iter()
        .map(|found| (found.relation(), found.relative_divisor()))
        .collect();
    assert_eq!(
        relations,
        [
            (&[0, 1, -1][..], 0.25),
            (&[1, -1, 0][..], 0.0),
            (&[1, 0, -1][..], 0.25)
        ]
    );

    // Three relations of rank two, and the lattice they generate is the one
    // that makes every frequency difference resonant. It is strictly larger
    // than the three rows, which is the saturation step of 0007 and is why a
    // caller reads the basis rather than the list.
    assert_eq!(over.basis(), [vec![1, 0, -1], vec![0, 1, -1]]);
}

/// Off the resonance by one part in `2^20`, the same relation is found and its
/// divisors are the expected small numbers.
#[test]
fn a_near_resonance_is_found_with_the_divisor_it_should_have() {
    let proposal =
        Proposal::detect(&perturbed(), 1e-5, 2).expect("the vector and the bound are sound");

    assert_eq!(proposal.relations().len(), 1);
    let found = &proposal.relations()[0];
    assert_eq!(found.relation(), [1, -1, 0]);
    assert_eq!(found.divisor(), -f64::powi(2.0, -20));
    assert_eq!(found.relative_divisor(), f64::powi(2.0, -22));
}

/// A tolerance of zero returns nothing at all, exact relations included.
///
/// The condition of 0007 is `rho(k) < tau` and `rho` of an exact relation is
/// zero, which is not below zero. So no relation the detection could call
/// inexact comes back, and neither does the exact one. The second half of this
/// test is what says the emptiness is the strictness and not a search that
/// found nothing: at the smallest positive tolerance binary64 has, the exact
/// relation is there and it is alone.
#[test]
fn a_tolerance_of_zero_returns_no_relation_at_all() {
    let none = Proposal::detect(&RESONANT, 0.0, 2).expect("the vector and the bound are sound");
    assert_eq!(none.relations(), []);
    assert_eq!(none.basis(), Vec::<Vec<i64>>::new());

    let smallest = Proposal::detect(&RESONANT, f64::MIN_POSITIVE, 2)
        .expect("the vector and the bound are sound");
    assert_eq!(smallest.relations().len(), 1);
    assert_eq!(smallest.relations()[0].relation(), [1, -1, 0]);
}

/// A proposal does not become a module, and the one conversion records that it
/// was taken.
///
/// Written as an enumeration of the constructors rather than as one case,
/// because what is being claimed is about all of them: `trivial` and `declare`
/// produce a declared module, `accept` is the only one that takes a
/// [`Proposal`] at all, and it stamps the tolerance and the order bound the
/// proposal came from. There is no fourth constructor, no `From` and no
/// accessor on `Proposal` that hands out a module, so a result built on a
/// detection cannot read as one built on relations somebody wrote down.
#[test]
fn every_constructor_of_a_module_says_where_it_came_from() {
    let trivial = ResonanceModule::trivial(3).expect("three freedoms is a phase space");
    assert_eq!(trivial.provenance(), Provenance::Declared);

    let declared =
        ResonanceModule::declare(3, &[vec![1, -1, 0]]).expect("one relation in three freedoms");
    assert_eq!(declared.provenance(), Provenance::Declared);

    let proposal = Proposal::detect(&RESONANT, 0.1, 2).expect("the vector and the bound are sound");
    let accepted = ResonanceModule::accept(&proposal).expect("the proposal has rank one");
    assert_eq!(
        accepted.provenance(),
        Provenance::Accepted {
            tolerance: 0.1,
            order_bound: 2
        }
    );

    // The same lattice by either route, and equality is over the lattice, so
    // the record is the only thing that tells the two apart.
    assert_eq!(accepted, declared);
    assert_ne!(accepted.provenance(), declared.provenance());
    assert_eq!(accepted.basis(), declared.basis());
}

/// Accepting a proposal that found nothing is still an acceptance.
#[test]
fn accepting_an_empty_proposal_records_that_the_look_happened() {
    let proposal = Proposal::detect(&RESONANT, 0.0, 2).expect("the vector and the bound are sound");
    let accepted =
        ResonanceModule::accept(&proposal).expect("an empty proposal is the trivial one");
    assert_eq!(accepted.dimension(), 0);
    assert_eq!(
        accepted.provenance(),
        Provenance::Accepted {
            tolerance: 0.0,
            order_bound: 2
        }
    );
}

/// The size of the coefficient a relation would reach, which is the context a
/// caller needs before chasing one.
///
/// Two degrees of freedom, so four variables `(q_1, q_2, p_1, p_2)` in the
/// order of item 1 of 0004, and the multi-index of a monomial is `a - b` by
/// item 8. The Hamiltonian below carries three terms:
///
/// - `q_1 p_2`, whose multi-index is `(1, -1)`, with coefficient 3;
/// - `q_1^2 p_2^2`, whose multi-index is `(2, -2)`, a multiple of the same
///   relation, with coefficient -7;
/// - `q_1 p_1`, whose multi-index is `(0, 0)`, with coefficient 100.
///
/// The answer for the relation `(1, -1)` is 7 and not 100. A monomial with
/// `a = b` is in the kernel for the reason item 8 gives, whatever the
/// frequencies are, so no relation is why it is kept and its coefficient is not
/// what a near resonance would reach.
#[test]
fn the_magnitude_a_relation_reaches_is_the_largest_it_actually_touches() {
    let proposal =
        Proposal::detect(&[1.0, 1.0], 0.1, 2).expect("the vector and the bound are sound");
    assert_eq!(proposal.relations().len(), 1);
    assert_eq!(proposal.relations()[0].relation(), [1, -1]);

    let mut hamiltonian: Series<f64> = Series::zero(2, 4).expect("order four is addressable");
    support::detection_fixture::set_monomial(&mut hamiltonian, &[1, 0, 0, 1], 3.0);
    support::detection_fixture::set_monomial(&mut hamiltonian, &[2, 0, 0, 2], -7.0);
    support::detection_fixture::set_monomial(&mut hamiltonian, &[1, 0, 1, 0], 100.0);

    assert_eq!(
        proposal
            .affected_magnitudes(&hamiltonian)
            .expect("the phase spaces agree"),
        [7.0]
    );
}

/// A relation that reaches nothing answers zero rather than being left out.
///
/// The Hamiltonian here carries only the term whose multi-index is zero, which
/// no relation reaches. Zero is the useful answer: it is what says the near
/// resonance the detection reported is not a problem on this input.
#[test]
fn a_relation_that_reaches_no_term_answers_zero() {
    let proposal =
        Proposal::detect(&[1.0, 1.0], 0.1, 2).expect("the vector and the bound are sound");
    let mut hamiltonian: Series<f64> = Series::zero(2, 2).expect("order two is addressable");
    support::detection_fixture::set_monomial(&mut hamiltonian, &[1, 0, 1, 0], 100.0);
    assert_eq!(
        proposal
            .affected_magnitudes(&hamiltonian)
            .expect("the phase spaces agree"),
        [0.0]
    );
}

/// A Hamiltonian in a different phase space is refused with both sides named.
#[test]
fn a_hamiltonian_from_another_phase_space_is_refused() {
    let proposal =
        Proposal::detect(&[1.0, 1.0], 0.1, 2).expect("the vector and the bound are sound");
    let hamiltonian: Series<f64> = Series::zero(3, 2).expect("order two is addressable");
    assert_eq!(
        proposal.affected_magnitudes(&hamiltonian).err(),
        Some(Error::FreedomsDiffer { left: 2, right: 3 })
    );
}

/// What detection refuses before it searches, each for the reason it names.
///
/// Four refusals and four reasons. The degenerate vector is the one 0007 names
/// first, because the relative divisor of 0006 divides by the largest entry.
/// The non-finite entry is the same division with a different failure, and it
/// carries its position because a caller who wrote six frequencies needs to be
/// told which one. The tolerance is refused rather than compared, because a
/// not-a-number tolerance makes every comparison false and returns an empty
/// proposal that reads like a clean result. The search size is refused before
/// anything is allocated.
#[test]
fn detection_refuses_what_it_cannot_answer() {
    assert_eq!(Proposal::detect(&[], 0.1, 2).err(), Some(Error::NoFreedoms));
    assert_eq!(
        Proposal::detect(&[0.0, 0.0, 0.0], 0.1, 2).err(),
        Some(Error::FrequenciesDegenerate { freedoms: 3 })
    );
    assert_eq!(
        Proposal::detect(&[1.0, f64::NAN, 2.0], 0.1, 2).err(),
        Some(Error::FrequencyNotFinite { at: 1 })
    );
    assert_eq!(
        Proposal::detect(&[1.0, 1.0, 2.0], f64::INFINITY, 2).err(),
        Some(Error::ToleranceNotInRange)
    );
    assert_eq!(
        Proposal::detect(&[1.0, 1.0, 2.0], f64::NAN, 2).err(),
        Some(Error::ToleranceNotInRange)
    );
    assert_eq!(
        Proposal::detect(&[1.0, 1.0, 2.0], -0.1, 2).err(),
        Some(Error::ToleranceNotInRange)
    );
    assert_eq!(
        Proposal::detect(&[1.0; 6], 0.1, 17).err(),
        Some(Error::DetectionTooLarge {
            candidates: 1_334_262,
            ceiling: DETECTION_CEILING
        })
    );
}

/// The count the ceiling is applied to is the number the search actually
/// visits.
///
/// The closed form of 0007 against an enumeration of the search, over every
/// phase space and bound small enough to enumerate. A ceiling computed from a
/// formula that disagrees with the walk is a ceiling on nothing, and the two
/// are written independently: one is a sum of binomials and the other counts
/// what came back.
#[test]
fn the_closed_form_for_the_search_size_agrees_with_the_search() {
    for freedoms in 1..=4usize {
        for bound in 0..=5u32 {
            let frequencies: Vec<f64> = (0..freedoms).map(|entry| 1.0 + entry as f64).collect();
            // A tolerance above one takes everything, since `rho` is at most
            // one, so the proposal is the whole search.
            let proposal = Proposal::detect(&frequencies, 2.0, bound)
                .expect("these vectors and bounds are sound");
            assert_eq!(
                proposal.relations().len() as u64,
                support::detection_fixture::candidates(freedoms, bound),
                "v={freedoms} N={bound}"
            );
        }
    }
}
