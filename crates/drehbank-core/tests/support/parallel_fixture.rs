//! The case where the reduction order is visible, and the kernel that gets it
//! wrong.
//!
//! Everything the determinism suite runs over the generated corpus is built
//! from small integers, so every intermediate value is exact and no reordering
//! of any sum could move it. That corpus proves the two kernels agree; it
//! cannot prove that the check would notice if they did not, because on those
//! coefficients there is nothing to notice.
//!
//! So the near miss is built by hand here, and it is the one somebody would
//! actually write: private accumulators combined in whatever order the threads
//! finished in, which is the shape
//! `docs/decisions/0009-parallelism-and-memory.md` refuses by name.

use drehbank_core::coefficient::Coefficient;
use drehbank_core::error::Error;
use drehbank_core::monomial::product_index;
use drehbank_core::parallel::Pool;
use drehbank_core::series::Series;

/// The signature the shipped parallel product and the broken one share, so the
/// check can be run against either without being written twice.
pub type Kernel<C> = fn(&Series<C>, &Series<C>, Pool) -> Result<Series<C>, Error>;

/// The exponent vectors of the near-miss case, in the variable order of item 1
/// of `docs/decisions/0004-conventions.md`: one degree of freedom, so `q` then
/// `p`.
const CONSTANT: [u32; 2] = [0, 0];
const Q: [u32; 2] = [1, 0];
const Q_SQUARED: [u32; 2] = [2, 0];

/// Half the gap above one, which is the coefficient that makes the order
/// visible.
///
/// Adding it to one is a tie, and a tie rounds to even, so `1 + h + h` is one
/// and `(h + h) + 1` is the next representable number above one. Nothing about
/// that is exotic; it is the smallest arrangement in which two orders of the
/// same three terms differ, and it is why the reduction order in 0009 is a rule
/// rather than a preference.
pub const HALF_STEP: f64 = 1.1102230246251565e-16;

/// The two operands of the near-miss case.
///
/// Their product has three contributions to the coefficient of `q^2`, one from
/// each pair of degrees, and they arrive as one, then half a step, then half a
/// step. Summed in that order the answer is one. Summed the other way round it
/// is the next number above one.
pub fn near_miss() -> Result<(Series<f64>, Series<f64>), Error> {
    let mut left = Series::zero(1, 2)?;
    let mut right = Series::zero(1, 2)?;
    left.set_coefficient(0, product_index_of(&CONSTANT, 0)?, 1.0)?;
    left.set_coefficient(1, product_index_of(&Q, 1)?, HALF_STEP)?;
    left.set_coefficient(2, product_index_of(&Q_SQUARED, 2)?, HALF_STEP)?;
    right.set_coefficient(0, product_index_of(&CONSTANT, 0)?, 1.0)?;
    right.set_coefficient(1, product_index_of(&Q, 1)?, 1.0)?;
    right.set_coefficient(2, product_index_of(&Q_SQUARED, 2)?, 1.0)?;
    Ok((left, right))
}

/// Where the coefficient of `q^2` sits in the degree-two array.
pub fn q_squared_index() -> Result<u64, Error> {
    product_index_of(&Q_SQUARED, 2)
}

fn product_index_of(exponents: &[u32], degree: u32) -> Result<u64, Error> {
    Ok(drehbank_core::monomial::index_of(exponents, degree)?)
}

/// The convolution with one private accumulator per left degree, combined in
/// decreasing degree.
///
/// This is the second shape 0009 names, private accumulators rather than a
/// partitioned output, with the one thing that shape has to get right got
/// wrong. The pool argument is ignored, and that is deliberate: a completion
/// order is not something a test can reproduce, so the fixture pins one order
/// that completion order can produce and holds it still. What the check refuses
/// is any order but the fixed one, and reversing it is the cheapest way to be
/// any other order.
pub fn product_reducing_in_completion_order<C: Coefficient>(
    left: &Series<C>,
    right: &Series<C>,
    _pool: Pool,
) -> Result<Series<C>, Error> {
    if left.freedoms() != right.freedoms() {
        return Err(Error::FreedomsDiffer {
            left: left.freedoms(),
            right: right.freedoms(),
        });
    }
    if left.order() != right.order() {
        return Err(Error::OrderDiffers {
            left: left.order(),
            right: right.order(),
        });
    }
    let order = left.order();
    let variables = 2 * left.freedoms();
    // One accumulator per left degree, each holding every contribution that
    // degree makes to every output degree.
    let mut partials: Vec<Vec<Vec<C>>> = Vec::with_capacity(order as usize + 1);
    for left_degree in 0..=order {
        let mut partial: Vec<Vec<C>> = Vec::with_capacity(order as usize + 1);
        for degree in 0..=order {
            partial.push(vec![C::zero(); left.dimension(degree)? as usize]);
        }
        for right_degree in 0..=(order - left_degree) {
            let sum_degree = left_degree + right_degree;
            for left_index in 0..left.dimension(left_degree)? {
                let left_value = left.coefficient(left_degree, left_index)?;
                for right_index in 0..right.dimension(right_degree)? {
                    let right_value = right.coefficient(right_degree, right_index)?;
                    let target = product_index(
                        left_index,
                        left_degree,
                        right_index,
                        right_degree,
                        variables,
                    )?;
                    let raised = partial[sum_degree as usize][target as usize]
                        .add(&left_value.multiply(&right_value));
                    partial[sum_degree as usize][target as usize] = raised;
                }
            }
        }
        partials.push(partial);
    }
    let mut result = Series::zero(left.freedoms(), order)?;
    for degree in 0..=order {
        for index in 0..result.dimension(degree)? {
            let mut total = C::zero();
            // The one thing. 0009 says increasing chunk index; this is the
            // other direction.
            for partial in partials.iter().rev() {
                total = total.add(&partial[degree as usize][index as usize]);
            }
            result.set_coefficient(degree, index, total)?;
        }
    }
    Ok(result)
}
