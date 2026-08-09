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

### Added

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
