# 0004. The canonical conventions, pinned in one place

Status: decided. Raised in issue #6.

This document is the only place any of the conventions below is fixed. Every
other document and every place in the code that depends on one references this
document by name and does not restate it. 0005 was written before this one and
declared its own convention in full; the declarations agree, and the check is at
the end of this document.

## 1. The variables and their order

Phase space has `v` degrees of freedom and `m = 2v` variables, ordered in two
blocks, positions first:

    z = (z_1, ..., z_m) = (q_1, ..., q_v, p_1, ..., p_v)

so `z_j = q_j` for `1 <= j <= v` and `z_(v+j) = p_j` for `1 <= j <= v`. The
conjugate partner of `z_j` is `z_(j+v)`, `v` places away, and never adjacent.

This ordering is the one the exponent vector of 0003 is written in. A monomial
exponent `a = (a_1, ..., a_m)` has `a_1` the exponent of `q_1` and `a_m` the
exponent of `p_v`, and the grevlex index of 0003 is computed on that vector in
that order. The monomial ordering and the variable ordering are one convention in
two documents, and changing either changes the index of every coefficient.

Blocked rather than interleaved, because the linear symplectic structure is then
a single block matrix that a person can read when it is printed, and because the
complexification of item 5 acts on the pairs `(z_j, z_(j+v))` which are easy to
address by an offset. The cost is locality: the partner of a variable is `v`
places away rather than adjacent, so the bracket and the complexification touch
two separated regions of the exponent vector. That is a small and measurable cost
against a large and unmeasurable one, which is a user comparing against a
published table and getting a sign wrong.

## 2. The symplectic form and the equations of motion

With `I` the `v` by `v` identity,

    J = [  0   I ]
        [ -I   0 ]

and the equations of motion are

    dz/dt = J grad H(z)

which written out is `dq_j/dt = dH/dp_j` and `dp_j/dt = -dH/dq_j`.

## 3. The Poisson bracket

    {f, g} = sum over j = 1..v of ( df/dq_j * dg/dp_j - df/dp_j * dg/dq_j )

equivalently `{f, g} = (grad f)^T J (grad g)`. With this sign, the evolution of a
function is `df/dt = {f, H}`, and in particular `dz/dt = {z, H}`.

The other convention in circulation is the negative of this one, under which
`df/dt = {H, f}`. Both appear in the literature this package's users arrive from.
Item 9 below says what converting from it costs.

Consequences fixed by this choice and used everywhere below: the bracket is
antisymmetric, `{f, g} = -{g, f}`; and for `f` homogeneous of degree `a` and `g`
homogeneous of degree `b`, `{f, g}` is homogeneous of degree `a + b - 2`.

## 4. The Lie operator and the generating function

The Lie operator of a generating function `chi` is

    L_chi f = {f, chi}

with the generating function in the **second** argument. The canonical
transformation is its exponential,

    exp(L_chi) f = f + {f, chi} + (1/2!) {{f, chi}, chi} + ...

and the inverse transformation is `exp(L_(-chi))`, which is the same series with
the sign of the generator reversed and no second implementation.

The generating function is expanded in homogeneous degree with no factorial and
no sign attached to the expansion:

    chi = chi_3 + chi_4 + chi_5 + ...

with `chi_d` homogeneous of degree `d`. The Hamiltonian is expanded the same way
about an equilibrium at the origin:

    H = H_2 + H_3 + H_4 + ...

There is no small parameter in the expansion. The grading is the polynomial
degree, and any factorial that appears in a recursion is a consequence of
restating a small-parameter recursion in that grading rather than a factor
carried in the definition of `chi`. 0005 carries that restatement, with the
factorial ratios that produce it.

The homological operator is `L_(H_2)` acting on the generator piece, and the
equation that closes group `d` is the one 0005 derives:

    K_(d+2) = R_(d+2) + (1/d) { H_2, B_d }

The `1/d` is derived there from the telescoping product down the group. It is not
a normalisation choice made here, and it is not an alternative spelling of a
factorial in the definition of the generator.

## 5. The complexification near an elliptic equilibrium

The complexification is a transformation the user applies explicitly. The normal
form does not perform it behind the user's back, so a user can always tell which
variables the output is expressed in, and the round trip through it is testable
on its own.

For each degree of freedom `j`, with `i` the imaginary unit:

    x_j = ( q_j + i p_j ) / sqrt(2)
    y_j = ( i q_j + p_j ) / sqrt(2)

and the inverse

    q_j = ( x_j - i y_j ) / sqrt(2)
    p_j = ( y_j - i x_j ) / sqrt(2)

The complex variables are stored in the same blocked order as the real ones,

    (x_1, ..., x_v, y_1, ..., y_v)

