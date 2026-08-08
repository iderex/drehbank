# 0008. What a remainder estimate is allowed to claim

Status: decided. Raised in issue #11.

## Why this is decided before anything is computed

A package that computes a normal form and says nothing about where it stops being
valid has rebuilt the one-off it was meant to replace. So what the package is
allowed to say about validity is defined here, before any of it is computed,
because otherwise the definition gets reverse engineered from whatever the code
happens to produce.

Three different statements get called an estimate in this field, and they are not
interchangeable. They are three types here, and the package refuses to print one
as another.

## The three classes, and the exact statement each licenses

Write `R` for the remainder, `N` for the truncation order, `r` for the radius
vector of a polydisc `P(r) = { z : |z_j| <= r_j for every j }`, and `H` for the
input Hamiltonian.

**Formal.** The statement licensed is exactly this and no more:

> Every term of `R` has total degree greater than `N`.

True by construction, says nothing about any domain, and is what most codes
report if they report anything. It carries no number, no radius and no
hypotheses, and it is a legitimate answer when nothing better was computed. What
it may not do is appear next to a number, because a number next to it will be
read as a size.

**Numerical estimate.** The statement licensed is exactly this:

> Computed in floating point arithmetic without directed rounding, the majorant
> norm of the truncated remainder on `P(r)` is approximately `E`.

Two words in that sentence carry the whole difference from the class below.
Approximately, because the rounding of the arithmetic that produced `E` was not
controlled. And truncated, because `E` is computed from the coefficients that
were kept, so it is an estimate of the tail that was computed and not of the tail
that exists. It is useful, it is honest when labelled, and it is not a bound. It
may not be described as a bound, an upper bound, a limit, a guarantee or a worst
case in any output the package produces.

**Rigorous bound.** The statement licensed is exactly this:

> For every `z` in `P(r)`, `|R(z)| <= B`, under the hypotheses listed with the
> estimate, with every arithmetic step in the computation of `B` rounded outward.

This is the one that can be used in a proof, and it is what an in-house code
never has. It is reachable only from the interval coefficient type, and the rule
below has no exception.

## The norm and the weight

The norm is the majorant norm on the polydisc. For a polynomial
`f = sum over multi-indices k of c_k z^k`,

    ||f||_r = sum over k of |c_k| * r^k

where `r^k` is the product of `r_i^(k_i)`. The weight attached to a monomial is
`r^k`, so the weight attached to a homogeneous piece of degree `d` under a
uniform radius `r` is `r^d`. That is the whole weight rule, and it is chosen
because it is the one norm on truncated series that is both submultiplicative and
directly comparable to the supremum:

    sup over z in P(r) of |f(z)| <= ||f||_r

which follows term by term from `|z^k| <= r^k` on `P(r)`, and

    ||f g||_r <= ||f||_r ||g||_r

which follows from `|sum of products| <= sum of |products|` on the coefficients of
the product. Both are why a bound stated in this norm converts into a statement
about values without a second argument.

## The majorant inequality for the Poisson bracket

The whole recursion in 0005 is brackets, additions and scalings, so one
inequality carries the estimate through it.

**Statement.** Let `f` be homogeneous of degree `a` and `g` homogeneous of degree
`b` in `2v` variables, and let

    rho = min over j = 1..v of ( r_(q_j) * r_(p_j) )

be the smallest radius product over conjugate pairs. Then

    ||{f, g}||_r <= (a * b / rho) * ||f||_r * ||g||_r

**Where the constant comes from.** It is derived here rather than cited, in three
steps that are short enough to check by reading. The published derivations of
this kind of bound differ in their constants, and a package that took one
inequality from one derivation and a second from another would produce a number
that is a bound under neither. So this document derives every constant it uses,
and the derivation is the source.

Step one, an exact identity. For `f` homogeneous of degree `a`,

    sum over all 2v variables z_i of ( r_i * ||df/dz_i||_r ) = a * ||f||_r

because `r_i ||df/dz_i||_r = sum over k of k_i |c_k| r^k`, and summing that over
`i` replaces `k_i` by `sum_i k_i = a`.

Step two, the bracket term by term. From the definition of the bracket, the
triangle inequality, and submultiplicativity,

    ||{f, g}||_r <= sum over j of ( ||df/dq_j||_r ||dg/dp_j||_r
                                  + ||df/dp_j||_r ||dg/dq_j||_r )

Step three, insert the radii. Multiplying and dividing each product by
`r_(q_j) r_(p_j)` and bounding that below by `rho` gives

    ||{f, g}||_r <= (1/rho) * sum over j of ( X_j Y'_j + X'_j Y_j )

