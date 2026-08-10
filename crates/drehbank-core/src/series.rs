//! The truncated series and its arithmetic.
//!
//! This is the substrate. Everything above it, the bracket, the Lie transform,
//! the normal form and the estimate, is an operation on these, so an error here
//! is invisible everywhere else and fatal everywhere else.
//!
//! The storage is the one `docs/decisions/0003-series-representation.md`
//! decides: graded by total degree, and each degree a dense array of
//! coefficients over the monomials of that degree in `m = 2v` variables, in the
//! grevlex order of that document, with the exponent vector recovered from the
//! array index rather than stored beside it. The bijection and its tables are
//! [`crate::monomial`].
//!
//! Two things are data rather than convention, which is what issue #29 asks
//! for. The degrees of freedom and the truncation order are carried by the
//! series, so a caller cannot combine two series that disagree about either and
//! get an answer.
//!
//! # A degree that is not stored
//!
//! A degree may be held as an empty array, which means every coefficient of
//! that degree is zero. This is not a second representation with its own rules.
//! It is the same series stored smaller, and the Hamiltonians this package is
//! aimed at are exactly the shape that benefits: an expansion about an
//! equilibrium that carries a quadratic part and a handful of higher degrees
//! and nothing else.
//!
//! What it costs is that equality cannot be the derived one, because the
//! derived one would call an unstored degree different from a stored degree of
//! zeros. So [`PartialEq`] is written rather than derived, and it compares the
//! coefficients a caller can read rather than the arrays that hold them.

use crate::coefficient::Coefficient;
use crate::error::Error;
use crate::monomial::{
    DegreeTable, IndexError, Scratch, exponents_into, index_of, product_index_with,
};

/// The truncation order a bracket of these two orders carries.
///
/// Not the order of either argument, which is the bookkeeping issue #30 names:
/// a driver that assumes the arguments' order loses the top of every bracket it
/// takes, silently, because the terms it drops are the ones the arguments were
/// carried to that order for.
///
/// It is derived rather than chosen. The bracket of item 3 of 0004 is a sum of
/// products of one derivative of the left with one derivative of the right, a
/// derivative lowers the order by one, and a product carries the sum of the two
/// orders. So the answer is `(left - 1) + (right - 1)`, which for arguments of
/// order two and above is the `d + e - 2` grading item 3 states.
///
/// The subtractions saturate rather than wrap, and what that covers is an
/// argument of order zero, which is a constant. Its derivative is zero, so the
/// bracket is the zero series whatever the other argument is, and the order
/// stated here is then higher than the grading rule would give. A zero series
/// carried to a higher order is still zero at every degree, so the statement is
/// true rather than merely safe.
///
/// # What this order is a statement about
///
/// It is the highest degree the two arguments *as they stand* determine. Where
/// they are themselves truncations of longer series, the top degrees of the
/// bracket are missing the contributions of the terms that were truncated away,
/// and the caller is the one who knows that. [`Series::truncated`] is how the
/// caller says what it can stand behind, for the reason
/// [`Series::add_in_place`] gives: taking the smaller order here would be the
/// arithmetic deciding what the caller meant.
pub fn bracket_order(left: u32, right: u32) -> u32 {
    left.saturating_sub(1)
        .saturating_add(right.saturating_sub(1))
}

/// Which way a convolution goes into its destination.
///
/// The two terms of item 3 of 0004 differ only in this, and a boolean argument
/// at the call site would read as `true` and `false` where the thing that
/// matters is the minus sign the whole antisymmetry of the bracket rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Sign {
    Add,
    Subtract,
}

/// The exponent brought down by a derivative, as a coefficient.
///
/// Separate from the loop that uses it so the refusal has somewhere to be
/// tested from. See [`Error::MultiplierBeyondCoefficient`] for why an exponent
/// that does not fit is refused rather than cast.
fn derivative_multiplier<C: Coefficient>(exponent: u32) -> Result<C, Error> {
    let multiplier =
        i32::try_from(exponent).map_err(|_| Error::MultiplierBeyondCoefficient { exponent })?;
    Ok(C::from_small_integer(multiplier))
}

