//! The kernels run across threads, with an answer that does not depend on how
//! many.
//!
//! `docs/decisions/0009-parallelism-and-memory.md` decides three things this
//! module implements. Parallelism is at the term level and nowhere above it, so
//! one product or one bracket is run across the pool and the callers above stay
//! sequential. The partition is over the output array, in contiguous chunks of
//! a fixed target length, cut the same way on every machine because the rule
//! reads the storage layout and never the scheduler. And the answer does not
//! move with the thread count.
//!
//! # How the answer is held still
//!
//! 0009 offers two shapes and this module takes the first one. Each chunk of
//! the output is written by exactly one thread, which accumulates every
//! contribution to that chunk itself, so no partial sum ever crosses a thread
//! and there is no reduction to fix an order for. The second shape, private
//! accumulators combined in increasing chunk index, is what an operation that
//! cannot be written this way would need, and neither kernel here is such an
//! operation.
//!
//! That costs a different kernel rather than a wrapper around the sequential
//! one. [`crate::series::Series::product`] is a scatter: it walks pairs of
//! input terms and adds each product into whichever output slot it lands in,
//! which no partition of the output can be cut out of. The kernels here are the
//! same convolution read backwards, a gather: for one output monomial, walk the
//! pairs of input monomials that multiply to it. Same terms, same additions,
//! and the whole of one output slot computed in one place.
//!
//! # Why the two agree bit for bit
//!
//! Reading the convolution backwards would normally change the order the
//! contributions to a slot are added in, and floating point addition is not
//! associative, so that would be a different number on the same input. It is
//! not, and the reason is the enumeration order in [`Walk`].
//!
//! The scatter reaches a slot of degree `s` in the order its loops run: left
//! degree `d` ascending, then left index ascending, with the right index
//! determined by the two. The gather walks the same contributions in the same
//! order, because it takes `d` ascending and enumerates the divisors of the
//! output exponent vector in ascending left index. So every slot receives the
//! same terms in the same order and rounds identically, at every pool size,
//! including one. `tests/parallel.rs` is where that is checked rather than
//! asserted, bit for bit over a generated corpus, and it is the test that would
//! fail first if either kernel's loop order moved.
//!
//! # What this costs
//!
//! A bracket holds all `4v` derivative operands alive at once, where
//! [`crate::series::Series::bracket`] holds two at a time, because a slot's
//! contributions from every term of the sum have to be added in the sum's own
//! order and that means every operand is reachable while the slot is being
//! filled. That is a real increase in the peak the memory ceiling of 0009 is
//! checked against, and it is stated here rather than found by a run that was
//! refused.

use std::num::NonZero;
use std::sync::Mutex;

use crate::coefficient::Coefficient;
use crate::error::Error;
use crate::monomial::{IndexError, Scratch, binomial, exponents_into, maximum_degree};
use crate::series::{Series, Sign, bracket_order};

/// The target length of a chunk of the output array, in coefficients.
///
/// 0009 requires this to be a property of the build and not a function of the
/// pool size, the core count or the load, so that two runs of one input cut the
/// same chunks with the same indices whatever the machine is doing. It is that,
/// and it is one number in one place for that reason.
///
/// The value has not been tuned against a measurement. It is large enough that
/// claiming a chunk costs less than filling one and small enough that a degree
/// of a few thousand monomials still splits across a pool, which is a reason to
/// start here rather than evidence that it is the best number. What would move
/// it is a measurement, and the speedup record of issue #49 is where such a
/// measurement is kept.
pub const CHUNK: usize = 256;

/// How many threads a kernel runs on.
///
/// A [`NonZero`] rather than a `usize` with a refusal, because a pool of no
/// threads is not a case with an error message, it is a value that cannot be
/// built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pool {
    threads: NonZero<usize>,
}

impl Pool {
    /// The pool the runtime's available parallelism suggests.
    ///
    /// One thread where the runtime declines to say, which it is allowed to do.
    /// That is the sequential path taken deliberately rather than a guess at a
    /// core count, and the answer is the same either way, which is the whole
    /// point of the module.
    pub fn available() -> Self {
        Pool {
            threads: std::thread::available_parallelism().unwrap_or(NonZero::<usize>::MIN),
        }
    }

    /// A pool of a stated size.
    ///
    /// Oversubscription is allowed and is not a mistake. The determinism suite
    /// runs sixteen threads on machines with fewer cores on purpose, because
    /// oversubscription makes completion order vary more rather than less.
    pub fn of(threads: NonZero<usize>) -> Self {
        Pool { threads }
    }

