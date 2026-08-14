# 0002. What a coefficient is, and what each type licenses

Status: decided. Raised in issue #4.

## The decision

One coefficient abstraction with three implementations behind it. The
implementation that produced a result is part of the result, carried by the type
rather than by a note somebody has to remember to write.

The three are machine floating point, an exact rational, and an outward rounded
interval. Machine floating point is the default and is the only one the
throughput path uses.

## What each type licenses, in one sentence each

These are the sentences the package is allowed to say about a result, and they
are the reason the decision exists at all.

**Machine floating point.** From a result computed in machine floating point the
package may claim that the coefficients are the ones the stated algorithm
produces in IEEE 754 binary64 arithmetic in the order 0009 fixes, and nothing at
all about how far any of them is from the exact value.

**Exact rational.** From a result computed in the exact rational type the package
may claim that every coefficient is the exact value the algorithm defines, with
no rounding anywhere in the computation, on the input as given.

**Interval.** From a result computed in the interval type the package may claim
that the exact value of each coefficient lies inside the returned enclosure,
because every arithmetic step was rounded outward.

The three sentences are different claims about different things and the package
never substitutes one for another. 0008 is where that is turned into a rule with
teeth: the rigorous estimate class is constructible only where the coefficient
parameter is the interval type, so a bound cannot be produced from a floating
point run by any route.

## The three implementations

**Machine floating point** is IEEE 754 binary64. It is the default because the
job that fixes the shape of this package is a normal form to order ten in six
variables, that job is bounded by arithmetic throughput, and binary64 is what a
machine does at full speed. Eight bytes per coefficient, which is the number the
memory ceiling arithmetic of 0009 needs.

**The exact rational** is an arbitrary precision rational, used for the oracle
the algebra is tested against and for small cases where somebody wants the exact
answer. It is never used in the throughput path. A coefficient has no fixed
width, so the memory ceiling arithmetic of 0009 cannot be evaluated ahead of a
group for this type from a width alone; what it uses instead is the measured
width of the live coefficients at the end of the previous group, and the refusal
is made on that. That is a weaker prediction than the fixed width case and it is
stated here rather than discovered when a rational run is killed.

The implementation of the rational is not chosen here. Several of the fastest are
bindings onto a copyleft C library and carry that library's license, which
interacts with the license of this repository. That coupling is an entry in issue
#2 and no work depending on the answer starts before the answer is written there.
What this document fixes is that the type exists, what it is for, and that it is
absent from the throughput path.

**The interval** is a pair of binary64 endpoints, lower and upper, with every
operation rounded outward so the enclosure is never narrower than the truth.
Sixteen bytes per coefficient. It is written as a pair of endpoints and never as
a midpoint with a radius, and 0010 carries the same rule for the file, so that no
reader can silently narrow an enclosure by rounding a midpoint.

Outward rounding is implemented by computing each operation in the ambient
rounding mode and then widening each endpoint by one representable step, downward
on the lower endpoint and upward on the upper. That is sound, because the true
result of a single binary64 operation is within one step of the computed one, and
it does not require the rounding mode to be changed, which the language does not
offer on a stable toolchain. It costs at most one step of extra width per
operation against a hardware directed rounding implementation, and that cost is
paid here so the package needs no assembly and no per-platform rounding mode
handling. The step exists on the pinned toolchain of 0001. Save this as
`nextup.rs` to reproduce it:

    fn main() {
        let a: f64 = 1.0;
        println!("next_up(1.0)   = {:?}", a.next_up());
        println!("next_down(1.0) = {:?}", a.next_down());
    }

    $ rustc --edition 2024 -O nextup.rs -o nextup.exe && ./nextup.exe
    next_up(1.0)   = 1.0000000000000002
    next_down(1.0) = 0.9999999999999999

Widening by one step in each direction is the whole mechanism, so the property
that has to be proved is not that the arithmetic is clever but that the widening
is never skipped. See below.

## The rule that a result is labelled

Every series, every normal form and every estimate carries its coefficient type
as part of its type, and the file written for it names the type in a required
field, which is 0010. There is no build-time flag that switches the coefficient
of an already built result, and there is no conversion that changes the label
without changing the numbers.

A result computed in floating point is reported as an estimate and is refused the
word rigorous. That refusal is by the type rather than by a convention: 0008
makes the rigorous estimate variant constructible only in the interval
specialisation, so the sentence cannot be produced from a floating point run even
by a caller who wants it.

Where a conversion between types exists it is explicit, it is named for what it
does, and it never improves a claim. Rounding an exact rational result to
floating point is available and produces a floating point result carrying the
floating point sentence. Enclosing a floating point coefficient in a degenerate
interval is deliberately not available, because a degenerate interval around
a rounded number is an enclosure of nothing, and offering it would be a route
from the weakest claim to the strongest one.

## The generic surface, and why it stays small

The core arithmetic is generic over the coefficient. That costs compile time, and
the response is to keep the generic surface small rather than to accept the cost.

The abstraction carries addition, subtraction, multiplication, negation, zero,
one, and construction from a small integer. That is what the series arithmetic of
0003 and the recursion of 0005 are built from, and it is the whole list.