/// A polynomial series in `2v` variables, truncated at a total degree.
///
/// `v` is the number of degrees of freedom and the variable order is the one
/// item 1 of `docs/decisions/0004-conventions.md` fixes: positions first, so
/// the first `v` variables are `q_1 .. q_v` and the last `v` are `p_1 .. p_v`.
#[derive(Debug, Clone)]
pub struct Series<C> {
    freedoms: usize,
    order: u32,
    table: DegreeTable,
    /// One entry per degree from 0 to `order`. An entry is either empty, which
    /// means the degree is all zero, or exactly `M(degree, 2v)` coefficients.
    degrees: Vec<Vec<C>>,
}

impl<C: Coefficient> Series<C> {
    /// The zero series.
    ///
    /// Nothing is allocated per degree, because the zero series is the case
    /// where every degree is unstored. A series is grown by writing into it.
    pub fn zero(freedoms: usize, order: u32) -> Result<Self, Error> {
        if freedoms == 0 {
            return Err(Error::NoFreedoms);
        }
        let variables = freedoms.checked_mul(2).ok_or(Error::NoFreedoms)?;
        // The table refuses an order the index arithmetic cannot address, and
        // that boundary is far below `u32::MAX`, so the length below cannot
        // wrap once the table has been built.
        let table = DegreeTable::new(variables, order)?;
        Ok(Series {
            freedoms,
            order,
            table,
            degrees: vec![Vec::new(); order as usize + 1],
        })
    }

    /// The series that is the constant one.
    ///
    /// The multiplicative identity of the arithmetic below, and the starting
    /// point of every exponential the Lie transforms will build.
    pub fn unit(freedoms: usize, order: u32) -> Result<Self, Error> {
        let mut series = Series::zero(freedoms, order)?;
        series.set_coefficient(0, 0, C::one())?;
        Ok(series)
    }

    /// The number of degrees of freedom, `v`.
    pub fn freedoms(&self) -> usize {
        self.freedoms
    }

    /// The number of variables, `2v`.
    pub fn variables(&self) -> usize {
        self.table.variables()
    }

    /// The total degree above which this series carries nothing.
    pub fn order(&self) -> u32 {
        self.order
    }

    /// The number of monomials at a degree this series carries.
    pub fn dimension(&self, degree: u32) -> Result<u64, Error> {
        self.check_degree(degree)?;
        Ok(self.table.dimension(degree)?)
    }

    /// The coefficient of one monomial.
    ///
    /// The degree and the index inside it, in the order of 0003. A degree that
    /// is not stored answers zero rather than refusing, because whether a
    /// degree is stored is a fact about the storage and not about the series.
    pub fn coefficient(&self, degree: u32, index: u64) -> Result<C, Error> {
        self.check_index(degree, index)?;
        let stored = &self.degrees[degree as usize];
        if stored.is_empty() {
            return Ok(C::zero());
        }
        Ok(stored[index as usize].clone())
    }

    /// Write the coefficient of one monomial.
    pub fn set_coefficient(&mut self, degree: u32, index: u64, value: C) -> Result<(), Error> {
        self.check_index(degree, index)?;
        self.store(degree)?;
        self.degrees[degree as usize][index as usize] = value;
        Ok(())
    }

    /// Add another series to this one.
    ///
    /// Both arguments have to agree about the degrees of freedom and about the
    /// truncation order. The second is not pedantry: the sum of a series known
    /// to order five and one known to order three is known to order three, and
    /// taking the smaller silently would be the arithmetic deciding what the
    /// caller meant. [`Series::truncated`] is how the caller says it instead.
    pub fn add_in_place(&mut self, other: &Self) -> Result<(), Error> {
        self.check_combinable(other)?;
        for degree in 0..=self.order {
            if other.degrees[degree as usize].is_empty() {
                continue;
            }
            self.store(degree)?;
            for (slot, addend) in self.degrees[degree as usize]
                .iter_mut()
                .zip(other.degrees[degree as usize].iter())
            {
                *slot = slot.add(addend);
            }
        }
        Ok(())
    }

