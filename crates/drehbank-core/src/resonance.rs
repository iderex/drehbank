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
//! # Detection is a different operation and it decides nothing
//!
//! [`Proposal::detect`] reads a frequency vector, a tolerance and an order
//! bound, and returns every multi-index whose relative divisor is under the
//! tolerance. It is **advisory**. It does not produce a module, nothing in this
//! package turns its answer into the module in force, and there is no path from
//! a [`Proposal`] to a [`ResonanceModule`] except [`ResonanceModule::accept`],
//! which a caller has to write.
//!
//! What accepting commits a caller to, in plain terms. The relations the
//! proposal found are treated as **exact** from then on: every monomial whose
//! multi-index lies in the lattice they generate is kept in the normal form
//! instead of being removed, and no divisor is computed for it. That is the
//! right thing when the near resonance is near enough that dividing by its
//! divisor would destroy the result, and it is the wrong thing when the
//! resonance is far enough away that the ordinary path would have handled it,
//! because it keeps terms the normal form could have removed. Nothing here
//! decides which of the two a case is. What it does instead is report the
//! divisor, the order and, through [`Proposal::affected_magnitudes`], how large
//! a coefficient the relation would actually reach, so that the caller decides
//! against numbers rather than against a tolerance somebody picked.
//!
//! An acceptance is recorded. A module that came from a proposal carries
//! [`Provenance::Accepted`] with the tolerance and the order bound it came
//! from, and one that was declared carries [`Provenance::Declared`], so a
//! result built on either says which it was.

use crate::error::Error;
use crate::monomial::{dimension, exponents_into};
use crate::series::Series;

/// A resonance lattice in `Z^v`, held as its canonical basis.
///
/// Constructed from declared relations and never from a tolerance. Two
/// declarations that generate lattices with the same saturation produce the
/// same value here, entry for entry, which is the property the whole design
/// rests on.
#[derive(Debug, Clone)]
pub struct ResonanceModule {
    freedoms: usize,
    /// The canonical basis, in the Hermite normal form of 0007: rows in
    /// echelon order by leading column, each pivot positive, every entry above
    /// a pivot reduced into `[0, pivot)`. Empty for the trivial module.
    basis: Vec<Vec<i64>>,
    provenance: Provenance,
}

/// Where a module came from, which 0007 requires a result to record.
///
/// Not a label a caller sets. It is fixed by which constructor built the
/// module, so there is no way to produce one that came from a detection and
/// says it was declared.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Provenance {
    /// The caller wrote the relations down.
    Declared,

    /// The caller accepted a proposal, with the tolerance and the order bound
    /// it came from.
    ///
    /// Both numbers travel because neither is recoverable from the basis: the
    /// same lattice comes out of many different pairs, and a reader asking how
    /// near the relations actually were is asking about the tolerance rather
    /// than about the lattice.
    Accepted {
        /// The tolerance on the relative divisor of 0006.
        tolerance: f64,
        /// The bound on `|k|_1` the search ran to.
        order_bound: u32,
    },
}

/// Equality is over the lattice and the phase space, and not over the
/// provenance.
///
/// The property 0007 is built on is that canonicalisation is a function of the
/// lattice, so two declarations of the same physics have to compare equal. A
/// derived implementation would additionally compare the provenance, which
/// would make the same lattice unequal to itself depending on how a caller
/// arrived at it, and the provenance is a fact about the arrival rather than
/// about the module. It travels with the module and is read through
/// [`ResonanceModule::provenance`] instead.
///
/// It is also what keeps this type [`Eq`]: [`Provenance::Accepted`] carries a
/// binary64, which has no total equality.
impl PartialEq for ResonanceModule {
    fn eq(&self, other: &Self) -> bool {
        self.freedoms == other.freedoms && self.basis == other.basis
    }
}

