//! An exact complex coefficient, held here as a fixture.
//!
//! Item 8 of `docs/decisions/0004-conventions.md` states the eigenvalue of the
//! homological operator in the complex variables of item 5, where the quadratic
//! part is `H_2 = -i sum_j omega_j x_j y_j` and every divisor carries a factor
//! of `i`. Checking that statement against the shipped bracket needs a
//! coefficient in which `i` is exact, and the smallest such thing is the
//! Gaussian integers.
//!
//! **It is a fixture and not a fourth implementation**, on the same terms as
//! [`super::exact::Exact`]. 0002 names three coefficient types the package
//! ships and this is none of them: nothing here is reachable from the library
//! and nothing here is published. What it exists for is one comparison, against
//! a closed form that was derived and checked outside this package.
//!
//! The frequencies the fixture uses are integers, which keeps the whole
//! computation inside the ring: `omega_j` enters only through
//! `-i <a - b, omega>`, and an integer frequency vector makes every coefficient
//! in the case a Gaussian integer with nothing rounded and nothing approximated.

use drehbank_core::coefficient::Coefficient;

/// `real + imaginary * i`, exact within its width.
///
/// `i128` with checked arithmetic, for the reason
/// [`super::exact::Exact`] gives: a fixture that wraps makes a property fail
/// for a reason that has nothing to do with the thing under test, which is
/// worse than no fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gaussian {
    pub real: i128,
    pub imaginary: i128,
}

impl Gaussian {
    /// The real integer `value`.
    pub fn real(value: i128) -> Self {
        Gaussian {
            real: value,
            imaginary: 0,
        }
    }

    /// `value * i`.
    pub fn imaginary(value: i128) -> Self {
        Gaussian {
            real: 0,
            imaginary: value,
        }
    }
}

fn add(left: i128, right: i128) -> i128 {
    left.checked_add(right)
        .expect("the fixture ranges keep every sum inside i128")
}

fn subtract(left: i128, right: i128) -> i128 {
    left.checked_sub(right)
        .expect("the fixture ranges keep every difference inside i128")
}

fn multiply(left: i128, right: i128) -> i128 {
    left.checked_mul(right)
        .expect("the fixture ranges keep every product inside i128")
}

impl Coefficient for Gaussian {
    fn zero() -> Self {
        Gaussian::real(0)
    }

    fn one() -> Self {
        Gaussian::real(1)
    }

    fn from_small_integer(value: i32) -> Self {
        Gaussian::real(i128::from(value))
    }

    fn add(&self, other: &Self) -> Self {
        Gaussian {
            real: add(self.real, other.real),
            imaginary: add(self.imaginary, other.imaginary),
        }
    }

    fn subtract(&self, other: &Self) -> Self {
        Gaussian {
            real: subtract(self.real, other.real),
            imaginary: subtract(self.imaginary, other.imaginary),
        }
    }

    /// `(a + bi)(c + di) = (ac - bd) + (ad + bc) i`.
    fn multiply(&self, other: &Self) -> Self {
        Gaussian {
            real: subtract(
                multiply(self.real, other.real),
                multiply(self.imaginary, other.imaginary),
            ),
            imaginary: add(
                multiply(self.real, other.imaginary),
                multiply(self.imaginary, other.real),
            ),
        }
    }

    fn negate(&self) -> Self {
        Gaussian {
            real: subtract(0, self.real),
            imaginary: subtract(0, self.imaginary),
        }
    }
}