    /// Subtract another series from this one.
    pub fn subtract_in_place(&mut self, other: &Self) -> Result<(), Error> {
        self.check_combinable(other)?;
        for degree in 0..=self.order {
            if other.degrees[degree as usize].is_empty() {
                continue;
            }
            self.store(degree)?;
            for (slot, subtrahend) in self.degrees[degree as usize]
                .iter_mut()
                .zip(other.degrees[degree as usize].iter())
            {
                *slot = slot.subtract(subtrahend);
            }
        }
        Ok(())
    }

    /// The sum of two series.
    pub fn sum(&self, other: &Self) -> Result<Self, Error> {
        let mut result = self.clone();
        result.add_in_place(other)?;
        Ok(result)
    }

    /// The difference of two series.
    pub fn difference(&self, other: &Self) -> Result<Self, Error> {
        let mut result = self.clone();
        result.subtract_in_place(other)?;
        Ok(result)
    }

    /// Scale this series by a coefficient.
    ///
    /// An unstored degree stays unstored, because a multiple of zero is zero
    /// whatever the factor is, including a factor that is itself zero.
    pub fn scale_in_place(&mut self, factor: &C) {
        for stored in &mut self.degrees {
            for slot in stored.iter_mut() {
                *slot = slot.multiply(factor);
            }
        }
    }

    /// This series scaled by a coefficient.
    pub fn scaled(&self, factor: &C) -> Self {
        let mut result = self.clone();
        result.scale_in_place(factor);
        result
    }

    /// The negation of this series.
    pub fn negated(&self) -> Self {
        let mut result = self.clone();
        for stored in &mut result.degrees {
            for slot in stored.iter_mut() {
                *slot = slot.negate();
            }
        }
        result
    }

    /// The product of two series, truncated at the order both carry.
    ///
    /// The convolution of 0003: for every pair of degrees whose sum is within
    /// the truncation order, a walk over two contiguous arrays writing into a
    /// third, with the destination index computed by the index arithmetic
    /// rather than found by a search. Pairs whose degrees sum above the order
    /// are never visited, which is what makes truncation free here rather than
    /// a filter applied afterwards.
    pub fn product(&self, other: &Self) -> Result<Self, Error> {
        self.check_combinable(other)?;
        let mut scratch = Scratch::new(self.variables())?;
        let mut result: Series<C> = Series::zero(self.freedoms, self.order)?;
        for left_degree in 0..=self.order {
            if self.degrees[left_degree as usize].is_empty() {
                continue;
            }
            for right_degree in 0..=(self.order - left_degree) {
                if other.degrees[right_degree as usize].is_empty() {
                    continue;
                }
                let sum_degree = left_degree + right_degree;
                result.store(sum_degree)?;
                let left = &self.degrees[left_degree as usize];
                let right = &other.degrees[right_degree as usize];
                let destination = &mut result.degrees[sum_degree as usize];
                for (left_index, left_value) in left.iter().enumerate() {
                    for (right_index, right_value) in right.iter().enumerate() {
                        let target = product_index_with(
                            left_index as u64,
                            left_degree,
                            right_index as u64,
                            right_degree,
                            &mut scratch,
                        )?;
                        let raised =
                            destination[target as usize].add(&left_value.multiply(right_value));
                        destination[target as usize] = raised;
                    }
                }
            }
        }
        Ok(result)
    }

    /// The partial derivative of this series with respect to one variable.
    ///
    /// The variables are numbered from zero in the order item 1 of 0004 fixes,
    /// so `0 .. v` are `q_1 .. q_v` and `v .. 2v` are `p_1 .. p_v`. The
    /// conjugate partner of variable `j` is `j + v` and never `j + 1`.
    ///
    /// A derivative lowers the degree by one, so the result carries one order
    /// less than this series and the order zero case is the zero series. It is
    /// a scatter with a multiplier, exactly as the product is: each term is
    /// written to the index of its own exponent vector with one exponent taken
    /// down, and that index is computed rather than searched for.
    pub fn derivative(&self, variable: usize) -> Result<Self, Error> {
        let variables = self.variables();
        if variable >= variables {
            return Err(Error::VariableBeyondPhaseSpace {
                variables,
                given: variable,
            });
        }
        let mut result = Series::zero(self.freedoms, self.order.saturating_sub(1))?;
        let mut exponents = vec![0u32; variables];
        let mut shifted = vec![0u64; variables - 1];
        // Degree zero contributes nothing, and at order zero this range is
        // empty, which is the zero series the documentation promises.
        for degree in 1..=self.order {
            if self.degrees[degree as usize].is_empty() {
                continue;
            }
            for index in 0..self.degrees[degree as usize].len() {
                exponents_into(index as u64, degree, &mut exponents, &mut shifted)?;
                let exponent = exponents[variable];
                if exponent == 0 {
                    continue;
                }
                let multiplier: C = derivative_multiplier(exponent)?;
                exponents[variable] = exponent - 1;
                let target = index_of(&exponents, degree - 1)?;
                let value = self.degrees[degree as usize][index].multiply(&multiplier);
                result.store(degree - 1)?;
                let slot: &mut C = &mut result.degrees[(degree - 1) as usize][target as usize];
                *slot = slot.add(&value);
            }
        }
        Ok(result)
    }

