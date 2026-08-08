# 0003. How a truncated series is stored

Status: decided. Raised in issue #5.

## The decision

A truncated series is stored graded. It is a vector indexed by total degree, and
the entry at degree `d` is a dense array of coefficients over the monomials of
degree `d` in `m = 2v` variables, in one fixed order, with the exponent vector
recovered from the array index by a bijection tabulated once.

The exponent vector is never stored per term. At the orders this package is aimed
at, storing exponents alongside coefficients would cost more than the
coefficients do.

## How many terms that is

A homogeneous polynomial of degree `d` in `m` variables has

    M(d, m) = C(d + m - 1, m - 1)

monomials, and a truncation carrying every degree up to `D` has `C(D + m, m)` of
them in total. At `m = 6`, which is three degrees of freedom:

    $ python -c "from math import comb; print([comb(d+5,5) for d in range(2,11)])"
    [21, 56, 126, 252, 462, 792, 1287, 2002, 3003]
    $ python -c "from math import comb; print(sum(comb(d+5,5) for d in range(0,11)), comb(16,6))"
    8008 8008

Multiplication is a convolution over degree, so the pair count in the inner loop
at degrees `a` and `b` is `M(a, m) * M(b, m)`. The access pattern in that loop is
what this package is, and it is the reason the layout is decided before anything
is built on it.

## The monomial ordering

Graded reverse lexicographic order, ascending, on the exponent vector
`a = (a_1, ..., a_m)` with `a_1` the first variable. Index 0 within a degree is
the grevlex-least monomial of that degree, and the index increases with the
grevlex order.

Grevlex compares two exponent vectors of equal total degree at the last position
where they differ, and the vector with the smaller exponent there is the greater
of the two. For `m = 3` and `d = 2` the ascending order is

    (0,0,2) (0,1,1) (1,0,1) (0,2,0) (1,1,0) (2,0,0)

with indices 0 through 5.

The order is named here because two representations that disagree about it
produce different files from the same mathematics, and 0010 has to be able to
state which one a file is written in.

## The bijection, explicitly

Write the partial sums of the exponent vector

    s_k = a_1 + a_2 + ... + a_k,        k = 1, ..., m - 1

so that `0 <= s_1 <= s_2 <= ... <= s_{m-1} <= d`, and shift them apart

    c_k = s_k + (k - 1)

so that `0 <= c_1 < c_2 < ... < c_{m-1} <= d + m - 2`. The index is then the rank
of that strictly increasing tuple in the combinatorial number system:

    index(a) = sum over k = 1 .. m-1 of C(c_k, k)

This is a bijection from the monomials of degree `d` in `m` variables onto
`0, 1, ..., M(d, m) - 1`.

The inverse runs greedily downward. Given an index `i`, for `k = m-1` down to `1`
take `c_k` to be the largest integer with `C(c_k, k) <= i`, subtract `C(c_k, k)`
from `i`, and continue. Then `s_k = c_k - (k - 1)`, and

    a_1 = s_1,   a_k = s_k - s_{k-1} for k = 2 .. m-1,   a_m = d - s_{m-1}

Both directions were checked exhaustively for `m = 1..6` and `d = 0..8` while
this document was written, against the definition of grevlex rather than against
a second spelling of the same formula: that the ranks of the monomials of each
degree are exactly `0 .. M(d,m)-1`, that unranking a rank returns the monomial it
came from, and that sorting by rank reproduces ascending grevlex. Nothing is
added to the tree for it, because the tree carries no language yet and this is a
check on a specification rather than on code. Paste it to reproduce the line:

    python - <<'EOF'
    from math import comb
    from itertools import product

    def rank(a):
        m = len(a); s = 0; idx = 0
        for k in range(1, m):
            s += a[k-1]
            idx += comb(s + k - 1, k)
        return idx

    def unrank(i, m, d):
        c = [0]*(m-1); rem = i
        for k in range(m-1, 0, -1):
            ck = k - 1
            while comb(ck + 1, k) <= rem:
                ck += 1
            c[k-1] = ck; rem -= comb(ck, k)
        s = [c[k-1] - (k-1) for k in range(1, m)]
        a = []; prev = 0
        for k in range(m-1):
            a.append(s[k] - prev); prev = s[k]
        a.append(d - prev)
        return tuple(a)

    def grevlex_less(a, b):
        for i in range(len(a)-1, -1, -1):
            if a[i] != b[i]:
                return a[i] > b[i]
        return False

    ok = True
    for m in range(1, 7):
        for d in range(0, 9):
            mons = [a for a in product(range(d+1), repeat=m) if sum(a) == d]
            if sorted(rank(a) for a in mons) != list(range(comb(d+m-1, m-1))):
                ok = False; print('BIJECTION FAIL', m, d)
            for a in mons:
                if unrank(rank(a), m, d) != a:
                    ok = False; print('INVERSE FAIL', m, d, a)
            srt = sorted(mons, key=rank)
            for i in range(len(srt)-1):
                if not grevlex_less(srt[i], srt[i+1]):
                    ok = False; print('ORDER FAIL', m, d, srt[i], srt[i+1])
    print('all checks pass for m=1..6, d=0..8:', ok)
    EOF

    all checks pass for m=1..6, d=0..8: True

