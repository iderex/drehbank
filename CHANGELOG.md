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

None. The series arithmetic and the evaluation of a series at a point are the
first numbers the package produces, so they are new rather than moved and there
is no input whose answer a caller could have pinned. #29

The resonance lattice answers a membership question and a count of retained
terms rather than producing a coefficient, so nothing there moved either. #39

The scaling harness measures how long a product takes and computes what it
would cost in memory. It changes no coefficient the package returns. #51

### Added

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

None.