    /// The Poisson bracket of two series, in the sign convention of item 3 of
    /// 0004.
    ///
    /// ```text
    /// {f, g} = sum over j = 1..v of ( df/dq_j * dg/dp_j - df/dp_j * dg/dq_j )
    /// ```
    ///
    /// Under this sign the evolution of a function is `df/dt = {f, H}`, the
    /// bracket is antisymmetric, and the bracket of a piece of degree `d` with
    /// one of degree `e` is homogeneous of degree `d + e - 2`.
    ///
    /// # The order of the answer
    ///
    /// [`bracket_order`] is that order and states where it comes from. Unlike
    /// [`Series::product`], this operation does not require the two arguments
    /// to carry the same truncation order, because it does not have to choose
    /// between them: each argument contributes its own derivative and the
    /// answer is known to the sum of the two derivative orders. Both arguments
    /// still have to be in the same phase space, and that is refused with both
    /// sides named.
    pub fn bracket(&self, other: &Self) -> Result<Self, Error> {
        if self.freedoms != other.freedoms {
            return Err(Error::FreedomsDiffer {
                left: self.freedoms,
                right: other.freedoms,
            });
        }
        let freedoms = self.freedoms;
        let mut result = Series::zero(freedoms, bracket_order(self.order, other.order))?;
        let mut scratch = Scratch::new(self.variables())?;
        for position in 0..freedoms {
            let momentum = position + freedoms;
            let left_position = self.derivative(position)?;
            let right_momentum = other.derivative(momentum)?;
            result.accumulate(&left_position, &right_momentum, Sign::Add, &mut scratch)?;
            let left_momentum = self.derivative(momentum)?;
            let right_position = other.derivative(position)?;
            result.accumulate(
                &left_momentum,
                &right_position,
                Sign::Subtract,
                &mut scratch,
            )?;
        }
        Ok(result)
    }

    /// Add or subtract the product of two series into this one.
    ///
    /// The convolution of [`Series::product`] with two differences: it
    /// accumulates into a destination that already holds something, and the
    /// destination's order is the sum of the two arguments' orders rather than
    /// the order they share. Every pair of degrees therefore lands inside the
    /// destination and none is skipped, which is what makes the caller above
    /// the only place the bracket's truncation is decided.
    ///
    /// Private, and its callers keep that invariant: [`Series::bracket`] sizes
    /// the destination with [`bracket_order`] off the same two orders these
    /// arguments were differentiated from.
    fn accumulate(
        &mut self,
        left: &Self,
        right: &Self,
        sign: Sign,
        scratch: &mut Scratch,
    ) -> Result<(), Error> {
        for left_degree in 0..=left.order {
            if left.degrees[left_degree as usize].is_empty() {
                continue;
            }
            for right_degree in 0..=right.order {
                if right.degrees[right_degree as usize].is_empty() {
                    continue;
                }
                let sum_degree = left_degree + right_degree;
                self.store(sum_degree)?;
                let left_values = &left.degrees[left_degree as usize];
                let right_values = &right.degrees[right_degree as usize];
                let destination = &mut self.degrees[sum_degree as usize];
                for (left_index, left_value) in left_values.iter().enumerate() {
                    for (right_index, right_value) in right_values.iter().enumerate() {
                        let target = product_index_with(
                            left_index as u64,
                            left_degree,
                            right_index as u64,
                            right_degree,
                            scratch,
                        )?;
                        let term = left_value.multiply(right_value);
                        let slot = &mut destination[target as usize];
                        *slot = match sign {
                            Sign::Add => slot.add(&term),
                            Sign::Subtract => slot.subtract(&term),
                        };
                    }
                }
            }
        }
        Ok(())
    }

