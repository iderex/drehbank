//! What the bracket properties are run over, and the three brackets that are
//! wrong on purpose.
//!
//! Item 3 of `docs/decisions/0004-conventions.md` writes the bracket as a sum
//! of `v` differences, each one a derivative with respect to a position times a
//! derivative with respect to its conjugate momentum. Three things in that
//! sentence can be got wrong independently, and each of them is a mistake
//! somebody makes rather than one somebody invents:
//!
//! - which variable is the conjugate partner, `j + v` and not `j + 1`;
//! - the minus sign between the two products;
//! - the truncation order of the answer.
//!
//! So the shape here is one assembler, written against the public API, and
//! three callers of it that each differ from the right one in exactly one of
//! those. The assembler is itself a second implementation of the bracket, which
//! is why [`bracket_via_the_public_api`] exists: a property comparing it
//! against `Series::bracket` is what says the three below differ from the
//! shipped kernel in the thing they name and in nothing else.

use std::ops::RangeInclusive;

use drehbank_core::coefficient::Coefficient;
use drehbank_core::error::Error;
use drehbank_core::monomial::{dimension, product_index};
use drehbank_core::series::{Series, bracket_order};
use proptest::prelude::*;

use super::series_fixture::Draft;

/// The signature every bracket here and the shipped one share, so a property
/// can be run against any of them without being written twice.
pub type Bracket<C> = fn(&Series<C>, &Series<C>) -> Result<Series<C>, Error>;

/// One term of a bracket: which variable each side is differentiated by, and
/// whether the product goes in with a plus or a minus.
#[derive(Debug, Clone, Copy)]
pub struct Term {
    pub left: usize,
    pub right: usize,
    pub add: bool,
}

/// The terms item 3 of 0004 gives: position against its partner momentum,
/// minus momentum against its partner position, summed over the degrees of
/// freedom.
fn terms_of_the_convention(freedoms: usize) -> Vec<Term> {
    let mut terms = Vec::with_capacity(2 * freedoms);
    for position in 0..freedoms {
        let momentum = position + freedoms;
        terms.push(Term {
            left: position,
            right: momentum,
            add: true,
        });
        terms.push(Term {
            left: momentum,
            right: position,
            add: false,
        });
    }
    terms
}

