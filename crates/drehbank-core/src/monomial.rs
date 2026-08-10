//! The monomial index and the degree tables.
//!
//! Everything else in this crate addresses a coefficient through here. A
//! monomial of total degree `d` in `m` variables is named by an integer in
//! `0 .. M(d, m)`, and the map between that integer and the exponent vector is
//! the bijection written out in `docs/decisions/0003-series-representation.md`.
//! The variable order the exponent vector is written in is item 1 of
//! `docs/decisions/0004-conventions.md`.
//!
//! Nothing here reads a global. Every entry point takes the number of variables
//! and the degree, because a package that carries them in ambient state produces
//! a wrong answer when two series of different sizes are alive at once, and the
//! wrong answer is a plausible number rather than a crash.

use core::fmt;

/// What an entry point refuses, and what it says when it does.
///
/// Refusing is the point. An index computation that wraps reads the wrong
/// coefficient and returns a plausible number, which is the failure this package
/// is least able to detect after the fact, so every boundary is a refusal with
/// the boundary named in it rather than a debug assertion that vanishes in
/// release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexError {
    /// The number of variables was zero. `m = 2v` and `v >= 1`, so `m = 0` is
    /// not a degenerate case to handle, it is a caller mistake.
    NoVariables,
    /// The degree is beyond what this many variables can be indexed at in 64
    /// bits. `maximum` is the largest degree that is accepted.
    DegreeBeyondMaximum {
        variables: usize,
        degree: u32,
        maximum: u32,
    },
    /// The index is not a monomial of that degree.
    IndexBeyondDimension {
        variables: usize,
        degree: u32,
        index: u64,
        dimension: u64,
    },
    /// The exponent vector does not have the total degree it was used at.
    DegreeMismatch { stated: u32, actual: u64 },
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexError::NoVariables => {
                write!(f, "the number of variables is zero")
            }
            IndexError::DegreeBeyondMaximum {
                variables,
                degree,
                maximum,
            } => write!(
                f,
                "degree {degree} is beyond the maximum {maximum} for {variables} variables"
            ),
            IndexError::IndexBeyondDimension {
                variables,
                degree,
                index,
                dimension,
            } => write!(
                f,
                "index {index} is beyond the {dimension} monomials of degree {degree} \
                 in {variables} variables"
            ),
            IndexError::DegreeMismatch { stated, actual } => {
                write!(f, "exponent vector has total degree {actual}, not {stated}")
            }
        }
    }
}

impl core::error::Error for IndexError {}

/// `C(n, k)`, or `None` when it does not fit in 64 bits.
///
/// Multiplicative rather than a Pascal triangle so that no table has to be sized
/// before the bound it is being used to find is known. The division at each step
/// is exact because the running product is `k` consecutive integers.
pub(crate) fn binomial(n: u64, k: u64) -> Option<u64> {
    if k > n {
        return Some(0);
    }
    let k = k.min(n - k);
    let mut result: u64 = 1;
    for i in 0..k {
        result = result.checked_mul(n - i)?;
        // Exact: after i+1 steps the numerator is a product of i+1 consecutive
        // integers, which is divisible by (i+1)!.
        result /= i + 1;
    }
    Some(result)
}

/// `M(d, m) = C(d + m - 1, m - 1)`, the number of monomials of total degree `d`
/// in `m` variables, or `None` when it does not fit in 64 bits.
fn dimension_unchecked(variables: usize, degree: u32) -> Option<u64> {
    let m = variables as u64;
    let d = u64::from(degree);
    binomial(d.checked_add(m)?.checked_sub(1)?, m - 1)
}

/// The largest degree this many variables may be used at.
///
/// Two quantities overflow first and the smaller of the two is the boundary,
/// which is the whole reason this is computed rather than written down. `M(d, m)`
/// is the size of one degree and bounds what can be counted. `M(a, m) * M(b, m)`
/// for `a + b = d` is the pair count of the multiplication inner loop and bounds
/// what can be walked, and it is the smaller bound at every width that matters,
/// so a maximum taken from the dimension alone would accept a degree whose
/// convolution cannot be addressed.
///
/// Computed once per variable count, by walking upward until a checked
/// multiplication refuses, so the number follows the arithmetic rather than a
/// constant somebody has to keep in step with it.
pub fn maximum_degree(variables: usize) -> Result<u32, IndexError> {
    if variables == 0 {
        return Err(IndexError::NoVariables);
    }
    let mut degree: u32 = 0;
    loop {
        let next = degree + 1;
        if !degree_is_addressable(variables, next) {
            return Ok(degree);
        }
        degree = next;
    }
}

