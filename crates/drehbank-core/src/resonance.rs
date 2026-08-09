//! The resonance module, as a lattice.
//!
//! `docs/decisions/0007-resonance-module.md` decides what this is. A user
//! declares integer relations, the package replaces them by the canonical basis
//! of the lattice those relations generate, and from then on membership is an
//! exact integer computation with no tolerance anywhere.
//!
//! # Why a lattice and not the list that was typed
//!
//! The exact resonance module of a frequency vector is
//! `{ k in Z^v : <k, omega> = 0 }`, which is a subgroup and is saturated: if
//! `c k` lies in it for a non-zero integer `c` then so does `k`, because
//! `<c k, omega> = c <k, omega>`. A user who writes `(2, -2, 0)` has said
//! `omega_1 = omega_2`, which makes `(1, -1, 0)` resonant whether they wrote it
//! or not. A code that keeps the list it was given produces a normal form that
//! is not one, and the failure arrives at the first degree where the implied
//! relation has a term, which is late enough to be attributed to something
//! else.
//!
//! So a declaration is canonicalised in three steps, which is the whole of what
//! this module does: saturate, put in Hermite normal form, refuse the
//! degenerate cases. The Hermite normal form with the convention of 0007 is
//! unique for a lattice, so it is a name for the lattice rather than one basis
//! of it, and that is what makes two declarations of the same physics
//! comparable.
//!
//! Detection, which proposes a declaration from a frequency vector, is a
//! different operation and is issue #40. Nothing here reads a frequency vector
//! at all.

use crate::error::Error;
use crate::monomial::{dimension, exponents_into};

/// A resonance lattice in `Z^v`, held as its canonical basis.
///
/// Constructed from declared relations and never from a tolerance. Two
/// declarations that generate lattices with the same saturation produce the
/// same value here, entry for entry, which is the property the whole design
/// rests on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResonanceModule {
    freedoms: usize,
    /// The canonical basis, in the Hermite normal form of 0007: rows in
    /// echelon order by leading column, each pivot positive, every entry above
    /// a pivot reduced into `[0, pivot)`. Empty for the trivial module.
    basis: Vec<Vec<i64>>,
}

impl ResonanceModule {
    /// The trivial module, which is the non-resonant case.
    ///
    /// It contains the zero vector and nothing else, so the terms it retains
    /// are the ones with `a = b`, which are in the kernel for the reason 0004
    /// item 8 gives and are retained whatever the frequencies are.
    pub fn trivial(freedoms: usize) -> Result<Self, Error> {
        if freedoms == 0 {
            return Err(Error::NoFreedoms);
        }
        Ok(ResonanceModule {
            freedoms,
            basis: Vec::new(),
        })
    }

