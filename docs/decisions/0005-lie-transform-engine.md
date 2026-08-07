# 0005. The Lie transform engine and its memory schedule

Status: decided, with one dependency named below that has not landed yet. Raised
in issue #7.

## The decision

The canonical transformation is organised as the Deprit recursion. The generating
function is held as one homogeneous piece per order, and the recursion is
evaluated as a triangle of intermediate homogeneous series. The transformation
and its inverse come out of the same recursion with the sign of the generator
reversed.

## The convention this document is written in, and its dependency

The recursion below is written in one convention, stated here in full so that the
indices can be read without guessing.

Phase space is `z = (q_1, ..., q_v, p_1, ..., p_v)`, with `v` degrees of freedom
and `m = 2v` variables. The Poisson bracket is

    {f, g} = sum over j = 1..v of ( df/dq_j * dg/dp_j - df/dp_j * dg/dq_j )

The Hamiltonian is expanded in homogeneous degree about an equilibrium at the
origin, `H = H_2 + H_3 + H_4 + ...`, with `H_d` homogeneous of degree `d` in `z`.
The generating function is `chi = chi_3 + chi_4 + ...`, with `chi_{j+2}`
homogeneous of degree `j + 2`.

0004 is where the canonical conventions are fixed for the whole package, and it
has not landed. Until it does, the paragraph above is this document's own
declaration and not a reference to a settled one. If 0004 fixes a different sign
in the bracket or a different variable ordering, this document is re-read against
it and the indices below are restated, not reinterpreted. That is the dependency,
and issue #7 stays open on it.

## The recursion, with indices

Deprit's recursion is stated for an expansion in a small parameter. Write

    H(z; e) = sum over n >= 0 of (e^n / n!) H_n^(0)(z)
    W(z; e) = sum over n >= 1 of (e^(n-1) / (n-1)!) W_n(z)

Then the transformed Hamiltonian is `K = sum over k >= 0 of (e^k / k!) H_0^(k)`,
and the triangle entries obey, for `k >= 1` and `n >= 0`,

    H_n^(k) = H_(n+1)^(k-1) + sum over j = 0..n of C(n, j) * { H_(n-j)^(k-1), W_(j+1) }

This package has no small parameter. The grading is the polynomial degree, so the
recursion is restated in degree, which removes the factorials from the arithmetic
and is the form the implementation is read against. Put

    A_n^(k) = H_n^(k) / (n! k!)        B_j = W_j / (j-1)!

Substituting into the line above and dividing through by `n! k!` gives, for
`k >= 1` and `n >= 0`,

    k * A_n^(k) = (n+1) * A_(n+1)^(k-1) + sum over j = 0..n of { A_(n-j)^(k-1), B_(j+1) }

The three factorial ratios that produce it are `(n+1)!(k-1)! / (n! k!) = (n+1)/k`
on the first term, `C(n,j) * (n-j)! * j! / n! = 1` on each bracket, and `1/k`
pulled out front. The identification with the degree expansion is

    A_n^(0) = H_(n+2)        B_(j+1) = chi_(j+3)        A_0^(k) = K_(k+2)

and every entry is homogeneous:

    A_n^(k) has degree n + k + 2

which is checked on the recursion itself. The first term `A_(n+1)^(k-1)` has
degree `(n+1) + (k-1) + 2 = n+k+2`. The bracket of a degree `a` and a degree `b`
homogeneous polynomial has degree `a + b - 2`, and
`{A_(n-j)^(k-1), B_(j+1)}` has degree `((n-j) + (k-1) + 2) + (j + 3) - 2 = n+k+2`.

The first two orders, written out, so that an implementation can be read against
something short:

    A_0^(1) = A_1^(0) + { A_0^(0), B_1 }
            = H_3 + { H_2, chi_3 }                           and this is K_3

    A_1^(1) = 2 A_2^(0) + { A_1^(0), B_1 } + { A_0^(0), B_2 }
            = 2 H_4 + { H_3, chi_3 } + { H_2, chi_4 }

    2 A_0^(2) = A_1^(1) + { A_0^(1), B_1 }                   and A_0^(2) is K_4

## The order the triangle is evaluated in

Entries are grouped by `d = n + k`, which is the degree offset: every entry with
`n + k = d` is homogeneous of degree `d + 2`. Within group `d` the entries are
computed in increasing `k`, starting from the given `A_d^(0) = H_(d+2)` and
ending at `A_0^(d) = K_(d+2)`.

That order is forced by the recursion rather than chosen. The first term on the
right, `A_(n+1)^(k-1)`, sits in the same group with a superscript one lower, so
it has to exist first. Every other term on the right sits in a strictly lower
group and was finished earlier.

The generator piece `B_d` is the unknown at group `d`. It enters the group in
exactly one place, the `j = n` term of the entry with `k = 1`, where it appears
as `{ A_0^(0), B_d } = { H_2, B_d }`, and from there it is carried down the group
by the first term of each following step. Its coefficient at the bottom of the
group is the telescoping product

    product over k = 2..d of (d - k + 1)/k = 1/d

so the equation that closes group `d` is

    K_(d+2) = R_(d+2) + (1/d) { H_2, B_d }

with `R_(d+2)` everything the group produced that does not contain `B_d`. It is
linear in `B_d`, which is what makes it solvable order by order. How it is solved,
and what happens when the operator on the left is not invertible, is 0006 and
0007 rather than this document.

## Why this engine