/// The ceiling every variable count is held to, whatever its arithmetic allows.
///
/// At few enough variables nothing overflows at any degree a machine could
/// reach, and at `m = 1` every degree has exactly one monomial, so the
/// arithmetic on its own gives no boundary and the search for one would not
/// terminate. This is the boundary in that case. It is refused above, not merely
/// stopped at, so that `maximum_degree` and the entry points agree about one
/// number rather than two.
///
/// It is far above any truncation order this package targets. The plan's largest
/// case is order ten.
pub const MAXIMUM_DEGREE_CAP: u32 = 4096;

/// Whether both the size of degree `d` and every pair count reaching it fit in
/// 64 bits.
///
/// The pair count is checked at the balanced split alone rather than at every
/// split, and that is not a shortcut taken on faith. `M(a, m) / M(a - 1, m)` is
/// `(a + m - 2) / a`, which decreases as `a` grows, so `log M` is concave in the
/// degree, so `log M(a, m) + log M(d - a, m)` is concave in `a` and is largest
/// at `a = d / 2`. A split that overflows therefore implies the balanced one
/// does, and the whole sweep would answer the same question `m` times more
/// slowly on a path every index computation runs through.
fn degree_is_addressable(variables: usize, degree: u32) -> bool {
    if degree > MAXIMUM_DEGREE_CAP {
        return false;
    }
    let half = degree / 2;
    let (Some(_size), Some(left), Some(right)) = (
        dimension_unchecked(variables, degree),
        dimension_unchecked(variables, half),
        dimension_unchecked(variables, degree - half),
    ) else {
        return false;
    };
    left.checked_mul(right).is_some()
}

/// `M(d, m)`, refusing a degree beyond the maximum instead of wrapping.
///
/// The accepted path asks whether this degree is addressable, which is a fixed
/// amount of work. Only the refusal walks up to find the maximum, because that
/// number is wanted for the message and nowhere else.
pub fn dimension(variables: usize, degree: u32) -> Result<u64, IndexError> {
    if variables == 0 {
        return Err(IndexError::NoVariables);
    }
    if !degree_is_addressable(variables, degree) {
        return Err(IndexError::DegreeBeyondMaximum {
            variables,
            degree,
            maximum: maximum_degree(variables)?,
        });
    }
    Ok(dimension_unchecked(variables, degree).expect("the degree is addressable"))
}

/// The dimensions of every degree up to a bound, computed once.
///
/// The inner loop of a multiplication consults `M(d, m)` for every degree it
/// touches, and recomputing a binomial there is a multiply-and-divide chain per
/// lookup for a number that never changes. The table is built once, refuses at
/// construction rather than at use, and is the only place the bound is stated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DegreeTable {
    variables: usize,
    bound: u32,
    dimensions: Vec<u64>,
    cumulative: Vec<u64>,
}

impl DegreeTable {
    /// Build the table for `variables` variables and every degree from 0 to
    /// `bound`.
    ///
    /// Refuses a bound beyond the maximum, and refuses a total that does not fit
    /// in 64 bits. The second one is separate from the first on purpose: every
    /// degree can be addressable while the truncation carrying all of them is
    /// not, and the total is what a caller allocates against.
    pub fn new(variables: usize, bound: u32) -> Result<Self, IndexError> {
        let maximum = maximum_degree(variables)?;
        if bound > maximum {
            return Err(IndexError::DegreeBeyondMaximum {
                variables,
                degree: bound,
                maximum,
            });
        }
        let mut dimensions = Vec::with_capacity(bound as usize + 1);
        let mut cumulative = Vec::with_capacity(bound as usize + 1);
        let mut running: u64 = 0;
        for degree in 0..=bound {
            let size = dimension(variables, degree)?;
            let Some(next) = running.checked_add(size) else {
                return Err(IndexError::DegreeBeyondMaximum {
                    variables,
                    degree,
                    maximum,
                });
            };
            running = next;
            dimensions.push(size);
            cumulative.push(running);
        }
        Ok(DegreeTable {
            variables,
            bound,
            dimensions,
            cumulative,
        })
    }

    /// The number of variables this table was built for.
    pub fn variables(&self) -> usize {
        self.variables
    }

    /// The highest degree in the table.
    pub fn bound(&self) -> u32 {
        self.bound
    }

    /// `M(d, m)`, refusing a degree the table was not built for.
    pub fn dimension(&self, degree: u32) -> Result<u64, IndexError> {
        self.dimensions
            .get(degree as usize)
            .copied()
            .ok_or(IndexError::DegreeBeyondMaximum {
                variables: self.variables,
                degree,
                maximum: self.bound,
            })
    }

    /// The number of monomials of every degree from 0 to `degree` together,
    /// which is `C(degree + m, m)`.
    pub fn cumulative(&self, degree: u32) -> Result<u64, IndexError> {
        self.cumulative
            .get(degree as usize)
            .copied()
            .ok_or(IndexError::DegreeBeyondMaximum {
                variables: self.variables,
                degree,
                maximum: self.bound,
            })
    }
}