impl Eq for ResonanceModule {}

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
            provenance: Provenance::Declared,
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
        // Ahead of the width loop, and it is not interchangeable with the same
        // check inside `from_relations`. A caller asking for no phase space and
        // handing over a relation of one entry has made one mistake, and the
        // refusal that names it is this one; the other order answers that the
        // relation is the wrong width for a phase space that does not exist.
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
        ResonanceModule::from_relations(freedoms, relations, Provenance::Declared)
    }

    /// The module a proposal's relations generate, with the acceptance
    /// recorded.
    ///
    /// This is the only way a [`Proposal`] becomes a module, and it is a call a
    /// caller has to write. Nothing in the package makes it, and the module
    /// that comes out carries [`Provenance::Accepted`] with the tolerance and
    /// the order bound, so a result built on it cannot be read as one built on
    /// relations somebody wrote down.
    ///
    /// A proposal that found nothing accepts to the trivial module, which is
    /// the honest answer to "I looked at this tolerance and there was nothing
    /// there", and it still records that the look happened.
    pub fn accept(proposal: &Proposal) -> Result<Self, Error> {
        let relations: Vec<Vec<i64>> = proposal
            .relations()
            .iter()
            .map(|found| found.relation().to_vec())
            .collect();
        ResonanceModule::from_relations(
            proposal.freedoms(),
            &relations,
            Provenance::Accepted {
                tolerance: proposal.tolerance(),
                order_bound: proposal.order_bound(),
            },
        )
    }

    /// The canonicalisation both public constructors run, with the provenance
    /// each of them supplies.
    ///
    /// Its callers guarantee a phase space: [`ResonanceModule::declare`]
    /// refuses one of no degrees of freedom above, and
    /// [`ResonanceModule::accept`] takes its width from a [`Proposal`], which
    /// [`Proposal::detect`] refuses to build without one. There is no check for
    /// it here, because a third one that no input can reach is a guard nothing
    /// could prove bites.
    fn from_relations(
        freedoms: usize,
        relations: &[Vec<i64>],
        provenance: Provenance,
    ) -> Result<Self, Error> {
        let declared: Matrix = relations
            .iter()
            .filter(|relation| relation.iter().any(|entry| *entry != 0))
            .map(|relation| relation.iter().map(|entry| i128::from(*entry)).collect())
            .collect();
        if declared.is_empty() {
            return Ok(ResonanceModule {
                freedoms,
                basis: Vec::new(),
                provenance,
            });
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
        Ok(ResonanceModule {
            freedoms,
            basis,
            provenance,
        })
    }

    /// Whether this module was declared or accepted from a detection.
    pub fn provenance(&self) -> Provenance {
        self.provenance
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

/// One near resonance a detection found.
///
/// Everything a caller needs to judge how near it is, except the size of the
/// coefficient it would reach, which needs a Hamiltonian and is
/// [`Proposal::affected_magnitudes`].
#[derive(Debug, Clone, PartialEq)]
pub struct NearResonance {
    relation: Vec<i64>,
    order: u32,
    divisor: f64,
    relative_divisor: f64,
}

impl NearResonance {
    /// The multi-index `k`, in `Z^v`.
    ///
    /// One of `k` and `-k`, never both, because they are the same relation. The
    /// one returned is the one whose first non-zero entry is positive.
    pub fn relation(&self) -> &[i64] {
        &self.relation
    }

    /// `|k|_1`, which is the order the search bound is stated against.
    pub fn order(&self) -> u32 {
        self.order
    }

    /// `<k, omega>`, the divisor of item 8 of 0004.
    ///
    /// The quantity that is actually divided by, carrying its sign, so a caller
    /// can see how near zero it is in the units their frequencies are in.
    pub fn divisor(&self) -> f64 {
        self.divisor
    }

    /// `rho(k)` of 0006, which is what the tolerance is compared against.
    ///
    /// `|<k, omega>| / ( |k|_1 * ||omega||_inf )`, between zero and one and
    /// unchanged by rescaling either the frequencies or the multi-index, so a
    /// tolerance stated against it means the same thing for the same physics in
    /// different units.
    pub fn relative_divisor(&self) -> f64 {
        self.relative_divisor
    }
}

/// The largest number of multi-indices a detection will visit.
///
/// A bound on the allocation rather than on the physics. 0007 tabulates the
/// search size and its largest entry is 67122, at six degrees of freedom and an
/// order bound of ten, so this sits more than an order of magnitude above the
/// widest case that document contemplates. What it refuses is an order bound
/// nobody meant, which at six degrees of freedom first passes this at
/// seventeen:
///
/// ```text
/// $ python -c "from math import comb
/// f=lambda v,n: (sum(2**j*comb(v,j)*comb(n,j) for j in range(min(v,n)+1))-1)//2
/// print([(n, f(6,n)) for n in (10, 16, 17)])"
/// [(10, 67122), (16, 942480), (17, 1334262)]
/// ```
///
/// The refusal is what 0011 asks for in place of an allocation the host cannot
/// serve, and it names the count so a caller can see how far past they are.
pub const DETECTION_CEILING: u64 = 1_000_000;

/// A proposed declaration, which decides nothing.
///
/// Produced by [`Proposal::detect`] and turned into a module only by
/// [`ResonanceModule::accept`]. There is no other conversion, no `From`, and
/// nothing in this package calls that function on a caller's behalf.
#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    freedoms: usize,
    tolerance: f64,
    order_bound: u32,
    found: Vec<NearResonance>,
    basis: Vec<Vec<i64>>,
}

impl Proposal {
    /// Every multi-index under the tolerance, with the lattice they generate.
    ///
    /// The inputs are the ones 0007 fixes and none of them has a default:
    /// `frequencies` is the vector as given, including negative entries by item
    /// 7 of 0004; `tolerance` is on the relative divisor of 0006 rather than on
    /// the divisor itself, so it means the same thing in different units; and
    /// `order_bound` is the bound on `|k|_1`. A default on either of the last
    /// two would be this package deciding the physics.
    ///
    /// The comparison is `rho(k) < tolerance`, strictly, as 0007 writes it and
    /// as 0006 writes its own threshold. A tolerance of zero therefore returns
    /// nothing at all, exact relations included.
    ///
    /// The search visits one of `k` and `-k`, since they are the same relation,
    /// and never `k = 0`.
    pub fn detect(frequencies: &[f64], tolerance: f64, order_bound: u32) -> Result<Self, Error> {
        let freedoms = frequencies.len();
        if freedoms == 0 {
            return Err(Error::NoFreedoms);
        }
        for (at, frequency) in frequencies.iter().enumerate() {
            if !frequency.is_finite() {
                return Err(Error::FrequencyNotFinite { at });
            }
        }
        let largest = frequencies
            .iter()
            .map(|frequency| frequency.abs())
            .fold(0.0f64, f64::max);
        if largest == 0.0 {
            return Err(Error::FrequenciesDegenerate { freedoms });
        }
        if !(tolerance.is_finite() && tolerance >= 0.0) {
            return Err(Error::ToleranceNotInRange);
        }
        let candidates = candidate_count(freedoms, order_bound);
        if candidates > DETECTION_CEILING {
            return Err(Error::DetectionTooLarge {
                candidates,
                ceiling: DETECTION_CEILING,
            });
        }

        let mut relation = vec![0i64; freedoms];
        let mut found = Vec::new();
        walk(
            0,
            order_bound,
            false,
            &mut relation,
            &mut |relation: &[i64]| {
                let order: u32 = relation
                    .iter()
                    .map(|entry| entry.unsigned_abs() as u32)
                    .sum();
                let divisor: f64 = relation
                    .iter()
                    .zip(frequencies.iter())
                    .map(|(entry, frequency)| *entry as f64 * frequency)
                    .sum();
                let relative_divisor = divisor.abs() / (f64::from(order) * largest);
                if relative_divisor < tolerance {
                    found.push(NearResonance {
                        relation: relation.to_vec(),
                        order,
                        divisor,
                        relative_divisor,
                    });
                }
            },
        );

        let generators: Matrix = found
            .iter()
            .map(|near| {
                near.relation
                    .iter()
                    .map(|entry| i128::from(*entry))
                    .collect()
            })
            .collect();
        let basis = if generators.is_empty() {
            Vec::new()
        } else {
            canonical(generators, freedoms)?
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|entry| i64::try_from(entry).map_err(|_| Error::LatticeOverflow))
                        .collect::<Result<Vec<i64>, Error>>()
                })
                .collect::<Result<Vec<Vec<i64>>, Error>>()?
        };

        Ok(Proposal {
            freedoms,
            tolerance,
            order_bound,
            found,
            basis,
        })
    }

    /// The degrees of freedom this proposal lives in.
    pub fn freedoms(&self) -> usize {
        self.freedoms
    }

    /// The tolerance the search was run at.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }

    /// The bound on `|k|_1` the search was run to.
    pub fn order_bound(&self) -> u32 {
        self.order_bound
    }

    /// What was found, in the order the search visited it.
    pub fn relations(&self) -> &[NearResonance] {
        &self.found
    }

    /// The canonical basis of the lattice the relations found generate.
    ///
    /// This is what accepting would put in force, and it can be strictly larger
    /// than the list above for the saturation reason at the top of this module.
    /// A caller who wants to see what they are agreeing to reads this rather
    /// than the list.
    pub fn basis(&self) -> &[Vec<i64>] {
        &self.basis
    }

    /// The largest coefficient magnitude the Hamiltonian carries on the
    /// monomials each relation reaches, one per relation, in the order of
    /// [`Proposal::relations`].
    ///
    /// A near resonance whose coefficient is zero is not a problem, and a
    /// package that reports the relation without this sends a caller chasing
    /// nothing. The monomials a relation `k` reaches are those whose
    /// multi-index `a - b` is a non-zero integer multiple of `k`, which is
    /// exactly the set whose divisors are the multiples of `<k, omega>`.
    ///
    /// A relation that reaches no term the Hamiltonian carries answers zero,
    /// which is the useful answer rather than an absence.
    ///
    /// Binary64 only, and that is 0002 rather than a shortcut: the coefficient
    /// abstraction carries no ordering, because a comparison on an interval is
    /// partial and a shared total order would have to lie for one of the three
    /// types. A magnitude lives on the type that has one.
    pub fn affected_magnitudes(&self, hamiltonian: &Series<f64>) -> Result<Vec<f64>, Error> {
        if hamiltonian.freedoms() != self.freedoms {
            return Err(Error::FreedomsDiffer {
                left: self.freedoms,
                right: hamiltonian.freedoms(),
            });
        }
        let variables = 2 * self.freedoms;
        let mut exponents = vec![0u32; variables];
        let mut shifted = vec![0u64; variables - 1];
        let mut multi_index = vec![0i64; self.freedoms];
        let mut largest = vec![0.0f64; self.found.len()];
        for degree in 0..=hamiltonian.order() {
            for index in 0..hamiltonian.dimension(degree)? {
                let coefficient = hamiltonian.coefficient(degree, index)?;
                if coefficient == 0.0 {
                    continue;
                }
                exponents_into(index, degree, &mut exponents, &mut shifted)?;
                let (positions, momenta) = exponents.split_at(self.freedoms);
                for ((slot, first), second) in multi_index
                    .iter_mut()
                    .zip(positions.iter())
                    .zip(momenta.iter())
                {
                    *slot = i64::from(*first) - i64::from(*second);
                }
                for (near, slot) in self.found.iter().zip(largest.iter_mut()) {
                    if is_multiple_of(&multi_index, &near.relation) {
                        *slot = slot.max(coefficient.abs());
                    }
                }
            }
        }
        Ok(largest)
    }
}

