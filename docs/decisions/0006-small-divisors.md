# 0006. What happens when a divisor is small

Status: decided. Raised in issue #8.

## The decision

Fail closed. The solver refuses rather than divides when a divisor is small, and
the threshold below which it refuses is part of the problem statement with a
default derived from the frequency vector rather than from a bare constant.

Exact zero is not a separate case. It is the limiting case of small, and treating
it separately is how a code ends up with a tolerance nobody chose.

## The divisor

The conventions are 0004 and are not restated. In the complex variables of 0004
item 5, a monomial is `x^a y^b` with `a` and `b` non-negative integer vectors in
`Z^v`, and the homological operator acts on it as a scalar with the divisor

    d(k) = <k, omega> = sum over j = 1..v of k_j * omega_j        where k = a - b

Solving the homological equation at group `d` divides the coefficient of each
monomial by that divisor, so a small `d(k)` multiplies that coefficient by a
large number and the generator, which is what the remainder bound of 0008 is
built from, silently becomes large.

Terms with `k = 0` are never divided by. They are the kernel of the homological
operator for every frequency vector and they are retained in the normal form, so
this document is about `k` non-zero only.

## The relative divisor, which is what the threshold is applied to

A threshold on `|d(k)|` alone would be a bare constant carrying the physical
dimension of a frequency, so it would mean different things for the same system
described in different units and it would have to be re-chosen for every problem.
The threshold is applied instead to a dimensionless quantity:

    rho(k) = |<k, omega>| / ( |k|_1 * ||omega||_inf )

with `|k|_1 = sum over j of |k_j|` and `||omega||_inf = max over j of |omega_j|`.

Three properties fix why this and not another scaling.

**It lies in `[0, 1]`.** By the triangle inequality
`|<k,omega>| <= sum |k_j| |omega_j| <= ||omega||_inf * |k|_1`, so `rho <= 1`, and
it is non-negative by construction. It reaches 1 exactly when no cancellation
occurred: every `j` with `k_j` non-zero has `|omega_j| = ||omega||_inf` and every
term of the sum carries the same sign.

**It measures cancellation and nothing else.** `|k|_1 * ||omega||_inf` is the
largest the inner product could have been for that multi-index and that frequency
vector, so `rho(k)` is the fraction of that size which survived. Being near a
resonance is exactly cancellation in this sum, so `rho` is small precisely in the
cases the refusal exists for.

**It is invariant under rescaling the frequencies.** Replacing `omega` by
`c * omega` for any non-zero real `c` multiplies numerator and denominator by
`|c|` and leaves `rho` unchanged. A rescaling of the frequency vector is a
rescaling of time, so a threshold that moved under it would refuse different
terms for the same physics described in different units. An absolute threshold on
`|d(k)|` does exactly that, and that is the alternative this rules out.

The bound and the attainment of 1 are checked exactly over the rationals rather
than asserted, by the block at the end of this document.

## The default threshold

The solver refuses the term when

    rho(k) < theta          with the default        theta = 2^-32 = 2.328306e-10

**Where the lower constraint comes from.** Below some level the computed relative
divisor carries no correct digit at all, and a threshold beneath that level is
not a policy, it is arithmetic noise. Computing `<k, omega>` in binary64 in the
natural order has an error bounded by `gamma_v * sum |k_j omega_j|`, with
`gamma_v = v u / (1 - v u)` and `u = 2^-53` the unit roundoff, so the computed
`rho` has an absolute uncertainty of about `gamma_v`. The default sits well above
that, and the block at the end of this document prints the margin at one, two,
three and six degrees of freedom:

    v=6  gamma_v = v*u/(1-v*u) = 6.661338e-16   theta/gamma_v = 3.495253e+05

so at six degrees of freedom the default is about `3.5e5` times the level at
which `rho` stops being a number. That is the margin, and it is the reason for
the exponent.

**Where the upper constraint comes from, and why the default is small rather than
comfortable.** The threshold's job is to refuse a division whose result is
meaningless, not to decide whether a bound is good enough. The second judgement
is already made visible: 0008 requires every result to carry an estimate stating
the value, the optimal order, and whether the order reached is past that optimum,
so a moderately small divisor degrades a number the user can see rather than one
they cannot. Raising the threshold to protect the bound would hide cases the
estimate would have reported honestly, and would refuse work the user is entitled
to do. So the default is set as low as the first constraint allows and no lower.