/// The index of an exponent vector within its degree.
///
/// `exponents` is written in the variable order of item 1 of 0004: positions
/// first, so `exponents[0]` is the exponent of `q_1` and the last entry is the
/// exponent of `p_v`. The total degree is the sum, and `degree` is passed as
/// well so that a caller who thinks it is working at one degree and hands over a
/// vector from another is refused instead of silently indexed into the wrong
/// array.
pub fn index_of(exponents: &[u32], degree: u32) -> Result<u64, IndexError> {
    let variables = exponents.len();
    if variables == 0 {
        return Err(IndexError::NoVariables);
    }
    if !degree_is_addressable(variables, degree) {
        return Err(IndexError::DegreeBeyondMaximum {
            variables,
            degree,
            maximum: maximum_degree(variables)?,
        });
    }
    let mut total: u64 = 0;
    for &exponent in exponents {
        total = total
            .checked_add(u64::from(exponent))
            .ok_or(IndexError::DegreeMismatch {
                stated: degree,
                actual: u64::MAX,
            })?;
    }
    if total != u64::from(degree) {
        return Err(IndexError::DegreeMismatch {
            stated: degree,
            actual: total,
        });
    }

    // The rank of 0003: partial sums shifted apart into a strictly increasing
    // tuple, then its rank in the combinatorial number system. The last variable
    // contributes nothing, which is why the loop stops one short.
    let mut partial: u64 = 0;
    let mut index: u64 = 0;
    for k in 1..variables {
        partial += u64::from(exponents[k - 1]);
        // Both of these fit whenever the degree is addressable, which was
        // established above: every term is at most `M(d, m)` and the sum is the
        // index, which is less than it. They are written checked anyway, because
        // the alternative to a refusal here is a wrapped index that reads the
        // wrong coefficient.
        let term =
            binomial(partial + k as u64 - 1, k as u64).and_then(|term| index.checked_add(term));
        match term {
            Some(next) => index = next,
            None => {
                return Err(IndexError::DegreeBeyondMaximum {
                    variables,
                    degree,
                    maximum: maximum_degree(variables)?,
                });
            }
        }
    }
    Ok(index)
}

/// The exponent vector of an index within a degree, the inverse of
/// [`index_of`].
///
/// Runs the greedy descent of 0003: peel the largest binomial that still fits at
/// each position, which recovers the strictly increasing tuple, and undo the
/// shift to get the partial sums back.
pub fn exponents_of(index: u64, variables: usize, degree: u32) -> Result<Vec<u32>, IndexError> {
    if variables == 0 {
        return Err(IndexError::NoVariables);
    }
    let mut exponents = vec![0u32; variables];
    let mut shifted = vec![0u64; variables - 1];
    exponents_into(index, degree, &mut exponents, &mut shifted)?;
    Ok(exponents)
}

/// Working space for the index computations, so an inner loop does not allocate
/// per term.
///
/// [`exponents_of`] and [`product_index`] each build one of these and throw it
/// away, which is right for a caller asking one question and wrong for the
/// multiplication kernel, where the allocator would dominate the arithmetic
/// being measured.
#[derive(Debug, Clone)]
pub struct Scratch {
    variables: usize,
    shifted: Vec<u64>,
    left: Vec<u32>,
    right: Vec<u32>,
    sum: Vec<u32>,
}

impl Scratch {
    /// Working space for this many variables.
    pub fn new(variables: usize) -> Result<Self, IndexError> {
        if variables == 0 {
            return Err(IndexError::NoVariables);
        }
        Ok(Scratch {
            variables,
            shifted: vec![0; variables - 1],
            left: vec![0; variables],
            right: vec![0; variables],
            sum: vec![0; variables],
        })
    }

    /// The number of variables this space was built for.
    pub fn variables(&self) -> usize {
        self.variables
    }
}

