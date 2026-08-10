//! The algebraic properties of the truncated series, in two coefficient types,
//! and the proof that they refuse a convolution whose truncation bound is
//! wrong.
//!
//! # Why every property is run twice
//!
//! A truncated series over a commutative ring is a commutative ring. Every
//! property below is that sentence, so what the property tests is the
//! convolution and the index arithmetic, and what it inherits is the
//! coefficient. Run in one coefficient type only, a failure cannot be placed:
//! an associativity that fails in `f64` is a defect in the kernel or it is the
//! rounding, and nothing in the run says which.
//!
//! So each property runs in `support::exact::Exact`, where the arithmetic is
//! exact and a failure is the kernel, and again in `f64`, which is the type the
//! throughput path uses and the one a defect would actually ship in.
//!
//! # Why the `f64` comparison is exact rather than toleranced
//!
//! Machine floating point is not a ring, so the properties below are not
//! true of `f64` in general. They are true of the cases generated here, and
//! that is arranged rather than hoped for. Every coefficient a draft carries is
//! an integer of magnitude at most 8, which binary64 represents exactly, and
//! every operation the properties perform on those is exact as long as the
//! result stays below `2^53`. It does, by this bound.
//!
//! One product raises the magnitude by at most a factor of `8 * P`, where `P`
//! is the number of monomial pairs that can land on one output monomial, and
//! `P` is at most the size of the largest degree in play. The properties run at
//! `m = 6` variables and order 4 at the widest, where the largest degree holds
//! `M(4, 6) = 126` monomials:
//!
//!     $ python -c "from math import comb; print(comb(4+5,5))"
//!     126
//!
//! So a coefficient after one product is at most `8 * 8 * 126 = 8064`, after
//! the second product of an associativity case at most `8064 * 8 * 126`, which
//! is 8128512, and a distributivity case adds two such together. All of those
//! are integers below `2^53`, which is 9007199254740992, so every intermediate
//! value is exactly representable and every sum and product of them is exact.
//! The equality below is therefore an equality and not a tolerance.
//!
//! This paragraph used to end by saying it was the only place in the suite that
//! compares binary64 values directly, and that was never true: the detection
//! tests compared divisors against literals in the change that added them, which
//! was already landed when the sentence was written. It is removed rather than
//! repaired with a count, because a count in a comment drifts against the tree
//! it describes, and it licensed nothing outside this file in the first place.
//! Every other site owes its own argument and none of them gets one from here,
//! which is issue #98 and, as an invariant over the tree, issue #57.
//!
//! That is a statement about these ranges and about nothing else. It is not the
//! differential test against an exact oracle over arbitrary inputs, which is
//! issue #31 and which needs the exact coefficient type of 0002 rather than the
//! fixture ring used here.

mod support;

use drehbank_core::coefficient::Coefficient;
use drehbank_core::error::Error;
use drehbank_core::series::Series;
use proptest::prelude::*;
use proptest::test_runner::TestError;

use support::exact::Exact;
use support::series_fixture::{Draft, Multiply, drafts, drafts_and_target_order};

/// How many cases each property below runs.
///
/// Smaller than the `CASES` of the index properties, and for a stated reason:
/// one case here is up to four convolutions over graded arrays, which is
/// several thousand index computations, where one case there is a single one.
/// The generator's whole space is narrow, one to three degrees of freedom and
/// order zero to four, so this many cases walks it many times over.
const CASES: u32 = 256;

fn refuse(error: Error) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn over_drafts<F>(count: usize, body: F) -> Result<(), TestError<Vec<Draft>>>
where
    F: Fn(Vec<Draft>) -> Result<(), TestCaseError>,
{
    support::fixed_seed_runner_with(CASES).run(&drafts(count), body)
}

/// `f + g = g + f`.
fn addition_is_commutative<C>() -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(2, |drafts| {
        let left = drafts[0].build::<C>();
        let right = drafts[1].build::<C>();
        prop_assert_eq!(
            left.sum(&right).map_err(refuse)?,
            right.sum(&left).map_err(refuse)?
        );
        Ok(())
    })
}

/// `(f + g) + h = f + (g + h)`.
fn addition_is_associative<C>() -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(3, |drafts| {
        let first = drafts[0].build::<C>();
        let second = drafts[1].build::<C>();
        let third = drafts[2].build::<C>();
        let left = first
            .sum(&second)
            .map_err(refuse)?
            .sum(&third)
            .map_err(refuse)?;
        let right = first
            .sum(&second.sum(&third).map_err(refuse)?)
            .map_err(refuse)?;
        prop_assert_eq!(left, right);
        Ok(())
    })
}

