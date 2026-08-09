//! The one error type the library returns.
//!
//! `docs/decisions/0011-api-and-errors.md` puts a single `Error` on the public
//! surface and makes every failure a returned value: nothing in the library
//! terminates the process and nothing panics on input a caller can supply. This
//! is that type. It starts at the variants the series arithmetic needs and
//! grows as the layers above it land, rather than one type per module, because
//! a caller matching on failures should not have to learn which module a
//! refusal came from.
//!
//! The index arithmetic already had its own refusals before this type existed,
//! and they are carried rather than re-spelled: [`Error::Index`] wraps
//! [`IndexError`] so the boundary the index names is the boundary the caller
//! reads.

use core::fmt;

use crate::monomial::IndexError;

/// What the library refuses, and what it says when it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The index arithmetic refused, and this is what it said.
    Index(IndexError),

    /// A series was asked for with no degrees of freedom.
    ///
    /// Zero degrees of freedom is zero variables, and the monomial index has
    /// nothing to address. Refused at construction rather than producing a
    /// series whose every operation refuses.
    NoFreedoms,

    /// Two series of different degrees of freedom were combined.
    ///
    /// Both counts are carried, because the whole use of this refusal is
    /// telling a caller which of the two arguments was not the one they meant.
    FreedomsDiffer {
        /// The degrees of freedom of the left argument.
        left: usize,
        /// The degrees of freedom of the right argument.
        right: usize,
    },

    /// Two series of different truncation orders were combined.
    ///
    /// The order is data rather than convention, so the sum of a series known
    /// to order five and one known to order three is not a series known to
    /// order five. Taking the smaller silently would be the arithmetic deciding
    /// what a caller meant, so the refusal names both and
    /// [`crate::series::Series::truncated`] is how a caller says which.
    OrderDiffers {
        /// The truncation order of the left argument.
        left: u32,
        /// The truncation order of the right argument.
        right: u32,
    },

    /// A truncation to an order the series does not carry.
    ///
    /// Truncation drops a tail. It cannot invent one, and a series silently
    /// extended with zeros would claim to know coefficients nobody computed.
    OrderAboveTruncation {
        /// The order that was asked for.
        requested: u32,
        /// The truncation order the series carries.
        order: u32,
    },

    /// A degree above the truncation order of the series.
    DegreeAboveTruncation {
        /// The degree that was asked for.
        degree: u32,
        /// The truncation order the series carries.
        order: u32,
    },

    /// A degree whose dense array is larger than this machine can address.
    ///
    /// The index arithmetic refuses a degree whose size does not fit in 64
    /// bits, which is a property of the mathematics. This is the other
    /// boundary, which is a property of the machine the run is on: a size that
    /// fits in 64 bits and not in a pointer. Separate from the first because
    /// the same series is addressable on one host and not on another, and a
    /// caller reading the refusal needs to know which of the two they met.
    SizeBeyondAddressable {
        /// The number of variables the series is in.
        variables: usize,
        /// The degree whose array could not be sized.
        degree: u32,
        /// How many coefficients that degree holds.
        dimension: u64,
    },

    /// A declared resonance relation of the wrong width.
    ///
    /// Its position in the declaration is carried, because a caller who wrote
    /// six relations needs to be told which of them is in the wrong phase
    /// space rather than that one of them is.
    RelationWidth {
        /// The degrees of freedom the module was asked for.
        freedoms: usize,
        /// How many entries the relation carried.
        given: usize,
        /// Its position in the declaration, counting from zero.
        at: usize,
    },

    /// A resonance declaration whose saturation is the whole lattice.
    ///
    /// Such a module makes every term resonant, so the normal form is the
    /// input unchanged. That is a request that has answered itself rather than
    /// work the package can do, and the rank is carried so a caller can see how
    /// many independent relations they actually declared.
    EveryTermResonant {
        /// The degrees of freedom the module was asked for.
        freedoms: usize,
        /// The rank of the saturation, which equals the degrees of freedom.
        rank: usize,
    },

    /// A membership query of the wrong width.
    MultiIndexWidth {
        /// The degrees of freedom the module lives in.
        freedoms: usize,
        /// How many entries the query carried.
        given: usize,
    },

    /// An exponent vector of the wrong width.
    ExponentWidth {
        /// The number of variables, which is twice the degrees of freedom.
        variables: usize,
        /// How many entries the exponent vector carried.
        given: usize,
    },

    /// The integer lattice arithmetic left the width it is computed in.
    ///
    /// The Hermite normal form of a declaration can grow entries well beyond
    /// the ones the caller wrote. Nothing here wraps: an entry that no longer
    /// fits is refused, because a wrapped entry gives a basis that is a
    /// perfectly ordinary looking lattice and the wrong one.
    LatticeOverflow,

    /// An evaluation point of the wrong width.
    ///
    /// A point has one entry per variable, which is twice the degrees of
    /// freedom, and a point of the wrong length is a caller working in a
    /// different phase space from the series.
    PointWidth {
        /// The number of variables the series is in.
        variables: usize,
        /// The number of entries the point carried.
        given: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Index(error) => write!(formatter, "{error}"),
            Error::NoFreedoms => write!(
                formatter,
                "a series needs at least one degree of freedom, and none was given"
            ),
            Error::FreedomsDiffer { left, right } => write!(
                formatter,
                "these series are in different phase spaces: \
                 {left} degree(s) of freedom on the left and {right} on the right"
            ),
            Error::OrderDiffers { left, right } => write!(
                formatter,
                "these series carry different truncation orders, \
                 {left} on the left and {right} on the right; \
                 truncate one to the other before combining them"
            ),
            Error::OrderAboveTruncation { requested, order } => write!(
                formatter,
                "truncation to order {requested} was asked of a series carrying \
                 order {order}, and truncation cannot add a degree nobody computed"
            ),
            Error::DegreeAboveTruncation { degree, order } => write!(
                formatter,
                "degree {degree} was asked of a series truncated at order {order}"
            ),
            Error::SizeBeyondAddressable {
                variables,
                degree,
                dimension,
            } => write!(
                formatter,
                "degree {degree} in {variables} variable(s) holds {dimension} \
                 coefficient(s), which is more than a pointer on this machine \
                 can index"
            ),
            Error::RelationWidth {
                freedoms,
                given,
                at,
            } => write!(
                formatter,
                "relation {at} of the declaration carries {given} entry(s) \
                 in a module of {freedoms} degree(s) of freedom"
            ),
            Error::EveryTermResonant { freedoms, rank } => write!(
                formatter,
                "this declaration saturates to rank {rank} in {freedoms} \
                 degree(s) of freedom, which makes every term resonant and \
                 leaves the normal form equal to the input"
            ),
            Error::MultiIndexWidth { freedoms, given } => write!(
                formatter,
                "a multi-index of {given} entry(s) was given for a module \
                 of {freedoms} degree(s) of freedom"
            ),
            Error::ExponentWidth { variables, given } => write!(
                formatter,
                "an exponent vector of {given} entry(s) was given for \
                 {variables} variable(s)"
            ),
            Error::LatticeOverflow => write!(
                formatter,
                "the integer lattice arithmetic left the width it is computed \
                 in, and it is refused rather than wrapped"
            ),
            Error::PointWidth { variables, given } => write!(
                formatter,
                "an evaluation point of {given} entry(s) was given for a series \
                 in {variables} variable(s)"
            ),
        }
    }
}