    /// The module a set of declared relations generates.
    ///
    /// A relation of all zeros is dropped rather than refused, because it says
    /// nothing and a declaration is allowed to say nothing. A relation of the
    /// wrong width is refused with its position named, because it is a caller
    /// working in a different phase space from the one they asked for. A
    /// declaration whose saturation has rank `v` is refused, because it makes
    /// every term resonant and the normal form is then the input unchanged,
    /// which is a request that has answered itself.
    pub fn declare(freedoms: usize, relations: &[Vec<i64>]) -> Result<Self, Error> {
        if freedoms == 0 {
            return Err(Error::NoFreedoms);
        }
        for (at, relation) in relations.iter().enumerate() {
            if relation.len() != freedoms {
                return Err(Error::RelationWidth {
                    freedoms,
                    given: relation.len(),
                    at,
                });
            }
        }
        let declared: Matrix = relations
            .iter()
            .filter(|relation| relation.iter().any(|entry| *entry != 0))
            .map(|relation| relation.iter().map(|entry| i128::from(*entry)).collect())
            .collect();
        if declared.is_empty() {
            return ResonanceModule::trivial(freedoms);
        }
        let basis = canonical(declared, freedoms)?;
        if basis.len() == freedoms {
            return Err(Error::EveryTermResonant {
                freedoms,
                rank: basis.len(),
            });
        }
        let basis = basis
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|entry| i64::try_from(entry).map_err(|_| Error::LatticeOverflow))
                    .collect::<Result<Vec<i64>, Error>>()
            })
            .collect::<Result<Vec<Vec<i64>>, Error>>()?;
        Ok(ResonanceModule { freedoms, basis })
    }

    /// The number of degrees of freedom this lattice lives in.
    pub fn freedoms(&self) -> usize {
        self.freedoms
    }

    /// The dimension of the module, which is the number of rows of the
    /// canonical basis.
    pub fn dimension(&self) -> usize {
        self.basis.len()
    }

    /// The canonical basis.
    ///
    /// Exposed because a user has to see what was actually in force rather than
    /// what they typed: the saturation step can return a lattice strictly
    /// larger than the declaration, and this is where that is visible.
    pub fn basis(&self) -> &[Vec<i64>] {
        &self.basis
    }

    /// Whether a multi-index lies in the module.
    ///
    /// Integer division and integer subtraction throughout. There is no
    /// tolerance here and no frequency vector, which is the point of declaring
    /// the module: once it exists, nothing downstream has to decide how near is
    /// near.
    pub fn contains(&self, multi_index: &[i64]) -> Result<bool, Error> {
        if multi_index.len() != self.freedoms {
            return Err(Error::MultiIndexWidth {
                freedoms: self.freedoms,
                given: multi_index.len(),
            });
        }
        let mut reduced: Vec<i128> = multi_index.iter().map(|entry| i128::from(*entry)).collect();
        self.reduce(&mut reduced)
    }

    /// The multi-index `k = a - b` of a monomial, from its exponent vector.
    ///
    /// The exponent vector is read in the blocked order of item 1 of 0004,
    /// which item 5 of that document says the complex variables are stored in
    /// too: the first `v` entries are `a` and the last `v` are `b`. The
    /// quantity that can be small is `<a - b, omega>` by item 8, and that
    /// difference is what this lattice is a lattice in.
    pub fn multi_index_of(&self, exponents: &[u32]) -> Result<Vec<i64>, Error> {
        let variables = 2 * self.freedoms;
        if exponents.len() != variables {
            return Err(Error::ExponentWidth {
                variables,
                given: exponents.len(),
            });
        }
        let (positions, momenta) = exponents.split_at(self.freedoms);
        Ok(positions
            .iter()
            .zip(momenta.iter())
            .map(|(first, second)| i64::from(*first) - i64::from(*second))
            .collect())
    }

    /// How many monomials of each degree from 0 to `order` the normal form
    /// keeps.
    ///
    /// This is the number somebody about to spend a day of compute needs, and
    /// it is cheap to produce and expensive to discover late: it is how much of
    /// the Hamiltonian survives normalisation. It is derived from the same
    /// bijection the storage uses, so it counts the monomials that will
    /// actually be there rather than a count of an independently enumerated
    /// set that could disagree with them.
    pub fn retained_per_degree(&self, order: u32) -> Result<Vec<u64>, Error> {
        let variables = 2 * self.freedoms;
        let mut exponents = vec![0u32; variables];
        let mut shifted = vec![0u64; variables - 1];
        let mut multi_index = vec![0i128; self.freedoms];
        let mut counts = Vec::with_capacity(order as usize + 1);
        for degree in 0..=order {
            let size = dimension(variables, degree)?;
            let mut count = 0u64;
            for index in 0..size {
                exponents_into(index, degree, &mut exponents, &mut shifted)?;
                let (positions, momenta) = exponents.split_at(self.freedoms);
                for ((slot, first), second) in multi_index
                    .iter_mut()
                    .zip(positions.iter())
                    .zip(momenta.iter())
                {
                    *slot = i128::from(*first) - i128::from(*second);
                }
                let mut reduced = multi_index.clone();
                if self.reduce(&mut reduced)? {
                    count += 1;
                }
            }
            counts.push(count);
        }
        Ok(counts)
    }

    /// Reduce a vector against the canonical basis, destroying it, and say
    /// whether it reached zero.
    ///
    /// At each pivot column the entry has to be divisible by the pivot, and
    /// that multiple of the row is subtracted. A vector that reaches zero is in
    /// the lattice and one that does not is not, which is exact because every
    /// step is an integer operation.
    fn reduce(&self, vector: &mut [i128]) -> Result<bool, Error> {
        for row in &self.basis {
            let (column, pivot) = row
                .iter()
                .enumerate()
                .find(|(_, entry)| **entry != 0)
                .map(|(column, entry)| (column, i128::from(*entry)))
                .expect("the canonical basis carries no zero row");
            if vector[column] % pivot != 0 {
                return Ok(false);
            }
            let multiple = vector[column] / pivot;
            if multiple != 0 {
                for (slot, entry) in vector.iter_mut().zip(row.iter()) {
                    *slot = multiple
                        .checked_mul(i128::from(*entry))
                        .and_then(|scaled| slot.checked_sub(scaled))
                        .ok_or(Error::LatticeOverflow)?;
                }
            }
        }
        Ok(vector.iter().all(|entry| *entry == 0))
    }
}

