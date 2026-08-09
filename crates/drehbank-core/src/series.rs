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
use crate::monomial::{DegreeTable, IndexError, Scratch, exponents_into, product_index_with};

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
    fn check_combinable(&self, other: &Self) -> Result<(), Error> {
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
    use super::Series;
    use crate::error::Error;

    /// The refusal that keeps two phase spaces apart names both of them, on
    /// every operation that takes two series.
    ///
    /// Every one of them rather than a representative, because the guard is per
    /// call site and a new operation that forgets it is exactly the failure
    /// this is here for. Delete the `freedoms` arm of `check_combinable` and
    /// this goes red, which is the proof the guard bites rather than a sentence
    /// saying it would.
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
