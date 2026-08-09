//! The algebraic properties of the Poisson bracket, the eigenvalue that pins
//! the convention, and the proof that each of them refuses a bracket which is
//! wrong in the one way that property is about.
//!
//! # What each property can and cannot catch
//!
//! The four identities below are internal consistency, and internal
//! consistency does not pin a convention. A bracket built on a constant
//! antisymmetric bivector is bilinear, antisymmetric and satisfies the Jacobi
//! identity whatever entries the bivector has, so all four hold for a package
//! that has paired every variable with the wrong partner. That is not a
//! supposition here: `support::bracket_fixture::bracket_with_the_partner_adjacent`
//! is such a bracket, it is in the tree, and the properties below run against
//! it and do not refuse it.
//!
//! What refuses it is the eigenvalue of item 8 of
//! `docs/decisions/0004-conventions.md`, which is a statement about this
//! convention and no other. So the fixture and the identities are not two ways
//! of testing one thing; each covers what the other cannot, and the file says
//! which is which rather than leaving a reader to assume the identities were
//! enough.
//!
//! # Why the `f64` comparison is exact rather than toleranced
//!
//! The stated tolerance for every floating point comparison in this file is
//! zero, and that is arranged rather than hoped for. It follows the argument in
//! the header of `tests/series.rs`: every coefficient a draft carries is an
//! integer of magnitude at most 8, binary64 represents such integers exactly,
//! and every sum and product of exactly represented integers is exact as long
//! as the result stays below `2^53`, which is 9007199254740992.
//!
//! The Jacobi case is the widest and it is bounded here rather than assumed.
//! It runs at `freedoms` at most 3, so `m = 6` variables, and at order at most
//! 3, so the largest degree in play holds `M(3, 6) = 56` monomials:
//!
//!     $ python -c "from math import comb; print(comb(3+5,5))"
//!     56
//!
//! Differentiating brings down an exponent of at most 3, so a derivative
//! coefficient is at most `8 * 3 = 24`. One output coefficient of a bracket is
//! a sum over at most `2v = 6` terms, over at most 4 pairs of degrees, of at
//! most 56 monomial pairs each, so at most `6 * 4 * 56 = 1344` products of two
//! derivative coefficients: at most `1344 * 24 * 24 = 774144`. The second
//! bracket differentiates that, at most `774144 * 4 = 3096576`, against a
//! derivative of the third argument at most 24, over at most `6 * 4 * 56` terms
//! again: at most `1344 * 3096576 * 24 = 99883155456`. Three of those summed is
//! at most `299649466368`, which is below `2^53`, so every intermediate value
//! of the identity is exactly representable and the residual is exactly zero.
//!
//! That is a statement about these ranges and about nothing else. How far the
//! Jacobi residual moves in binary64 at coefficient magnitudes where the
//! arithmetic does round is NOT MEASURED here, and nothing in this file should
//! be read as a bound on it.

mod support;

use drehbank_core::coefficient::Coefficient;
use drehbank_core::error::Error;
use drehbank_core::monomial::index_of;
use drehbank_core::series::{Series, bracket_order};
use proptest::prelude::*;
use proptest::test_runner::TestError;

use support::bracket_fixture::{
    Bracket, bracket_dropping_the_top_degree, bracket_via_the_public_api,
    bracket_with_the_partner_adjacent, bracket_without_the_antisymmetric_sign, drafts_over,
};
use support::exact::Exact;
use support::gaussian::Gaussian;
use support::series_fixture::Draft;

/// How many cases the identities that take one bracket per case run.
///
/// One case here is up to two brackets, each of them `2v` convolutions over
/// arrays wider than either argument's, which is several times the cost of one
/// case in `tests/series.rs`. This number and the two below were chosen against
/// a measured run rather than by feel, and the measurement is in issue #30.
const CASES: u32 = 128;