Division is not on the abstraction. It appears in exactly one place, the
solution of the homological equation, where the divisor is a small divisor and
where each type behaves differently: floating point and rational apply the
threshold of 0006, and the interval refuses when the enclosure contains zero.
Putting division on the shared surface would make those three behaviours one
call site with a hidden branch, which is where a package quietly divides by
something it should have refused.

Ordering is not on the abstraction either. Comparisons on an interval are
partial, so a shared total order would have to lie for one of the three. Anything
that needs a comparison needs it on a magnitude, and a magnitude of an interval
is an interval, so those operations live on the types that have them and not on
the abstraction.

## The proofs these types owe

**Outward rounding is verified rather than assumed.** The near miss to aim at is
the one somebody will actually write: an operation that widens one endpoint and
forgets the other, or one that widens in the wrong direction on a negative
operand. So the test is not that the enclosure contains the exact answer on an
easy case. It is a case where the naive implementation, the one that computes
both endpoints in the ambient mode and widens neither, produces an enclosure that
excludes the exact value, and the shipped implementation does not. Products of
operands of mixed sign are where that bites, because the endpoints of the product
do not come from the endpoints of the operands in a fixed pairing.

**The exact type is the oracle and is checked against something other than
itself.** A test that compares the rational path to the floating point path to a
tolerance cannot distinguish an algebraic error from accumulated rounding, which
is the failure this whole decision is aimed at. So the rational path is checked
against identities that hold exactly, which is issue #30 for the bracket, and
against published cases, which is issue #38.

Both obligations belong to the issues that build the code. This document is where
they are named, so that neither arrives as a surprise.

## Why not fewer types

One type only, floating point, is the shape of every in-house code this package
exists to replace, and it is why those codes cannot state a domain of validity.
It would have cost the milestone that justifies the project.

One type only, exact, is unusable past low order. Rational coefficients grow in
size as the recursion proceeds, so the arithmetic slows down as the run
continues, and the target case is unreachable by a wide margin.

Two types, floating point and exact, gives correct tests and no rigorous bound.
It is the tempting middle, because it removes the hardest of the three
implementations and keeps the one that catches algebra errors. It loses the
statement that no in-house code can make, which is the reason this package is
worth writing.

## The alternatives, and what each would have cost

**A single coefficient type chosen at build time by a feature flag.** The core
carries no generic parameter at all, and a build produces one package for one
type. It would have cost the label. A result would carry no evidence of which
build produced it, two builds would produce files that look identical and mean
different things, and 0008's rule that a rigorous bound is reachable only from
intervals would become a claim about how somebody configured their build rather
than a property of a type. It is also worse in practice: the oracle and the
throughput path have to run in the same process for a test to compare them.

**Arbitrary precision floating point with a caller chosen mantissa width, instead
of intervals.** One type, tunable, and it covers the oracle case and the fast
case at their extremes. It would have cost the only claim that matters. More
digits is not a bound. A computation in a thousand bits that is not tracking its
own error is exactly as unable to say where the remainder lies as one in
fifty-three bits, and the package would have gained a slow path that still could
not use the word rigorous.

**A dynamically typed coefficient, one enum with three variants, dispatched at
run time.** No generic parameter, no compile time cost, and the label is carried
as data. It would have cost the inner loop and the guarantee. Every coefficient
operation becomes a branch, in the kernel that is the whole cost model of 0003,
and the label becomes a value that a mixed series could disagree with itself
about. Making the rigorous class unreachable from a floating point run would then
be a run time check somebody has to get right everywhere rather than something
the type refuses.

**A residue number or fixed point representation for exactness without growth.**
It would have cost the mathematics. The coefficients here are not bounded, the
recursion divides, and a representation that cannot divide or cannot grow is not
an oracle for this algorithm.

## What it costs, stated rather than skipped

Compile time on a generic numeric core is unpleasant and the small surface above
is the mitigation rather than a cure.

Interval arithmetic is not a drop-in and this document does not pretend it is.
Comparisons are partial. Division by an interval containing zero has no answer.
The small divisors this field is full of are exactly the intervals that will
contain zero, so the interval path will refuse cases the floating point path
appears to handle. That is not a defect of the choice. It is the honest form of a
problem the floating point path hides, and 0006 is where the response is decided.

Interval width grows through a long recursion, so some cases will produce a
numerical estimate and no bound at all. 0008 already says the package reports
that rather than degrading quietly into the weaker class.

Three implementations is three times the surface for the arithmetic tests. The
mitigation is that the abstraction is seven operations wide, so the shared test
set is written once against the abstraction and instantiated three times, which
is issue #31.

## What this document does not decide

It does not decide the storage layout or the monomial order, which is 0003. It
does not decide which rational implementation is used, which waits on issue #2.
It does not decide what happens when a divisor is small, which is 0006. It does
not decide what an estimate may claim, which is 0008, though the three sentences
above are what that document's classes rest on. It does not decide the on-disk
spelling of any of the three, which is 0010.