    /// This series truncated to a lower order.
    ///
    /// Dropping the tail of the degree vector, which is what 0003 buys with the
    /// graded layout. Truncating to the order the series already carries is the
    /// identity; truncating to a higher one is refused, because the degrees
    /// above the order were never computed and filling them with zeros would
    /// claim they were.
    pub fn truncated(&self, order: u32) -> Result<Self, Error> {
        if order > self.order {
            return Err(Error::OrderAboveTruncation {
                requested: order,
                order: self.order,
            });
        }
        let mut degrees = self.degrees.clone();
        degrees.truncate(order as usize + 1);
        Ok(Series {
            freedoms: self.freedoms,
            order,
            table: DegreeTable::new(self.variables(), order)?,
            degrees,
        })
    }

    /// The value of this series at a point.
    ///
    /// The point carries one entry per variable, in the variable order of 0004.
    /// Written plainly and term by term: the falsifier of the remainder
    /// milestone is the only caller, and a fast evaluation would be a second
    /// numerical path with nothing to check it against.
    pub fn evaluate(&self, point: &[C]) -> Result<C, Error> {
        let variables = self.variables();
        if point.len() != variables {
            return Err(Error::PointWidth {
                variables,
                given: point.len(),
            });
        }
        let mut exponents = vec![0u32; variables];
        let mut shifted = vec![0u64; variables - 1];
        let mut total = C::zero();
        for degree in 0..=self.order {
            let stored = &self.degrees[degree as usize];
            if stored.is_empty() {
                continue;
            }
            for (index, value) in stored.iter().enumerate() {
                exponents_into(index as u64, degree, &mut exponents, &mut shifted)?;
                let mut term = value.clone();
                for (variable, &exponent) in exponents.iter().enumerate() {
                    for _ in 0..exponent {
                        term = term.multiply(&point[variable]);
                    }
                }
                total = total.add(&term);
            }
        }
        Ok(total)
    }

    /// The coefficients of one degree, or an empty slice where the degree is
    /// unstored or above the truncation.
    ///
    /// For [`crate::parallel`], which reads the operand arrays directly rather
    /// than through [`Series::coefficient`] because it walks whole degrees at a
    /// time. Empty for a degree above the order rather than a refusal, because
    /// the caller's own loop bounds already exclude those and a second refusal
    /// there would be an unreachable branch nothing could prove bites.
    pub(crate) fn degree_slice(&self, degree: u32) -> &[C] {
        self.degrees.get(degree as usize).map_or(&[], Vec::as_slice)
    }

    /// [`Series::store`], for [`crate::parallel`].
    pub(crate) fn store_degree(&mut self, degree: u32) -> Result<(), Error> {
        self.store(degree)
    }

    /// One degree's coefficients, to be written into.
    ///
    /// For [`crate::parallel`], which partitions this slice into chunks and
    /// gives each chunk to one thread. Empty under the same condition as
    /// [`Series::degree_slice`] and for the same reason.
    pub(crate) fn degree_mut(&mut self, degree: u32) -> &mut [C] {
        self.degrees
            .get_mut(degree as usize)
            .map_or(&mut [], Vec::as_mut_slice)
    }

    /// Materialise a degree, if it is not stored already.
    fn store(&mut self, degree: u32) -> Result<(), Error> {
        if !self.degrees[degree as usize].is_empty() {
            return Ok(());
        }
        let count = self.width(degree)?;
        self.degrees[degree as usize] = vec![C::zero(); count];
        Ok(())
    }

    /// The number of coefficients one degree holds, as a length.
    fn width(&self, degree: u32) -> Result<usize, Error> {
        let dimension = self.table.dimension(degree)?;
        usize::try_from(dimension).map_err(|_| Error::SizeBeyondAddressable {
            variables: self.variables(),
            degree,
            dimension,
        })
    }

    fn check_degree(&self, degree: u32) -> Result<(), Error> {
        if degree > self.order {
            return Err(Error::DegreeAboveTruncation {
                degree,
                order: self.order,
            });
        }
        Ok(())
    }

