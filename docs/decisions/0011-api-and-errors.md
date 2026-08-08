# 0011. The public surface, the error model and the versioning rule

Status: decided. Raised in issue #13.

## The decision

A small public surface, errors as returned values with no exceptions, and a
versioning rule under which a change to a convention or to the meaning of a file
field is breaking even when no signature moves.

## The public surface

The surface is small on purpose. A surface can be widened later on request and
cannot be narrowed, so everything not on this list is internal until somebody
asks for it with a reason.

The public items are these, and nothing else:

- `Series`, the truncated series of 0003, graded by total degree, parameterised
  by the coefficient type of 0002.
- `Hamiltonian`, a series carrying what makes it a Hamiltonian: the number of
  degrees of freedom, and the statement that it is expanded about an equilibrium
  at the origin.
- `ResonanceModule`, the lattice of resonance relations of 0007, declared by the
  caller.
- `Polydisc`, the explicit radius vector every estimate is stated on.
- `Driver`, the normal form driver, together with the configuration it takes:
  target order, polydisc, memory ceiling, pool size, small divisor threshold, and
  the resonance module where one is declared.
- `NormalFormResult`, carrying the normalised Hamiltonian, the generator, and the
  estimate.
- `Estimate`, the object of 0008, with its class, domain, order, value, optimal
  order and hypotheses.
- `Reader` and `Writer`, for the formats of 0010.
- `Error`, one error type for the whole library.
- The coefficient trait that `Series` is parameterised over, and the concrete
  coefficient types 0002 provides.

Everything else, including the monomial index tables, the chunking, the triangle
and the majorant recursion, is internal. Widening the list is a change to this
document first and to the code second.

## The error model

Errors are returned values. Nothing in the library terminates the process, and
nothing panics on input a user can supply.

The inputs a user can supply are the ones this rule is about, and each is an
error naming the specific thing and, where there is one, the specific term or
index:

- a malformed or truncated input file, naming the byte offset and what was
  expected there,
- a degenerate or inconsistent frequency vector,
- a resonance declaration inconsistent with itself or with the frequency vector,
- an order beyond the memory ceiling, naming the group, the need and the ceiling,
  as 0009 sets out,
- a divisor below the threshold, naming the multi-index and the divisor,
- an interval that contains zero where a division was required,
- a result whose estimate could not be computed,
- an order or degree so large that the index arithmetic would not fit its
  integer type.

Numerical failure is an error too. None of the cases above returns a number with
a note attached, because a number with a note attached gets copied into a paper
without the note.

The last item deserves its own sentence. The index arithmetic of 0003 is where
this package produces a plausible wrong answer rather than a crash, so overflow
checks on it are on in every build profile, not only in tests, and an overflow is
an error rather than a wrap.

**The rule has no exceptions.** A panic is therefore a bug by definition, and
that is what makes the property testable: fuzz the reader and the constructors,
and any panic at all is a finding, with no triage step asking whether this
particular panic was intended. Issue #56 is where that runs.

The rule is about the library's own code, and there is one process abort it
cannot reach, which is stated here rather than left to be discovered. A global
allocator that cannot satisfy a request aborts the process, and that is the
runtime's behaviour and not a return value the library can produce. The memory
ceiling of 0009 exists to keep the library away from it, and reaching it is a
defect in the prediction that 0009 requires be reported and fixed. That is a
limit on what the rule can reach, and it is not permission for the library to
panic anywhere.

## What counts as a breaking change

Versioning follows the usual meaning, with additions that a numerical package
usually gets wrong. The following are breaking:

- Any removal from or incompatible change to the public list above. The ordinary
  case.
- A change to any canonical convention fixed in 0004. The bracket sign, the
  ordering of the variables, the placement of the momenta, and the normalisation
  of the generator are all in this class. Every signature can be unchanged and
  every file can still parse while the answer means something different, which is
  exactly why it is named here.
- A change to the monomial ordering of 0003. It changes the bytes of every file
  the package writes and the index of every coefficient the package returns,
  while the types stay identical.
- A change to the meaning of any field in the file format of 0010, including a
  unit, a sign, a normalisation, or what a field is measured relative to. Adding
  a field is not breaking; changing what an existing field means is.
- A change to what any of the three estimate classes of 0008 licenses.
- A change that moves the computed number for the same input on the same build,
  unless it is the correction of a defect. Where it is a correction, it is still
  named in the changelog, with what was wrong, how it was found, and a command
  that shows the difference between the two answers.

That list is the part of this decision most likely to be argued with later, so it
is written down before there is a release to be inconvenienced by it.

## Stability before the first release

Before the first release the public surface is explicitly unstable. It may change
in any release, including in ways that break callers, and version numbers stay
below `1.0`.

A user reads that in three places, and the wording is the same in all three: the
first screen of the README, the top of the library documentation, next to the
description of the surface itself, and the release notes of every pre-release
version. Putting it next to the description of the surface is the part that
matters, because that is where somebody arrives when they are about to depend on
it.

After the first release the rules above are in force and the changelog is where a
break is announced. Issue #64 is where the versioning policy, the changelog and
the release checklist are written against this document.

## What it costs

Returning errors everywhere makes the internal code more verbose than panicking
would. That is the price of the property being testable at all, and it is paid
knowingly.

A small surface means users will ask for access to internals. Each request is
answered with a decision written down, not by widening reflexively, and the
answer that widens the surface amends the list above in the same change.

## What this document does not decide

It does not decide the coefficient types, which is 0002. It does not decide the
conventions whose change it calls breaking, which is 0004. It does not decide the
file format whose fields it calls breaking, which is 0010. It does not decide the
release process, which is issue #64.