/// An integer matrix, row major.
type Matrix = Vec<Vec<i128>>;

/// Floor division, which is what the algorithm of 0007 is written in.
///
/// Not the language's `/`, which truncates toward zero. The two differ on a
/// negative quotient, and the reduction below is a gcd descent whose
/// termination argument is that the remainder shrinks, which is a statement
/// about the floored quotient. Getting this wrong gives a loop that terminates
/// anyway on most inputs and a basis that is not the canonical one on some.
fn floor_div(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator / denominator;
    if numerator % denominator != 0 && (numerator < 0) != (denominator < 0) {
        quotient - 1
    } else {
        quotient
    }
}

fn subtract_multiple(row: &mut [i128], other: &[i128], multiple: i128) -> Result<(), Error> {
    for (slot, entry) in row.iter_mut().zip(other.iter()) {
        *slot = multiple
            .checked_mul(*entry)
            .and_then(|scaled| slot.checked_sub(scaled))
            .ok_or(Error::LatticeOverflow)?;
    }
    Ok(())
}

/// The Hermite normal form of `rows`, with the unimodular transform that
/// produced it.
///
/// Row style, echelon by leading column, every pivot positive, every entry
/// above a pivot reduced into `[0, pivot)`. The transform `U` satisfies
/// `U * rows = H`, and its rows against the zero rows of `H` are a basis of the
/// integer kernel, which is what makes one routine serve all three steps of the
/// canonicalisation.
///
/// The elimination inside a column is a gcd descent: the row with the smallest
/// non-zero entry in that column is subtracted a floored number of times out of
/// every other, which strictly shrinks each of their entries in that column, so
/// the loop ends with one non-zero entry left.
fn hnf_with_transform(rows: Matrix, columns: usize) -> Result<(Matrix, Matrix), Error> {
    let height = rows.len();
    let mut form = rows;
    let mut transform: Matrix = (0..height)
        .map(|row| {
            (0..height)
                .map(|column| i128::from(row == column))
                .collect()
        })
        .collect();
    let mut pivots: Vec<(usize, usize)> = Vec::new();
    let mut rank = 0;
    for column in 0..columns {
        loop {
            let mut nonzero: Vec<usize> = (rank..height)
                .filter(|row| form[*row][column] != 0)
                .collect();
            if nonzero.len() <= 1 {
                break;
            }
            nonzero.sort_by_key(|row| form[*row][column].abs());
            let smallest = nonzero[0];
            for &row in &nonzero[1..] {
                let multiple = floor_div(form[row][column], form[smallest][column]);
                if multiple != 0 {
                    let pivot_row = form[smallest].clone();
                    let pivot_transform = transform[smallest].clone();
                    subtract_multiple(&mut form[row], &pivot_row, multiple)?;
                    subtract_multiple(&mut transform[row], &pivot_transform, multiple)?;
                }
            }
        }
        let Some(found) = (rank..height).find(|row| form[*row][column] != 0) else {
            continue;
        };
        if found != rank {
            form.swap(found, rank);
            transform.swap(found, rank);
        }
        if form[rank][column] < 0 {
            for entry in &mut form[rank] {
                *entry = -*entry;
            }
            for entry in &mut transform[rank] {
                *entry = -*entry;
            }
        }
        pivots.push((rank, column));
        rank += 1;
        if rank == height {
            break;
        }
    }
    for &(pivot_row, pivot_column) in &pivots {
        for row in 0..pivot_row {
            let multiple = floor_div(form[row][pivot_column], form[pivot_row][pivot_column]);
            if multiple != 0 {
                let above = form[pivot_row].clone();
                let above_transform = transform[pivot_row].clone();
                subtract_multiple(&mut form[row], &above, multiple)?;
                subtract_multiple(&mut transform[row], &above_transform, multiple)?;
            }
        }
    }
    Ok((form, transform))
}