    fn check_index(&self, degree: u32, index: u64) -> Result<(), Error> {
        self.check_degree(degree)?;
        let dimension = self.table.dimension(degree)?;
        if index >= dimension {
            return Err(Error::Index(IndexError::IndexBeyondDimension {
                variables: self.variables(),
                degree,
                index,
                dimension,
            }));
        }
        Ok(())
    }

    /// The guard every binary operation runs first.
    ///
    /// Two series in different phase spaces, or carrying different truncation
    /// orders, are refused with both sides named. The alternative is an
    /// arithmetic that reads one series with the other's index tables, which
    /// returns a number rather than an error and is the failure this package is
    /// least able to notice afterwards.
    pub(crate) fn check_combinable(&self, other: &Self) -> Result<(), Error> {
        if self.freedoms != other.freedoms {
            return Err(Error::FreedomsDiffer {
                left: self.freedoms,
                right: other.freedoms,
            });
        }
        if self.order != other.order {
            return Err(Error::OrderDiffers {
                left: self.order,
                right: other.order,
            });
        }
        Ok(())
    }
}

/// Equality over the coefficients rather than over the arrays that hold them.
///
/// A degree held as an empty array and a degree held as an array of zeros are
/// the same series, so the derived implementation would be wrong. It would also
/// be wrong in a way that is hard to see: two series that a caller cannot tell
/// apart through any accessor would compare unequal, and every property test
/// over the arithmetic would then be testing the storage.
impl<C: Coefficient + PartialEq> PartialEq for Series<C> {
    fn eq(&self, other: &Self) -> bool {
        if self.freedoms != other.freedoms || self.order != other.order {
            return false;
        }
        self.degrees
            .iter()
            .zip(other.degrees.iter())
            .all(|(left, right)| match (left.is_empty(), right.is_empty()) {
                (true, true) => true,
                (false, false) => left == right,
                (true, false) => right.iter().all(|value| *value == C::zero()),
                (false, true) => left.iter().all(|value| *value == C::zero()),
            })
    }
}

impl<C: Coefficient + Eq> Eq for Series<C> {}

#[cfg(test)]
mod tests {
    use super::{Series, bracket_order, derivative_multiplier};
    use crate::error::Error;

    /// The refusal that keeps two phase spaces apart names both of them, on
    /// every operation that takes two series.
    ///
    /// Every one of them rather than a representative, because the guard is per
    /// call site and a new operation that forgets it is exactly the failure
    /// this is here for. Delete the `freedoms` arm of `check_combinable` and
    /// the first five go red, which is the proof the guard bites rather than a
    /// sentence saying it would.
    ///
    /// The bracket is the sixth and it is the reason this test is worth
    /// keeping. It does not go through `check_combinable`, because it is the
    /// one binary operation that accepts two different truncation orders, so
    /// it carries its own copy of the phase space arm and deleting that copy
    /// reds this test on its own.
    #[test]
    fn combining_two_phase_spaces_is_refused_and_names_both() {
        let mismatch = Error::FreedomsDiffer { left: 2, right: 3 };
        let mut left: Series<f64> = Series::zero(2, 4).expect("two freedoms is addressable");
        let right: Series<f64> = Series::zero(3, 4).expect("three freedoms is addressable");
        assert_eq!(left.sum(&right), Err(mismatch));
        assert_eq!(left.difference(&right), Err(mismatch));
        assert_eq!(left.product(&right), Err(mismatch));
        assert_eq!(left.add_in_place(&right), Err(mismatch));
        assert_eq!(left.subtract_in_place(&right), Err(mismatch));
        assert_eq!(left.bracket(&right), Err(mismatch));
    }

