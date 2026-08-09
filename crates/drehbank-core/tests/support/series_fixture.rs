//! What the series properties are run over, and the multiplication that is
//! wrong on purpose.
//!
//! A draft is a coefficient-free description of a series: how many degrees of
//! freedom, what truncation order, and one small integer per monomial. The
//! properties generate drafts and build them in whichever coefficient type they
//! are being run in, so the same case runs in both instantiations and the
//! generator never has to know which.

use std::ops::RangeInclusive;

use drehbank_core::coefficient::Coefficient;
use drehbank_core::error::Error;
use drehbank_core::monomial::{dimension, product_index};
use drehbank_core::series::Series;
use proptest::prelude::*;

/// The degrees of freedom the properties run at.
///
/// Three is six variables, which is the shape the scale milestone targets, so
/// the properties are exercised at the width the package is for. One is where a
/// bijection with a single variable degenerates, which is the other end worth
/// having.
pub const FREEDOMS: RangeInclusive<usize> = 1..=3;

/// The truncation orders the properties run at.
///
/// Zero is a series that is a constant, where every convolution bound is at its
/// boundary at once. Four is where the graded arrays are large enough that an
/// index mistake has somewhere to hide: `M(4, 6)` is 126.
pub const ORDER: RangeInclusive<u32> = 0..=4;

/// The coefficients a draft is filled from.
///
/// Small integers, and the range is not a convenience. Every coefficient here
/// is exactly representable in binary64, and the bound is what keeps every
/// intermediate value of the properties exactly representable too, so the
/// `f64` instantiation can be compared exactly rather than to a tolerance. The
/// arithmetic behind that bound is in the header of `tests/series.rs`.
pub const MAGNITUDE: RangeInclusive<i32> = -8..=8;

/// A series before it has a coefficient type.
#[derive(Debug, Clone)]
pub struct Draft {
    /// Degrees of freedom, so `2 * freedoms` variables.
    pub freedoms: usize,
    /// The truncation order.
    pub order: u32,
    /// How many monomials each degree from 0 to `order` holds.
    pub widths: Vec<usize>,
    /// One value per monomial, degree by degree, in the layout `widths` gives.
    pub values: Vec<i32>,
}

impl Draft {
    /// Build this draft in a coefficient type.
    ///
    /// A zero is not written, so a degree whose values are all zero is left
    /// unstored. That is deliberate: it means the properties run over both
    /// storage states of the same series rather than over the materialised one
    /// only, which is where an arithmetic that treats them differently would
    /// otherwise hide.
    pub fn build<C: Coefficient>(&self) -> Series<C> {
        let mut series =
            Series::zero(self.freedoms, self.order).expect("every draft is inside the ranges");
        let mut at = 0;
        for (degree, &width) in self.widths.iter().enumerate() {
            for index in 0..width {
                let value = self.values[at + index];
                if value != 0 {
                    series
                        .set_coefficient(degree as u32, index as u64, C::from_small_integer(value))
                        .expect("every draft is inside the ranges");
                }
            }
            at += width;
        }
        series
    }
}

/// `count` drafts that share a phase space and a truncation order.
///
/// They have to share both, because the arithmetic refuses two series that do
/// not, and a generator producing pairs the arithmetic refuses would run every
/// property over the refusal instead of over the algebra.
pub fn drafts(count: usize) -> impl Strategy<Value = Vec<Draft>> {
    (FREEDOMS, ORDER)
        .prop_flat_map(move |(freedoms, order)| {
            let widths: Vec<usize> = (0..=order)
                .map(|degree| {
                    dimension(2 * freedoms, degree).expect("every case here is addressable")
                        as usize
                })
                .collect();
            let total: usize = widths.iter().sum();
            (
                Just(freedoms),
                Just(order),
                Just(widths),
                proptest::collection::vec(proptest::collection::vec(MAGNITUDE, total), count),
            )
        })
        .prop_map(|(freedoms, order, widths, blocks)| {
            blocks
                .into_iter()
                .map(|values| Draft {
                    freedoms,
                    order,
                    widths: widths.clone(),
                    values,
                })
                .collect()
        })
}

/// `count` drafts and an order to truncate them to.
///
/// The order is drawn inside the drafts' own order, because truncation to a
/// higher order is refused and a generator that produced one would run the
/// property over the refusal.
pub fn drafts_and_target_order(count: usize) -> impl Strategy<Value = (Vec<Draft>, u32)> {
    drafts(count).prop_flat_map(|drafts| {
        let order = drafts[0].order;
        (Just(drafts), 0..=order)
    })
}

/// The signature the real multiplication and the broken one share, so a
/// property can be run against either without being written twice.
pub type Multiply<C> = fn(&Series<C>, &Series<C>) -> Result<Series<C>, Error>;

/// The convolution with the truncation bound one character wrong.
///
/// `Series::product` walks the pairs of degrees whose sum is at most the
/// truncation order, which it writes as `0..=(order - left_degree)`. This is
/// the same walk with that written as `0..(order - left_degree)`, which is the
/// mistake somebody makes reaching for a half-open range out of habit. It costs
/// the whole top degree of every product, and it is invisible to any property
/// that truncates both sides the same way, which is why the suite has to carry
/// a property that does not.
///
/// Written through the public API rather than inside the module, so that what
/// it proves is a property of what a caller gets.
pub fn product_dropping_the_top_degree<C: Coefficient>(
    left: &Series<C>,
    right: &Series<C>,
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
    let mut accumulated: Vec<Vec<C>> = Vec::with_capacity(order as usize + 1);
    for degree in 0..=order {
        let width = left.dimension(degree)? as usize;
        accumulated.push(vec![C::zero(); width]);
    }
    for left_degree in 0..=order {
        // The one character. The shipped kernel has `0..=(order - left_degree)`.
        for right_degree in 0..(order - left_degree) {
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
                    let raised = accumulated[sum_degree as usize][target as usize]
                        .add(&left_value.multiply(&right_value));
                    accumulated[sum_degree as usize][target as usize] = raised;
                }
            }
        }
    }
    let mut result = Series::zero(left.freedoms(), order)?;
    for (degree, values) in accumulated.into_iter().enumerate() {
        for (index, value) in values.into_iter().enumerate() {
            result.set_coefficient(degree as u32, index as u64, value)?;
        }
    }
    Ok(result)
}