/// A basis of the integer vectors `x` with `x * rows = 0`, read off the
/// transform against the zero rows of the Hermite normal form of the transpose.
fn integer_kernel(rows: &Matrix, columns: usize) -> Result<Matrix, Error> {
    if rows.is_empty() {
        return Ok(identity(columns));
    }
    let transposed: Matrix = (0..columns)
        .map(|column| rows.iter().map(|row| row[column]).collect())
        .collect();
    let (form, transform) = hnf_with_transform(transposed, rows.len())?;
    Ok(form
        .iter()
        .zip(transform.iter())
        .filter(|(row, _)| row.iter().all(|entry| *entry == 0))
        .map(|(_, kernel)| kernel.clone())
        .collect())
}

fn identity(width: usize) -> Matrix {
    (0..width)
        .map(|row| (0..width).map(|column| i128::from(row == column)).collect())
        .collect()
}

/// The canonical basis of the saturation of the lattice `rows` generate.
///
/// The saturation is the integer kernel of the integer kernel, which is exact
/// and needs no rational arithmetic: the first kernel is a basis of the vectors
/// orthogonal to the rational row span, and the kernel of that is the set of
/// integer vectors in the span, which is the saturation. Where the first kernel
/// is empty the span is everything, so the saturation is `Z^v`.
fn canonical(rows: Matrix, freedoms: usize) -> Result<Matrix, Error> {
    let null = integer_kernel(&rows, freedoms)?;
    let saturated = if null.is_empty() {
        identity(freedoms)
    } else {
        integer_kernel(&null, freedoms)?
    };
    let (form, _) = hnf_with_transform(saturated, freedoms)?;
    Ok(form
        .into_iter()
        .filter(|row| row.iter().any(|entry| *entry != 0))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::ResonanceModule;
    use crate::error::Error;

    /// The worked example of 0007, which is the case a list of relations gets
    /// wrong.
    ///
    /// The rows of the first declaration do not contain `(1, -1, 0)` in the
    /// lattice they generate over the integers. They do contain it in the
    /// saturation, because `2 omega_1 - 2 omega_2 = 0` says
    /// `omega_1 = omega_2`. A code that kept the declared list would answer no
    /// here and would then normalise a term it should have retained.
    #[test]
    fn the_saturation_recovers_a_relation_the_declaration_only_implies() {
        let declared =
            ResonanceModule::declare(3, &[vec![2, -2, 0], vec![0, 3, -3], vec![2, 1, -3]])
                .expect("the declaration has rank two in three freedoms");
        assert_eq!(declared.basis(), &[vec![1, 0, -1], vec![0, 1, -1]]);
        assert!(
            declared
                .contains(&[1, -1, 0])
                .expect("the query is three wide"),
            "the saturation step is what puts this relation back"
        );
        assert!(
            !declared
                .contains(&[1, 1, 0])
                .expect("the query is three wide")
        );
    }

    /// The two declarations of 0007's worked example name one lattice.
    #[test]
    fn two_declarations_of_the_same_physics_canonicalise_to_one_basis() {
        let first = ResonanceModule::declare(3, &[vec![2, -2, 0], vec![0, 3, -3], vec![2, 1, -3]])
            .expect("rank two in three freedoms");
        let second = ResonanceModule::declare(3, &[vec![1, -1, 0], vec![0, 1, -1]])
            .expect("rank two in three freedoms");
        assert_eq!(first, second);
    }

    /// The counts of 0007, which that document produced by brute force.
    #[test]
    fn the_retained_counts_are_the_ones_the_decision_tabulates() {
        let trivial = ResonanceModule::trivial(3).expect("three freedoms");
        let one = ResonanceModule::declare(3, &[vec![1, -1, 0]]).expect("rank one");
        let two = ResonanceModule::declare(3, &[vec![1, -1, 0], vec![0, 1, -1]]).expect("rank two");
        let counts = |module: &ResonanceModule| {
            module
                .retained_per_degree(8)
                .expect("degree eight in six variables is addressable")
        };
        assert_eq!(
            counts(&trivial)[2..=8],
            [3, 0, 6, 0, 10, 0, 15],
            "the non-resonant column"
        );
        assert_eq!(
            counts(&one)[2..=8],
            [5, 0, 14, 0, 30, 0, 55],
            "the column for the module generated by (1, -1, 0)"
        );
        assert_eq!(
            counts(&two)[2..=8],
            [9, 0, 36, 0, 100, 0, 225],
            "the column for the module generated by (1, -1, 0) and (0, 1, -1)"
        );
    }

    /// A relation of the wrong width names its own position.
    #[test]
    fn a_relation_in_the_wrong_phase_space_is_refused_with_its_position() {
        assert_eq!(
            ResonanceModule::declare(3, &[vec![1, -1, 0], vec![1, -1]]).err(),
            Some(Error::RelationWidth {
                freedoms: 3,
                given: 2,
                at: 1
            })
        );
    }

    /// A declaration whose saturation is everything is refused, with the rank.
    #[test]
    fn a_declaration_that_makes_every_term_resonant_is_refused() {
        assert_eq!(
            ResonanceModule::declare(2, &[vec![1, 0], vec![0, 1]]).err(),
            Some(Error::EveryTermResonant {
                freedoms: 2,
                rank: 2
            })
        );
    }

    /// A relation of all zeros says nothing and is dropped rather than refused.
    #[test]
    fn a_zero_relation_is_dropped() {
        let module = ResonanceModule::declare(3, &[vec![0, 0, 0], vec![1, -1, 0]])
            .expect("one relation is left after the zero is dropped");
        assert_eq!(module.dimension(), 1);
        let only = ResonanceModule::declare(3, &[vec![0, 0, 0]]).expect("nothing is declared");
        assert_eq!(only, ResonanceModule::trivial(3).expect("three freedoms"));
    }

    /// The multi-index of a monomial is the first block minus the second.
    #[test]
    fn the_multi_index_is_the_difference_of_the_two_exponent_blocks() {
        let module = ResonanceModule::trivial(3).expect("three freedoms");
        assert_eq!(
            module
                .multi_index_of(&[2, 0, 1, 0, 3, 1])
                .expect("six entries in three freedoms"),
            vec![2, -3, 0]
        );
        assert_eq!(
            module.multi_index_of(&[1, 2]).err(),
            Some(Error::ExponentWidth {
                variables: 6,
                given: 2
            })
        );
    }
}
