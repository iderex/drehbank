# Changelog

Kept per change rather than assembled at release time. A change that a user
could notice adds its line here in the same pull request that makes it, because
a changelog written at the end is written by whoever is doing the release from
whatever they can reconstruct, and what they cannot reconstruct is exactly the
part that mattered.

`docs/versioning.md` is the policy this file serves and
`docs/decisions/0011-api-and-errors.md` holds the list of what counts as
breaking.

## How to add a line

Put it under `Unreleased`, in the section it belongs to. Write what changed from
the position of somebody using the package, not from the position of the diff.
Name the issue.

The **Numerical results** section is not optional and it is not omitted when it
is empty. An empty section under the word `None.` is a claim somebody made after
looking. An absent section is a question nobody asked, and the two read the same
way to everyone except the person who wrote it.

A change belongs in that section when the package computes a different number
for the same input on the same build. That is a breaking change under the
policy, including when it is the correction of a defect, and the line says which
case, by how much, and carries the command that shows the difference.

## Unreleased

Nothing has been released, so everything below is the whole history of the
package as far as this file is concerned. This file starts at commit
`ecbb5c7bbb6492e128c71734f4056c082061d892` and does not reconstruct what came
before it. What came before is on the tracker and in the history:

    git log --oneline origin/main | wc -l
    43

Reconstructing forty-three commits into entries after the fact is the thing this
file exists to avoid doing, and doing it once to start the file would be the
same mistake with a better excuse. Nothing has been released, so nobody is
relying on those entries existing.

### Numerical results

None. Stating the workflow rules in the contribution guide adds prose to one
document and replaces a comment in one workflow file. No workflow step, no crate
source and no dependency moved, so no coefficient the package returns can have
moved either. #57

None, and here that is a measured statement rather than an argument. The
parallel product and the parallel bracket return the same bits as the sequential
ones on every input, at every thread count, which is what
`crates/drehbank-core/tests/parallel.rs` compares: the bit pattern of every
coefficient a caller can read, over the generated corpus, at one, two, three,
seven, eight and sixteen threads and twice at each.

    cargo test --locked --offline --test parallel
    running 6 tests
    test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Nothing a caller had pinned moves, because the sequential kernels are untouched
and the new ones agree with them rather than replacing them. #49

None. Correcting what the workflow files say about themselves changes comments
and one error message. No workflow step and no crate source moved, so no
coefficient the package returns can have moved either. #57

None. The supply chain triage reads what an audit reported about the repository
and writes down what was accepted. It compiles nothing and no coefficient the
package returns passes through it. #59

None. Near resonance detection reports divisors of a frequency vector rather
than computing a coefficient, and no path in the package consults it, so nothing
a caller could have pinned moves. Accepting a proposal produces the same
canonical basis the same relations would have produced if they had been declared
by hand, which is asserted by a test rather than by this line. #40

None. The Poisson bracket and the partial derivative are new operations, so
there is no input whose bracket a caller could have pinned and no coefficient
the package already returned has moved. The sign they are written in is the one
`docs/decisions/0004-conventions.md` already fixed, so nothing about the
convention moved either. #30

None. The series arithmetic and the evaluation of a series at a point are the
first numbers the package produces, so they are new rather than moved and there
is no input whose answer a caller could have pinned. #29

The resonance lattice answers a membership question and a count of retained
terms rather than producing a coefficient, so nothing there moved either. #39

The scaling harness measures how long a product takes and computes what it
would cost in memory. It changes no coefficient the package returns. #51

### Added

- A "Changing a workflow" section in the contribution guide, which is where
  somebody about to write one looks. It carries the four rules a workflow here
  has to satisfy, each with the command that shows the tree meets it today:
  actions pinned to a commit, checkout without persisted credentials, write
  scopes granted on the job rather than at workflow level, and no cache restored
  in a job that publishes. It also says what the workflow audit does not decide,
  so a rule the audit does not cover is visibly one a person applies. The
  workflow file that used to restate the rules now points at the section instead
  of carrying a copy of it. #57

- The product and the Poisson bracket across a pool of threads, in
  `drehbank_core::parallel`. The pool size is the caller's, it defaults to what
  the runtime reports, and the answer does not depend on it: the output array is
  partitioned into contiguous chunks of a fixed length, each chunk is written by
  one thread, and no partial sum crosses a thread. What comes back is the same
  bits as the sequential kernel returns, at every pool size, so a run made on
  one machine can be reproduced on another with a different core count. The
  scaling harness now runs on the pool it is given rather than ignoring it, and
  every line it appends carries the thread count and the chunk length the run
  was measured under. #49

- `docs/supply-chain-acceptances.md`, which holds the triage of the supply chain
  self audit. Every check that audit reported on one named commit is answered
  there, either by the change that fixes it or by an acceptance carrying its
  reason and what would end it. The page also says where the audit runs and
  where its findings are published, with the commands that show both. #59

- Near resonance detection, which is advisory and applies nothing. Given a
  frequency vector, a tolerance on the relative divisor and a bound on the order
  of the relations to consider, it returns every multi-index under the
  tolerance, each with its divisor, its order and its relative divisor, together
  with the canonical basis of the lattice they generate. The output is a
  proposal and not a module: the only way it becomes the module in force is an
  explicit acceptance, and a module that came from one records the tolerance and
  the order bound it came from, so a result can say whether its resonance was
  declared or accepted. How large a coefficient each relation would actually
  reach in a given Hamiltonian is available too, because a near resonance whose
  coefficient is zero is not a problem. #40

- The Poisson bracket of two series, and the partial derivative it is built
  from. The sign is the one item 3 of `docs/decisions/0004-conventions.md`
  fixes, under which the evolution of a function is `{f, H}`. The bracket states
  its own truncation order, which is not the order of its arguments: it is the
  sum of the two derivative orders, so a bracket of two series of order `N`
  answers at order `2N - 2` and nothing that the arguments determine is dropped.
  It is the one binary operation that accepts two arguments of different orders,
  because it has no reason to choose between them. #30

- The scaling and measurement harness says what it needs before it runs
  anything. It prints that it is not part of the gate, computes the peak live
  set of each case from the case rather than quoting a number, refuses a case
  that does not fit the memory ceiling it was given, and prints what a skipped
  case would have cost. Measurements that need a privileged hardware counter are
  reported as not made, and no privilege is asked for on any host. Every
  recorded run carries the machine it ran on, in `docs/scaling-runs.md`. #51
- The resonance module as a lattice. A declaration of integer relations is
  replaced by the canonical basis of the lattice they generate, which includes
  the relations the declaration only implies, so two ways of writing one
  resonance name one module. Membership is decided in integer arithmetic with no
  tolerance anywhere, and the number of terms the normal form retains at each
  degree is available before a run is started. #39
- The truncated series, its arithmetic and the coefficient abstraction it is
  written over. A series carries its degrees of freedom and its truncation
  order, so two series that disagree about either are refused rather than
  combined. Addition, subtraction, scaling, negation, the graded product,
  truncation to a lower order and evaluation at a point. #29
- The versioning policy, this changelog and the release checklist. #64

### Changed

None.

### Removed

None.

### Fixed

- The sign-off check no longer sends a contributor to a file that is not here.
  Its failure message named `./DCO`, which does not exist in this tree, at the
  moment somebody is already stuck; it now names the section of
  `CONTRIBUTING.md` that states the rule. The headers of three workflows
  described a release pipeline, an issue tracker and a branch policy belonging
  to another project, and they now describe this one. #57