    /// The bracket of the position with the quadratic part is the linear flow,
    /// on a case written out by hand.
    ///
    /// One degree of freedom, so two variables `(q_1, p_1)` in the order of
    /// item 1 of 0004. In two variables the rank of 0003 reduces to
    /// `index = a_1`, so at degree one index 0 is `p_1` and index 1 is `q_1`,
    /// and at degree two index 0 is `p_1^2` and index 2 is `q_1^2`.
    ///
    /// Item 6 of 0004 normalises the quadratic part as
    /// `H_2 = (omega/2)(q_1^2 + p_1^2)`, so at `omega = 2` it is
    /// `q_1^2 + p_1^2` with no fraction to represent. Item 3's bracket then
    /// gives
    ///
    ///     {q_1, H_2} = 1 * 2 p_1 - 0 = 2 p_1        {p_1, H_2} = 0 - 1 * 2 q_1 = -2 q_1
    ///
    /// which is `dq/dt = omega p` and `dp/dt = -omega q`, the equations of
    /// motion of item 2 and the rotation of item 6. A package that took the
    /// opposite bracket sign would return both with the signs exchanged, and a
    /// package that halved the quadratic part would return half of each.
    #[test]
    fn the_bracket_with_the_quadratic_part_is_the_linear_flow() {
        let mut quadratic: Series<f64> = Series::zero(1, 2).expect("order two is addressable");
        quadratic
            .set_coefficient(2, 2, 1.0)
            .expect("degree two holds q_1^2 at index two");
        quadratic
            .set_coefficient(2, 0, 1.0)
            .expect("degree two holds p_1^2 at index zero");

        let mut position: Series<f64> = Series::zero(1, 1).expect("order one is addressable");
        position
            .set_coefficient(1, 1, 1.0)
            .expect("degree one holds q_1 at index one");
        let mut momentum: Series<f64> = Series::zero(1, 1).expect("order one is addressable");
        momentum
            .set_coefficient(1, 0, 1.0)
            .expect("degree one holds p_1 at index zero");

        let mut flow_of_position: Series<f64> =
            Series::zero(1, 1).expect("order one is addressable");
        flow_of_position
            .set_coefficient(1, 0, 2.0)
            .expect("degree one holds p_1 at index zero");
        let mut flow_of_momentum: Series<f64> =
            Series::zero(1, 1).expect("order one is addressable");
        flow_of_momentum
            .set_coefficient(1, 1, -2.0)
            .expect("degree one holds q_1 at index one");

        assert_eq!(
            position
                .bracket(&quadratic)
                .expect("both are in the same phase space"),
            flow_of_position
        );
        assert_eq!(
            momentum
                .bracket(&quadratic)
                .expect("both are in the same phase space"),
            flow_of_momentum
        );
    }

    /// A derivative with respect to a variable the phase space has not got is
    /// refused, and the refusal names the width.
    ///
    /// Two degrees of freedom is four variables numbered 0 to 3, so 4 is the
    /// first one that is not there and it is what a caller counting from one
    /// reaches for.
    #[test]
    fn a_derivative_outside_the_phase_space_is_refused() {
        let series: Series<f64> = Series::zero(2, 3).expect("order three is addressable");
        assert_eq!(
            series.derivative(4).err(),
            Some(Error::VariableBeyondPhaseSpace {
                variables: 4,
                given: 4
            })
        );
    }

    /// The multiplier a derivative brings down is refused rather than cast when
    /// it does not fit.
    ///
    /// Delete the `try_from` in `derivative_multiplier` and write `exponent as
    /// i32` instead, and this goes red: the cast turns `u32::MAX` into `-1`,
    /// which is a coefficient the arithmetic accepts and the wrong one.
    ///
    /// Tested here rather than through a series, because the series that would
    /// carry such an exponent needs a truncation order above two billion and no
    /// host can allocate one. That is what the second half of this assertion is
    /// for: an exponent an ordinary series does carry passes through.
    #[test]
    fn the_derivative_multiplier_refuses_an_exponent_it_cannot_carry() {
        assert_eq!(
            derivative_multiplier::<f64>(u32::MAX).err(),
            Some(Error::MultiplierBeyondCoefficient { exponent: u32::MAX })
        );
        assert_eq!(derivative_multiplier::<f64>(3).ok(), Some(3.0));
    }

    /// The order of a bracket, against the grading item 3 of 0004 states.
    ///
    /// The first three are `d + e - 2` written out. The last two are the
    /// saturating cases, where one argument is a constant, the bracket is the
    /// zero series and the order stated is above what the grading would give.
    #[test]
    fn a_bracket_carries_the_order_the_grading_gives() {
        assert_eq!(bracket_order(4, 4), 6);
        assert_eq!(bracket_order(2, 3), 3);
        assert_eq!(bracket_order(1, 1), 0);
        assert_eq!(bracket_order(0, 5), 4);
        assert_eq!(bracket_order(0, 0), 0);
    }