What that admits, stated so it is not a surprise. At the default the amplification
applied to a single coefficient is bounded by

    1 / |d(k)| <= 4.294967e+09 / ( |k|_1 * ||omega||_inf )

which is a large number. A run that divides by something near the threshold
produces a generator with a very large coefficient, and the estimate says so.

**The constant is a judgement, not a measurement.** The scaling in `rho` is
derived above and the lower constraint is arithmetic, but the choice of `2^-32`
rather than `2^-28` or `2^-36` inside the admissible range is an argument and not
a number produced by a command. Nothing in this repository has measured the
distribution of `rho` over cases people actually run, because no case has been
run. What would move it is that measurement, over the published cases reproduced
in issues #38 and #42, and moving it is a change to this document with the
measurement quoted.

## The threshold is global, not per order

One threshold is in force for a whole run and it is applied to every term at
every order.

A per-order threshold would make the same monomial admissible at one order and
refused at another, for no reason connected to the monomial or to the frequency
vector, both of which are fixed for the run. It would also turn the hypothesis
that 0008 requires an estimate to carry from one number into a vector of numbers,
and a reader comparing two results would have to compare two vectors before
knowing whether the results are comparable at all.

`rho` already contains the only order-dependence that is real: `|k|_1` grows with
the degree, so a divisor of a given absolute size is judged against a larger
denominator at higher order. That is the correct direction and it comes out of
the scaling rather than out of a schedule.

## The refusal, and the three responses

A refusal is an error returned as a value, in the sense of 0011. It is not a
warning written to a log nobody reads, and it does not return a number with a
note attached.

The error names, for the first term that fell below the threshold:

- the degree of the group and the grevlex index of the monomial within it, which
  together identify the term in the layout of 0003,
- the multi-index `k = a - b`,
- the divisor `<k, omega>`,
- the relative divisor `rho(k)`,
- the threshold `theta` in force and whether it is the default.

The error then names the three responses that are available, because a user who
meets this for the first time is arriving from a code that never refused
anything:

**Raise the threshold deliberately.** The caller sets a smaller `theta`, accepts
the amplification, and the result records that the threshold was not the default.
This is the response when the user has looked at `rho(k)` and judged the case
well enough separated for what they want to claim.

**Declare the resonance and normalise with those terms kept.** The caller adds
`k` to the resonance module of 0007, which moves the term out of the range of the
homological operator and into the normal form, so it is retained rather than
divided by. This is the response when the near resonance is the physics.

**Stop at the previous order.** The caller takes the normal form to the last
order that completed. The remainder estimate at that order is a real number about
a real object, and 0008's optimal truncation order often says that this is the
better answer anyway.

## What the result records

The result records the threshold in force and whether it is the default, and it
records the smallest relative divisor that was actually divided by during the run
together with its multi-index and the group it occurred in.

Those are three items rather than a list of every division, because a list grows
with the run and the worst case is what a reader needs. A reader who sees a
worst-case `rho` just above the threshold knows the run was close to a refusal
without reading anything else.

Both reach any remainder estimate computed from the result, because 0008 requires
the estimate object to carry every hypothesis assumed, including the small
divisor threshold in force. That is what stops a bound obtained by declaring a
near resonance harmless from being read as a bound on a well separated case, and
0008 says so in the same words.

0010 records all of it in the file, so the same reading is available a year later
from the file alone.

## What each coefficient type does

**Binary64.** As above. The threshold may be lowered by the caller but not below
`gamma_v`, and a request below that floor is an error naming the floor and the
degrees of freedom it was computed at. Beneath it the comparison is being made
against a quantity with no correct digits, so a threshold there would be a
refusal policy driven by rounding noise.

**Exact rational.** The divisor is computed exactly, so the lower constraint
above does not apply, and `theta = 0` is accepted. At `theta = 0` the solver
refuses exactly the terms with `d(k) = 0`, which is the exact kernel and nothing
else. The default is still `2^-32` for the other reason: an exact rational run of
a near-resonant case produces an exactly correct and enormous generator, and the
default is a policy about amplification rather than about rounding.

**Interval.** No threshold is applied at all. The divisor is an interval, and the
solver refuses exactly when that interval contains zero, because an interval
containing zero cannot be divided by and there is no number the division could
return. That is not a special case worked around. It is the correct answer
arriving from the arithmetic instead of from a constant somebody chose, and it is
one of the reasons the interval path is worth having.