/// `f + 0 = f`, where the zero series is the one that stores nothing.
fn zero_is_the_additive_identity<C>() -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(1, |drafts| {
        let series = drafts[0].build::<C>();
        let zero = Series::<C>::zero(drafts[0].freedoms, drafts[0].order).map_err(refuse)?;
        prop_assert_eq!(series.sum(&zero).map_err(refuse)?, series.clone());
        Ok(())
    })
}

/// `(f + g) - g = f`, which is what makes the subtraction the addition's
/// inverse rather than a second operation that happens to look like one.
fn subtraction_undoes_addition<C>() -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(2, |drafts| {
        let left = drafts[0].build::<C>();
        let right = drafts[1].build::<C>();
        let mut walked = left.clone();
        walked.add_in_place(&right).map_err(refuse)?;
        walked.subtract_in_place(&right).map_err(refuse)?;
        prop_assert_eq!(walked, left);
        Ok(())
    })
}

/// `f + (-f) = 0`, which is what makes the negation a negation.
fn a_series_and_its_negation_cancel<C>() -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(1, |drafts| {
        let series = drafts[0].build::<C>();
        let zero = Series::<C>::zero(drafts[0].freedoms, drafts[0].order).map_err(refuse)?;
        prop_assert_eq!(series.sum(&series.negated()).map_err(refuse)?, zero);
        Ok(())
    })
}

/// Scaling by a coefficient is multiplying by the series that is that
/// coefficient.
///
/// Scaling is a separate code path from the convolution, which is why it is
/// worth having: it is fast because it touches only the stored coefficients,
/// and the price of that is that it could disagree with the multiplication it
/// is a shortcut for. The two are compared rather than assumed to agree.
fn scaling_agrees_with_multiplying_by_a_constant<C>() -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(2, |drafts| {
        let series = drafts[0].build::<C>();
        // The second draft is used for one number only, its degree zero
        // coefficient, so the factor varies with the case rather than being a
        // constant the property could be passing by.
        let factor = drafts[1].build::<C>().coefficient(0, 0).map_err(refuse)?;
        let mut constant =
            Series::<C>::zero(drafts[0].freedoms, drafts[0].order).map_err(refuse)?;
        constant
            .set_coefficient(0, 0, factor.clone())
            .map_err(refuse)?;
        prop_assert_eq!(
            series.scaled(&factor),
            series.product(&constant).map_err(refuse)?
        );
        Ok(())
    })
}

/// `f * g = g * f`.
fn multiplication_is_commutative<C>(multiply: Multiply<C>) -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(2, move |drafts| {
        let left = drafts[0].build::<C>();
        let right = drafts[1].build::<C>();
        prop_assert_eq!(
            multiply(&left, &right).map_err(refuse)?,
            multiply(&right, &left).map_err(refuse)?
        );
        Ok(())
    })
}

/// `(f * g) * h = f * (g * h)`, to the truncation order.
fn multiplication_is_associative<C>(multiply: Multiply<C>) -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(3, move |drafts| {
        let first = drafts[0].build::<C>();
        let second = drafts[1].build::<C>();
        let third = drafts[2].build::<C>();
        let left = multiply(&multiply(&first, &second).map_err(refuse)?, &third).map_err(refuse)?;
        let right =
            multiply(&first, &multiply(&second, &third).map_err(refuse)?).map_err(refuse)?;
        prop_assert_eq!(left, right);
        Ok(())
    })
}

/// `f * (g + h) = f * g + f * h`, to the truncation order.
fn multiplication_distributes_over_addition<C>(
    multiply: Multiply<C>,
) -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(3, move |drafts| {
        let first = drafts[0].build::<C>();
        let second = drafts[1].build::<C>();
        let third = drafts[2].build::<C>();
        let left = multiply(&first, &second.sum(&third).map_err(refuse)?).map_err(refuse)?;
        let right = multiply(&first, &second)
            .map_err(refuse)?
            .sum(&multiply(&first, &third).map_err(refuse)?)
            .map_err(refuse)?;
        prop_assert_eq!(left, right);
        Ok(())
    })
}