/// How many cases the Jacobi identity runs.
///
/// Nine brackets per case, three of them over an argument that is itself a
/// bracket and therefore of higher order than the drafts. It is the most
/// expensive property in the tree by a wide margin, which is why it runs fewer
/// cases over a narrower order range and says so.
const JACOBI_CASES: u32 = 24;

/// How many cases the properties that run the public API assembly do.
///
/// The assembly in `support::bracket_fixture` allocates working space per term
/// pair and reads every coefficient through an accessor, so it is slower than
/// the shipped kernel by a large factor. It is a fixture rather than a path
/// anybody ships, and what it has to do is disagree, which it does on the first
/// case or not at all.
const FIXTURE_CASES: u32 = 16;

/// The degrees of freedom the identities run at.
///
/// One is where the two blocks of item 1 of 0004 degenerate into a single
/// conjugate pair, and it is the case where a bracket that pairs neighbours
/// rather than partners is indistinguishable from the right one. Three is six
/// variables, the width the scale milestone targets.
const FREEDOMS: std::ops::RangeInclusive<usize> = 1..=3;

/// The truncation orders the identities run at.
const ORDER: std::ops::RangeInclusive<u32> = 0..=4;

/// The truncation orders the Jacobi identity runs at.
///
/// One order lower than the rest, and it is not a rounding of the number. A
/// Jacobi case at order `N` computes brackets of brackets, whose order is
/// `3N - 4`, so the arrays it walks at order 4 are those of degree 8 in 6
/// variables, `M(8, 6) = 1287`, against `M(5, 6) = 252` at order 3:
///
///     $ python -c "from math import comb; print(comb(8+5,5), comb(5+5,5))"
///     1287 252
///
/// The narrower range is what keeps this property inside a suite somebody runs
/// before every push, and it is a coverage decision rather than an accident.
const JACOBI_ORDER: std::ops::RangeInclusive<u32> = 0..=3;

/// The truncation orders the Leibniz rule runs at.
///
/// Two is the floor and it is forced by the statement rather than chosen. The
/// derivation in [`leibniz_residual`] truncates a bracket of order `2N - 2`
/// down to `N`, which is a truncation upward and refused when `N` is below 2.
const LEIBNIZ_ORDER: std::ops::RangeInclusive<u32> = 2..=4;

fn refuse(error: Error) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn over<F>(
    cases: u32,
    count: usize,
    order: std::ops::RangeInclusive<u32>,
    body: F,
) -> Result<(), TestError<Vec<Draft>>>
where
    F: Fn(Vec<Draft>) -> Result<(), TestCaseError>,
{
    support::fixed_seed_runner_with(cases).run(&drafts_over(count, FREEDOMS, order), body)
}

/// `{f, g} = -{g, f}`, exactly, in both coefficient types.
///
/// Item 3 of 0004 names antisymmetry as a consequence of the sign it fixes, so
/// this is that sentence run against the kernel. It holds for the zero series
/// and for a constant, which are the cases where both sides are zero for
/// different reasons, and the generator produces both.
#[test]
fn the_bracket_is_antisymmetric() {
    fn body<C: Coefficient + PartialEq>(drafts: Vec<Draft>) -> Result<(), TestCaseError> {
        let left: Series<C> = drafts[0].build();
        let right: Series<C> = drafts[1].build();
        let forward = left.bracket(&right).map_err(refuse)?;
        let backward = right.bracket(&left).map_err(refuse)?;
        prop_assert!(forward == backward.negated());
        Ok(())
    }
    over(CASES, 2, ORDER, body::<Exact>).expect("antisymmetry holds in the exact fixture");
    over(CASES, 2, ORDER, body::<f64>).expect("antisymmetry holds in binary64 at these ranges");
}