so the exponent vector and the grevlex index of 0003 apply unchanged, with `x_j`
in the slot `q_j` occupied and `y_j` in the slot `p_j` occupied.

`(x_j, y_j)` is a canonical pair under the bracket of item 3, with `x_j` in the
position slot and `y_j` in the momentum slot:

    {x_i, y_j} = 1 if i = j, and 0 otherwise
    {x_i, x_j} = {y_i, y_j} = 0 for all i, j

which is what makes the transformation canonical rather than merely invertible,
so the bracket in item 3 may be evaluated on the complex variables with the same
formula and the same sign.

The normalisation of the imaginary unit factor is the one that follows from the
map above and is not a separate choice: for each `j`,

    x_j y_j = i ( q_j^2 + p_j^2 ) / 2        equivalently        ( q_j^2 + p_j^2 ) / 2 = -i x_j y_j

The other normalisation in circulation swaps the roles of the two factors of `i`,
taking `y_j = ( q_j + i p_j ) / sqrt(2)` and `x_j = ( i q_j + p_j ) / sqrt(2)`,
which is this map with `x` and `y` exchanged and therefore flips the sign of
every divisor in item 8. Item 9 says what converting from it costs.

## 6. The quadratic part and the action variables

The quadratic part at an elliptic equilibrium is normalised so that the frequency
appears once and not halved:

    H_2 = sum over j = 1..v of ( omega_j / 2 ) * ( q_j^2 + p_j^2 )

so `omega_j` is the angular frequency of the linear oscillation in the `j`-th
degree of freedom, and the linear flow is `q_j + i p_j` rotating as
`exp(-i omega_j t)`.

The action variables are

    I_j = ( q_j^2 + p_j^2 ) / 2 = -i x_j y_j

so that `H_2 = sum over j of omega_j I_j` with no factor in front, in both the
real and the complex variables.

In the complex variables the quadratic part is therefore

    H_2 = -i * sum over j = 1..v of omega_j x_j y_j

The other normalisation in circulation writes `H_2 = sum of omega'_j (q_j^2 +
p_j^2)`, whose frequency is half of this one. Item 9 says what converting from it
costs.

## 7. The sign of the frequency vector

The frequency vector is taken **as given**, including negative entries. It is not
sorted, not normalised to a fixed sign, and not scaled.

Forcing every `omega_j` positive would require reflecting a variable, which
changes the sign of one coordinate and therefore the sign of one component of
every exponent difference in item 8. The resonance module of 0007 is a lattice in
those exponent differences, so a silent reflection would silently relabel the
user's resonance relations. The sign is also physical: it is the signature of the
quadratic form, which distinguishes cases the linear analysis treats differently,
and a package that normalises it away has thrown that information out before the
user could use it.

The cost is that two users who supply the same physical system with a different
sign convention on one variable get different exponent labels. That is visible in
the file, because 0010 records the frequency vector as given, and it is
recoverable. A silent normalisation would not be.

## 8. The divisor of a monomial

In the complex variables, write a monomial as `x^a y^b` with `a` and `b` in
`Z^v`, non-negative. The homological operator acts on it as a scalar:

    { x^a y^b , H_2 } = -i * <a - b, omega> * x^a y^b

so the eigenvalue is `-i <a-b, omega>` and the quantity that can be small is

    d(a, b) = <a - b, omega> = sum over j = 1..v of ( a_j - b_j ) * omega_j

That integer vector `k = a - b` in `Z^v` is the multi-index every other document
means when it says the divisor is the inner product of a multi-index with the
frequency vector. 0006 fixes the threshold on `|d(a,b)|` and 0007 fixes the
lattice of `k` with `d = 0`.

Note what the eigenvalue says about degree. A monomial with `a = b` has `k = 0`
and divisor zero for every frequency vector, so those terms are in the kernel for
trivial reasons and are always retained. They exist only in even degree, since
the degree is `|a| + |b|`.

## 9. Converting from another convention

The table below is keyed by what the other source does rather than by its name,
so a reader who has checked their own source against item 1 through item 8 can
convert without trusting anybody's attribution.

**Interleaved variable order,** `(q_1, p_1, q_2, p_2, ...)`. Permute the exponent
vector by the permutation that sends slot `2j-1` to `j` and slot `2j` to `v+j`.
Nothing else changes: no sign moves and no formula changes. This is the cheapest
of the conversions and the one most often left undone, because a formula written
in either order looks correct in the other.

**Opposite bracket sign,** `{f, g}' = -{f, g}`. Every bracket in the source
becomes the negative of the bracket here, so a Lie operator `L'_chi f = {f,
chi}'` equals `L_(-chi) f`. Converting means negating the generating function:
`chi = -chi'`. A recursion transcribed without that negation produces a
transformation that is the inverse of the intended one, which is a plausible
wrong answer rather than a visible failure.