That is a check on this document, not on the package. The obligation to prove the
same three properties of the shipped index is issue #28's, and it discharges it
with tests in the tree rather than with a paste.

## Why graded and dense

The inner loop of a graded multiplication becomes a walk over two contiguous
arrays writing into a third, with the destination index found by a table lookup
rather than by hashing. That is the difference between a memory-bound kernel that
prefetches and one that does not.

Truncation to order `N` is dropping the tail of the degree vector. It is free,
and it is the operation this field performs constantly.

Iteration order is fixed by the layout, so a reduction over terms is reproducible
without sorting anything first. The determinism requirement in 0009 rests on
this.

Chunking for parallel execution is slicing an array, and the chunks come out
equal sized by construction, which is what makes the chunk index a stable name
for a partial sum.

## Where it is wrong, and what is built for that

Dense per degree is wrong when the series is sparse, and Hamiltonians in this
field often are. Many physically interesting ones have most coefficients exactly
zero, and a dense layout then spends its whole budget on zeros.

So the decision has two halves. Dense per degree is the representation the
arithmetic is written against. A sparse representation of exponent and
coefficient pairs exists at the boundary, for reading a file, writing a file and
holding an input Hamiltonian, and it is converted to the dense form on entry. No
sparse throughput path is built.

## The measurement that would overturn the dense choice

The threshold is the fill fraction at the degrees that dominate the cost. For a
series `f` and degree `d`, the fill fraction is the number of coefficients that
are not exactly zero divided by `M(d, m)`.

The dense decision is overturned when, on a Hamiltonian somebody actually wants
to run rather than on a constructed example, the fill fraction stays below

    1/16

at every degree from half the truncation order upward. Below that, a sparse
throughput path is worth building; above it, it is not.

The reasoning behind the number, stated so that it can be argued with. A sparse
term carries an index next to its coefficient and is written through a lookup
rather than a computed offset, so the same arithmetic costs several times more
per stored term than the dense walk does. A fill fraction has to beat that
multiple before it starts saving anything, and the ratio also has to be clear
enough to be worth carrying a second representation with its own tests, its own
determinism argument and its own file path. One sixteenth is where those two put
it. The crossing point itself has not been measured, and this document does not
claim it has been.

The command that produces the number is

    drehbank inspect --fill <hamiltonian> --degrees <lo>-<hi>

which prints one fill fraction per degree. That command does not exist yet. It is
part of the command line built in issue #60, and no fill fraction can be quoted
before it does. Naming it here fixes what has to be produced rather than
recording a result.

## The alternatives, and what each would have cost

A hash map from exponent vector to coefficient. Simple, sparse for free, and what
most one-off implementations use. It would have cost the inner loop: a hash per
output term, an iteration order that depends on the hasher's seed, and no useful
chunking. The seed dependence alone is fatal, because 0009 requires the answer
not to move between runs. It is a reasonable choice for a script and the wrong
one for a package.

A flat sorted list of exponent and coefficient pairs, merged on multiply. Cache
friendly and sparse. It would have cost a merge with a sort in every
multiplication, and it would have turned truncation from a slice into a scan.

A recursive representation, a polynomial in one variable whose coefficients are
polynomials in the rest. Elegant, and it makes some substitutions cheap. It would
have cost locality, and it would have made degree-graded truncation awkward,
because the total degree is not visible at any single level of the nesting.

## What this document does not decide

It does not decide what a coefficient is, which is 0002. It does not decide the
on-disk form of either representation, which is 0010. It does not decide how a
degree array is chunked for parallel work or in what order the chunks are
reduced, which is 0009.