/// The bracket is linear in its left argument, over the coefficient type.
///
/// Both halves, because they fail differently: an operation that forgot to
/// accumulate would break the additive half and pass the scalar one.
#[test]
fn the_bracket_is_bilinear_in_its_left_argument() {
    fn body<C: Coefficient + PartialEq>(drafts: Vec<Draft>) -> Result<(), TestCaseError> {
        let first: Series<C> = drafts[0].build();
        let second: Series<C> = drafts[1].build();
        let right: Series<C> = drafts[2].build();

        let of_the_sum = first
            .sum(&second)
            .map_err(refuse)?
            .bracket(&right)
            .map_err(refuse)?;
        let sum_of_the_brackets = first
            .bracket(&right)
            .map_err(refuse)?
            .sum(&second.bracket(&right).map_err(refuse)?)
            .map_err(refuse)?;
        prop_assert!(of_the_sum == sum_of_the_brackets);

        let factor = C::from_small_integer(-3);
        let of_the_scaled = first.scaled(&factor).bracket(&right).map_err(refuse)?;
        let scaled_bracket = first.bracket(&right).map_err(refuse)?.scaled(&factor);
        prop_assert!(of_the_scaled == scaled_bracket);
        Ok(())
    }
    over(CASES, 3, ORDER, body::<Exact>).expect("bilinearity holds in the exact fixture");
    over(CASES, 3, ORDER, body::<f64>).expect("bilinearity holds in binary64 at these ranges");
}

/// The two sides of the Leibniz rule, truncated to where both are complete.
///
/// The rule is `{f, gh} = {f, g} h + g {f, h}`, and "valid to the truncation
/// order" is the whole of the work here, because neither side is complete at
/// its own stated order once the arguments are truncated series.
///
/// Write `N` for the order the three drafts share. `gh` is the product at order
/// `N`, so it is the true product only up to degree `N`; the terms of the true
/// product above `N` would reach the left side at degrees `N` and above, so the
/// left side agrees with the true `{f, gh}` up to degree `N - 1`. On the right,
/// `{f, g}` and `{f, h}` are exact at order `2N - 2`, and each is truncated to
/// `N` before multiplying, which loses only what the product would truncate
/// away anyway, so each product is the true one up to degree `N`.
///
/// Both sides are therefore complete up to `N - 1`, and that is where they are
/// compared. Comparing at `N` instead would compare one side's missing terms
/// against the other's, which passes or fails for a reason that has nothing to
/// do with the rule.
fn leibniz_residual<C: Coefficient + PartialEq>(
    first: &Series<C>,
    second: &Series<C>,
    third: &Series<C>,
) -> Result<(Series<C>, Series<C>), Error> {
    let order = first.order();
    let target = order - 1;
    let left = first.bracket(&second.product(third)?)?.truncated(target)?;
    let right = first
        .bracket(second)?
        .truncated(order)?
        .product(third)?
        .sum(&second.product(&first.bracket(third)?.truncated(order)?)?)?
        .truncated(target)?;
    Ok((left, right))
}

/// The Leibniz rule, to the truncation order the derivation above gives.
#[test]
fn the_leibniz_rule_holds_to_the_truncation_order() {
    fn body<C: Coefficient + PartialEq>(drafts: Vec<Draft>) -> Result<(), TestCaseError> {
        let first: Series<C> = drafts[0].build();
        let second: Series<C> = drafts[1].build();
        let third: Series<C> = drafts[2].build();
        let (left, right) = leibniz_residual(&first, &second, &third).map_err(refuse)?;
        prop_assert!(left == right);
        Ok(())
    }
    over(CASES, 3, LEIBNIZ_ORDER, body::<Exact>).expect("the Leibniz rule holds in the fixture");
    over(CASES, 3, LEIBNIZ_ORDER, body::<f64>)
        .expect("the Leibniz rule holds in binary64 at these ranges");
}