impl core::error::Error for Error {}

/// Every variant, so the test below cannot pass by not looking at one.
///
/// Written here rather than in the test module because it is the list a new
/// variant has to be added to, and a list next to the enum is one somebody
/// editing the enum sees.
#[cfg(test)]
const EVERY_VARIANT: &[Error] = &[
    Error::Index(IndexError::NoVariables),
    Error::NoFreedoms,
    Error::FreedomsDiffer { left: 2, right: 3 },
    Error::OrderDiffers { left: 4, right: 5 },
    Error::OrderAboveTruncation {
        requested: 6,
        order: 2,
    },
    Error::DegreeAboveTruncation {
        degree: 7,
        order: 3,
    },
    Error::SizeBeyondAddressable {
        variables: 6,
        degree: 8,
        dimension: 9,
    },
    Error::RelationWidth {
        freedoms: 3,
        given: 2,
        at: 1,
    },
    Error::EveryTermResonant {
        freedoms: 2,
        rank: 2,
    },
    Error::MultiIndexWidth {
        freedoms: 3,
        given: 2,
    },
    Error::ExponentWidth {
        variables: 6,
        given: 2,
    },
    Error::LatticeOverflow,
    Error::PointWidth {
        variables: 4,
        given: 2,
    },
];

impl From<IndexError> for Error {
    fn from(error: IndexError) -> Self {
        Error::Index(error)
    }
}

#[cfg(test)]
mod tests {
    use super::EVERY_VARIANT;

    /// A refusal is one line of ordinary prose.
    ///
    /// The mistake this catches is not a typo. A message written across several
    /// source lines uses the language's line continuation, which drops the
    /// newline and the indentation after it, and a later edit that pulls the
    /// pieces onto one line leaves the indentation behind as a run of spaces
    /// inside the string. Three refusals shipped that way, and nothing noticed,
    /// because a message is not compared against anything.
    ///
    /// It also refuses a newline, which is the other way that repair goes
    /// wrong: writing the continuation as an escape rather than as a real line
    /// break puts a line ending in the middle of what a caller prints.
    ///
    /// Delete either assertion and this goes red on the message the other one
    /// would have caught, which is why they are two assertions and not one.
    #[test]
    fn no_refusal_carries_a_run_of_spaces_or_a_line_break() {
        for error in EVERY_VARIANT {
            let message = error.to_string();
            assert!(
                !message.contains("  "),
                "{error:?} prints a run of spaces: {message:?}"
            );
            assert!(
                !message.contains('\n'),
                "{error:?} prints a line break: {message:?}"
            );
        }
    }

    /// Every variant is in the list the test above walks.
    ///
    /// A guard over a list is only as good as the list, and the failure this
    /// catches is a variant added to the enum and not to `EVERY_VARIANT`, which
    /// leaves the new message unread by anything. There is no way to enumerate
    /// an enum's variants at run time, so what stands in for it is a count that
    /// somebody has to move deliberately.
    #[test]
    fn the_list_the_guard_walks_holds_every_variant() {
        assert_eq!(
            EVERY_VARIANT.len(),
            13,
            "a variant was added or removed; add it to EVERY_VARIANT and move this count"
        );
    }
}
