# The release checklist

A document a person walks through before a tag, top to bottom, writing the
result of each item down as they go. Nothing here is automated and nothing here
is a check a machine refuses, which is why each item asks for an output rather
than for a tick.

The record of a walk belongs in the release's own issue, so that a later reader
can see what each item said on the day rather than that somebody said it was
fine.

## What this checklist can and cannot be walked against today

Several items below name an artefact that does not exist yet. They are written
now rather than when their artefact lands, because a checklist assembled at the
first release is a checklist nobody has read, and because an item naming a thing
that is missing is the cheapest way to see what a release still needs.

Each such item says which issue owes its artefact. Walking the checklist before
those land means writing "the artefact does not exist" against the item, which
is a result, and a release that goes out with several of those is a release
whose notes say so.

## 1. The gate is green on the commit being tagged

Not on the branch, not on an earlier commit, and not on a run somebody
remembers. Read the checks on the exact commit:

    gh api repos/iderex/drehbank/commits/<sha>/check-runs \
      --jq '.check_runs[]|"\(.name)=\(.conclusion)"' | sort

Paste the output. Every check `success`, and the list itself is worth reading:
a check that is absent from it did not run, and an absent check and a green one
look nothing alike on the page but exactly alike in a summary.

## 2. The changelog is complete, including its numerical section

`CHANGELOG.md` carries a line for every change a user could notice since the
last release, and the **Numerical results** section is filled in, with `None.`
where nothing moved.

The one to read carefully is the numerical section, because a wrong entry there
is a wrong statement about somebody's published table. Where a number moved,
its line names the case, the size of the move, and the command that shows it.

Move the `Unreleased` heading to the version being released and open a fresh
`Unreleased` above it, with its own empty sections, in the same change.

## 3. The release notes answer whether any result changed

Explicitly, in words, even when the answer is no. `docs/versioning.md` says why
the answer is written rather than left out.

The pre-release wording of `docs/decisions/0011-api-and-errors.md` goes in the
notes of every version below `1.0`: the public surface is explicitly unstable
and may change in any release, including in ways that break callers.

## 4. The version number follows the policy

Read `docs/versioning.md` and
`docs/decisions/0011-api-and-errors.md` before choosing the number rather than
after. A change to a convention, to the monomial ordering, to the meaning of a
field in the file format, to what an estimate class licenses, or to the computed
number for the same input is a breaking change even where nothing about the
types moved and the whole gate is green.

The number is one number for every crate in the workspace.

## 5. The worked examples reproduce their expected output

Run them and paste what they printed, rather than the fact that they ran.

Owed by issue #61, which is where the examples and the gate step that runs them
land. Until then the result of this item is that there are no worked examples.

## 6. The documentation is checked against the conventions decision

`docs/decisions/0004-conventions.md` is the authority. Every formula and every
sign in the user-facing documentation is read side by side against it, and the
result of the reading is written down.

This is the item most likely to be skipped, because a code change can make a
conventions page wrong without touching it and nothing goes red. The three
documents this reads are owed by issue #62.

## 7. The scaling record is updated where the scale milestone was touched

If anything under `M7` changed, the recorded runs are re-run or the record says
plainly that they were not, with what stopped them.

An extrapolation is labelled as one and carries its method. A number with no
machine attached is not a measurement: core count, memory, operating system and
toolchain version, or it does not go in. Owed by issues #51 and #52.

## 8. The bill of materials and the provenance are attached

A bill of materials in a standard format, listing every dependency with its
version and license, attached to the release artefacts. Provenance attestation
emitted for every artefact.

Then verify them, against the real artefact, and paste the verification output.
An attestation nobody has verified is a file. Owed by issue #58.

## 9. The dependency page matches the graph

`docs/dependencies.md` lists every direct dependency with its reason, its
license read from the crate's own metadata, and whether it reaches the numbers.
Re-read the graph rather than the page:

    cargo tree --package <member> --locked --offline --edges normal,dev --depth 1 --prefix none

An entry on the page for a dependency that is gone, or a dependency in the graph
with no entry, is a defect in the page and is fixed before the tag rather than
after.

## 10. The artefact is verified on a clean machine

Not on the machine that built it. Owed by issue #63, which is where the
packaging and the clean-machine verification land.

## 11. The tag

Only after every item above has a written result. Owed by issue #65.

## What refuses a violation of this

Nothing. No check reads this file, no check requires that a walk happened, and
nothing compares the version number against what changed. Every item here is an
item somebody does, and the record of the walk is the only evidence that any of
them was done.

That is the same position `docs/versioning.md` and the contribution guide both
state about themselves, and it is stated here rather than left to be assumed
from the absence of a red check.
