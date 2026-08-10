//! The answer does not move with the thread count, and the check that says so
//! can tell when it does.
//!
//! `docs/decisions/0009-parallelism-and-memory.md` writes the reproducibility
//! requirement in the form a test is written against: for every fixture, the
//! complete result computed on `t` threads on run `i` equals the result on one
//! thread on run one, for `t` in one, two, three, seven, eight and sixteen, and
//! for two runs at each. Three of those counts are not powers of two and one is
//! prime, so the mapping from chunks to threads is uneven in several different
//! ways rather than in one, and running sixteen on a machine with fewer cores
//! is the point rather than a compromise: oversubscription makes completion
//! order vary more.
//!
//! # What is compared, and what 0009 asks for
//!
//! 0009 compares the byte serialisation of the result, produced by the writer
//! of `docs/decisions/0010-file-format.md`. There is no writer in this tree.
//! That is issue #32 and it has not landed, so the comparison here is over the
//! bit pattern of every coefficient a caller can read out of the result, which
//! is what such a writer would be given. It is not the serialisation and this
//! file does not claim it is. What it does cover is the whole of what the
//! result holds, including the sign of a zero, which an equality on `f64` would
//! not.
//!
//! # Why the generated corpus cannot prove the check bites
//!
//! Every coefficient a draft carries is a small integer and every intermediate
//! value the properties below produce is exactly representable, for the reason
//! the header of `tests/series.rs` works out. On coefficients like that no
//! reordering of any sum moves anything, so the corpus proves the kernels agree
//! and proves nothing about whether disagreement would be caught. The near miss
//! is built by hand in `support::parallel_fixture` for that reason, and the
//! last two tests here are the proof that the check refuses a reduction in any
//! order but the fixed one.

mod support;

use std::num::NonZero;

use drehbank_core::error::Error;
use drehbank_core::parallel::{self, Pool};
use drehbank_core::series::Series;
use proptest::prelude::*;
use proptest::test_runner::TestError;

use support::exact::Exact;
use support::parallel_fixture::{
    HALF_STEP, Kernel, near_miss, product_reducing_in_completion_order, q_squared_index,
};
use support::series_fixture::{Draft, drafts};

/// The thread counts of 0009.
const POOLS: [usize; 6] = [1, 2, 3, 7, 8, 16];

/// How many runs at each thread count.
///
/// Two, because a scheduler that varies between runs at one fixed count fails
/// in the same way as one that varies between counts, and a single run at each
/// count would not see it.
const RUNS: usize = 2;

/// How many cases each property below runs.
///
/// Small, and the reason is arithmetic rather than impatience. One case here is
/// one sequential kernel and twelve parallel ones, each of which starts and
/// joins its whole pool once per output degree, so a case costs of the order of
/// a hundred thread lifetimes on top of the convolutions. The generator's space
/// is one to three degrees of freedom and order zero to four, which this many
/// cases walks several times over.
const CASES: u32 = 24;

fn refuse(error: Error) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

fn pool(threads: usize) -> Pool {
    Pool::of(NonZero::new(threads).expect("every count in POOLS is above zero"))
}

/// Every coefficient a caller can read, as bits.
///
/// Through the public accessor rather than off the arrays, so a degree held
/// empty and a degree held as zeros read the same, which is the equality the
/// series type itself uses and the thing a writer would serialise.
fn readable(series: &Series<f64>) -> Result<Vec<u64>, Error> {
    let mut bits = Vec::new();
    for degree in 0..=series.order() {
        for index in 0..series.dimension(degree)? {
            bits.push(series.coefficient(degree, index)?.to_bits());
        }
    }
    Ok(bits)
}