**Generator in the first argument,** `L'_chi f = {chi, f}` with this document's
bracket sign. By antisymmetry `L'_chi = -L_chi`, so again `chi = -chi'`. Note
that a source that flips both the bracket sign and the argument order agrees with
this document exactly and needs no conversion at all, which is why the two have
to be read together and never one at a time.

**Halved quadratic part,** `H_2 = sum omega'_j (q_j^2 + p_j^2)`. Then
`omega = 2 omega'`. Every divisor of item 8 doubles, so a threshold quoted
against `omega'` has to double with it. This is the conversion most likely to
pass unnoticed, because doubling every divisor changes no resonance relation and
no normal form structure, and shows up only in the number a threshold is compared
against and in the size of the generator.

**Exchanged complexification,** the other normalisation named in item 5. It is
this map with `x` and `y` exchanged, so `k = a - b` becomes `-k` and every
divisor changes sign. The resonance module is unchanged, because a lattice
contains `-k` whenever it contains `k`, and the magnitude of every divisor is
unchanged, so 0006 is unaffected. What changes is the sign reported for an
individual divisor, and any expression that uses the divisor rather than its
magnitude.

**Small-parameter grading,** a source that expands in `e` with factorials, as
Deprit's recursion is stated. The substitution into the degree grading is in
0005, with the three factorial ratios that produce it, and it is not repeated
here.

## What this document records about two published sources

The two entries below are what this document records about two sources a user of
this package is likely to arrive from. **They are claims and not measurements.**
They were written from the method as it is normally presented and not from a
reading of a copy of either text made when this document was written, and no
command in this repository can check them. They are checked when the published
case from each source is reproduced as a fixture, which is issue #38, and a wrong
attribution here is a defect that issue finds. The table above is what a reader
should rely on in the meantime, because it is keyed by the convention rather than
by the source.

**Deprit, "Canonical transformations depending on a small parameter", Celestial
Mechanics 1 (1969).** Recorded as: the same bracket sign as item 3, the generator
in the second argument as in item 4, and an expansion in a small parameter with
factorial weights rather than in polynomial degree. Conversion: the
small-parameter row of the table, which is the substitution 0005 derives. No sign
change.

**Meyer, Hall and Offin, "Introduction to Hamiltonian Dynamical Systems and the
N-Body Problem".** Recorded as: blocked variable order as in item 1, `J` as in
item 2, and the bracket as in item 3. Conversion: none for those three. Its
treatment of the Birkhoff normal form near an elliptic equilibrium is where the
complexification and the quadratic normalisation of items 5 and 6 have to be read
against the table, and this document records no claim about which row applies.

## Changing a convention is a breaking change

A change to any item above is a breaking change. It gets a major version bump
under the rule of 0011 and an entry in the changelog naming the item, what it was
and what it became.

This is stated separately because the ordinary signal for a breaking change is
absent here. Every signature can be unchanged, every file can still parse and
every test that compares the package against itself can still pass, while the
answer means something different. The bracket sign, the variable ordering, the
placement of the generator, the complexification normalisation and the quadratic
part normalisation are each in that class, and 0011 already names them as
breaking for exactly this reason.

A convention is never changed silently, and it is never changed as part of a
change about something else. It gets an issue of its own, and the issue states
which published results the change makes comparable and which it makes
incomparable.

## The check that 0005 and this document agree

0005 declared items 1, 3 and 4 in full, under the heading "The convention this
document is written in, and its dependency", because this document had not
landed. It declares the same variable order, the same bracket sign written as the
same formula, and the same placement of the generator in the second argument, so
no index in 0005 is restated and none is reinterpreted.

That is a reading of two documents rather than a command. Nothing in this
repository compares two prose declarations, and the sentence above is a claim
about what a reader will find, checked by reading the two passages side by side.

It is also a comparison across two branches at the time of writing: 0005 is on
the open pull request that carries the odd-numbered decisions of this milestone
and is not on the default branch. Where that document lands changed, this section
is what has to be re-read.

Items 5, 6, 7 and 8 are decided here for the first time and 0005 declares none of
them.

## The identities in items 5, 6 and 8 are checked

Items 5, 6 and 8 are the ones where a sign error would be invisible, so they are
checked exactly rather than asserted. The block below verifies, over the
Gaussian rationals with no floating point anywhere, that the complexification of
item 5 is canonical under the bracket of item 3, that the quadratic part of item
6 in the real variables equals the complex form stated there, and that the
eigenvalue of item 8 is what item 8 says it is. It checks the last one for every
`a` and `b` in `{0,1,2}^v`, which reaches mixed monomials of both signs and both
the `k = 0` and `k` non-zero cases.