/// `{{f, g}, h} + {{g, h}, f} + {{h, f}, g} = 0`.
///
/// Nine brackets whose errors do not cancel by accident, which is what makes
/// this the strongest test of both the bracket and the multiplication under it.
/// All three terms carry the same order, because [`bracket_order`] is symmetric
/// and each term is a bracket of an order `2N - 2` series with an order `N`
/// one, so they can be summed without a truncation anywhere.
fn jacobi_sum<C: Coefficient + PartialEq>(
    first: &Series<C>,
    second: &Series<C>,
    third: &Series<C>,
    bracket: Bracket<C>,
) -> Result<Series<C>, Error> {
    let one = bracket(&bracket(first, second)?, third)?;
    let two = bracket(&bracket(second, third)?, first)?;
    let three = bracket(&bracket(third, first)?, second)?;
    one.sum(&two)?.sum(&three)
}

/// The Jacobi identity, in both coefficient types.
#[test]
fn the_jacobi_identity_holds() {
    fn body<C: Coefficient + PartialEq>(drafts: Vec<Draft>) -> Result<(), TestCaseError> {
        let first: Series<C> = drafts[0].build();
        let second: Series<C> = drafts[1].build();
        let third: Series<C> = drafts[2].build();
        let sum = jacobi_sum(&first, &second, &third, Series::bracket).map_err(refuse)?;
        let order = sum.order();
        let zero: Series<C> = Series::zero(first.freedoms(), order).map_err(refuse)?;
        prop_assert!(sum == zero);
        Ok(())
    }
    over(JACOBI_CASES, 3, JACOBI_ORDER, body::<Exact>)
        .expect("the Jacobi identity holds in the exact fixture");
    over(JACOBI_CASES, 3, JACOBI_ORDER, body::<f64>)
        .expect("the Jacobi identity holds in binary64 at these ranges");
}

/// The bracket carries the top degree its arguments determine.
///
/// `{q_1^n, p_1^m} = n m q_1^(n-1) p_1^(m-1)`, whose degree is `n + m - 2`,
/// which is exactly the order [`bracket_order`] states. A bracket answering at
/// the order of its arguments would return a series that is a perfectly
/// ordinary shorter one, with this term simply absent, which is the failure
/// issue #30 names.
///
/// Two variables, so the rank of 0003 reduces to `index = a_1`: the monomial
/// `q_1^a p_1^b` sits at index `a` within its degree.
fn top_degree_disagreements<C: Coefficient + PartialEq + std::fmt::Debug>(
    bracket: Bracket<C>,
) -> Vec<String> {
    let mut disagreements = Vec::new();
    for left_degree in 1..=4u32 {
        for right_degree in 1..=4u32 {
            let mut left: Series<C> = Series::zero(1, left_degree).expect("addressable");
            left.set_coefficient(
                left_degree,
                index_of(&[left_degree, 0], left_degree).expect("addressable"),
                C::one(),
            )
            .expect("addressable");
            let mut right: Series<C> = Series::zero(1, right_degree).expect("addressable");
            right
                .set_coefficient(
                    right_degree,
                    index_of(&[0, right_degree], right_degree).expect("addressable"),
                    C::one(),
                )
                .expect("addressable");

            let stated = bracket_order(left_degree, right_degree);
            let mut expected: Series<C> = Series::zero(1, stated).expect("addressable");
            expected
                .set_coefficient(
                    stated,
                    index_of(&[left_degree - 1, right_degree - 1], stated).expect("addressable"),
                    C::from_small_integer(
                        i32::try_from(left_degree * right_degree).expect("small"),
                    ),
                )
                .expect("addressable");

            match bracket(&left, &right) {
                Ok(answer) if answer.order() == stated && answer == expected => {}
                Ok(answer) => disagreements.push(format!(
                    "{{q^{left_degree}, p^{right_degree}}} carries order {} where {stated} was stated",
                    answer.order()
                )),
                Err(error) => disagreements.push(format!(
                    "{{q^{left_degree}, p^{right_degree}}} was refused: {error}"
                )),
            }
        }
    }
    disagreements
}