/// [`exponents_of`] writing into space the caller owns.
///
/// `exponents` is the output, `variables` long. `shifted` is working space, one
/// shorter, and its contents on entry are ignored.
pub fn exponents_into(
    index: u64,
    degree: u32,
    exponents: &mut [u32],
    shifted: &mut [u64],
) -> Result<(), IndexError> {
    let variables = exponents.len();
    if variables == 0 {
        return Err(IndexError::NoVariables);
    }
    if shifted.len() + 1 != variables {
        return Err(IndexError::DegreeMismatch {
            stated: degree,
            actual: shifted.len() as u64,
        });
    }
    let size = dimension(variables, degree)?;
    if index >= size {
        return Err(IndexError::IndexBeyondDimension {
            variables,
            degree,
            index,
            dimension: size,
        });
    }

    let mut remainder = index;
    for k in (1..variables).rev() {
        let k64 = k as u64;
        let mut candidate = k64 - 1;
        // `binomial` returns `Some(0)` while the candidate is below `k`, so the
        // walk starts inside the range where the value is real.
        while binomial(candidate + 1, k64).is_some_and(|value| value <= remainder) {
            candidate += 1;
        }
        shifted[k - 1] = candidate;
        remainder -= binomial(candidate, k64).expect("candidate was reached by a value that fits");
    }

    let mut previous: u64 = 0;
    for k in 1..variables {
        let partial = shifted[k - 1] - (k as u64 - 1);
        exponents[k - 1] = (partial - previous) as u32;
        previous = partial;
    }
    exponents[variables - 1] = (u64::from(degree) - previous) as u32;
    Ok(())
}

/// The index of the product of two monomials, recomputed rather than looked up.
///
/// Given the index of a monomial at degree `left_degree` and the index of one at
/// `right_degree`, the index of their product at `left_degree + right_degree`.
/// This is the arithmetic the multiplication inner loop needs for every pair of
/// terms, and [`ProductTable`] is the same map stored instead.
pub fn product_index(
    left: u64,
    left_degree: u32,
    right: u64,
    right_degree: u32,
    variables: usize,
) -> Result<u64, IndexError> {
    let mut scratch = Scratch::new(variables)?;
    product_index_with(left, left_degree, right, right_degree, &mut scratch)
}

/// [`product_index`] using space the caller owns, which is what the
/// multiplication kernel would call.
///
/// Separate from [`product_index`] because the difference is the whole
/// measurement: two heap allocations per term against none, on a function called
/// once per pair of terms in the inner loop.
pub fn product_index_with(
    left: u64,
    left_degree: u32,
    right: u64,
    right_degree: u32,
    scratch: &mut Scratch,
) -> Result<u64, IndexError> {
    let variables = scratch.variables;
    // Saturating rather than checked: a sum that would wrap is refused by the
    // addressability test on the next line anyway, and the saturated value is
    // what the error message should carry.
    let sum_degree = left_degree.saturating_add(right_degree);
    if !degree_is_addressable(variables, sum_degree) {
        return Err(IndexError::DegreeBeyondMaximum {
            variables,
            degree: sum_degree,
            maximum: maximum_degree(variables)?,
        });
    }
    let Scratch {
        shifted,
        left: left_exponents,
        right: right_exponents,
        sum,
        ..
    } = scratch;
    exponents_into(left, left_degree, left_exponents, shifted)?;
    exponents_into(right, right_degree, right_exponents, shifted)?;
    for k in 0..variables {
        sum[k] = left_exponents[k] + right_exponents[k];
    }
    index_of(sum, sum_degree)
}

/// The same map as [`product_index`], stored for one pair of degrees.
///
/// Held so the two can be measured against each other on a case of realistic
/// size rather than argued about. Which one the multiplication should use is the
/// fourth clause of issue #28 and the measurement is in that issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductTable {
    variables: usize,
    left_degree: u32,
    right_degree: u32,
    right_dimension: u64,
    entries: Vec<u64>,
}

impl ProductTable {
    /// Build the table for one pair of degrees.
    ///
    /// The size is `M(left, m) * M(right, m)` entries, which grows fast, so the
    /// multiplication that computes it is checked and the allocation is refused
    /// rather than attempted when it does not fit.
    pub fn new(variables: usize, left_degree: u32, right_degree: u32) -> Result<Self, IndexError> {
        let left_dimension = dimension(variables, left_degree)?;
        let right_dimension = dimension(variables, right_degree)?;
        let maximum = maximum_degree(variables)?;
        let count =
            left_dimension
                .checked_mul(right_dimension)
                .ok_or(IndexError::DegreeBeyondMaximum {
                    variables,
                    degree: left_degree.saturating_add(right_degree),
                    maximum,
                })?;
        let count = usize::try_from(count).map_err(|_| IndexError::DegreeBeyondMaximum {
            variables,
            degree: left_degree.saturating_add(right_degree),
            maximum,
        })?;
        let mut entries = Vec::with_capacity(count);
        for left in 0..left_dimension {
            for right in 0..right_dimension {
                entries.push(product_index(
                    left,
                    left_degree,
                    right,
                    right_degree,
                    variables,
                )?);
            }
        }
        Ok(ProductTable {
            variables,
            left_degree,
            right_degree,
            right_dimension,
            entries,
        })
    }