/// The sum of the terms given, truncated at the order given, through the public
/// API only.
///
/// Nothing here reaches into the crate: the derivatives come from
/// `Series::derivative`, the destination index from `product_index`, and every
/// coefficient is read and written through the accessors a caller has. What a
/// property proves against this is therefore a property of what a caller gets.
pub fn assemble<C: Coefficient>(
    left: &Series<C>,
    right: &Series<C>,
    terms: &[Term],
    order: u32,
) -> Result<Series<C>, Error> {
    if left.freedoms() != right.freedoms() {
        return Err(Error::FreedomsDiffer {
            left: left.freedoms(),
            right: right.freedoms(),
        });
    }
    let variables = 2 * left.freedoms();
    let mut accumulated: Vec<Vec<C>> = Vec::with_capacity(order as usize + 1);
    for degree in 0..=order {
        accumulated.push(vec![C::zero(); dimension(variables, degree)? as usize]);
    }

    for term in terms {
        let left_derivative = left.derivative(term.left)?;
        let right_derivative = right.derivative(term.right)?;
        for left_degree in 0..=left_derivative.order() {
            for right_degree in 0..=right_derivative.order() {
                let sum_degree = left_degree + right_degree;
                if sum_degree > order {
                    continue;
                }
                for left_index in 0..left_derivative.dimension(left_degree)? {
                    let left_value = left_derivative.coefficient(left_degree, left_index)?;
                    for right_index in 0..right_derivative.dimension(right_degree)? {
                        let right_value =
                            right_derivative.coefficient(right_degree, right_index)?;
                        let target = product_index(
                            left_index,
                            left_degree,
                            right_index,
                            right_degree,
                            variables,
                        )?;
                        let product = left_value.multiply(&right_value);
                        let slot = &accumulated[sum_degree as usize][target as usize];
                        accumulated[sum_degree as usize][target as usize] = if term.add {
                            slot.add(&product)
                        } else {
                            slot.subtract(&product)
                        };
                    }
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

/// The bracket of item 3 of 0004, assembled the long way.
///
/// The same object `Series::bracket` returns, computed through the public API
/// with the terms and the order spelled out at the call site. It is the control
/// the three below are read against.
pub fn bracket_via_the_public_api<C: Coefficient>(
    left: &Series<C>,
    right: &Series<C>,
) -> Result<Series<C>, Error> {
    assemble(
        left,
        right,
        &terms_of_the_convention(left.freedoms()),
        bracket_order(left.order(), right.order()),
    )
}

/// The bracket pairing each variable with its neighbour instead of with its
/// conjugate partner.
///
/// Item 1 of 0004 orders the variables in two blocks, positions first, so the
/// partner of `z_j` is `v` places away and never adjacent. This is the same
/// assembly with `j + 1` written where `j + v` belongs, which is what happens
/// when a formula written for the interleaved order of the conversion table is
/// transcribed without the permutation.
///
/// It is still bilinear and still antisymmetric, because exchanging the two
/// arguments still negates it, and a constant bivector satisfies the Jacobi
/// identity whatever its entries are. So neither of those properties refuses
/// it and the property that does is the one that pins the convention: the
/// eigenvalue of item 8. At one degree of freedom it coincides with the right
/// answer, because `j + 1` and `j + v` are the same variable there, so the case
/// that refuses it has to be at two or more.
pub fn bracket_with_the_partner_adjacent<C: Coefficient>(
    left: &Series<C>,
    right: &Series<C>,
) -> Result<Series<C>, Error> {
    let freedoms = left.freedoms();
    let mut terms = Vec::with_capacity(2 * freedoms);
    for position in 0..freedoms {
        // The one substitution. The convention has `position + freedoms`.
        let neighbour = (position + 1) % (2 * freedoms);
        terms.push(Term {
            left: position,
            right: neighbour,
            add: true,
        });
        terms.push(Term {
            left: neighbour,
            right: position,
            add: false,
        });
    }
    assemble(
        left,
        right,
        &terms,
        bracket_order(left.order(), right.order()),
    )
}

/// The bracket with the two products added instead of subtracted.
///
/// One character, and it is the sign the whole antisymmetry of item 3 rests on.
/// What it produces is a symmetric bilinear form, so it agrees with the bracket
/// on every argument pair where one of the two products vanishes, which is
/// most small hand-written examples.
pub fn bracket_without_the_antisymmetric_sign<C: Coefficient>(
    left: &Series<C>,
    right: &Series<C>,
) -> Result<Series<C>, Error> {
    let freedoms = left.freedoms();
    let mut terms = terms_of_the_convention(freedoms);
    for term in &mut terms {
        // The one character. The convention alternates.
        term.add = true;
    }
    assemble(
        left,
        right,
        &terms,
        bracket_order(left.order(), right.order()),
    )
}

/// The bracket truncated one order below what its arguments determine.
///
/// The mistake issue #30 names: a bracket that answers at the order of its
/// arguments rather than at the order the grading gives loses the top of every
/// answer, and it loses it silently, because the result is a perfectly ordinary
/// series that is simply shorter. Written as one order lower rather than as the
/// arguments' order, because that is the near miss: it is wrong by the smallest
/// amount that is still wrong.
pub fn bracket_dropping_the_top_degree<C: Coefficient>(
    left: &Series<C>,
    right: &Series<C>,
) -> Result<Series<C>, Error> {
    assemble(
        left,
        right,
        &terms_of_the_convention(left.freedoms()),
        // The one change. The convention is `bracket_order` itself.
        bracket_order(left.order(), right.order()).saturating_sub(1),
    )
}

/// `count` drafts sharing a phase space and a truncation order, over ranges the
/// caller states.
///
/// The generator in `series_fixture` fixes both ranges, which is right for the
/// ring properties, where one case is a convolution. One case here is up to
/// nine brackets, each of them several convolutions over arrays that are larger
/// than either argument's, so the properties below choose their own ranges and
/// say what each one costs.
pub fn drafts_over(
    count: usize,
    freedoms: RangeInclusive<usize>,
    order: RangeInclusive<u32>,
) -> impl Strategy<Value = Vec<Draft>> {
    (freedoms, order)
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
                proptest::collection::vec(
                    proptest::collection::vec(super::series_fixture::MAGNITUDE, total),
                    count,
                ),
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