/// `f * 1 = f`.
///
/// The cheapest of the properties and the one that catches the most, because
/// it is the only one here that compares a product against something that was
/// not itself produced by the same multiplication. A convolution that drops a
/// degree agrees with itself in every property above and disagrees with `f`
/// here.
fn the_unit_series_is_the_multiplicative_identity<C>(
    multiply: Multiply<C>,
) -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(1, move |drafts| {
        let series = drafts[0].build::<C>();
        let unit = Series::<C>::unit(drafts[0].freedoms, drafts[0].order).map_err(refuse)?;
        prop_assert_eq!(multiply(&series, &unit).map_err(refuse)?, series.clone());
        Ok(())
    })
}

/// Multiplying and then truncating agrees with truncating and then
/// multiplying.
///
/// This is the direction that has to hold: the terms a truncation would remove
/// cannot contribute to the terms it keeps, because a product of monomials has
/// the sum of their degrees. A convolution whose bound is off by one breaks it
/// first, which is why it is written as a property and not assumed.
fn truncation_commutes_with_multiplication<C>(
    multiply: Multiply<C>,
) -> Result<(), TestError<(Vec<Draft>, u32)>>
where
    C: Coefficient + PartialEq,
{
    support::fixed_seed_runner_with(CASES).run(
        &drafts_and_target_order(2),
        move |(drafts, target)| {
            let left = drafts[0].build::<C>();
            let right = drafts[1].build::<C>();
            let multiplied_then_truncated = multiply(&left, &right)
                .map_err(refuse)?
                .truncated(target)
                .map_err(refuse)?;
            let truncated_then_multiplied = multiply(
                &left.truncated(target).map_err(refuse)?,
                &right.truncated(target).map_err(refuse)?,
            )
            .map_err(refuse)?;
            prop_assert_eq!(multiplied_then_truncated, truncated_then_multiplied);
            Ok(())
        },
    )
}

/// Evaluation is additive: `(f + g)(z) = f(z) + g(z)`.
///
/// Evaluation walks the exponent vectors where the arithmetic walks the
/// indices, so the two reach the same monomial by different routes, and this is
/// the identity between them that no truncation can weaken. The multiplicative
/// identity is not tested here, because a product truncates and the value of a
/// truncated product is not the product of the values.
fn evaluation_is_additive<C>() -> Result<(), TestError<Vec<Draft>>>
where
    C: Coefficient + PartialEq,
{
    over_drafts(2, |drafts| {
        let left = drafts[0].build::<C>();
        let right = drafts[1].build::<C>();
        // The point is fixed and alternating rather than generated. What is
        // being checked is that the two walks agree, and a generated point
        // would raise the magnitudes without widening what the case covers.
        let point: Vec<C> = (0..left.variables())
            .map(|variable| C::from_small_integer(if variable.is_multiple_of(2) { 1 } else { -1 }))
            .collect();
        let together = left
            .sum(&right)
            .map_err(refuse)?
            .evaluate(&point)
            .map_err(refuse)?;
        let separately = left
            .evaluate(&point)
            .map_err(refuse)?
            .add(&right.evaluate(&point).map_err(refuse)?);
        prop_assert!(together == separately);
        Ok(())
    })
}

macro_rules! in_both_coefficients {
    ($exact:ident, $binary64:ident, $property:ident) => {
        #[test]
        fn $exact() {
            $property::<Exact>().expect("the property holds in the exact coefficient");
        }

        #[test]
        fn $binary64() {
            $property::<f64>().expect("the property holds in binary64 on these ranges");
        }
    };
}

macro_rules! in_both_coefficients_multiplying {
    ($exact:ident, $binary64:ident, $property:ident) => {
        #[test]
        fn $exact() {
            $property::<Exact>(Series::<Exact>::product)
                .expect("the property holds in the exact coefficient");
        }

        #[test]
        fn $binary64() {
            $property::<f64>(Series::<f64>::product)
                .expect("the property holds in binary64 on these ranges");
        }
    };
}