    /// The stored index of the product.
    pub fn get(&self, left: u64, right: u64) -> Result<u64, IndexError> {
        if right >= self.right_dimension {
            return Err(IndexError::IndexBeyondDimension {
                variables: self.variables,
                degree: self.right_degree,
                index: right,
                dimension: self.right_dimension,
            });
        }
        let offset = left * self.right_dimension + right;
        self.entries
            .get(offset as usize)
            .copied()
            .ok_or(IndexError::IndexBeyondDimension {
                variables: self.variables,
                degree: self.left_degree,
                index: left,
                dimension: self.entries.len() as u64 / self.right_dimension,
            })
    }

    /// How many entries the table holds.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table holds nothing, which happens only at a degree with no
    /// monomials and never for a valid pair.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every monomial of degree `d` in `m` variables, generated independently of
    /// the index under test so that the tests compare against the definition
    /// rather than against a second spelling of the same formula.
    fn every_monomial(variables: usize, degree: u32) -> Vec<Vec<u32>> {
        fn walk(remaining: u32, left: usize, prefix: &mut Vec<u32>, out: &mut Vec<Vec<u32>>) {
            if left == 1 {
                prefix.push(remaining);
                out.push(prefix.clone());
                prefix.pop();
                return;
            }
            for exponent in 0..=remaining {
                prefix.push(exponent);
                walk(remaining - exponent, left - 1, prefix, out);
                prefix.pop();
            }
        }
        let mut out = Vec::new();
        walk(degree, variables, &mut Vec::new(), &mut out);
        out
    }

    /// Ascending graded reverse lexicographic order, from 0003: compare at the
    /// last position where the vectors differ, and the smaller exponent there is
    /// the greater monomial.
    fn grevlex_less(a: &[u32], b: &[u32]) -> bool {
        for i in (0..a.len()).rev() {
            if a[i] != b[i] {
                return a[i] > b[i];
            }
        }
        false
    }

    /// The first Done-when clause of #28. Round trip at two, four and six
    /// variables, which is one, two and three degrees of freedom, for every
    /// index at every degree up to the bound rather than for a sample.
    #[test]
    fn index_and_exponents_are_inverse_for_every_monomial_up_to_degree_eight() {
        for &variables in &[2usize, 4, 6] {
            for degree in 0..=8u32 {
                let size = dimension(variables, degree).expect("degree is small");
                for index in 0..size {
                    let exponents = exponents_of(index, variables, degree).expect("index is valid");
                    let back = index_of(&exponents, degree).expect("exponents are valid");
                    assert_eq!(
                        back, index,
                        "round trip failed at {variables} variables, degree {degree}, index {index}"
                    );
                }
            }
        }
    }

    /// The second Done-when clause. Distinctness and total degree, checked
    /// against the independently generated set rather than against a count.
    #[test]
    fn every_index_at_a_degree_is_a_distinct_monomial_of_that_degree() {
        for &variables in &[2usize, 4, 6] {
            for degree in 0..=6u32 {
                let size = dimension(variables, degree).expect("degree is small");
                let mut seen = Vec::new();
                for index in 0..size {
                    let exponents = exponents_of(index, variables, degree).expect("index is valid");
                    assert_eq!(exponents.len(), variables);
                    let total: u32 = exponents.iter().sum();
                    assert_eq!(total, degree, "wrong total degree at index {index}");
                    seen.push(exponents);
                }
                let mut sorted = seen.clone();
                sorted.sort();
                sorted.dedup();
                assert_eq!(sorted.len(), seen.len(), "two indices gave one monomial");

                let mut expected = every_monomial(variables, degree);
                expected.sort();
                assert_eq!(sorted, expected, "the index misses or invents a monomial");
            }
        }
    }

    /// Ascending index has to be ascending grevlex, which is the property that
    /// makes a file written by one build readable by another. 0003 names the
    /// order and this is where the shipped index is held to it.
    #[test]
    fn index_order_is_ascending_grevlex() {
        for &variables in &[2usize, 3, 4, 6] {
            for degree in 0..=5u32 {
                let size = dimension(variables, degree).expect("degree is small");
                for index in 1..size {
                    let previous =
                        exponents_of(index - 1, variables, degree).expect("index is valid");
                    let current = exponents_of(index, variables, degree).expect("index is valid");
                    assert!(
                        grevlex_less(&previous, &current),
                        "index {index} at {variables} variables degree {degree} is out of order: \
                         {previous:?} then {current:?}"
                    );
                }
            }
        }
    }

    /// The worked example in 0003, quoted there for m = 3 and d = 2. A test
    /// against a literal from the decision document catches a change that keeps
    /// every internal property and still renumbers every coefficient in every
    /// file this package has ever written.
    #[test]
    fn the_worked_example_in_0003_holds() {
        let expected = [
            [0u32, 0, 2],
            [0, 1, 1],
            [1, 0, 1],
            [0, 2, 0],
            [1, 1, 0],
            [2, 0, 0],
        ];
        for (index, exponents) in expected.iter().enumerate() {
            assert_eq!(
                exponents_of(index as u64, 3, 2).expect("index is valid"),
                exponents.to_vec(),
                "index {index} does not match 0003"
            );
            assert_eq!(
                index_of(exponents, 2).expect("exponents are valid"),
                index as u64
            );
        }
    }