/// The shipped bracket keeps that term.
#[test]
fn the_bracket_carries_the_top_degree_its_arguments_determine() {
    assert_eq!(
        top_degree_disagreements::<Exact>(Series::bracket),
        Vec::<String>::new()
    );
    assert_eq!(
        top_degree_disagreements::<f64>(Series::bracket),
        Vec::<String>::new()
    );
}

/// A bracket that truncates one order lower is refused by that case.
///
/// This is the proof the case above bites. Remove the `saturating_sub(1)` from
/// `bracket_dropping_the_top_degree` and this test goes red, because there is
/// then nothing left for it to catch.
///
/// Fifteen of the sixteen pairs and not all sixteen. `{q^1, p^1}` has stated
/// order zero, where subtracting one saturates rather than moving, so that pair
/// is the one case in which the mistake is invisible. The count is written out
/// rather than asserted as "not empty", because a fixture that stopped being
/// wrong on fifteen of the sixteen would still pass an emptiness check.
#[test]
fn the_top_degree_case_refuses_a_bracket_that_drops_it() {
    let disagreements = top_degree_disagreements::<Exact>(bracket_dropping_the_top_degree);
    assert_eq!(disagreements.len(), 15, "{disagreements:?}");
}

/// The eigenvalue of the homological operator, against the closed form of item
/// 8 of 0004.
///
/// In the complex variables of item 5, with `H_2 = -i sum_j omega_j x_j y_j`,
/// item 8 states
///
///     { x^a y^b , H_2 } = -i * <a - b, omega> * x^a y^b
///
/// so every monomial is an eigenvector and the eigenvalue is a number this test
/// computes from `a`, `b` and `omega` without going near the bracket. The
/// closed form is not asserted here: it is derived in 0004 and checked there,
/// exactly over the Gaussian rationals, for every `a` and `b` in `{0,1,2}^v` at
/// one, two and three degrees of freedom. This is the same sweep against the
/// shipped kernel instead of against a second spelling of the derivation.
///
/// Item 5 puts `x_j` in the slot `q_j` occupies and `y_j` in the slot `p_j`
/// occupies, and says the bracket of item 3 may be evaluated on the complex
/// variables with the same formula and the same sign, so nothing here needs a
/// complexification to have been implemented.
///
/// This is the property that pins the convention. A bracket pairing each
/// variable with its neighbour rather than with its conjugate partner satisfies
/// every identity in this file and fails here.
fn eigenvalue_disagreements(bracket: Bracket<Gaussian>, freedoms: usize) -> Vec<String> {
    eigenvalue_disagreements_to_degree(bracket, freedoms, DEGREE_LIMIT)
}

/// The highest total degree the eigenvalue sweep runs at.
///
/// `|a| + |b|` reaches `4v` over `{0,1,2}^v`, so this is the whole sweep at one
/// and two degrees of freedom and part of it at three. What it leaves out at
/// three is every pair of total degree 9 to 12, which is 78 of the 729 pairs:
///
///     $ python -c "from itertools import product; c=[a+b for a in product(range(3),repeat=3) for b in product(range(3),repeat=3)]; print(sum(sum(k)>8 for k in c), len(c))"
///     78 729
///
/// It is a time budget and not a statement that those 78 hold. They are the
/// expensive end rather than a tenth of the work, because a degree 12 monomial
/// in six variables makes the bracket walk arrays of `M(12, 6) = 6188`
/// coefficients: dropping them took this test from 42.14 seconds to 23.79 on
/// the machine issue #30 records, under
///
///     cargo test --locked --offline --test bracket -- --exact the_homological_eigenvalue_is_the_one_item_8_states
///
/// Both numbers are that machine's and neither transfers. What they are used
/// for here is the ratio, which is what says the tail is where the time goes.
const DEGREE_LIMIT: u32 = 8;

