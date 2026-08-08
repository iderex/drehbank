# 0010. The on-disk format for a Hamiltonian and for a result

Status: decided, with one agreement named below that is checked across two
branches rather than one. Raised in issue #12.

## The decision

One text format, line oriented, ASCII, with a required version on the first line,
a canonical writer under which the same result written twice is byte identical,
and every convention named in the file rather than implied by it.

A run of this package can take a day, so its output has to survive being written
down, read back and compared against a run from a year earlier. The format is an
interface with the same status as the public surface of 0011 and is decided
before anything writes a file.

## The shape

A file is a sequence of lines separated by a single line feed, with a line feed
after the last one. Every byte is ASCII. There are no carriage returns, no
trailing spaces, no tabs and no blank lines.

A line is either a header record, a block delimiter or a term.

    <key> <value>              a header record, one space, value to end of line
    begin <block> <name>       opens a block
    end <block>                closes the most recently opened block
    t <a_1> ... <a_m> <coeff>  a term, inside a series block

Keys and block names are lower case with hyphens. Every header record of a file
appears before the first block.

## The first line, and what a reader does with it

    drehbank-format <major>.<minor>

This is the first line of every file, it is required, and a reader parses nothing
else until it has read it. An unversioned format is a format that can never
change, which is why this is required rather than optional and first rather than
somewhere.

A reader refuses a major it does not know, naming the major it found and the
majors it supports. Within a known major it refuses an unknown key too, naming
the key, the minor in the file and the minor it supports.

Refusing an unknown key is the fail-closed direction and it is a deliberate
trade. A field carrying meaning is never dropped in silence, and the cost is that
an older binary cannot read a file written by a newer one. Adding a field bumps
the minor. Under the rule of 0011 that is not a breaking change to the package,
because no existing field changes meaning and no caller stops compiling, and this
sentence is here so the two documents are not read as disagreeing.

## Every field a result file must carry

A result file that is missing any of these is refused, naming the field. The list
is the schema, and a reader may rely on all of it being present.

**Identity and version.**

    drehbank-format     the format version, first line
    kind                normal-form-result
    package-version     the version of the package that wrote the file
    input-digest        sha256:<64 hex characters>, over the bytes of the input file

**Shape.**

    degrees-of-freedom  v, so the number of variables is m = 2v
    variables           real or complex, which of the two coordinate sets the
                        series below are expressed in
    coefficient-type    binary64, rational or interval, from 0002
    truncation-order    the highest degree present in the series below

**Conventions, named rather than implied.** Each of these is a literal statement
and not a reference, so a file read without this repository to hand still says
what it means. A file that does not say which bracket sign it used is a file that
will eventually be read with the wrong one.

    conventions         0004, the document the five records below are taken from
    variable-order      q1..qv,p1..pv
    bracket-sign        df/dq_j*dg/dp_j - df/dp_j*dg/dq_j
    lie-operator        L_chi f = {f, chi}
    complexification    x_j=(q_j+i*p_j)/sqrt2, y_j=(i*q_j+p_j)/sqrt2
    quadratic-form      H2 = sum_j omega_j*(q_j^2+p_j^2)/2
    monomial-order      grevlex-ascending

**The run.**

    frequency           the frequency vector as given, v coefficients, in the
                        sign convention of 0004 item 7
    target-order        the order the run was asked for
    order-reached       the order it reached, which is lower when it stopped early
    polydisc            the radius vector the estimate is stated on, m coefficients
    polydisc-source     given or derived, so a reader can tell a domain the user
                        chose from one the package picked
    divisor-threshold   theta, the relative divisor threshold of 0006
    threshold-default   yes or no
    worst-divisor       the smallest relative divisor actually divided by, its
                        multi-index k and the group it occurred in
    chunk-target        T, the chunk length of 0009, so the partition that fixes
                        the reduction order can be reconstructed

**The resonance module,** as 0007 leaves it.

    begin module in-force
      the canonical basis, one relation per line, with the relative divisor of
      each basis vector
    end module
    module-source       declared, or detected with the tolerance and order bound
                        the proposal came from
    begin module detected-not-declared
      any relation detection found that the module in force does not contain,
      with its relative divisor
    end module

**The series.**

    begin series normalised-hamiltonian ... end series
    begin series generator ... end series

**The estimate,** which every result carries because 0008 admits no result
without one.

    begin estimate
      class             formal, numerical or rigorous
      order             the order the value refers to
      value             absent for the formal class, which carries no number
      optimal-order     the order at which the bound is smallest on this polydisc
      optimal-value     the value there
      position          at, before or past the optimum
      coefficient-type  the type the computation ran in
      hypotheses        every hypothesis assumed, one per line
    end estimate

A Hamiltonian file carries the identity, shape and convention records, one series
block, and the frequency vector where it is known. It carries none of the run
records, no estimate and no module.

## How a series is written

A series block holds one term per line and nothing else:

    t <a_1> ... <a_m> <coefficient>

with `a` the exponent vector in the variable order named in the header, so the
file does not depend on the monomial index bijection of 0003 at all. A reader can
check a term against the mathematics by reading it, and a diff shows which
monomial moved rather than which slot did.

