//! What the lattice properties are run over, and the two computations they are
//! checked against.
//!
//! Both computations here are written for this file rather than borrowed from
//! the crate. A membership check that reduces against the canonical basis, and
//! a test that checks it by reducing against the canonical basis, agree with
//! each other whatever either of them does. So membership is checked against a
//! rank comparison over the rationals, and the retained count is checked
//! against an enumeration of exponent pairs that never touches the monomial
//! index.

use std::ops::RangeInclusive;

use proptest::prelude::*;

/// The degrees of freedom the lattice properties run at.
///
/// Two is the smallest phase space where a proper sublattice exists at all,
/// and four is wide enough that the Hermite normal form has several pivots to
/// get wrong. Six is what the package targets and is left out here on purpose:
/// the properties below are quadratic in the degrees of freedom and this range
/// already exercises every branch of the elimination.
pub const FREEDOMS: RangeInclusive<usize> = 2..=4;

/// The entries a declared relation is drawn from.
pub const ENTRY: RangeInclusive<i64> = -4..=4;

/// One rewriting of a declaration that leaves the lattice it generates alone.
///
/// These are the two things a user does without meaning to when they write the
/// same physics down twice: add a multiple of one relation to another, and
/// write a relation again in a scaled form.
#[derive(Debug, Clone, Copy)]
pub struct Rewrite {
    /// Which relation is added into, reduced against the current count.
    pub into: usize,
    /// Which relation is added, reduced against the current count.
    pub from: usize,
    /// How many times.
    pub times: i64,
    /// Which relation is copied, reduced against the current count.
    pub copy: usize,
    /// The non-zero factor the copy is scaled by.
    pub factor: i64,
}

/// A declaration, together with a rewriting of it that has to name the same
/// lattice.
#[derive(Debug, Clone)]
pub struct Declaration {
    /// The degrees of freedom.
    pub freedoms: usize,
    /// The relations as first written.
    pub rows: Vec<Vec<i64>>,
    /// The rewritings applied to produce the equivalent declaration.
    pub rewrites: Vec<Rewrite>,
}

impl Declaration {
    /// The same lattice, written differently.
    pub fn rewritten(&self) -> Vec<Vec<i64>> {
        let mut rows = self.rows.clone();
        for rewrite in &self.rewrites {
            if rows.len() >= 2 {
                let into = rewrite.into % rows.len();
                let from = rewrite.from % rows.len();
                if into != from {
                    let addend = rows[from].clone();
                    for (slot, entry) in rows[into].iter_mut().zip(addend.iter()) {
                        *slot += rewrite.times * entry;
                    }
                }
            }
            let copy = rewrite.copy % rows.len();
            let scaled: Vec<i64> = rows[copy]
                .iter()
                .map(|entry| rewrite.factor * entry)
                .collect();
            rows.push(scaled);
        }
        rows
    }
}

/// Declarations of rank strictly below the degrees of freedom.
///
/// The count of relations is capped one below the degrees of freedom, so the
/// saturation can never be the whole lattice and the generator never produces
/// the declaration the constructor refuses. A generator that produced it would
/// run every property below over that refusal.
pub fn declarations() -> impl Strategy<Value = Declaration> {
    FREEDOMS.prop_flat_map(|freedoms| {
        (
            Just(freedoms),
            proptest::collection::vec(proptest::collection::vec(ENTRY, freedoms), 1..freedoms),
            proptest::collection::vec(
                (
                    0usize..8,
                    0usize..8,
                    -3i64..=3,
                    0usize..8,
                    prop::sample::select(vec![-3i64, -2, -1, 1, 2, 3]),
                ),
                1..=6,
            ),
        )
            .prop_map(|(freedoms, rows, rewrites)| Declaration {
                freedoms,
                rows,
                rewrites: rewrites
                    .into_iter()
                    .map(|(into, from, times, copy, factor)| Rewrite {
                        into,
                        from,
                        times,
                        copy,
                        factor,
                    })
                    .collect(),
            })
    })
}