fn eigenvalue_disagreements_to_degree(
    bracket: Bracket<Gaussian>,
    freedoms: usize,
    degree_limit: u32,
) -> Vec<String> {
    let frequencies: Vec<i128> = [3, 5, 7][..freedoms].to_vec();
    let variables = 2 * freedoms;

    let mut quadratic: Series<Gaussian> = Series::zero(freedoms, 2).expect("addressable");
    for (position, frequency) in frequencies.iter().enumerate() {
        let mut exponents = vec![0u32; variables];
        exponents[position] = 1;
        exponents[position + freedoms] = 1;
        quadratic
            .set_coefficient(
                2,
                index_of(&exponents, 2).expect("addressable"),
                Gaussian::imaginary(-frequency),
            )
            .expect("addressable");
    }

    let mut disagreements = Vec::new();
    let mut multi_indices = vec![vec![0u32; freedoms]];
    for _ in 0..freedoms {
        multi_indices = multi_indices
            .iter()
            .flat_map(|prefix| {
                (0..3u32).map(move |entry| {
                    let mut next = prefix.clone();
                    next.rotate_right(1);
                    next[0] = entry;
                    next
                })
            })
            .collect();
    }

    for left in &multi_indices {
        for right in &multi_indices {
            let mut exponents = vec![0u32; variables];
            exponents[..freedoms].copy_from_slice(left);
            exponents[freedoms..].copy_from_slice(right);
            let degree: u32 = exponents.iter().sum();
            if degree > degree_limit {
                continue;
            }

            let mut monomial: Series<Gaussian> =
                Series::zero(freedoms, degree).expect("addressable");
            monomial
                .set_coefficient(
                    degree,
                    index_of(&exponents, degree).expect("addressable"),
                    Gaussian::one(),
                )
                .expect("addressable");

            let divisor: i128 = (0..freedoms)
                .map(|entry| {
                    (i128::from(left[entry]) - i128::from(right[entry])) * frequencies[entry]
                })
                .sum();

            let stated = bracket_order(degree, 2);
            let mut expected: Series<Gaussian> =
                Series::zero(freedoms, stated).expect("addressable");
            expected
                .set_coefficient(
                    degree,
                    index_of(&exponents, degree).expect("addressable"),
                    Gaussian::imaginary(-divisor),
                )
                .expect("addressable");

            match bracket(&monomial, &quadratic) {
                Ok(answer) if answer == expected => {}
                Ok(_) => disagreements.push(format!(
                    "v={freedoms} a={left:?} b={right:?} divisor={divisor}"
                )),
                Err(error) => disagreements.push(format!(
                    "v={freedoms} a={left:?} b={right:?} was refused: {error}"
                )),
            }
        }
    }
    disagreements
}

/// The shipped bracket reproduces the eigenvalue at one, two and three degrees
/// of freedom.
#[test]
fn the_homological_eigenvalue_is_the_one_item_8_states() {
    for freedoms in 1..=3 {
        assert_eq!(
            eigenvalue_disagreements(Series::bracket, freedoms),
            Vec::<String>::new()
        );
    }
}

/// A bracket pairing neighbours rather than conjugate partners is refused by
/// the eigenvalue, and by nothing else in this file.
///
/// Both halves are asserted, because the second is the one that says why this
/// fixture is worth keeping. At one degree of freedom `j + 1` and `j + v` name
/// the same variable, so the fixture is the right bracket there and the sweep
/// finds nothing, which is why the refusal is read at two.
///
/// Two is where the refusal is read and three is not, for the reason
/// [`DEGREE_LIMIT`] gives one step further: the fixture assembles through the
/// public accessors and is slower than the kernel by a large factor, so the
/// sweep against it is held to the smallest phase space in which the mistake it
/// carries is a mistake at all. Whether the neighbour pairing is also refused
/// at three degrees of freedom is NOT MEASURED here.
#[test]
fn the_eigenvalue_refuses_a_bracket_that_pairs_neighbours() {
    assert_eq!(
        eigenvalue_disagreements(bracket_with_the_partner_adjacent, 1),
        Vec::<String>::new(),
        "at one degree of freedom the neighbour is the partner"
    );
    assert!(
        !eigenvalue_disagreements(bracket_with_the_partner_adjacent, 2).is_empty(),
        "the eigenvalue did not refuse the neighbour pairing at two freedoms"
    );
}