The generating function comes out one order at a time, and the order in progress
depends only on the pieces already fixed. That is what makes an arbitrary-order
driver possible without knowing the final order in advance, and it is what lets a
run stop at the order the remainder estimate says is optimal rather than at an
order guessed before the run started. 0008 is where that optimum is defined, and
it is not knowable until the coefficients exist.

The transformation and its inverse come from the same recursion with the sign of
the generator reversed, so the inverse is not a second implementation carrying
its own bugs. The round trip is then a test of the machinery rather than a
comparison of two independent guesses, which is what issue #37 rests on.

The recursion is a sequence of Poisson brackets, additions and scalings, and
nothing else. The whole package above the series core is those three operations,
so the surface that can be wrong is small and every part of it can be checked
against an exact oracle.

The intermediate triangle is the memory problem, and this engine makes it
explicit rather than hiding it inside a recursive call. That is what allows the
schedule below to be planned instead of discovered when a run dies.

## The memory schedule

To normalise a Hamiltonian up to degree `D`, the triangle needed is every entry
with `n + k <= M`, where

    M = D - 2

Nothing in that triangle can be discarded before the last group. An entry
`A_n^(r)` is consumed by every entry of row `r + 1` with subscript at least `n`,
and row `r + 1` is not complete until group `M` is reached. So the live set grows
monotonically until group `M` begins, and it falls off only during the final
sweep: once group `M` has produced `A_(M-k)^(k)`, row `k - 1` is dead and can be
released, for `k = 1, 2, ..., M` in that order.

The peak is therefore at the start of group `M` and it is the whole triangle plus
the whole generator. As a count of homogeneous pieces, in the order alone:

    pieces(M) = (M + 1)(M + 2)/2 + M

As a count of coefficients, in the order and the degrees of freedom, using
`mon(d, v) = C(d + 2v - 1, 2v - 1)` for the monomial count of a homogeneous
polynomial of degree `d` in `2v` variables:

    coeffs(M, v) = sum over d = 0..M of (d + 1) * mon(d + 2, v)      the triangle
                 + sum over j = 1..M of mon(j + 2, v)                the generator

The triangle sum has `d + 1` entries in group `d`, each of degree `d + 2`. The
generator sum has one piece `B_j` of degree `j + 2` for each `j`.

Evaluated at three degrees of freedom:

    $ python - <<'EOF'
    from math import comb
    def mon(d, v): return comb(d + 2*v - 1, 2*v - 1)
    def peak_pieces(M): return (M + 1)*(M + 2)//2 + M
    def peak_coeffs(M, v):
        tri = sum((m + 1)*mon(m + 2, v) for m in range(0, M + 1))
        gen = sum(mon(j + 2, v) for j in range(1, M + 1))
        return tri, gen, tri + gen
    for D in (8, 10):
        M = D - 2
        tri, gen, tot = peak_coeffs(M, 3)
        print(f"D={D} v=3 M={M} pieces={peak_pieces(M)} triangle={tri} generators={gen} total={tot}")
    EOF

    D=8 v=3 M=6 pieces=34 triangle=17590 generators=2975 total=20565
    D=10 v=3 M=8 pieces=53 triangle=60633 generators=7980 total=68613

So order eight at three degrees of freedom peaks at 34 live homogeneous pieces
and 20565 live coefficients, and order ten peaks at 53 pieces and 68613
coefficients.

## What those numbers say, and what they do not

At three degrees of freedom the peak live set is small. Whatever a coefficient
turns out to be in 0002, sixty-eight thousand of them is not a number that needs
a memory schedule. The constraint at the target case is arithmetic rather than
bytes: the work is the bracket pair counts, which grow much faster than the
storage, and 0003 is where the pair count is written down.

That is worth saying plainly, because the plan describes order eight to ten in
six variables as needing a real machine, and the reason is time and not memory.
The formula is stated in `M` and `v` rather than as a number so that the scale
milestone can evaluate it where the memory does bind, which is at more degrees of
freedom rather than at higher order. The peak grows roughly like the top group,
and the top group grows like `mon(M + 2, v)`, which is combinatorial in `v`.

The intermediate triangle is held in memory. Nothing is streamed to disk. The
reason is the numbers above: at the target case there is nothing to stream, and a
spill path would be a second code path with its own determinism argument to
discharge, since a result that changed when the triangle spilled would be a
defect and not a trade. If a case is ever reached where the peak does not fit,
what happens is a refusal before the group starts rather than a spill, and that
behaviour is 0009.

## The alternatives, and what each would have cost

The Hori approach, with a single generating function and the averaging carried
through it. It produces the same normal forms and it is closer to how the
celestial mechanics literature states the method, which is a real advantage for a
reader arriving from there. It would have cost bookkeeping. Recovering the
transformation at general order needs care that the Deprit triangle makes
structural, and the inverse becomes a separate derivation rather than a sign,
which doubles the surface that issue #37 has to test.

Mixed variable generating functions of the classical kind, `F(q, P)` and its
relatives. Familiar from textbooks. They would have cost the ability to evaluate
the transformation at all without solving an implicit system at every point, and
composing several of them compounds that.

A direct substitution approach, building the transformation as an explicit map
and composing maps. Conceptually the simplest of the four. It would have cost the
symplectic structure: the composition is symplectic only up to the truncation
error, and it degrades exactly at the orders where the normal form is supposed to
be saying something.

## What this document does not decide

It does not fix the canonical conventions, which is 0004 and which this document
depends on. It does not decide how the homological equation is solved, which is
issue #35 in the non-resonant case and issue #41 on a resonant subspace. It does
not decide what happens when the small divisor is small, which is 0006. It does
not decide the parallelism, the chunk order or the memory ceiling behaviour,
which is 0009.