/// The rank over the rationals of a set of integer rows.
///
/// Fraction-free Gaussian elimination: a row below the pivot is replaced by
/// `pivot_entry * row - row_entry * pivot_row`, which stays integral, and each
/// row is divided through by the greatest common divisor of its entries to keep
/// the numbers from growing. This decides the rational span, which is a
/// different question from the one the crate answers and reaches the same
/// answer, because the module is saturated.
pub fn rank_over_rationals(rows: &[Vec<i64>]) -> usize {
    let mut matrix: Vec<Vec<i128>> = rows
        .iter()
        .map(|row| row.iter().map(|entry| i128::from(*entry)).collect())
        .collect();
    if matrix.is_empty() {
        return 0;
    }
    let columns = matrix[0].len();
    let mut rank = 0;
    for column in 0..columns {
        let Some(found) = (rank..matrix.len()).find(|row| matrix[*row][column] != 0) else {
            continue;
        };
        matrix.swap(found, rank);
        let pivot = matrix[rank].clone();
        for row in matrix.iter_mut().skip(rank + 1) {
            let factor = row[column];
            if factor == 0 {
                continue;
            }
            for (entry, above) in row.iter_mut().zip(pivot.iter()) {
                *entry = pivot[column] * *entry - factor * above;
            }
            reduce_by_gcd(row);
        }
        rank += 1;
        if rank == matrix.len() {
            break;
        }
    }
    rank
}

/// Whether `vector` lies in the rational span of `rows`.
///
/// The rank does not move when the vector is added exactly when the vector was
/// already in the span. This is the independent answer the membership property
/// is compared against, and it is the right one because the module the crate
/// builds is the saturation, which is precisely the integer points of that
/// span.
pub fn lies_in_the_rational_span(rows: &[Vec<i64>], vector: &[i64]) -> bool {
    let without = rank_over_rationals(rows);
    let mut with: Vec<Vec<i64>> = rows.to_vec();
    with.push(vector.to_vec());
    rank_over_rationals(&with) == without
}

fn reduce_by_gcd(row: &mut [i128]) {
    let mut divisor: i128 = 0;
    for entry in row.iter() {
        divisor = gcd(divisor, entry.abs());
    }
    if divisor > 1 {
        for entry in row.iter_mut() {
            *entry /= divisor;
        }
    }
}

fn gcd(mut left: i128, mut right: i128) -> i128 {
    while right != 0 {
        let next = left % right;
        left = right;
        right = next;
    }
    left
}

/// Every vector of `length` non-negative entries summing to `total`.
fn compositions(length: usize, total: u32) -> Vec<Vec<u32>> {
    if length == 0 {
        return if total == 0 {
            vec![Vec::new()]
        } else {
            Vec::new()
        };
    }
    let mut out = Vec::new();
    for first in 0..=total {
        for mut rest in compositions(length - 1, total - first) {
            let mut one = vec![first];
            one.append(&mut rest);
            out.push(one);
        }
    }
    out
}

/// How many monomials of a degree a lattice retains, counted by enumerating
/// the exponent pairs directly.
///
/// The crate walks the monomials of a degree through the bijection of 0003.
/// This walks every pair `(a, b)` with `|a| + |b| = d` and asks the same
/// question of each, so the two agree only if the bijection reaches each
/// monomial exactly once and the membership is the same membership. A count
/// that borrowed the bijection would prove neither.
pub fn retained_by_enumeration(
    freedoms: usize,
    degree: u32,
    member: &dyn Fn(&[i64]) -> bool,
) -> u64 {
    let mut count = 0;
    for left in 0..=degree {
        for first in compositions(freedoms, left) {
            for second in compositions(freedoms, degree - left) {
                let multi_index: Vec<i64> = first
                    .iter()
                    .zip(second.iter())
                    .map(|(one, other)| i64::from(*one) - i64::from(*other))
                    .collect();
                if member(&multi_index) {
                    count += 1;
                }
            }
        }
    }
    count
}