/// Whether a kernel returns the sequential answer, bit for bit, at every thread
/// count of 0009 and on both runs at each.
///
/// This is the check. The two tests at the bottom of this file run it against
/// the shipped kernel and against a kernel whose reduction order is wrong, and
/// what makes it a guard rather than a hope is that the second one comes back
/// false.
fn holds_across_thread_counts(
    kernel: Kernel<f64>,
    left: &Series<f64>,
    right: &Series<f64>,
) -> Result<bool, Error> {
    let sequential = readable(&left.product(right)?)?;
    for threads in POOLS {
        for _ in 0..RUNS {
            if readable(&kernel(left, right, pool(threads))?)? != sequential {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// The parallel product is the sequential product, bit for bit, at every thread
/// count.
#[test]
fn the_parallel_product_is_the_sequential_product_at_every_thread_count()
-> Result<(), TestError<Vec<Draft>>> {
    support::fixed_seed_runner_with(CASES).run(&drafts(2), |drafts| {
        let left = drafts[0].build::<f64>();
        let right = drafts[1].build::<f64>();
        let sequential = readable(&left.product(&right).map_err(refuse)?).map_err(refuse)?;
        for threads in POOLS {
            for run in 1..=RUNS {
                let parallel = parallel::product(&left, &right, pool(threads)).map_err(refuse)?;
                prop_assert_eq!(
                    readable(&parallel).map_err(refuse)?,
                    sequential.clone(),
                    "at {} thread(s), run {}",
                    threads,
                    run
                );
            }
        }
        Ok(())
    })
}

/// The parallel bracket is the sequential bracket, bit for bit, at every thread
/// count.
///
/// Its own property rather than a corollary of the product's. The bracket is a
/// sum of `2v` convolutions with alternating signs, so it has an order the
/// product does not have, and a slot's contributions cross the terms of that
/// sum as well as the degrees inside each one.
#[test]
fn the_parallel_bracket_is_the_sequential_bracket_at_every_thread_count()
-> Result<(), TestError<Vec<Draft>>> {
    support::fixed_seed_runner_with(CASES).run(&drafts(2), |drafts| {
        let left = drafts[0].build::<f64>();
        let right = drafts[1].build::<f64>();
        let sequential = readable(&left.bracket(&right).map_err(refuse)?).map_err(refuse)?;
        for threads in POOLS {
            for run in 1..=RUNS {
                let parallel = parallel::bracket(&left, &right, pool(threads)).map_err(refuse)?;
                prop_assert_eq!(
                    readable(&parallel).map_err(refuse)?,
                    sequential.clone(),
                    "at {} thread(s), run {}",
                    threads,
                    run
                );
            }
        }
        Ok(())
    })
}

/// Both kernels agree with the sequential ones in the exact coefficient type
/// too.
///
/// Not a repeat of the two properties above. Those compare bit patterns, where
/// a disagreement could be the gather walking the pairs in a different order
/// rather than walking different pairs. This one runs where addition is
/// associative, so a disagreement can only be a different set of terms, which
/// separates an ordering defect from an enumeration defect.
///
/// The fixture ring is `support::exact::Exact` and not the exact rational type
/// of `docs/decisions/0002-coefficients.md`, which does not exist in this tree.
#[test]
fn both_kernels_agree_with_the_sequential_ones_in_the_exact_fixture_ring()
-> Result<(), TestError<Vec<Draft>>> {
    support::fixed_seed_runner_with(CASES).run(&drafts(2), |drafts| {
        let left = drafts[0].build::<Exact>();
        let right = drafts[1].build::<Exact>();
        let product = left.product(&right).map_err(refuse)?;
        let bracket = left.bracket(&right).map_err(refuse)?;
        for threads in POOLS {
            prop_assert_eq!(
                parallel::product(&left, &right, pool(threads)).map_err(refuse)?,
                product.clone(),
                "product at {} thread(s)",
                threads
            );
            prop_assert_eq!(
                parallel::bracket(&left, &right, pool(threads)).map_err(refuse)?,
                bracket.clone(),
                "bracket at {} thread(s)",
                threads
            );
        }
        Ok(())
    })
}

/// The near-miss case is a case where the reduction order is visible at all.
///
/// Without this the two tests below prove nothing: a check cannot be shown to
/// refuse a wrong order on an input where every order gives the same answer.
/// The coefficient of `q^2` receives one, then half a step above one, then half
/// a step, and the two ways of adding those three are different numbers.
///
/// Compared as bit patterns rather than as values, like everything else in this
/// file. `tests/series.rs` works out at length why its own binary64 equality is
/// exact rather than toleranced and says it is the only place in the suite that
/// compares binary64 directly; a second place here would make that sentence
/// false and would carry no such argument of its own. Bits are also the
/// stronger comparison, and this test is about which of two numbers came out.
#[test]
fn the_near_miss_case_has_two_orders_that_are_different_numbers() -> Result<(), Error> {
    let (left, right) = near_miss()?;
    let index = q_squared_index()?;
    let in_order = left.product(&right)?.coefficient(2, index)?;
    let reversed =
        product_reducing_in_completion_order(&left, &right, pool(1))?.coefficient(2, index)?;
    assert_eq!(in_order.to_bits(), 1.0_f64.to_bits());
    assert_eq!(reversed.to_bits(), (1.0 + 2.0 * HALF_STEP).to_bits());
    assert_ne!(in_order.to_bits(), reversed.to_bits());
    Ok(())
}

/// The shipped kernel passes the check on that case.
#[test]
fn the_shipped_kernel_holds_the_answer_still_on_the_near_miss_case() -> Result<(), Error> {
    let (left, right) = near_miss()?;
    assert!(holds_across_thread_counts(
        parallel::product,
        &left,
        &right
    )?);
    Ok(())
}

/// A kernel that combines its accumulators in the other order fails it.
///
/// This is the proof that the guard bites, and it bites for the reason it
/// names. Delete the `.rev()` in `product_reducing_in_completion_order` and
/// this test goes red, because the fixture then reduces in the order 0009
/// fixes and the check has nothing to refuse.
#[test]
fn the_check_refuses_a_reduction_that_is_not_in_the_fixed_order() -> Result<(), Error> {
    let (left, right) = near_miss()?;
    assert!(!holds_across_thread_counts(
        product_reducing_in_completion_order,
        &left,
        &right
    )?);
    Ok(())
}
