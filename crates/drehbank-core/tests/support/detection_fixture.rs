//! What the detection cases build their inputs from, and the search size
//! written a second time.

use drehbank_core::monomial::index_of;
use drehbank_core::series::Series;

/// Write one monomial into a series, naming it by its exponent vector.
///
/// The exponent vector is in the variable order of item 1 of 0004, positions
/// first. Naming the monomial rather than its index is what makes a case
/// readable: `[1, 0, 0, 1]` is `q_1 p_2` on the page, where the index it sits
/// at is a number nobody can check by eye.
pub fn set_monomial(series: &mut Series<f64>, exponents: &[u32], value: f64) {
    let degree: u32 = exponents.iter().sum();
    let index = index_of(exponents, degree).expect("the fixture monomials are addressable");
    series
        .set_coefficient(degree, index, value)
        .expect("the fixture monomials are inside the series");
}

/// How many multi-indices a search of this width and bound visits.
///
/// The closed form 0007 gives for the integer points of the `L1` ball, less the
/// origin, halved because `k` and `-k` are one relation. Written here rather
/// than reached for in the crate, because a fixture that borrows the code it is
/// meant to check proves nothing: the library computes the same number to
/// decide whether to refuse a search, and this is what says that number is the
/// number of things the search actually visits.
pub fn candidates(freedoms: usize, bound: u32) -> u64 {
    let mut total: u128 = 0;
    for corner in 0..=freedoms.min(bound as usize) {
        total += (1u128 << corner)
            * binomial(freedoms as u128, corner as u128)
            * binomial(u128::from(bound), corner as u128);
    }
    u64::try_from((total - 1) / 2).expect("the fixture is only used at sizes that fit")
}

fn binomial(n: u128, k: u128) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result: u128 = 1;
    for step in 0..k {
        result = result * (n - step) / (step + 1);
    }
    result
}
