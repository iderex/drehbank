# Contributing to drehbank

This package computes normal forms. A wrong number here looks exactly like a
right one, so most of what follows is about evidence rather than about style.

## Build and test

The toolchain is pinned in `rust-toolchain.toml` and rustup picks it up from the
repository, so there is nothing to select by hand. What version you actually got
is worth checking before you report anything:

    rustup show active-toolchain
    cargo --version
    rustc --version

Fetch the dependency set the lockfile names, without letting it move:

    cargo fetch --locked

Compile everything, including the scaling harness that no ordinary build
touches:

    cargo build --workspace --all-targets --locked --offline

The gate runs that step with `RUSTFLAGS` set to `-D warnings`, so a warning
there is an error. Set the same variable in your shell before you run it or you
will see green locally and red in the gate. The two warnings that matter most
here, an unused result and a comparison that is always true, are both symptoms
of an index mistake.

Run the suite:

    cargo test --locked --offline

That is the default set. It leaves out the scaling harness on purpose, because
what the harness runs needs a real machine and hours, and a suite that quietly
tries it fails on every laptop.

Every command above is a string `.github/workflows/build-and-test.yml` runs,
character for character, so a red check reproduces locally and a local pass
means something. If you find a difference between this file and that workflow,
the workflow is right and this file is a defect.

`--offline` is there so that a dependency that cannot be resolved is a different
failure from a compile error. If it refuses, run the fetch step again and read
what it says.

## Format and lint

Check the formatting:

    cargo fmt --all -- --check

`--check` writes nothing. It prints the difference and exits non-zero; drop it
and the formatter applies the change instead. `--all` reaches every workspace
member, so the scaling harness is formatted like everything else even though no
ordinary build touches it.

Lint the workspace:

    cargo clippy --workspace --all-targets --locked --offline

Neither command carries a level. The lint levels are in the `[workspace.lints]`
table in `Cargo.toml`, which is what `cargo clippy` reads, so the set the gate
refuses and the set you see are the same set and neither depends on a flag you
have to remember. The whole default compiler set and the whole default clippy
set are errors here, and so is `unsafe`.

Where a lint genuinely has to be allowed, allow it at the site with the reason
written next to it, so that it is one thing somebody argued. Lowering the level
in `Cargo.toml` turns it off for the tree and for everyone after you.

The formatter's own settings are in `rustfmt.toml`, and there are two of them.
The rest is rustfmt's defaults on purpose.

Both commands in this section are strings
`.github/workflows/format-and-lint.yml` runs, character for character, under the
same rule as above: where this file and that workflow differ, the workflow is
right and this file is the defect.

## Sign your work

Every commit carries a `Signed-off-by` line matching its author, which is the
Developer Certificate of Origin assertion, and the check refuses a commit
without one. Git writes the line for you:

    git commit -s

Whether contributions from outside are accepted at all is an open question in
#2 and is not answered here.

## What an issue has to say

Three things: what is wrong, what the evidence is, and what done means.

Where the evidence is a number, the command that produced it goes with it. This
is not decoration in a numerical package. "The bound got tighter" and "this is
forty per cent faster" are unreproducible by the next reader without the
invocation, the input and the machine, and the next reader is usually the person
who wrote it, a year later.

Where a number came from a measurement that was not made, say that instead. A
missing measurement and a measurement of zero are different statements, and the
second one is worse than saying nothing.

Done means a condition somebody else can check. "Improve the estimate" is not a
condition. "The falsifier runs ten thousand cases at order six and finds none
above the bound" is.

## What a change has to carry

Which decision it depends on. The decisions live in `docs/decisions/`, one file
each, and they are the reason the code is shaped the way it is. A change that
contradicts one is not wrong by definition, but it changes the decision first,
in its own change, and says why.

Where it adds a guard, the proof that the guard refuses what it names. A guard
that has never refused anything is untested by construction, and the way to
prove one bites is to remove it and watch the suite go red for the reason the
guard is about. Put that in the pull request body. A near-miss that could not
have failed proves less than one that nearly did, so pick the one-character
mistake somebody will actually make.

Where it adds a dependency, an entry in `docs/dependencies.md`. That page holds
the position on how large the graph should be and the four things every entry has
to answer, and a dependency arriving without one is a dependency nobody argued.

Where it changes what a user sees, which of the three documents that touches.

## The breaking change rule

A change to any convention, to the meaning of any field in the file format, or
to the numerical result on an unchanged input is a breaking change, even when
no signature moves and everything still compiles.

That last clause is the one that gets missed. A caller who pins a version and
gets a different answer from the same input has had the contract broken as
surely as one whose code no longer builds, and only one of the two finds out
from the compiler. The conventions are in `docs/decisions/0004-conventions.md`,
the file format is in `docs/decisions/0010-file-format.md`, and the shape of
this rule is argued in #13.

## What is not accepted

A speedup that changes results. If the numbers move, it is not a speedup, it is
a different computation, and it arrives as one: what changed, on which case, by
how much, and why the new answer is the right one.

A new numerical path with no test that could distinguish it from a wrong one. A
test that only checks the code ran, or that the output has the right shape, does
not distinguish anything. What distinguishes is an exact case computed
independently, an identity that has to hold, or a comparison against a published
value with its source named.

A number without the machine it was measured on. Core count, memory, operating
system and toolchain version, or it is not a measurement.

## What is not enforced

Nothing in this repository reads this file. There is no check that a pull
request body carries the guard proof, that an issue states its evidence, or that
a commit message says what failure it prevents. What a machine refuses today is
what the workflows in `.github/workflows/` do, and that directory is the
authority for what those are rather than a list in this file, which would drift
against it.

So every rule above is a rule somebody follows, not one the tree can stop you
from breaking. Where that changes, this section shrinks.