    /// The third Done-when clause. The table has to agree with the closed form
    /// at every degree, and the closed form is written here as a product rather
    /// than by calling the same binomial the table used.
    #[test]
    fn the_degree_table_agrees_with_the_closed_form() {
        for &variables in &[1usize, 2, 3, 4, 6, 12] {
            let table = DegreeTable::new(variables, 12).expect("bound is small");
            for degree in 0..=12u32 {
                // M(d, m) = C(d + m - 1, m - 1), written as a product of
                // (d + i) / i for i = 1 .. m - 1, which is a different
                // computation from the one under test.
                let mut expected: u128 = 1;
                for i in 1..variables as u128 {
                    expected = expected * (u128::from(degree) + i) / i;
                }
                assert_eq!(
                    u128::from(table.dimension(degree).expect("degree is in the table")),
                    expected,
                    "dimension disagrees at {variables} variables, degree {degree}"
                );
            }
            // And the running total is C(D + m, m), the size of the whole
            // truncation.
            for degree in 0..=12u32 {
                let mut expected: u128 = 1;
                for i in 1..=variables as u128 {
                    expected = expected * (u128::from(degree) + i) / i;
                }
                assert_eq!(
                    u128::from(table.cumulative(degree).expect("degree is in the table")),
                    expected,
                    "cumulative disagrees at {variables} variables, degree {degree}"
                );
            }
        }
    }

    /// The fifth Done-when clause. A degree beyond the maximum is refused, and
    /// the error names the maximum so a caller can act on it without guessing.
    #[test]
    fn a_degree_beyond_the_maximum_is_refused_with_the_maximum_named() {
        for &variables in &[4usize, 6, 8, 12] {
            let maximum = maximum_degree(variables).expect("variables is not zero");
            assert!(
                dimension(variables, maximum).is_ok(),
                "the maximum itself has to be accepted"
            );

            let error = dimension(variables, maximum + 1).expect_err("beyond the maximum");
            assert_eq!(
                error,
                IndexError::DegreeBeyondMaximum {
                    variables,
                    degree: maximum + 1,
                    maximum
                }
            );
            assert!(
                error.to_string().contains(&maximum.to_string()),
                "the message does not name the maximum: {error}"
            );

            // The same boundary on the other entry points, so the refusal is not
            // one function's manners.
            assert!(DegreeTable::new(variables, maximum + 1).is_err());
            assert!(exponents_of(0, variables, maximum + 1).is_err());
            let exponents = vec![0u32; variables];
            assert!(index_of(&exponents, maximum + 1).is_err());
        }
    }

    /// The maximum is the pair-count bound rather than the dimension bound, and
    /// that is the whole reason it is computed. At the target case the two
    /// differ by a wide margin, and a maximum taken from the dimension would
    /// accept a degree whose convolution cannot be addressed.
    #[test]
    fn the_maximum_is_set_by_the_pair_count_and_not_by_the_dimension() {
        for &variables in &[6usize, 8, 12] {
            let maximum = maximum_degree(variables).expect("variables is not zero");

            // Every pair reaching the maximum multiplies without overflow.
            for a in 0..=maximum {
                let left = dimension_unchecked(variables, a).expect("within the maximum");
                let right =
                    dimension_unchecked(variables, maximum - a).expect("within the maximum");
                assert!(
                    left.checked_mul(right).is_some(),
                    "pair count overflows inside the maximum at {variables} variables"
                );
            }

            // One degree further, some pair does not.
            let beyond = maximum + 1;
            let overflows = (0..=beyond).any(|a| {
                match (
                    dimension_unchecked(variables, a),
                    dimension_unchecked(variables, beyond - a),
                ) {
                    (Some(left), Some(right)) => left.checked_mul(right).is_none(),
                    _ => true,
                }
            });
            assert!(
                overflows,
                "nothing overflows one degree past the maximum at {variables} variables"
            );

            // The dimension alone survives further, which is what says the two
            // bounds are different numbers.
            assert!(
                dimension_unchecked(variables, beyond).is_some(),
                "the dimension bound is not the looser of the two at {variables} variables"
            );
        }
    }