/// The identities do not refuse that bracket, which is why the eigenvalue is in
/// this file.
///
/// A negative result stated as one. It runs the same antisymmetry and Jacobi
/// bodies over the neighbour pairing and asserts they pass, so the sentence in
/// the header is a measurement rather than an argument. If a later change makes
/// an identity strong enough to catch it, this test goes red and the header is
/// what has to be corrected.
#[test]
fn the_identities_do_not_refuse_the_neighbour_pairing() {
    let outcome = support::fixed_seed_runner_with(FIXTURE_CASES).run(
        &drafts_over(3, 2..=3, 0..=2),
        |drafts| {
            let first: Series<Exact> = drafts[0].build();
            let second: Series<Exact> = drafts[1].build();
            let third: Series<Exact> = drafts[2].build();

            let forward = bracket_with_the_partner_adjacent(&first, &second).map_err(refuse)?;
            let backward = bracket_with_the_partner_adjacent(&second, &first).map_err(refuse)?;
            prop_assert!(forward == backward.negated());

            let sum = jacobi_sum(
                &first,
                &second,
                &third,
                bracket_with_the_partner_adjacent as Bracket<Exact>,
            )
            .map_err(refuse)?;
            let zero: Series<Exact> =
                Series::zero(first.freedoms(), sum.order()).map_err(refuse)?;
            prop_assert!(sum == zero);
            Ok(())
        },
    );
    assert!(
        outcome.is_ok(),
        "an identity refused the neighbour pairing; the header of this file says it does not"
    );
}

/// Antisymmetry refuses a bracket that adds its two products instead of
/// subtracting them.
///
/// The proof that the antisymmetry property bites. Flip the `add` back in
/// `bracket_without_the_antisymmetric_sign` and this goes red.
#[test]
fn antisymmetry_refuses_a_bracket_without_the_sign() {
    let outcome = support::fixed_seed_runner_with(FIXTURE_CASES).run(
        &drafts_over(2, FREEDOMS, 1..=3),
        |drafts| {
            let left: Series<Exact> = drafts[0].build();
            let right: Series<Exact> = drafts[1].build();
            let forward = bracket_without_the_antisymmetric_sign(&left, &right).map_err(refuse)?;
            let backward = bracket_without_the_antisymmetric_sign(&right, &left).map_err(refuse)?;
            prop_assert!(forward == backward.negated());
            Ok(())
        },
    );
    assert!(
        outcome.is_err(),
        "antisymmetry passed a bracket whose two products are added"
    );
}

/// The assembly the fixtures are built from agrees with the shipped kernel.
///
/// This is what makes the three fixtures mean anything. Each of them is this
/// assembly with one thing changed, so a property refusing one of them is
/// refusing that one thing, and not the fact that a second implementation of a
/// bracket disagrees with the first for reasons nobody named.
///
/// It is also a genuine cross-check of the kernel: the assembly recomputes the
/// destination index from scratch per term pair and reads every coefficient
/// through the public accessors, where the kernel walks the stored arrays with
/// working space it carries between terms.
#[test]
fn the_public_api_assembly_agrees_with_the_shipped_bracket() {
    let outcome = over(FIXTURE_CASES, 2, ORDER, |drafts| {
        let left: Series<Exact> = drafts[0].build();
        let right: Series<Exact> = drafts[1].build();
        let shipped = left.bracket(&right).map_err(refuse)?;
        let assembled = bracket_via_the_public_api(&left, &right).map_err(refuse)?;
        prop_assert!(shipped == assembled);
        Ok(())
    });
    outcome.expect("the two implementations of the bracket agree");
}
