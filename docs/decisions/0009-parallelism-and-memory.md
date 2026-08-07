# 0009. The parallelism model, the determinism rule and the memory ceiling

Status: decided. Raised in issue #9.

## The decision, in three parts

Parallelism is at the term level, over chunks of the dense per-degree arrays from
0003, with a pool sized by default to the available parallelism and settable by
the caller. Nothing above that level is parallel.

The result does not depend on the number of threads, on the scheduler or on the
order in which work completes. This is a requirement, not a preference.

Memory has a ceiling that the caller sets and the package respects, checked
before each group of the recursion starts rather than discovered when an
allocation fails.

## The level of parallelism

One bracket at a time, parallel inside it. A Poisson bracket at the sizes this
package works at is enough to saturate sixteen cores on its own, and keeping the
parallelism to one level keeps the number of places that can race to one.

Nothing above that is parallel. The Deprit groups in 0005 are sequential because
the recursion makes them sequential, and within a group the entries are computed
in increasing superscript because each one needs the previous. Nothing is gained
by looking for parallelism there, and a driver that tried would have to
reintroduce the ordering afterwards to keep the result stable.

## The chunking rule

The chunking rule is derived from the storage layout and not from the scheduler,
because a chunk index that depended on the scheduler could not name a partial
result.

A degree `d` array holds `L = mon(d, v)` coefficients, contiguous, in the grevlex
order fixed by 0003. It is cut into contiguous chunks of a fixed target length
`T`, so there are `ceil(L / T)` of them and chunk `c` covers the index range

    [ c * T , min((c + 1) * T, L) )

`T` is a property of the build, chosen for cache behaviour, and it is recorded in
the result so that a reader can reconstruct the partition. It is not a function
of the pool size, of the core count, of the load or of anything else that moves
between runs. Two runs of the same input on the same build therefore have the
same chunks, with the same indices, whatever the machine is doing.

Work is assigned to threads by partitioning the output array. Each chunk of the
output is written by exactly one thread, which accumulates every contribution to
that chunk itself. Where an operation cannot be expressed that way and needs
private per-chunk accumulators, the accumulators are combined in increasing chunk
index and never in completion order.

## The fixed reduction order

Floating point addition is not associative, so a sum whose order moves is a
result that moves. Every reduction in this package is performed in increasing
chunk index. A thread that finishes first waits; it does not fold its partial sum
into a shared accumulator, and there is no atomic accumulation anywhere in the
package.

That costs a barrier per reduction, which is small next to the bracket that
produced the partials, and it costs the ability to use any library reduction
whose order is unspecified. Both costs are accepted here so that they are not
re-argued later under time pressure.

Why not the faster form. Atomic or completion-order accumulation is faster and
gives a different answer per run. The differences land in the smallest
coefficients, which are exactly the ones the remainder estimate in 0008 is most
sensitive to, so the cheap version moves the number the package exists to
produce. A package whose output moves when the machine changes cannot be used to
reproduce a published result, and reproducing published results is issues #38 and
#42.

## The reproducibility requirement, in the form a test is written against

The requirement is stated so that it can be a test rather than an intention.

For every fixture in the determinism suite, let `S(t, i)` be the byte
serialisation of the complete result, produced by the writer from 0010, computed
with a pool of `t` threads on run `i`. The requirement is

    S(t, i) == S(1, 1)   for every fixture, for every t in the list below,
                         and for i = 1 and i = 2

with the thread counts

    1, 2, 3, 7, 8, 16

Three of those are not powers of two and one is prime, so the mapping from chunks
to threads is uneven in several different ways rather than in one. Two runs at
each count are compared as well as two counts, because a scheduler that varies
between runs at a fixed thread count fails in the same way and would otherwise
pass.

The pool size is a parameter and not the core count, so the list is run in full
on a machine with fewer cores than sixteen. Oversubscription is the point: it
makes completion order vary more, not less.

The comparison is on bytes of the serialised result, not on a tolerance. A
tolerance would be a second thing to argue about, and the whole claim here is
that there is nothing to tolerate.

## The memory ceiling

The caller sets a ceiling in bytes. The package respects it, and the mechanism is
a refusal before an allocation rather than a failure during one.

Before group `d` of the recursion begins, the driver computes the peak live bytes
that group will need. The piece counts and coefficient counts come from the
formula in 0005, evaluated at the order and the degrees of freedom of this run;
the bytes per coefficient come from the coefficient type in 0002. If the computed
need exceeds the ceiling, the driver stops and returns an error naming three
numbers: the group it was about to start, the bytes it computed it would need,
and the ceiling it was given. It does not start the group.

The reason is what the two failures tell the user. A refusal before the group
says exactly which order cannot be afforded and by how much, which is the
information needed to choose a smaller order or a bigger machine. A kill during
the group loses the whole run and says nothing, and the user does not even learn
whether the answer would have been right.

The working set is large enough that the difference between fitting and not
fitting is one order, and one order can be a day.

## When the ceiling is reached anyway

If the ceiling is reached during a group, the prediction was wrong. That is a
defect in the prediction, not a case the package handles gracefully, and it is
reported as its own error so that it can be found and fixed rather than absorbed.

The error names the group, the bytes predicted for it, and the bytes allocated at
the point the ceiling was crossed. Those three numbers are what an issue about
the prediction needs, and the error exists so that such an issue can be opened
with evidence instead of with a description.

## No spill to disk

The triangle is held in memory and nothing is streamed to disk at any order. 0005
carries the reason: at the target case there is nothing worth streaming, and a
spill path would be a second code path owing its own determinism argument, since
a result that moved when the triangle spilled would be a defect rather than a
trade. What happens instead when the peak does not fit is the refusal above.

## What this document does not decide

It does not decide the storage layout or the monomial order, which is 0003. It
does not decide the coefficient type or its width in bytes, which is 0002 and
which the ceiling arithmetic needs. It does not decide the recursion or its peak
formula, which is 0005. It does not decide the serialisation the determinism test
compares, which is 0010.