    /// The maximum at the variable counts that matter, against the numbers
    /// worked out on #28 before any of this was written. Six, eight and twelve
    /// variables are three, four and six degrees of freedom, and the whole
    /// package is aimed at the last of those.
    #[test]
    fn the_maximum_matches_the_numbers_the_issue_worked_out() {
        assert_eq!(maximum_degree(6), Ok(434));
        assert_eq!(maximum_degree(8), Ok(152));
        assert_eq!(maximum_degree(12), Ok(62));
        // Few enough variables and the arithmetic gives no boundary at all, so
        // the stated ceiling is the boundary and it is refused above rather than
        // silently accepted.
        assert_eq!(maximum_degree(4), Ok(MAXIMUM_DEGREE_CAP));
        assert!(dimension(4, MAXIMUM_DEGREE_CAP).is_ok());
        assert!(dimension(4, MAXIMUM_DEGREE_CAP + 1).is_err());
    }

    /// The balanced split really is the worst one, which is what makes the
    /// cheap addressability check the same test as the full sweep. Checked over
    /// every split at every degree up to the maximum, so it is measured on the
    /// shipped bound rather than assumed from the concavity argument in the
    /// source.
    #[test]
    fn the_balanced_split_is_the_largest_pair_count() {
        for &variables in &[6usize, 8, 12] {
            let maximum = maximum_degree(variables).expect("variables is not zero");
            for degree in 0..=maximum {
                let half = degree / 2;
                let balanced = dimension_unchecked(variables, half)
                    .expect("within the maximum")
                    .checked_mul(
                        dimension_unchecked(variables, degree - half).expect("within the maximum"),
                    )
                    .expect("within the maximum");
                for a in 0..=degree {
                    let pair = dimension_unchecked(variables, a)
                        .expect("within the maximum")
                        .checked_mul(
                            dimension_unchecked(variables, degree - a).expect("within the maximum"),
                        )
                        .expect("within the maximum");
                    assert!(
                        pair <= balanced,
                        "split at {a} beats the balanced split at {variables} variables, \
                         degree {degree}"
                    );
                }
            }
        }
    }

    /// A vector whose entries do not add up to the degree it is used at is a
    /// caller working at one degree with a monomial from another, which would
    /// otherwise index into the wrong array and return a plausible number.
    #[test]
    fn an_exponent_vector_of_the_wrong_degree_is_refused() {
        let error = index_of(&[1, 1, 1], 2).expect_err("total degree is three");
        assert_eq!(
            error,
            IndexError::DegreeMismatch {
                stated: 2,
                actual: 3
            }
        );
    }

    /// An index at or past the dimension is not a monomial of that degree.
    #[test]
    fn an_index_beyond_the_dimension_is_refused() {
        let size = dimension(4, 3).expect("degree is small");
        let error = exponents_of(size, 4, 3).expect_err("index is past the end");
        assert_eq!(
            error,
            IndexError::IndexBeyondDimension {
                variables: 4,
                degree: 3,
                index: size,
                dimension: size
            }
        );
    }

    /// Zero variables is a caller mistake rather than an empty case to serve.
    #[test]
    fn zero_variables_is_refused() {
        assert_eq!(maximum_degree(0), Err(IndexError::NoVariables));
        assert_eq!(dimension(0, 0), Err(IndexError::NoVariables));
        assert_eq!(exponents_of(0, 0, 0), Err(IndexError::NoVariables));
        assert_eq!(index_of(&[], 0), Err(IndexError::NoVariables));
    }

    /// The stored table and the recomputed map are the same function. Whichever
    /// the multiplication ends up using, a disagreement between them is a wrong
    /// coefficient in every product.
    #[test]
    fn the_stored_table_agrees_with_the_recomputed_index() {
        for &(variables, left_degree, right_degree) in
            &[(2usize, 2u32, 3u32), (4, 2, 2), (6, 1, 3), (6, 2, 2)]
        {
            let table =
                ProductTable::new(variables, left_degree, right_degree).expect("case is small");
            let left_dimension = dimension(variables, left_degree).expect("degree is small");
            let right_dimension = dimension(variables, right_degree).expect("degree is small");
            assert_eq!(table.len() as u64, left_dimension * right_dimension);
            for left in 0..left_dimension {
                for right in 0..right_dimension {
                    let stored = table.get(left, right).expect("indices are valid");
                    let recomputed =
                        product_index(left, left_degree, right, right_degree, variables)
                            .expect("indices are valid");
                    assert_eq!(stored, recomputed);
                }
            }
        }
    }

