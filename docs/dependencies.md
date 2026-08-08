# The dependencies, and the position on how many there should be

This page lists every direct dependency of every crate in this workspace, with
the reason it is there and what would have to be written by hand to remove it.
Adding one is a change to this page as well as to a manifest.

## The list

There are none. Every crate in the workspace declares either nothing or a path
edge onto another crate in the same workspace:

    $ grep -n -A3 '^\[dependencies\]' crates/*/Cargo.toml
    crates/drehbank-cli/Cargo.toml:13:[dependencies]
    crates/drehbank-cli/Cargo.toml-14-drehbank-core = { path = "../drehbank-core" }
    crates/drehbank-core/Cargo.toml:12:[dependencies]
    crates/drehbank-scaling-harness/Cargo.toml:15:[dependencies]

    $ cargo metadata --format-version 1 --locked --offline \
        | tr ',' '\n' | grep -o '"name":"[^"]*"' | sort -u
    "name":"drehbank"
    "name":"drehbank_core"
    "name":"drehbank-cli"
    "name":"drehbank-core"
    "name":"drehbank-scaling-harness"

Five strings, three packages and two target names, and nothing from outside. So
the resolved graph is the workspace, and the count above is a fact about this
tree at the commit it was run on rather than a target anyone is holding to.

That is not a boast and it does not stay true. The first one this package wants
is named below.

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
