# 0001. The implementation language and toolchain

Status: decided. Raised in issue #3.

## The decision, in one sentence

drehbank is written in Rust on a pinned stable toolchain declared in the
repository, with cargo as the only build entry point.

## The pinned toolchain version

    1.97.0

That string is what the repository toolchain file carries, and issue #16 is where
that file is laid down. The two are required to be the same string, and this
document is the authority for what the string is.

The version was read from a stable toolchain rather than chosen from a release
calendar:

    $ rustc --version
    rustc 1.97.0 (2d8144b78 2026-07-07)
    $ rustup show active-toolchain
    stable-x86_64-pc-windows-msvc (default)

Raising the pin is a change like any other. It gets an issue, it says what moved,
and it lands on its own so that a compiler upgrade never arrives inside a change
about mathematics.

## What the artefact has to carry

The work is dense index arithmetic over exponent vectors, performed billions of
times, with the term count growing combinatorially in the order and in the number
of variables. It has to spread across cores without the answer depending on how
many cores were used. It has to hold a large working set without a garbage
collector deciding when to walk it. It has to be installable by an operator who
is not going to build a compiler first. It has to be testable, lintable and
benchmarkable by tools that already exist, rather than by an apparatus this
project invents and then has to maintain for as long as it lives.

## Why this means fits that

The test, benchmark, format and lint apparatus already exists and is one tool. A
project that has to assemble its own harness spends its first months there and
its later years maintaining it, and the harness is what decides whether the
quality parity milestone is reachable at all.

The code is almost entirely manual index arithmetic into flat buffers. That is
the exact shape where an off-by-one reads or writes outside its buffer and
produces a plausible wrong number instead of a crash. A wrong coefficient in a
normal form is not visible by looking at it, and no test that checks a plot would
catch it. Bounds behaviour that is defined rather than undefined is worth more
here than in most projects.

Term-level data parallelism over independent chunks with a reduction in a fixed
order is well travelled in this ecosystem. Determinism across thread counts is a
requirement of its own and is decided in 0009.

There is no runtime for the operator to install and no garbage collector to tune
against a working set of tens of gigabytes. A single static binary and a linkable
library come out of the same build.

A C ABI is available later if a binding for another language is wanted, without
rewriting the core. Whether the first release ships such a binding is not settled
here; it is an entry in issue #2.

## What it costs, stated rather than skipped

The symbolic algebra ecosystem here is thinner than in C++ and much thinner than
in the computer algebra systems this field usually reaches for. Anything
algebraic beyond truncated polynomial arithmetic is written in this repository or
is not used.

Exact rational and arbitrary-precision integer arithmetic comes from a
dependency, and the license of that dependency interacts with the license of this
repository. That coupling is an entry in issue #2 and is not settled here. The
consequence for this document is narrow: the choice of language does not decide
which exact-arithmetic crate is used, and no work that depends on that answer
starts before the answer is written into #2.

Compile times on a heavily generic numeric core become unpleasant. The response
is to keep the generic surface small rather than to accept the cost, and 0002 is
where the size of that surface is fixed.

## The alternatives, and what each would have cost

C++. The strongest case against the decision. Mature numerical libraries, a
compiler on every cluster, and the language the in-house accelerator codes are
already written in, which matters for adoption more than any argument in this
document. It loses on the apparatus. Testing, benchmarking, linting, dependency
pinning and reproducible builds are each a separate choice that each project
makes differently, and this project would have spent its scaffolding milestone
assembling them instead of doing mathematics. The memory-safety argument is real
and is secondary to that one.

Julia. The fastest of the candidates to write this particular mathematics in, and
the community closest to the audience. It would have cost delivery. The operator
gets a package that needs a Julia installation, pays a startup cost per
invocation, and is harder to pin reproducibly than it should be. A garbage
collector against a working set this size is an unforced risk, and the memory
ceiling in 0009 is much harder to enforce when allocation is not the caller's to
observe.

Python with a native kernel. The audience is there, and it is the shortest path
to somebody trying the package. It would have cost the same question twice. The
kernel ends up being the whole project, so the language decision is only
postponed, and the postponed version arrives with two toolchains and two test
harnesses to gate instead of one. A binding is a separate question from what the
core is written in, and it is an entry in issue #2.

A computer algebra system as the host. Exact arithmetic and series manipulation
would have come for free. It would have cost everything else: no control over
storage layout, which 0003 shows is the whole cost model; no useful parallelism
at this scale; and no artefact an operator can install.

## What this document does not decide

It does not decide the coefficient types, which is 0002. It does not decide any
dependency, and in particular not the exact-arithmetic dependency, whose license
is an entry in issue #2. It does not lay down the workspace or the build files,
which is issue #16.

Until this document existed, no build file was allowed in the tree. The tracked
tree carried none when this document was written, and the check is a grep over
the index rather than a look around the working directory:

    $ git ls-files | grep -Ei '(^|/)(Cargo\.toml|Cargo\.lock|Makefile|CMakeLists\.txt|build\.rs|meson\.build|BUILD\.bazel)$'; echo "exit=$?"
    exit=1

Exit 1 from grep is no match, which is the reading wanted here. Nothing refuses a
build file that lands before a decision document, so this is a statement about
what was true and not about what is prevented.