    /// The measurement the fourth clause of #28 asks for: the stored addition
    /// table against recomputing the exponent addition, on a case of realistic
    /// size.
    ///
    /// Ignored by default, because it is a timing run rather than a property and
    /// a timing run in the gate is a flaky check. Run it with
    ///
    ///     cargo test --release -p drehbank-core -- --ignored --nocapture
    ///
    /// The release profile is the one this is about. The test profile turns
    /// overflow checks on, which is right for correctness and would be measuring
    /// the wrong build here.
    #[test]
    #[ignore = "a timing run, not a property; see the command in the doc comment"]
    fn measure_the_stored_table_against_recomputing_it() {
        // A deterministic stream, so two runs on one machine differ only by the
        // machine. Not a good generator and it does not need to be: what it has
        // to do is defeat the prefetcher, which any non-sequential order does.
        fn next(state: &mut u64) -> u64 {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            *state
        }

        println!(
            "{:>4} {:>3} {:>3} {:>12} {:>10} {:>9} {:>11} {:>11} {:>11} {:>7}",
            "v",
            "a",
            "b",
            "entries",
            "build ms",
            "MiB",
            "stored ns",
            "scratch ns",
            "alloc ns",
            "ratio"
        );
        for &(variables, left_degree, right_degree) in
            &[(6usize, 4u32, 4u32), (12, 3, 3), (12, 4, 4)]
        {
            let left_dimension = dimension(variables, left_degree).expect("case is small");
            let right_dimension = dimension(variables, right_degree).expect("case is small");

            let start = std::time::Instant::now();
            let table = ProductTable::new(variables, left_degree, right_degree).expect("case fits");
            let build = start.elapsed();

            let samples: usize = 2_000_000;
            let mut state: u64 = 0x9E3779B97F4A7C15;

            let mut sink: u64 = 0;
            let start = std::time::Instant::now();
            for _ in 0..samples {
                let left = next(&mut state) % left_dimension;
                let right = next(&mut state) % right_dimension;
                sink ^= table.get(left, right).expect("indices are valid");
            }
            let stored = start.elapsed();

            // Recomputed with working space the caller owns, which is what a
            // multiplication kernel would do.
            let mut scratch = Scratch::new(variables).expect("variables is not zero");
            let mut state: u64 = 0x9E3779B97F4A7C15;
            let mut with_scratch: u64 = 0;
            let start = std::time::Instant::now();
            for _ in 0..samples {
                let left = next(&mut state) % left_dimension;
                let right = next(&mut state) % right_dimension;
                with_scratch ^=
                    product_index_with(left, left_degree, right, right_degree, &mut scratch)
                        .expect("indices are valid");
            }
            let scratched = start.elapsed();

            // And allocating per call, which is the convenience entry point.
            // Both are here because a measurement that decides an architecture
            // must not be decided by an allocation somebody could remove.
            let mut state: u64 = 0x9E3779B97F4A7C15;
            let mut allocating: u64 = 0;
            let start = std::time::Instant::now();
            for _ in 0..samples {
                let left = next(&mut state) % left_dimension;
                let right = next(&mut state) % right_dimension;
                allocating ^= product_index(left, left_degree, right, right_degree, variables)
                    .expect("indices are valid");
            }
            let allocated = start.elapsed();

            // Same stream and the same map, so the three loops have to agree.
            // Without this the compiler is free to delete any of them.
            assert_eq!(
                (sink, sink),
                (with_scratch, allocating),
                "the paths disagree, so none of the timings means anything"
            );

            let bytes = table.len() * core::mem::size_of::<u64>();
            println!(
                "{:>4} {:>3} {:>3} {:>12} {:>10.1} {:>9.1} {:>11.2} {:>11.2} {:>11.2} {:>7.1}",
                variables / 2,
                left_degree,
                right_degree,
                table.len(),
                build.as_secs_f64() * 1e3,
                bytes as f64 / (1024.0 * 1024.0),
                stored.as_secs_f64() * 1e9 / samples as f64,
                scratched.as_secs_f64() * 1e9 / samples as f64,
                allocated.as_secs_f64() * 1e9 / samples as f64,
                scratched.as_secs_f64() / stored.as_secs_f64(),
            );
        }
    }

    /// The product index is the index of the product monomial, checked against
    /// exponent addition done outside the index.
    #[test]
    fn the_product_index_is_the_index_of_the_sum_of_the_exponents() {
        let variables = 6;
        for left_degree in 0..=3u32 {
            for right_degree in 0..=3u32 {
                let left_dimension = dimension(variables, left_degree).expect("degree is small");
                let right_dimension = dimension(variables, right_degree).expect("degree is small");
                for left in 0..left_dimension {
                    for right in 0..right_dimension {
                        let a = exponents_of(left, variables, left_degree).expect("valid");
                        let b = exponents_of(right, variables, right_degree).expect("valid");
                        let sum: Vec<u32> = a.iter().zip(b.iter()).map(|(x, y)| x + y).collect();
                        assert_eq!(
                            product_index(left, left_degree, right, right_degree, variables)
                                .expect("valid"),
                            index_of(&sum, left_degree + right_degree).expect("valid")
                        );
                    }
                }
            }
        }
    }
}
