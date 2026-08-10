# The scaling runs, as they were made

One line per run of `drehbank-scaling-harness`, appended by the harness itself
and never rewritten by it. A regression is then a change over time in this file
rather than a number somebody remembers being different.

Every line carries the machine it ran on, because a number without its machine
is not a measurement of anything. What that means here is the parallelism the
runtime reported, the processor as the operating system names it, the platform,
and the compiler that built the binary.

Nothing in this repository refuses an edit to a line that has already been
written. The append-only rule is one somebody follows, and the harness is the
only thing in the tree that writes here.

What is not in any line, on purpose:

- The host's total memory. The harness does not read it. The ceiling a run was
  checked against was given on the command line by whoever ran it, and it is not
  a property of the machine.
- Anything from a hardware counter. Those need a privilege the harness does not
  ask for on any host, so a cache or instruction count is reported as not made
  rather than as a number.

The command that produces a line:

    cargo run --release --package drehbank-scaling-harness -- \
        --case <name> --memory <bytes>

With no `--case` the harness measures nothing and prints what each case would
cost instead.

## The runs

case product-3f-order-8 | freedoms 3 | order 8 | pool 32 requested, kernel sequential | peak-live-set 72072 bytes | wall-clock 26.3575ms | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-3f-order-12 | freedoms 3 | order 12 | pool 32 requested, kernel sequential | peak-live-set 445536 bytes | wall-clock 748.1713ms | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 1 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 5.3023806s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 1 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 4.978434s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 1 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 5.1629419s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 2 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 3.4449476s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 2 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 3.156685s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 2 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 3.2550912s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 4 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 1.6701983s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 4 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 1.6990688s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 4 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 1.7199263s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 8 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 1.1628633s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 8 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 1.0882741s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 8 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 1.1033455s | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 16 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 779.109ms | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 16 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 768.3382ms | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 16 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 784.5016ms | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 32 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 587.9084ms | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 32 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 441.915ms | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
case product-6f-order-10 | freedoms 6 | order 10 | pool 32 thread(s), chunk 256 | peak-live-set 15519504 bytes | wall-clock 626.3376ms | parallelism 32, processor AMD64 Family 25 Model 33 Stepping 2, AuthenticAMD, platform windows x86_64, toolchain rustc 1.97.0 (2d8144b78 2026-07-07)