This is a check on this document and not on the package. The obligation to prove
the same properties of the shipped code belongs to issues #30 and #33. Paste it
to reproduce the line:

    python - <<'EOF'
    from fractions import Fraction as F
    from itertools import product

    def cadd(a, b): return (a[0] + b[0], a[1] + b[1])
    def cmul(a, b): return (a[0]*b[0] - a[1]*b[1], a[0]*b[1] + a[1]*b[0])

    ONE = (F(1), F(0)); I = (F(0), F(1)); Z = (F(0), F(0))

    def padd(f, g):
        h = dict(f)
        for k, c in g.items():
            d = cadd(h.get(k, Z), c)
            if d == Z: h.pop(k, None)
            else: h[k] = d
        return h

    def pmul(f, g):
        h = {}
        for kf, cf in f.items():
            for kg, cg in g.items():
                k = tuple(a + b for a, b in zip(kf, kg))
                d = cadd(h.get(k, Z), cmul(cf, cg))
                if d == Z: h.pop(k, None)
                else: h[k] = d
        return h

    def pscale(f, c):
        return {k: cmul(cf, c) for k, cf in f.items() if cmul(cf, c) != Z}

    def pvar(i, n): return {tuple(1 if j == i else 0 for j in range(n)): ONE}
    def pconst(c, n): return {} if c == Z else {tuple(0 for _ in range(n)): c}

    def pderiv(f, i):
        h = {}
        for k, c in f.items():
            if k[i] == 0: continue
            kk = list(k); e = kk[i]; kk[i] -= 1
            h = padd(h, {tuple(kk): cmul(c, (F(e), F(0)))})
        return h

    def bracket(f, g, v):
        h = {}
        for j in range(v):
            h = padd(h, pmul(pderiv(f, j), pderiv(g, v + j)))
            h = padd(h, pscale(pmul(pderiv(f, v + j), pderiv(g, j)), (F(-1), F(0))))
        return h

    ok = True
    def check(name, cond):
        global ok
        if not cond:
            ok = False; print("FAIL", name)

    for v in (1, 2, 3):
        n = 3 * v                       # q_1..q_v, p_1..p_v, omega_1..omega_v
        q = [pvar(j, n) for j in range(v)]
        p = [pvar(v + j, n) for j in range(v)]
        w = [pvar(2 * v + j, n) for j in range(v)]
        half = (F(1, 2), F(0))

        # X_j = sqrt(2) x_j, Y_j = sqrt(2) y_j, so {x_a,y_b} = {X_a,Y_b}/2
        X = [padd(q[j], pscale(p[j], I)) for j in range(v)]
        Y = [padd(pscale(q[j], I), p[j]) for j in range(v)]

        for a in range(v):
            for b in range(v):
                want = pconst(ONE, n) if a == b else {}
                check("xy", pscale(bracket(X[a], Y[b], v), half) == want)
                check("xx", pscale(bracket(X[a], X[b], v), half) == {})
                check("yy", pscale(bracket(Y[a], Y[b], v), half) == {})

        H2 = {}
        for j in range(v):
            H2 = padd(H2, pscale(pmul(w[j], padd(pmul(q[j], q[j]), pmul(p[j], p[j]))), half))
        rhs = {}
        for j in range(v):
            rhs = padd(rhs, pmul(w[j], pscale(pmul(X[j], Y[j]), half)))
        check("H2", H2 == pscale(rhs, (F(0), F(-1))))

        xs = [pvar(j, n) for j in range(v)]
        ys = [pvar(v + j, n) for j in range(v)]
        K2 = {}
        for j in range(v):
            K2 = padd(K2, pmul(w[j], pmul(xs[j], ys[j])))
        K2 = pscale(K2, (F(0), F(-1)))
        for a in product(range(3), repeat=v):
            for b in product(range(3), repeat=v):
                mon = pconst(ONE, n)
                for j in range(v):
                    for _ in range(a[j]): mon = pmul(mon, xs[j])
                    for _ in range(b[j]): mon = pmul(mon, ys[j])
                div = {}
                for j in range(v):
                    div = padd(div, pscale(w[j], (F(a[j] - b[j]), F(0))))
                check("eig", bracket(mon, K2, v) == pscale(pmul(div, mon), (F(0), F(-1))))

    print("all convention checks pass for v=1,2,3:", ok)
    EOF

    all convention checks pass for v=1,2,3: True

## What this document does not decide

It does not decide the storage layout, though it fixes the variable order the
exponent vector of 0003 is written in. It does not decide the coefficient types,
which is 0002. It does not decide the recursion, which is 0005. It does not
decide the divisor threshold, which is 0006, nor the resonance lattice, which is
0007. It does not decide how a convention is named in a file, which is 0010.