An interval divisor that is narrow but excludes zero is divided by, and the
enclosure of the quotient is correspondingly wide. The width is the honest signal
and it propagates into the bound of 0008 without anybody having to notice it,
which is the behaviour the threshold in the other two types is approximating.

The error for the interval case names the same term and multi-index, and the
enclosure of the divisor in place of a value and a threshold.

## What it costs, stated rather than skipped

Users arriving from codes that never refuse will meet a refusal on cases those
codes appeared to handle. The documentation says plainly that the earlier answer
was not better, it was unchecked, and that the number that code returned was the
same number this one refuses to return without being told to. That is a support
cost and it is the correct one to pay.

The interval path will refuse more often than the other two, because an enclosure
that contains zero is a weaker condition than a magnitude below a threshold. Some
cases will therefore produce a numerical estimate and no rigorous bound, which is
already what 0008 says happens and is reported rather than degraded quietly.

Recording the worst relative divisor costs one comparison per division in the
inner loop of the solver. It is a scalar comparison against a running minimum in
a loop that is already doing a division, and it is paid so that the result can be
read without the run being repeated.

## The numbers in this document

Every number quoted above comes out of the block below. The `rho` check is exact,
over the rationals, with no floating point in it: it draws a frequency vector of
rational entries and an integer multi-index at random, forms `rho` as a rational,
and asserts the bound. The seed is fixed so the line reproduces. Paste it:

    python - <<'EOF'
    from fractions import Fraction as F
    import random

    u = 2.0 ** -53
    theta = 2.0 ** -32

    print(f"unit roundoff u = 2^-53 = {u:.6e}")
    for v in (1, 2, 3, 6):
        g = v * u / (1 - v * u)
        print(f"v={v}  gamma_v = v*u/(1-v*u) = {g:.6e}   theta/gamma_v = {theta/g:.6e}")
    print(f"theta = 2^-32 = {theta:.6e}")
    print(f"max amplification 1/(theta * |k|_1 * ||w||_inf) = {1.0/theta:.6e} / (|k|_1 * ||w||_inf)")

    bad = 0; one = 0
    random.seed(20260808)
    for _ in range(200000):
        v = random.randint(1, 6)
        w = [F(random.randint(-40, 40), random.randint(1, 17)) for _ in range(v)]
        k = [random.randint(-6, 6) for _ in range(v)]
        if all(x == 0 for x in k) or all(x == 0 for x in w):
            continue
        num = abs(sum(F(k[j]) * w[j] for j in range(v)))
        den = sum(abs(x) for x in k) * max(abs(x) for x in w)
        if den == 0:
            continue
        rho = num / den
        if not (F(0) <= rho <= F(1)):
            bad += 1
        if rho == 1:
            one += 1
    print(f"exact rho in [0,1] violations over 200000 random (v, omega, k): {bad}; rho == 1 attained {one} times")
    EOF

    unit roundoff u = 2^-53 = 1.110223e-16
    v=1  gamma_v = v*u/(1-v*u) = 1.110223e-16   theta/gamma_v = 2.097152e+06
    v=2  gamma_v = v*u/(1-v*u) = 2.220446e-16   theta/gamma_v = 1.048576e+06
    v=3  gamma_v = v*u/(1-v*u) = 3.330669e-16   theta/gamma_v = 6.990507e+05
    v=6  gamma_v = v*u/(1-v*u) = 6.661338e-16   theta/gamma_v = 3.495253e+05
    theta = 2^-32 = 2.328306e-10
    max amplification 1/(theta * |k|_1 * ||w||_inf) = 4.294967e+09 / (|k|_1 * ||w||_inf)
    exact rho in [0,1] violations over 200000 random (v, omega, k): 0; rho == 1 attained 33007 times

That is a check on this document rather than on the package. A random search
cannot prove the bound, which is why the bound is derived above and the search is
there to catch a derivation that is wrong rather than to stand in for one. The
obligation to prove the refusal bites in the shipped solver belongs to issue #35.

## What this document does not decide

It does not decide the conventions the divisor is written in, which is 0004. It
does not decide the coefficient types, which is 0002. It does not decide how a
resonance is declared or detected, which is 0007, though declaring one is the
second of the three responses above. It does not decide what the estimate
carrying the threshold may claim, which is 0008. It does not decide the file
fields that record any of it, which is 0010. It does not build the solver, which
is issue #35 in the non-resonant case and issue #41 on a resonant subspace.