/// Whether `vector` is a non-zero integer multiple of `relation`.
///
/// `relation` never carries a zero row, so its first non-zero entry fixes the
/// multiple, and the rest is a check. Zero is refused because the monomials
/// with `a = b` are in the kernel for the reason item 8 of 0004 gives, whatever
/// the frequencies are, and no relation is why they are kept.
fn is_multiple_of(vector: &[i64], relation: &[i64]) -> bool {
    let Some((column, pivot)) = relation
        .iter()
        .enumerate()
        .find(|(_, entry)| **entry != 0)
        .map(|(column, entry)| (column, *entry))
    else {
        return false;
    };
    if vector[column] == 0 || vector[column] % pivot != 0 {
        return false;
    }
    let multiple = vector[column] / pivot;
    vector
        .iter()
        .zip(relation.iter())
        .all(|(entry, generator)| *entry == multiple * generator)
}

/// The multi-indices the search visits, one of `k` and `-k`, `k = 0` excluded.
///
/// The closed form 0007 gives for the integer points of the `L1` ball of radius
/// `bound`, less the origin, halved. Saturating rather than checked, because a
/// count that overflowed is past the ceiling either way and the saturated value
/// is what the refusal should carry.
fn candidate_count(freedoms: usize, bound: u32) -> u64 {
    let mut total: u128 = 0;
    for corner in 0..=(freedoms.min(bound as usize)) {
        let term = (1u128 << corner.min(127))
            .saturating_mul(binomial(freedoms as u128, corner as u128))
            .saturating_mul(binomial(u128::from(bound), corner as u128));
        total = total.saturating_add(term);
    }
    u64::try_from(total.saturating_sub(1) / 2).unwrap_or(u64::MAX)
}

/// `C(n, k)` in `u128`, saturating.
fn binomial(n: u128, k: u128) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result: u128 = 1;
    for step in 0..k {
        result = result.saturating_mul(n - step) / (step + 1);
    }
    result
}

/// Every multi-index with `0 < |k|_1 <= remaining`, one of `k` and `-k`.
///
/// The half is taken by construction rather than by filtering afterwards: until
/// a non-zero entry has been written, the range starts at zero, so the first
/// non-zero entry of everything this emits is positive.
fn walk(
    position: usize,
    remaining: u32,
    seen: bool,
    relation: &mut Vec<i64>,
    visit: &mut impl FnMut(&[i64]),
) {
    if position == relation.len() {
        if seen {
            visit(relation);
        }
        return;
    }
    let span = i64::from(remaining);
    let low = if seen { -span } else { 0 };
    for entry in low..=span {
        relation[position] = entry;
        walk(
            position + 1,
            remaining - entry.unsigned_abs() as u32,
            seen || entry != 0,
            relation,
            visit,
        );
    }
    relation[position] = 0;
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