    /// How many threads this pool runs.
    pub fn threads(self) -> usize {
        self.threads.get()
    }

    /// The chunk target this build cuts the output with, which is [`CHUNK`].
    ///
    /// On the pool so that a caller recording a run can read the partition off
    /// the thing it ran on rather than off a constant it has to know the name
    /// of.
    pub fn chunk(self) -> usize {
        CHUNK
    }
}

/// One term of the sum a kernel is: two operands and the sign their product
/// enters the destination with.
///
/// A product is one of these. A bracket is `2v` of them, in the order item 3 of
/// `docs/decisions/0004-conventions.md` writes the sum, and the order matters
/// because it is the order a destination slot's contributions are added in.
struct Term<'a, C> {
    left: &'a Series<C>,
    right: &'a Series<C>,
    sign: Sign,
}

/// The product of two series, truncated at the order both carry, computed
/// across `pool`.
///
/// The same series [`crate::series::Series::product`] returns, coefficient for
/// coefficient and bit for bit, at every pool size.
pub fn product<C: Coefficient + Send + Sync>(
    left: &Series<C>,
    right: &Series<C>,
    pool: Pool,
) -> Result<Series<C>, Error> {
    left.check_combinable(right)?;
    let mut result: Series<C> = Series::zero(left.freedoms(), left.order())?;
    let terms = [Term {
        left,
        right,
        sign: Sign::Add,
    }];
    fill(&mut result, &terms, pool)?;
    Ok(result)
}

/// The Poisson bracket of two series, computed across `pool`.
///
/// The same series [`crate::series::Series::bracket`] returns, coefficient for
/// coefficient and bit for bit, at every pool size. The derivatives are taken
/// sequentially: each one is a single pass over the operand and is linear in
/// its length, where the convolutions below it are quadratic, so the level 0009
/// puts the parallelism at is the one that carries the cost.
pub fn bracket<C: Coefficient + Send + Sync>(
    left: &Series<C>,
    right: &Series<C>,
    pool: Pool,
) -> Result<Series<C>, Error> {
    if left.freedoms() != right.freedoms() {
        return Err(Error::FreedomsDiffer {
            left: left.freedoms(),
            right: right.freedoms(),
        });
    }
    let freedoms = left.freedoms();
    let mut result: Series<C> = Series::zero(freedoms, bracket_order(left.order(), right.order()))?;
    // Held rather than streamed, which is the memory cost the module header
    // states: every operand of the sum has to be reachable while a slot is
    // filled, because the slot's additions run in the sum's order.
    let mut operands: Vec<(Series<C>, Series<C>, Sign)> = Vec::with_capacity(2 * freedoms);
    for position in 0..freedoms {
        let momentum = position + freedoms;
        operands.push((
            left.derivative(position)?,
            right.derivative(momentum)?,
            Sign::Add,
        ));
        operands.push((
            left.derivative(momentum)?,
            right.derivative(position)?,
            Sign::Subtract,
        ));
    }
    let terms: Vec<Term<'_, C>> = operands
        .iter()
        .map(|(left, right, sign)| Term {
            left,
            right,
            sign: *sign,
        })
        .collect();
    fill(&mut result, &terms, pool)?;
    Ok(result)
}

/// Fill every degree of the destination from the terms, one degree at a time.
///
/// The barrier between degrees is the one 0009 accepts. It is a join of the
/// pool per output degree, which is small next to the convolution that
/// preceded it, and it is what keeps the partition a partition of one
/// contiguous array rather than of a ragged set of them.
fn fill<C: Coefficient + Send + Sync>(
    result: &mut Series<C>,
    terms: &[Term<'_, C>],
    pool: Pool,
) -> Result<(), Error> {
    let variables = result.variables();
    let order = result.order();
    // The ranks the divisor enumeration adds up, once per call rather than once
    // per output slot. `index_of` recomputes these from a binomial each time it
    // is asked, and the enumeration below asks for one per level per divisor.
    let ranks = RankTable::new(variables, order)?;
    for degree in 0..=order {
        if !contributes(terms, degree) {
            continue;
        }
        result.store_degree(degree)?;
        let destination = result.degree_mut(degree);
        fill_degree(destination, degree, terms, variables, &ranks, pool)?;
    }
    Ok(())
}

/// Whether any term reaches this output degree at all.
///
/// The same condition the sequential kernels store a degree under: a pair of
/// stored operand degrees whose degrees sum to this one. A degree no pair
/// reaches is left unstored, which is the same series held smaller, and a
/// degree some pair reaches is stored even when every contribution to it is
/// zero, because that is what the scatter does and the two have to agree on the
/// storage as well as on the coefficients.
fn contributes<C: Coefficient>(terms: &[Term<'_, C>], degree: u32) -> bool {
    terms
        .iter()
        .any(|term| splits(term, degree).next().is_some())
}

/// The left degrees of a term that reach an output degree, ascending.
///
/// Ascending is not a convenience. It is the order the scatter's outer loop
/// runs in, and a destination slot's contributions are added in it.
fn splits<'a, C: Coefficient>(
    term: &'a Term<'_, C>,
    degree: u32,
) -> impl Iterator<Item = (u32, u32)> + 'a {
    let low = degree.saturating_sub(term.right.order());
    let high = degree.min(term.left.order());
    (low..=high)
        .filter(move |&left_degree| {
            let right_degree = degree - left_degree;
            !term.left.degree_slice(left_degree).is_empty()
                && !term.right.degree_slice(right_degree).is_empty()
        })
        .map(move |left_degree| (left_degree, degree - left_degree))
}