The cost is `m` small integers per term instead of one index. It is paid because
what gets written is the result and not the intermediate triangle, so the file is
small next to the run that produced it, and because the alternative couples every
file ever written to an internal table.

Terms with an exactly zero coefficient are omitted. The file is the sparse
boundary representation of 0003, and the dense per-degree arrays are rebuilt on
entry.

Terms appear in ascending total degree, and within a degree in ascending grevlex,
which is the order named in the header. That is what makes the line order
canonical.

## How a coefficient is written

**binary64.** The normalised scientific spelling, always carrying an exponent, so
that no coefficient is ever mistaken for an integer field:

    5e-1    1e0    -1e0    2.5e-1    3.333333333333333e-1    -0e0

Every binary64 value round trips through it exactly. That was measured on the
pinned toolchain of 0001 rather than assumed, over the values above and over
two million random bit patterns. Save this as `coeff.rs` to reproduce it:

    fn main() {
        for x in [0.5f64, 1.0, -1.0, 0.25, 1.0 / 3.0, -0.0, f64::MIN_POSITIVE, 5e-324, 1e-300] {
            println!("{:>26} -> {:e}", format!("{:?}", x), x);
        }
        let mut s: u64 = 0x9E3779B97F4A7C15;
        let (mut n, mut bad) = (0u64, 0u64);
        for _ in 0..2_000_000u64 {
            s ^= s << 13; s ^= s >> 7; s ^= s << 17;
            let x = f64::from_bits(s);
            if x.is_nan() || x.is_infinite() { continue; }
            n += 1;
            if format!("{:e}", x).parse::<f64>().unwrap().to_bits() != x.to_bits() { bad += 1; }
        }
        println!("round-trip failures over {n} random finite bit patterns: {bad}");
    }

    $ rustc --edition 2024 -O coeff.rs -o coeff.exe && ./coeff.exe
                           0.5 -> 5e-1
                           1.0 -> 1e0
                          -1.0 -> -1e0
                          0.25 -> 2.5e-1
            0.3333333333333333 -> 3.333333333333333e-1
                          -0.0 -> -0e0
       2.2250738585072014e-308 -> 2.2250738585072014e-308
                        5e-324 -> 5e-324
                        1e-300 -> 1e-300
    round-trip failures over 1999009 random finite bit patterns: 0

Two million draws over a space of `2^64` values is a sample and not a proof, so
what that line rules out is a spelling that fails on an ordinary value, not one
that fails on a value nobody drew. The property is proved of the shipped writer
by issue #32, whose fixtures are named below.

The plain decimal spelling was rejected for this reason: it expands `1e-300` into
three hundred characters, which is not a format anybody reads, and it prints
`1.0` as `1`, which makes a coefficient indistinguishable from a count.

A coefficient that is not finite is refused at write time, naming the term. There
is no legitimate infinite or undefined coefficient in a normal form, so writing
one would be recording a defect in a form that reads like data.

**rational.** `<numerator>/<denominator>`, in lowest terms with a positive
denominator, so the spelling of a value is unique.

**interval.** `[<lower>,<upper>]` with each endpoint in the binary64 spelling. An
interval is written as its two endpoints and never as a midpoint with a radius,
so that a reader cannot silently narrow an enclosure by rounding one number.
0002 carries the same rule for the type.

## The canonical writer

The same result written twice is byte identical, and two results that are equal
as objects produce identical bytes. Concretely: the header records appear in the
order this document lists them, one space separates a key from its value, lines
end with a single line feed, terms are in the order above with exact zeros
omitted, coefficients are spelled as above, and nothing in the writer consults a
locale, an environment variable or a clock.

That makes a regression test a file comparison rather than a tolerance argument,
and it is what 0009's determinism requirement is stated against: that document
compares the byte serialisation of the complete result across thread counts and
across runs, and it needs this property to be able to.

Two obligations follow and belong to issue #32.

**Round trip.** For every fixture, `write(read(write(x)))` equals `write(x)` byte
for byte, and `read(write(x))` equals `x` as an object. Both directions, because
either alone admits a writer that drops a field the reader never asks for.

**A near miss that could have failed.** The fixtures include a coefficient whose
shortest decimal spelling differs from its shortest scientific spelling, a
negative zero, the smallest subnormal, and a value one step from a power of two,
because those are where a formatting shortcut stops round tripping. A round trip
test over well behaved numbers proves nothing about the property it is named for.

## What is deliberately not recorded

The list is 0013's, item for item, and the reason is 0013's: a file that records
the host is a disclosure waiting for the moment somebody shares the file.

- the host name,
- the user name, the user id, or the home directory,
- the absolute path of the input or of the output; a diagnostic that must name a
  file names the path the operator supplied and nothing derived from the
  environment,
- the wall clock time or the time zone,
- the machine identity in any form: CPU model, core count, serial number,
  network address, or machine id,
- the operating system and its version,
- the number of threads the run used, which is a property of the machine and, by
  0009, cannot change the answer,
- environment variables and locale.

