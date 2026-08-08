# The dependencies, and the position on how many there should be

This page lists every direct dependency of every crate in this workspace, with
the reason it is there and what would have to be written by hand to remove it.
Adding one is a change to this page as well as to a manifest.

## The list

One, and it is on the test edge rather than on the shipped one. What a caller
links is still the workspace and nothing else:

    $ cargo tree --workspace --locked --offline --edges normal --prefix none \
        | sed 's/ (.*//' | sort -u
    drehbank-cli v0.0.0
    drehbank-core v0.0.0
    drehbank-scaling-harness v0.0.0

    $ cargo tree --package drehbank-core --locked --offline --edges dev \
        --depth 1 --prefix none | sed 's/ (.*//' | sort -u
    drehbank-core v0.0.0
    proptest v1.11.0

The `sed` drops the checkout path each line ends with, which is a fact about
whoever ran it rather than about the tree.

### proptest 1.11.0, dev-dependency of drehbank-core

**What it is for.** Generating cases for the properties in
`crates/drehbank-core/tests/`, and shrinking a failure down to the smallest case
that still fails. Issue #19 asks for a property harness with a fixed seed
policy, and the shrinking is the half that is worth a dependency: a property
that fails at six variables and degree eight hands back a monomial nobody can
reason about, and the same failure shrunk to three variables and degree one is
the two-line fixture in `tests/fixtures/monomial-index/`.

**What removing it would cost.** The generators are a few dozen lines and the
tree already carries a hand-rolled xorshift in a test, so that half is cheap.
The shrinker is not: shrinking a compound value while keeping it inside the
strategy that produced it is the part that gets written wrong quietly, and a
shrinker that reports a case that does not actually fail is worse than none.
Call it the generators plus a correct integrated shrinker, and the shrinker is
the reason the answer here is a dependency.

**Its license, read from the crate's own metadata.** `MIT OR Apache-2.0`, from
the resolved graph rather than from memory:

    $ cargo metadata --format-version 1 --locked --offline \
        | tr '{' '\n' | grep '"name":"proptest"' | grep -o '"license":"[^"]*"'
    "license":"MIT OR Apache-2.0"

Every package it pulls in offers a permissive option too. One of them,
`r-efi`, offers `LGPL-2.1-or-later` as a third alternative, which is a choice
this repository does not have to take. That is a reading of the metadata and not
a legal opinion, and the license of this repository itself is still the first
open entry in issue #2.

**Whether it reaches the numbers.** No. It is a dev-dependency, so it is absent
from the first command above, and no coefficient passes through it. It does
reach the apparatus that judges the coefficients, which is a smaller risk and
not a zero one: a generator that quietly never produces a case is a suite that
passes over nothing, which is why
`every_regression_case_in_the_fixture_directory_holds` refuses an empty fixture
directory rather than passing over it.

### What it cost the graph

Thirteen packages on this host, twenty-three in the lockfile across every
platform it resolves for. The lockfile is the larger number because it carries
the platform-gated ones, `libc`, `r-efi`, `wasip2` and `wit-bindgen` among them,
which are in the file and are not built here.

    $ cargo tree --package drehbank-core --locked --offline --edges normal,dev \
        --prefix none | sed 's/ (.*//' | sort -u | grep -cv '^drehbank-core'
    13

`default-features = false` with `std` put back is what keeps it to that. The
default feature set adds process forking, a timeout thread and a temporary file
crate for persisting counterexamples, none of which this suite uses. Measured
by resolving both ways at the same commit: thirty-seven external packages with
the defaults, twenty-three without them.

That is the largest single move this graph has made, from nothing to twenty-three
in one change, and it is stated as the cost rather than folded into the entry
above.

## The position

Very few, and each one argued.

The reasoning is specific to what this package is rather than a general dislike
of dependencies. A normal form computation produces a number that looks exactly
like the right number when it is wrong. Every dependency is code that can change
that number and that this project does not test, does not read on upgrade and
cannot prove a property of. The cost of carrying one is therefore not the
download or the build time, it is that the set of code capable of producing a
plausible wrong answer grows by an amount nobody here has measured.

Against that, writing a well-tested library by hand is also a way to produce a
wrong number, and doing it badly is worse than depending on something the
community has hammered on for a decade. Arbitrary-precision arithmetic is the
obvious case: a hand-rolled one would be slower, buggier and unproven next to any
of the established implementations.

So the rule is not a count. It is that each dependency arrives with the four
answers below written down, and a dependency nobody can write them for does not
arrive.

## What a new dependency has to carry

Every entry on this page states, in this order:

**What it is for.** The operation this package cannot do without it, named
specifically enough that somebody can check whether the crate is still doing only
that.

**What removing it would cost.** What would have to be written by hand, and
roughly how much of it. An entry that cannot answer this is an entry for a
convenience rather than a need.

**Its license, read from the crate's own metadata at the time of the decision.**
Not from memory and not from this page later, because a license can change
between releases and several of the fastest implementations of the arithmetic
this package wants are bindings onto a copyleft C library and carry that
library's terms. Whether a copyleft dependency is acceptable at all is an open
entry in issue 2 and is not decided here.

**Whether it reaches the numbers.** A crate in the computation path and a crate
in the command line parser are different risks, and the boundary the workspace
declares is what keeps them apart: the core does not depend on the command line,
and a dependency that reaches the core reaches every coefficient.

## The one that is already spoken for

0010 takes a dependency on a SHA-256 implementation for the input digest and
gives its reason there. That entry is written and belongs on this page as soon as
the crate is chosen. It is listed here as owed rather than as present, because
nothing in the tree declares it yet and a page that anticipated it would be
describing a graph that does not exist.

## What refuses a violation of any of this

Nothing reads this page. There is no check that a new dependency arrives with an
entry here, that the entry names a license, or that the license matches what the
crate declares.

What machines do refuse is narrower and is worth knowing exactly. `Dependency
review` fails a pull request that introduces a package with a known advisory, at
any severity. `Advisory scan` fails weekly and on every pull request when the
resolved graph contains a vulnerability, an unmaintained crate or an unsound one.
Both read advisories. Neither reads a license, a count, or a reason, and neither
would notice a dependency added with no entry on this page.
