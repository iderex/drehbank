# Security policy

## Reporting a vulnerability

Use the private reporting form on this repository. It is under the Security tab,
as "Report a vulnerability", and it opens a report only the maintainers can read.

Do not open a public issue, a discussion or a pull request for a suspected
vulnerability. The issue tracker is public, and a report there is a disclosure
before there is a fix.

If the private form is unavailable to you for any reason, say so in a public
issue without describing the vulnerability, and a private route will be arranged
from there.

## What a reporter can expect

A report is acknowledged and read. It gets an assessment saying whether it is
accepted, and if it is not, the reason is given rather than the report being
closed in silence.

An accepted report gets a fix on the current development line, and the fix says
what was wrong and what it prevents, which is the same thing every change in this
repository says. A reporter who wants to be named is named; a reporter who wants
to stay anonymous is not named.

No interval is promised here. A published response time this policy could not
keep would be worth less than the absence of one.

## What the surface is

The parts of this package that touch input somebody else produced, or that act on
what an input file declares.

**The file reader.** It parses files an operator may have received from
elsewhere. Both memory safety and unbounded resource consumption live here. A
file header declares a degree and a number of variables, and the number of
coefficients those imply grows combinatorially, so a reader that trusts a header
allocates whatever the header asks for.

**The command line.** It takes paths and parameters and writes files where it is
told to write them.

**The dependency graph.** It is the surface that changes without anybody in this
project touching a line.

## Denial of service by resource exhaustion is in scope

It is stated explicitly because in this package a small input can legitimately
ask for an enormous computation. A ten kilobyte file can name a normal form that
no machine can hold, and that is not by itself an attack; it is what the package
is for.

What separates a legitimate large request from an abusive one is the memory
ceiling. The caller sets a ceiling, and the driver refuses before a group of the
recursion starts when the computed need exceeds it, naming the group, the need
and the ceiling. That is decided in
[`docs/decisions/0009-parallelism-and-memory.md`](docs/decisions/0009-parallelism-and-memory.md).

So the ceiling is a security control and not only an operational one, and the
reportable failures around it are these: an input that causes an allocation
before the check, an input that makes the computed need wrong in the direction
that under-predicts, and any path that reaches an allocation without consulting
the ceiling at all.

## In scope

- Memory safety in the reader or anywhere else: an out of bounds read or write,
  a use after free, an uninitialised read.
- Any panic or process abort reachable from input a user can supply. Errors are
  returned values in this package and a panic is a bug by definition, which is
  the rule in
  [`docs/decisions/0011-api-and-errors.md`](docs/decisions/0011-api-and-errors.md).
- Resource exhaustion as set out above.
- Integer overflow in index arithmetic that produces a wrong index rather than an
  error.
- Anything that causes data to leave the host. The package makes no network
  connection, and that rule and what it covers are in
  [`docs/decisions/0013-data-stays-on-the-host.md`](docs/decisions/0013-data-stays-on-the-host.md).
- A result file, a log line or a diagnostic that discloses the host or the person
  who ran it.
- A dependency reachable from this package that carries a known vulnerability
  this package's use of it reaches.

## Out of scope

- A mathematically wrong answer that is not one of the above. That is a
  correctness defect and it belongs on the public tracker, where it can be argued
  with in the open.
- A published advisory in a dependency with no path from this package reaching
  it. Report those upstream. If you can show the path, it is in scope.
- Anything that needs privileged access to the machine the package already runs
  on. An attacker who is already root does not need this package.
- The hosting platform, its accounts and its infrastructure.
- Missing hardening with no demonstrated failure behind it.

## Supported versions

Before the first release, only the current development line on the default branch
receives fixes. There is no maintained older line, and no version of this package
should be treated as supported until a release is tagged.