in_both_coefficients!(
    addition_is_commutative_exactly,
    addition_is_commutative_in_binary64,
    addition_is_commutative
);
in_both_coefficients!(
    addition_is_associative_exactly,
    addition_is_associative_in_binary64,
    addition_is_associative
);
in_both_coefficients!(
    zero_is_the_additive_identity_exactly,
    zero_is_the_additive_identity_in_binary64,
    zero_is_the_additive_identity
);
in_both_coefficients!(
    subtraction_undoes_addition_exactly,
    subtraction_undoes_addition_in_binary64,
    subtraction_undoes_addition
);
in_both_coefficients!(
    a_series_and_its_negation_cancel_exactly,
    a_series_and_its_negation_cancel_in_binary64,
    a_series_and_its_negation_cancel
);
in_both_coefficients!(
    scaling_agrees_with_multiplying_by_a_constant_exactly,
    scaling_agrees_with_multiplying_by_a_constant_in_binary64,
    scaling_agrees_with_multiplying_by_a_constant
);
in_both_coefficients!(
    evaluation_is_additive_exactly,
    evaluation_is_additive_in_binary64,
    evaluation_is_additive
);
in_both_coefficients_multiplying!(
    truncation_commutes_with_multiplication_exactly,
    truncation_commutes_with_multiplication_in_binary64,
    truncation_commutes_with_multiplication
);
in_both_coefficients_multiplying!(
    multiplication_is_commutative_exactly,
    multiplication_is_commutative_in_binary64,
    multiplication_is_commutative
);
in_both_coefficients_multiplying!(
    multiplication_is_associative_exactly,
    multiplication_is_associative_in_binary64,
    multiplication_is_associative
);
in_both_coefficients_multiplying!(
    multiplication_distributes_over_addition_exactly,
    multiplication_distributes_over_addition_in_binary64,
    multiplication_distributes_over_addition
);
in_both_coefficients_multiplying!(
    the_unit_series_is_the_multiplicative_identity_exactly,
    the_unit_series_is_the_multiplicative_identity_in_binary64,
    the_unit_series_is_the_multiplicative_identity
);

/// The proof that the properties bite.
///
/// `product_dropping_the_top_degree` is the shipped convolution with the
/// truncation bound written as a half-open range, which is one character. The
/// identity property refuses it. Without this test the properties above would
/// be nine assertions nobody has watched fail, which is the state a guard is
/// worth nothing in.
#[test]
fn the_identity_property_refuses_a_convolution_that_drops_the_top_degree() {
    let outcome = the_unit_series_is_the_multiplicative_identity::<Exact>(
        support::series_fixture::product_dropping_the_top_degree,
    );
    let failure = outcome.expect_err("a convolution missing its top degree is not an identity");
    assert!(
        matches!(failure, TestError::Fail(..)),
        "the property has to fail on the case rather than give up on the generator: {failure}"
    );
}

/// The same broken convolution, refused by the truncation property as well.
///
/// Two properties rather than one, because a proof resting on a single
/// assertion is one a later edit to that assertion silently retires. It is also
/// the property whose bound is worth stating: multiplying and then truncating
/// to an order below the series' own separates the two, and truncating to the
/// series' own order does not, because there both sides lose the same degree.
///
/// The test below records which properties do not refuse it.
#[test]
fn the_truncation_property_refuses_a_convolution_that_drops_the_top_degree() {
    let outcome = truncation_commutes_with_multiplication::<Exact>(
        support::series_fixture::product_dropping_the_top_degree,
    );
    let failure = outcome.expect_err("a convolution missing its top degree does not commute");
    assert!(
        matches!(failure, TestError::Fail(..)),
        "the property has to fail on the case rather than give up on the generator: {failure}"
    );
}

/// What the two proofs above do not cover, run rather than asserted.
///
/// Commutativity, associativity and distributivity each compare one product of
/// a multiplication against another product of the same multiplication, so a
/// convolution that loses its top degree loses it on both sides and agrees with
/// itself. Left as a sentence, that is a claim about three properties nobody
/// ran; here it is the run. The three pass against the broken convolution, and
/// this test says so.
///
/// It goes red if a later change makes one of them sharper, which is the point:
/// the bound of a proof moving is a thing somebody should have to look at
/// rather than a comment that quietly stops being true.
#[test]
fn the_other_properties_do_not_refuse_that_convolution() {
    let broken = support::series_fixture::product_dropping_the_top_degree;
    assert!(
        multiplication_is_commutative::<Exact>(broken).is_ok(),
        "commutativity was expected to be blind to a lost top degree"
    );
    assert!(
        multiplication_is_associative::<Exact>(broken).is_ok(),
        "associativity was expected to be blind to a lost top degree"
    );
    assert!(
        multiplication_distributes_over_addition::<Exact>(broken).is_ok(),
        "distributivity was expected to be blind to a lost top degree"
    );
}
