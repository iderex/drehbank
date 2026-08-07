# 0013. Personal data never leaves the host

Status: decided, with one agreement named below that cannot be checked yet.
Raised in issue #15.

## The rule

Personal data never leaves the host. The package makes no network connection,
sends no telemetry, reports no usage, checks for no update and uploads no crash
report. It reads the files it is pointed at, writes the files it is told to
write, and does nothing else.

The absence is structural rather than incidental. It is not a setting that is off
by default, because a setting that is off by default is a setting that can be on.

## Why a mathematics library needs this rule at all

Personal data reaches this package in ordinary ways. A file path contains a home
directory and therefore a name. An input file carries whatever the operator put
in its metadata. An error message quoting a path carries the name onward into a
log, a bug report or a paper's supplementary material. A crash report would carry
all of it at once, which is why there is none.

So the rule is not only about connections. It is also about what gets written
down, because a file that records the host is a disclosure waiting for the moment
somebody shares the file.

## What a result file records

A result file records:

- the package version,
- the digest of the input file,
- the target order and the order actually reached,
- the polydisc radii the estimate is stated on,
- the small divisor threshold in force,
- the resonance module as declared, and any resonance detected and not declared,
- the coefficient type the computation ran in,
- the chunk target that fixes the partition, so that the result can be
  reconstructed,
- the estimate object of 0008 in full.

## What a result file deliberately does not record

- the host name,
- the user name, the user id, or the home directory,
- the absolute path of the input or of the output; a diagnostic that must name a
  file names the path the operator supplied and nothing derived from the
  environment,
- the wall clock time or the time zone,
- the machine identity in any form: CPU model, core count, serial number,
  network address, or machine id,
- the operating system and its version,
- the number of threads the run used, which is a property of the machine and,
  by 0009, cannot change the answer,
- environment variables and locale.

Each of those is a deliberate omission. The rule for anyone adding a field later:
if a field would let a reader of the file identify the machine or the person who
ran it, it does not go in the file, and no argument about debugging convenience
overrides that.

**This list has to agree with 0010, and that cannot be checked yet.** The
on-disk format is decided in 0010, which has not landed. Until it does, the list
above is the constraint the format has to satisfy, not a statement about what the
format says. Issue #15 stays open on that agreement, and it is checked by reading
the two lists against each other once 0010 exists.

## The mechanical check

The rule holds by construction, and the construction is checked by a machine
rather than asserted in a document.

The check is a refusal over the whole dependency graph of the core library: no
crate reachable from the core, at any depth, may be capable of opening a socket.
A future dependency that pulls in a network stack fails the check, and the
failure is the point, because it is what makes anyone notice a capability
arriving that nobody asked for.

That check is built in issue #25 and is not built here. This document is the
decision and the reasons; issue #25 owes the mechanism, and the transitive scope
and the failure behaviour are settled there. Until it lands, this rule is a rule
in prose, and the honest reading of the paragraph above is that nothing in the
tree refuses a network-capable dependency today.

## Federation, if it is ever wanted

If a later feature shares results outside the host, a comparison against a
published catalogue for instance, then that is federation, and federation is
deliberate.

Deliberate means all four of these at once:

- the operator asks for it on that invocation, by an argument on that
  invocation,
- the documentation states, before they ask, which fields will leave the host and
  where they go, as a list and not as a description,
- it is off by default,
- there is no way to turn it on globally and forget. No environment variable, no
  configuration file entry, no build feature and no installed default enables it.
  If it is not on the command that ran, it did not happen.

A feature that phones home to be helpful is exactly the thing this rule forbids,
and the fourth condition is the one that stops it arriving as a convenience.

## Where a reader finds this

The user-facing documentation says it in plain words, on the first screen, in the
place a person reads before installing rather than in an appendix. An operator in
a regulated setting has to be able to answer what the tool sends and where, and
the answer has to be findable in under a minute. Issue #24 is where that sentence
is written into the documentation, and it says the same thing as this document
rather than a summary of it.

## What this document does not decide

It does not decide the on-disk format, which is 0010 and which this document
constrains. It does not build the dependency check, which is issue #25. It does
not decide what the documentation says about anything else, which is issue #24.
