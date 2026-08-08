# What leaves the host

Nothing.

This page is an inventory of what this software sends and where, written for an
operator who has to answer that question to somebody holding a checklist. It is
narrower than [the notice](../NOTICE.md), which is about lawful use. This page is
only about what crosses the edge of the machine.

## Read this first

**The tree carries no source code yet.** Every statement below describes what the
package does, and today there is no package: the repository holds decisions,
documents and workflow files and nothing that runs on an operator's machine. The
statements are therefore true of nothing rather than measured on something, and
this paragraph is here so that nobody reads the page as a report on a program
that was examined. It is what the software will hold to, written down before the
code exists so the code has to meet it. This paragraph is removed on the day
there is something to describe, and not before.

## The inventory

**No network connection.** The package makes none. It sends no telemetry, no
usage report and no crash report, and it does not check for updates. There is no
setting that turns any of that on, because a setting that is off by default is a
setting that can be on.

**Only the files you name.** It reads the files it is pointed at on the command
that ran, and writes the files it is told to write on that same command. It reads
no configuration file it was not given, and it writes nothing beside its output.

**Result files carry no machine identity.** The full list of what a result file
records is below, and the full list of what it deliberately does not is below
that.

**Diagnostics quote your path and nothing else.** A message that has to name a
file names the path you supplied, exactly as you supplied it, and nothing derived
from the environment. So a diagnostic pasted into a bug report carries no home
directory and therefore no user name.

## What a result file records

Item for item, this is the schema of the decision on the file format, which is
`docs/decisions/0010-file-format.md`. If the two ever disagree, that document is
the authority and this page is the defect.

Identity and version:

- the format version,
- the kind of file,
- the package version that wrote it,
- a SHA-256 digest of the bytes of the input file.

Shape:

- the number of degrees of freedom,
- whether the series are in the real or the complex coordinates,
- the coefficient type,
- the truncation order.

Conventions, written out in full so the file says what it means without this
repository to hand:

- the conventions document the next records come from,
- the variable order,
- the Poisson bracket sign,
- the Lie operator,
- the complexification,
- the normalisation of the quadratic part,
- the monomial ordering.

The run:

- the frequency vector as given,
- the order asked for and the order reached,
- the polydisc the estimate is stated on, and whether it was given or derived,
- the small divisor threshold and whether it is the default,
- the smallest relative divisor actually divided by, with its multi-index and
  the group it occurred in,
- the chunk target that fixes the partition.

The resonance module:

- the canonical basis in force, with the relative divisor of each basis vector,
- whether it was declared or accepted from a detection, and the tolerance and
  order bound in the latter case,
- any relation detection found that the module in force does not contain.

The series and the estimate:

- the normalised Hamiltonian,
- the generating function,
- the estimate: its class, the order, the value, the optimal order and the value
  there, whether the order reached is at, before or past that optimum, the
  coefficient type, and every hypothesis assumed.

Every one of those is about the mathematics, the input or the parameters you
chose. None of it is about you or about the machine.

## What a result file deliberately does not record

- the host name,
- the user name, the user id, or the home directory,
- the absolute path of the input or of the output,
- the wall clock time or the time zone,
- the machine identity in any form: CPU model, core count, serial number,
  network address, or machine id,
- the operating system and its version,
- the number of threads the run used,
- environment variables and locale.

Two of those are worth a sentence.

The **thread count** is absent while the chunk target is present, which looks
inconsistent and is not. The chunk target is part of what determines the answer.
The thread count determines nothing, so recording it would invite a reader to
believe it did.

The **wall clock** is absent, so a result file carries no date. A file that
travels into a paper's supplementary material carries no timestamp with it, and
where you keep the file is where the date lives.

## What is checked and what is stated

**None of this is enforced by a machine today, and the check that would enforce
part of it has not been built.**

The check is a refusal over the whole dependency graph of the core library: no
package reachable from the core, at any depth, may be capable of opening a
socket. That is issue #25 and it does not exist yet. Until it lands, the first
line of the inventory above is a claim in prose, and the honest reading is that
nothing in this repository refuses a network-capable dependency arriving
tomorrow.

The other lines are claims in the same sense. Reading only the files you name,
recording no machine identity and quoting only your own paths are properties of
the code, checked by the suite that tests the code, and there is no code yet.

This section states the gap rather than covering it. It is not rewritten into an
assurance when the check lands; it is rewritten to say what the check refuses and
what it still does not reach.

## If a later version can share results

If a later feature ever sends a result anywhere, for instance to compare it
against a published catalogue, that is federation, and federation is deliberate.
Deliberate means all four of these at once:

- you ask for it on the invocation that does it, by an argument on that command,
- this page states, before you ask, which fields leave the host and where they
  go, as a list and not as a description,
- it is off by default,
- there is no way to turn it on globally and forget. No environment variable, no
  configuration file entry, no build feature and no installed default enables it.
  If it is not on the command that ran, it did not happen.

The fourth is the one that stops it arriving as a convenience.

## Where this is decided

The fields are `docs/decisions/0010-file-format.md`, which is where the schema
above comes from. The rule that personal data never leaves the host is decision
0013, which is on an open pull request and not yet on the default branch, so this
page names it by number rather than by a path that would not resolve. The
mechanical check is issue #25.

If a future dependency does open a socket, this page changes rather than being
defended.