/// Fill one output degree across the pool.
///
/// The queue is the chunk iterator behind a lock, so a thread that finishes
/// early takes the next chunk rather than waiting for a static assignment it
/// was given at the start. What the lock hands out is the chunk and its index;
/// what is under the lock is the cursor and never a coefficient, so no
/// arithmetic happens inside it.
fn fill_degree<C: Coefficient + Send + Sync>(
    destination: &mut [C],
    degree: u32,
    terms: &[Term<'_, C>],
    variables: usize,
    ranks: &RankTable,
    pool: Pool,
) -> Result<(), Error> {
    let queue = Mutex::new(destination.chunks_mut(CHUNK).enumerate());
    let outcomes: Vec<Result<(), Error>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..pool.threads())
            .map(|_| {
                scope.spawn(|| {
                    let mut space = Space::new(variables, degree, ranks)?;
                    loop {
                        // The guard is dropped before the chunk is filled. The
                        // item outlives it, because a chunk borrows the
                        // destination and not the cursor.
                        let claimed = {
                            let mut cursor =
                                queue.lock().unwrap_or_else(|poison| poison.into_inner());
                            cursor.next()
                        };
                        let Some((chunk, values)) = claimed else {
                            return Ok(());
                        };
                        let start = chunk * CHUNK;
                        for (offset, slot) in values.iter_mut().enumerate() {
                            let index = (start + offset) as u64;
                            *slot = space.slot(index, terms)?;
                        }
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| match handle.join() {
                Ok(outcome) => outcome,
                // A panic in a worker is re-raised rather than turned into an
                // error, because it is a defect in this module and not a
                // condition a caller can act on.
                Err(payload) => std::panic::resume_unwind(payload),
            })
            .collect()
    });
    for outcome in outcomes {
        outcome?;
    }
    Ok(())
}

/// `C(p + k - 1, k)` for every level `k` and every prefix sum `p` a run can
/// reach.
///
/// This is the term the rank of `docs/decisions/0003-series-representation.md`
/// adds at level `k`, and the enumeration below adds one per level per divisor,
/// twice, so it is a table rather than a binomial in the innermost loop.
struct RankTable {
    /// Row `k` holds `C(p + k - 1, k)` for `p` in `0 ..= order`. Row zero is
    /// empty: the rank sum runs from level one.
    rows: Vec<Vec<u64>>,
}

impl RankTable {
    /// Build the table.
    ///
    /// The refusal is unreachable for a series that exists. Every entry is at
    /// most `M(order, variables)`, which the destination's own degree table
    /// computed without overflowing before this is called. It is written as a
    /// refusal anyway rather than as an unwrap, because the alternative to
    /// refusing here is a rank that wrapped, and a wrapped rank reads the wrong
    /// coefficient and returns a plausible number.
    fn new(variables: usize, order: u32) -> Result<Self, Error> {
        let mut rows = vec![Vec::new()];
        for level in 1..variables {
            let mut row = Vec::with_capacity(order as usize + 1);
            for prefix in 0..=u64::from(order) {
                let level = level as u64;
                let term =
                    binomial(prefix + level - 1, level).ok_or(IndexError::DegreeBeyondMaximum {
                        variables,
                        degree: order,
                        maximum: maximum_degree(variables)?,
                    })?;
                row.push(term);
            }
            rows.push(row);
        }
        Ok(RankTable { rows })
    }

    fn at(&self, level: usize, prefix: u64) -> u64 {
        self.rows[level][prefix as usize]
    }
}

/// One thread's working space, and the walk over one output slot.
///
/// Everything here is allocated once per thread per degree and reused across
/// the chunks that thread claims, because the alternative is an allocation per
/// output coefficient.
struct Space<'a> {
    variables: usize,
    degree: u32,
    ranks: &'a RankTable,
    /// The exponent vector of the output monomial being filled.
    exponents: Vec<u32>,
    /// Working space [`exponents_into`] wants.
    shifted: Vec<u64>,
    /// `prefix[k]` is the sum of the first `k` exponents of the output
    /// monomial, so `prefix[0]` is zero and `prefix[m]` is the degree.
    prefix: Vec<u64>,
    /// The divisor walk's state, one entry per level.
    walk: Walk,
}

impl<'a> Space<'a> {
    fn new(variables: usize, degree: u32, ranks: &'a RankTable) -> Result<Self, Error> {
        // Refuses a zero variable count in one place, so the vectors below can
        // be sized from it.
        Scratch::new(variables)?;
        Ok(Space {
            variables,
            degree,
            ranks,
            exponents: vec![0; variables],
            shifted: vec![0; variables - 1],
            prefix: vec![0; variables + 1],
            walk: Walk::new(variables),
        })
    }

    /// The coefficient of one output monomial.
    ///
    /// The sum of every contribution to it, in the order the scatter adds them:
    /// term by term in the order the terms were given, then left degree
    /// ascending, then left index ascending.
    fn slot<C: Coefficient>(&mut self, index: u64, terms: &[Term<'_, C>]) -> Result<C, Error> {
        exponents_into(index, self.degree, &mut self.exponents, &mut self.shifted)?;
        self.prefix[0] = 0;
        for k in 1..=self.variables {
            self.prefix[k] = self.prefix[k - 1] + u64::from(self.exponents[k - 1]);
        }
        let mut total = C::zero();
        for term in terms {
            for (left_degree, right_degree) in splits(term, self.degree) {
                let left = term.left.degree_slice(left_degree);
                let right = term.right.degree_slice(right_degree);
                self.walk
                    .start(u64::from(left_degree), &self.exponents, &self.prefix);
                while let Some((left_index, right_index)) =
                    self.walk.next(&self.exponents, &self.prefix, self.ranks)
                {
                    let product = left[left_index as usize].multiply(&right[right_index as usize]);
                    total = match term.sign {
                        Sign::Add => total.add(&product),
                        Sign::Subtract => total.subtract(&product),
                    };
                }
            }
        }
        Ok(total)
    }
}

/// The walk over the divisors of one output monomial at one left degree, in
/// ascending left index.
///
/// # Why the order is the ascending one
///
/// A divisor is fixed by its prefix sums `P_1 .. P_{m-1}`, and the rank of 0003
/// is `sum_k C(P_k + k - 1, k)` over those. That is the combinatorial number
/// system on the strictly increasing tuple `P_k + k - 1`, whose ranking is
/// colexicographic: `P_{m-1}` decides first, then `P_{m-2}`, down to `P_1`. So
/// choosing `P_{m-1}` ascending in the outermost level and descending no level
/// anywhere emits the divisors in ascending index, with no sort and no second
/// pass.
///
/// The two indices come out of the same walk. `P_k` is a prefix sum of the
/// divisor and `A_k - P_k` is the matching prefix sum of the cofactor, so one
/// level of the walk adds one rank term to each of the two indices, and the
/// pair is complete when the walk reaches level one.
struct Walk {
    variables: usize,
    /// `pick[k]` is the chosen `P_k`. `pick[m]` is the left degree, which is
    /// the value the outermost level is bounded by rather than a choice.
    pick: Vec<u64>,
    low: Vec<u64>,
    high: Vec<u64>,
    /// `left[k]` is the divisor's rank summed over levels `m-1` down to `k`,
    /// and `right[k]` the cofactor's. Entry `m` is the empty sum.
    left: Vec<u64>,
    right: Vec<u64>,
    level: usize,
    done: bool,
}

impl Walk {
    fn new(variables: usize) -> Self {
        Walk {
            variables,
            pick: vec![0; variables + 1],
            low: vec![0; variables + 1],
            high: vec![0; variables + 1],
            left: vec![0; variables + 1],
            right: vec![0; variables + 1],
            level: 0,
            done: true,
        }
    }

    /// Begin the divisors of `exponents` whose degree is `degree`.
    fn start(&mut self, degree: u64, exponents: &[u32], prefix: &[u64]) {
        let top = self.variables;
        self.pick[top] = degree;
        self.left[top] = 0;
        self.right[top] = 0;
        self.level = top - 1;
        self.bound(self.level, exponents, prefix);
        self.pick[self.level] = self.low[self.level];
        self.done = false;
    }

    /// The bounds on `P_k` given the level above it.
    ///
    /// Upward from zero because a divisor's exponents are non-negative, and
    /// downward from the cofactor's because `beta_k = P_{k+1} - P_k` may not
    /// exceed the output monomial's own exponent there. `P_k` is also capped by
    /// the output monomial's prefix sum, which is the same statement one level
    /// further left.
    fn bound(&mut self, level: usize, exponents: &[u32], prefix: &[u64]) {
        let above = self.pick[level + 1];
        self.low[level] = above.saturating_sub(u64::from(exponents[level]));
        self.high[level] = above.min(prefix[level]);
    }

    /// The next divisor and cofactor index pair, or `None` when the walk is
    /// spent.
    fn next(&mut self, exponents: &[u32], prefix: &[u64], ranks: &RankTable) -> Option<(u64, u64)> {
        if self.done {
            return None;
        }
        loop {
            let level = self.level;
            if self.pick[level] > self.high[level] {
                if level + 1 == self.variables {
                    self.done = true;
                    return None;
                }
                self.level += 1;
                self.pick[self.level] += 1;
                continue;
            }
            let chosen = self.pick[level];
            self.left[level] = self.left[level + 1] + ranks.at(level, chosen);
            self.right[level] = self.right[level + 1] + ranks.at(level, prefix[level] - chosen);
            if level == 1 {
                let pair = (self.left[1], self.right[1]);
                self.pick[1] += 1;
                return Some(pair);
            }
            self.level -= 1;
            self.bound(self.level, exponents, prefix);
            self.pick[self.level] = self.low[self.level];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CHUNK, Pool, bracket, product};
    use crate::series::Series;
    use std::num::NonZero;

    fn pool(threads: usize) -> Pool {
        Pool::of(NonZero::new(threads).expect("the test pools are all non-zero"))
    }

    /// The one series both kernels are exercised on here, and nothing about it
    /// is special. What is special is checked in `tests/parallel.rs`, over a
    /// generated corpus and through the public API.
    fn ramp(freedoms: usize, order: u32) -> Series<f64> {
        let mut series = Series::zero(freedoms, order).expect("inside the ranges");
        let mut value = 1.0;
        for degree in 0..=order {
            let width = series.dimension(degree).expect("inside the ranges");
            for index in 0..width {
                series
                    .set_coefficient(degree, index, value)
                    .expect("inside the ranges");
                value += 1.0;
            }
        }
        series
    }

    #[test]
    fn the_product_agrees_with_the_sequential_kernel() {
        let left = ramp(2, 4);
        let right = ramp(2, 4);
        let sequential = left.product(&right).expect("inside the ranges");
        for threads in [1, 3, 8] {
            let parallel = product(&left, &right, pool(threads)).expect("inside the ranges");
            assert_eq!(parallel, sequential, "at {threads} thread(s)");
        }
    }

    #[test]
    fn the_bracket_agrees_with_the_sequential_kernel() {
        let left = ramp(2, 4);
        let right = ramp(2, 3);
        let sequential = left.bracket(&right).expect("inside the ranges");
        for threads in [1, 3, 8] {
            let parallel = bracket(&left, &right, pool(threads)).expect("inside the ranges");
            assert_eq!(parallel, sequential, "at {threads} thread(s)");
        }
    }

    /// A degree longer than one chunk is what makes the partition a partition
    /// at all, and the fixtures above are shorter than [`CHUNK`] at every
    /// degree. This one is not, so the queue hands out more than one chunk and
    /// a chunk offset that was wrong would show.
    #[test]
    fn a_degree_longer_than_one_chunk_is_partitioned_and_still_agrees() {
        let left = ramp(3, 6);
        let right = ramp(3, 6);
        let widest = (0..=6)
            .map(|degree| left.dimension(degree).expect("inside the ranges"))
            .max()
            .expect("the range is not empty");
        assert!(
            widest > CHUNK as u64,
            "this fixture is meant to exceed one chunk of {CHUNK}, and its widest \
             degree holds {widest}"
        );
        let sequential = left.product(&right).expect("inside the ranges");
        let parallel = product(&left, &right, pool(4)).expect("inside the ranges");
        assert_eq!(parallel, sequential);
    }

    #[test]
    fn a_pool_from_the_runtime_is_at_least_one_thread() {
        assert!(Pool::available().threads() >= 1);
    }

    #[test]
    fn the_pool_reports_the_chunk_target_the_build_cuts_with() {
        assert_eq!(pool(2).chunk(), CHUNK);
    }
}