Three of those deserve a sentence each, because they are the ones somebody will
propose adding.

The **thread count** is absent and the **chunk target** is present, which looks
inconsistent and is not. The chunk target fixes the partition and therefore the
reduction order, so it is part of what determines the answer. The thread count
determines nothing, by 0009, and recording it would invite a reader to believe it
did.

The **wall clock** is absent, so a file carries no date. A reader who wants to
know when a result was produced gets that from wherever they stored the file, and
the package does not put it in the file where it travels with the result into a
paper's supplementary material.

The **operating system** is absent, which means a file does not say which
platform wrote it. If a platform difference ever moves a result, that is a defect
under 0011's last breaking-change case and is found by the matrix in the gate,
not by a field in a file.

**The agreement with 0013, and how it was checked.** The two lists agree item for
item. At the time this document was written, 0013 was on an open pull request and
not on the default branch, so the comparison was made against the head commit of
that branch and not against a working copy:

    $ git fetch origin && git show origin/m1-decision-records:docs/decisions/0013-data-stays-on-the-host.md | sed -n '/^## What a result file deliberately does not record$/,/^## The mechanical check$/p'

Where that document lands changed, this section is what has to be re-read. Issue
#15 holds the same agreement from the other side and closes on the same reading.

The recorded-fields list of 0013 is the other half of the same agreement, and
every item in it is a required field above: package version, input digest, target
order and order reached, polydisc radii, divisor threshold, resonance module
declared and detected, coefficient type, chunk target, and the estimate object of
0008 in full.

## The digest, and the dependency it creates

`input-digest` is SHA-256 over the bytes of the input file, written as
`sha256:` and sixty-four lower case hexadecimal characters.

It is a digest of the file rather than of the parsed object, because the question
it answers is whether two runs were given the same input, and a digest of the
parsed object would call two files the same when they differ in a way this
version happens to ignore.

This is the one direct dependency this decision creates. Writing a cryptographic
primitive by hand in a numerical package is a worse trade than taking a small,
widely reviewed implementation of it. Issue #26 requires every direct dependency
to be listed in the documentation with the reason it is there and what would have
to be written by hand to remove it, and this paragraph is that reason.

## A worked example

A Hamiltonian at one degree of freedom, `H = (1/2)(q^2 + p^2) + q^3`, truncated
at degree three, with `omega = 1`. The exponent vector is `(a_q, a_p)`, and the
degree two terms appear in the ascending grevlex order of 0003, which puts
`(0,2)` before `(2,0)`.

    drehbank-format 1.0
    kind hamiltonian
    package-version 0.1.0
    degrees-of-freedom 1
    variables real
    coefficient-type binary64
    truncation-order 3
    conventions 0004
    variable-order q1..qv,p1..pv
    bracket-sign df/dq_j*dg/dp_j - df/dp_j*dg/dq_j
    lie-operator L_chi f = {f, chi}
    complexification x_j=(q_j+i*p_j)/sqrt2, y_j=(i*q_j+p_j)/sqrt2
    quadratic-form H2 = sum_j omega_j*(q_j^2+p_j^2)/2
    monomial-order grevlex-ascending
    frequency 1e0
    begin series hamiltonian
    t 0 2 5e-1
    t 2 0 5e-1
    t 3 0 1e0
    end series

Every coefficient spelling in it is one of the lines the block above prints:
`5e-1` for one half and `1e0` for one. None of them was typed by hand.

## The alternatives, and what each would have cost

**A binary format keyed to the in-memory layout.** Fast, compact, and the obvious
choice for a package whose working set is tens of gigabytes. It would have cost
everything the file is for. It is unreadable, so nobody checks it; it is
unportable across versions and across platforms, so a file from a year ago is a
liability; and it cannot be diffed in a pull request, so a change to a fixture is
invisible in review.

**An existing scientific serialisation container.** Portable, well supported, and
somebody else maintains the reader. It would have cost a dependency with its own
build requirements, in a package that is trying to have very few, and it would
have solved less of this decision than it looks: the structure inside the
container is still project specific, so the schema, the required fields, the
canonical ordering and the coefficient spelling would all still be here. The
byte-identical property would also have to be re-established against the
container's own writer, which is not usually a property those writers promise.

**JSON.** Familiar and diffable, which are the two properties that matter most
here. It would have cost the number. JSON has no canonical spelling for a
floating point value, so the byte-identical property becomes a property of
whichever encoder is linked, and the natural encodings do not round trip a
binary64 exactly unless the encoder is chosen for it. It also has no comments and
no line-oriented structure, so a several-megabyte series becomes one array whose
diff is unreadable.

**No format at all, with results printed.** The one-off behaviour the project
exists to replace.

## What this document does not decide

It does not decide the storage layout or the monomial ordering, which is 0003,
though it names the ordering it writes in. It does not decide the coefficient
types, which is 0002. It does not decide the conventions it records, which is
0004. It does not decide the divisor threshold it records, which is 0006, nor the
module, which is 0007. It does not decide what the estimate may claim, which is
0008. It does not decide the reader and writer signatures, which is 0011. It does
not build either of them, which is issue #32.