where `X_j = r_(q_j)||df/dq_j||_r`, `X'_j = r_(p_j)||df/dp_j||_r`, and `Y_j`,
`Y'_j` are the same for `g`. Every one of those quantities is non-negative, and
the sum on the right is a subset of the terms in the expansion of

    ( sum over j of (X_j + X'_j) ) * ( sum over j of (Y_j + Y'_j) )

which by step one is `a ||f||_r * b ||g||_r`. That gives the statement.

**It cannot be improved as stated.** Take `v = 1`, `r = (1, 1)`, `f = q^2/2`,
`g = p^2/2`. Then `a = b = 2`, `rho = 1`, `{f, g} = q p`, so `||{f,g}||_r = 1`,
and the right-hand side is `4 * (1/2) * (1/2) = 1`. Equality, so no smaller
constant works for every homogeneous pair.

## How the bound reaches the remainder

The bound is not a closed form. It is the same recursion as 0005, run on norms
instead of on coefficients.

Every entry of the Deprit triangle is built from entries below it by brackets,
additions and scalings. Replacing each entry by an upper bound on its majorant
norm, each bracket by the inequality above, each addition by the triangle
inequality and each scaling by the absolute value of the scalar gives an upper
bound on the norm of every triangle entry, computed by the same sweep in the same
order. The generator pieces enter through the solution of the equation that
closes each group, so a bound on the generator needs a lower bound on the
homological operator, which is the small divisor, and that is 0006 rather than
this document.

The remainder after normalising to degree `N` is bounded by the sum of the
majorant norms of the pieces that were not normalised, on the stated polydisc.
Because the norm dominates the supremum, that sum is a bound on `|R(z)|` for
every `z` in the polydisc.

Two consequences worth stating, because both change what the number means. The
bound depends on `r` through `rho` and through `r^d`, so it is a statement about
one polydisc and never about a neighbourhood in general. And it is monotone in
nothing: as `N` grows the discarded tail starts later but the factors `a b / rho`
accumulate, so the bound first falls and then rises.

## The optimal truncation order

Because the bound is not monotone in `N`, a bound reported at a fixed order says
nothing on its own about whether the method was being used well. So every
rigorous or numerical estimate reports, alongside the value at the order that was
computed:

- the order at which the bound is smallest on the given polydisc,
- the value of the bound at that order,
- whether the reported order is at, before or past that optimum.

Reporting a bound at a fixed order without saying whether that order was past the
optimum is how a result gets read as worse than the method is, and this is the
rule that prevents it.

## The estimate object

Every normal form result carries an estimate object. The object states

- which of the three classes it is,
- the polydisc, as an explicit radius vector, never a scalar and never a default
  left unnamed,
- the truncation order the value refers to,
- the value,
- the optimal order and the value there,
- the coefficient type the computation ran in,
- every hypothesis assumed, including the small divisor threshold in force, any
  resonance that was declared, and any resonance that was detected and not
  declared.

The small divisor threshold is in that list because raising it changes what the
number means, and a reader who cannot see the threshold cannot tell a bound on a
well separated case from a bound that was obtained by declaring a near
resonance harmless.

## The two rules that have no exception

**No path through the public surface returns a normal form without an estimate
object.** In the form the API review can check: the normal form type has one
public constructor, that constructor takes an estimate by value, the estimate
field is not an option and has no default, and no public function returns the
normal form type built any other way. An omitted estimate is therefore not a
state the type can be in, rather than a state the code is careful to avoid. The
review checks it by reading the type definition and the signature of every public
function that mentions the type, which is a finite list fixed in 0011.

**The rigorous class is reachable only from the interval coefficient type.** The
estimate type is parameterised by the coefficient type, the rigorous variant is
constructible only where that parameter is the interval type, and there is no
conversion, cast, flag, builder or feature that produces a rigorous estimate from
a computation that ran in any other type. Enforced by the type and not by a
convention, which means the review checks it by finding no constructor of the
rigorous variant outside the interval specialisation.

## What it costs, stated rather than skipped

The interval path is slower by a large factor, and its domain of applicability is
smaller, because interval width grows through a long recursion. Some cases will
produce a numerical estimate and no bound, and the package says so rather than
degrading quietly into the class above.

Requiring an explicit polydisc forces the user to choose one, which is a real
usability cost. The mitigation is a default derived from the input, and the
default is labelled as a default in the estimate object, so a reader can tell a
domain the user chose from one the package picked.

## What this document does not decide

It does not decide the coefficient types or the interval type, which is 0002. It
does not decide what happens when a divisor is small, which is 0006. It does not
decide how a resonance is declared or detected, which is 0007. It does not decide
the public signatures themselves, which is 0011. It does not name a published
derivation to compare its constant against; comparing this derivation with the
literature is part of reproducing published cases, which is issue #38.