    /// The derivative of a constant is the zero series, and it says so at order
    /// zero rather than refusing.
    #[test]
    fn the_derivative_of_a_constant_is_zero() {
        let constant: Series<f64> = Series::unit(2, 0).expect("order zero is addressable");
        let derivative = constant.derivative(0).expect("variable zero is in range");
        assert_eq!(derivative.order(), 0);
        assert_eq!(
            derivative,
            Series::zero(2, 0).expect("order zero is addressable")
        );
    }

    /// A series evaluated at a point, against a value computed by hand.
    ///
    /// One degree of freedom, so two variables `(q_1, p_1)` in the order item 1
    /// of 0004 fixes, and order two. In two variables the rank of 0003 reduces
    /// to `index = C(a_1, 1) = a_1`, so at degree one index 0 is `p_1` and
    /// index 1 is `q_1`, and at degree two index 2 is `q_1^2`.
    ///
    /// The series is `3 p_1 + 5 q_1^2` and the point is `q_1 = 2, p_1 = 7`, so
    /// the value is `3*7 + 5*4 = 41`. Written out because it pins the variable
    /// order and the monomial order together: a package that got either wrong
    /// would evaluate this to 3*2 + 5*49, which is 251.
    #[test]
    fn a_series_evaluates_to_the_value_computed_by_hand() {
        let mut series: Series<f64> = Series::zero(1, 2).expect("order two is addressable");
        series
            .set_coefficient(1, 0, 3.0)
            .expect("degree one has two monomials");
        series
            .set_coefficient(2, 2, 5.0)
            .expect("degree two has three monomials");
        assert_eq!(
            series
                .evaluate(&[2.0, 7.0])
                .expect("the point has one entry per variable"),
            41.0
        );
    }

    /// An evaluation point of the wrong width is refused rather than padded.
    #[test]
    fn an_evaluation_point_of_the_wrong_width_is_refused() {
        let series: Series<f64> = Series::zero(2, 2).expect("order two is addressable");
        assert_eq!(
            series.evaluate(&[1.0, 1.0]).err(),
            Some(Error::PointWidth {
                variables: 4,
                given: 2
            })
        );
    }

    /// The same for the truncation order.
    #[test]
    fn combining_two_truncation_orders_is_refused_and_names_both() {
        let left: Series<f64> = Series::zero(2, 4).expect("order four is addressable");
        let right: Series<f64> = Series::zero(2, 3).expect("order three is addressable");
        assert_eq!(
            left.sum(&right),
            Err(Error::OrderDiffers { left: 4, right: 3 })
        );
    }

    /// Truncation drops a tail and never invents one.
    #[test]
    fn truncating_upward_is_refused() {
        let series: Series<f64> = Series::zero(1, 2).expect("order two is addressable");
        assert_eq!(
            series.truncated(5).err(),
            Some(Error::OrderAboveTruncation {
                requested: 5,
                order: 2
            })
        );
    }

    /// A series with no degrees of freedom has no variables to be a series in.
    #[test]
    fn a_series_needs_a_degree_of_freedom() {
        assert_eq!(Series::<f64>::zero(0, 2).err(), Some(Error::NoFreedoms));
    }

    /// An unstored degree and a stored degree of zeros are the same series.
    ///
    /// Written against the accessors a caller has, so it is the claim the
    /// module documentation makes rather than a restatement of the
    /// implementation.
    #[test]
    fn an_unstored_degree_is_a_degree_of_zeros() {
        let unstored: Series<f64> = Series::zero(2, 3).expect("order three is addressable");
        let mut stored = unstored.clone();
        stored
            .set_coefficient(2, 0, 0.0)
            .expect("degree two has a monomial at index zero");
        assert_eq!(unstored, stored);
        assert_eq!(
            unstored
                .coefficient(2, 0)
                .expect("degree two has a monomial at index zero"),
            stored
                .coefficient(2, 0)
                .expect("degree two has a monomial at index zero")
        );
    }
}
