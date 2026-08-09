# The versioning policy

A version number is a promise about what changed. In this package the promise is
easy to break without noticing, because the signature stays the same and the
answer moves.

## The scheme

Three numbers, `MAJOR.MINOR.PATCH`, with the usual meaning. A breaking change
raises the major number, an addition that keeps every existing caller working
raises the minor, and a fix that changes neither the surface nor any result
raises the patch.

The version of the workspace is one number for every crate in it. They are built
together, tested together and released together, so a caller that pins one and
not the others is pinning a combination nothing in the gate has ever run.

## The clause that this package exists to get right

**A change that moves the computed number for the same input on the same build
is a breaking change, even when nothing about the types moves and everything
still compiles.**

A caller who pins a version and gets a different answer from the same input has
had the contract broken as surely as one whose code no longer builds, and only
one of the two hears about it from the compiler. The other reads a paper draft
six months later and cannot reproduce their own table.

There is one exception and it is not an escape. Where the move is the correction
of a defect, the correction still raises the major number and it is named in the
changelog with what was wrong, how it was found, and a command that shows the
difference between the two answers. Correcting a wrong number is not a reason to
say nothing about it; it is the case where saying something matters most,
because somebody has published the old one.

`docs/decisions/0011-api-and-errors.md` holds the full list of what counts as
breaking, including the cases a numerical package usually misses: a change to a
convention, to the monomial ordering, to the meaning of a field in the file
format, and to what an estimate class licenses. The list is there and is not
restated here, because a second copy drifts against the first and the reader has
no way to tell which one is current.

## Before the first release

The public surface is explicitly unstable. It may change in any release,
including in ways that break callers, and version numbers stay below `1.0`.

0011 requires that sentence in three places, in the same wording: the first
screen of the readme, the top of the library documentation next to the
description of the surface itself, and the release notes of every pre-release
version. This policy is what puts it in the third. The other two are not in the
tree today, which is a gap in what 0011 asks for rather than something this page
settles, and the release checklist reads them as items rather than assuming
them.

Nothing carries a release number yet, and no tag exists on the remote a reader
will fetch rather than only in somebody's clone:

    $ git ls-remote --tags origin; echo "exit=$?"
    exit=0
    $ git grep -n '^version = ' origin/main -- Cargo.toml
    origin/main:Cargo.toml:22:version = "0.0.0"

Exit zero with no output is an empty tag list, which is what says the first
release has not happened rather than that the command failed.

## The answer every release records

Every release records, in its notes, whether any result changed.

The answer is written even when it is no, because an absent answer and an answer
of no are different statements and only one of them says somebody looked. Where
the answer is yes, the notes say which case, by how much, and carry the command
that shows it.

That is the same rule as the one the contribution guide states for evidence in
general, and it lands here because a release is the moment the numbers reach
somebody who was not watching the individual changes.

## Pre-releases and yanking

A pre-release is the ordinary version with a suffix, `1.2.0-rc.1`, and it makes
the same promises as the release it precedes. It is not a place to put a change
that has not been argued.

A release that is found to produce a wrong number is not quietly replaced. The
correction is a new version under the rule above, and the notes of the new
version name the old one and what it got wrong. Removing the old artefact where
a registry allows it is a separate decision, and it does not substitute for
saying what was wrong, because somebody has already downloaded it.

## What refuses a violation of this

Nothing. There is no check that a change moving a number raises the major
number, that the changelog carries an entry, or that the release notes answer
the question above. What machines refuse in this repository is what the
workflows in `.github/workflows/` do, and none of them reads a version, a tag or
this page.

So every rule here is a rule somebody follows. `docs/release-checklist.md` is
the place a person walks through before a tag, and it is an administrative
control rather than an engineering one. Where that changes, this section
shrinks.
