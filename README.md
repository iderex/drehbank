# drehbank

Canonical perturbation theory has no maintained open-source implementation. What exists are one-off builds and in-house accelerator codes, and everyone rebuilds Deprit-Hori. One clean package with Lie transformations, Birkhoff normal form, resonance handling and remainder estimates serves celestial mechanics, accelerator physics and KAM theory at once. The Poisson brackets grow combinatorially, so order eight to ten in six variables needs a real machine and parallelises well.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

## What leaves your machine

Nothing. The package makes no network connection, sends no telemetry, no usage
report and no crash report, and does not check for updates. It reads the files
you point it at and writes the files you tell it to write. A result file records
the package version, a digest of your input, and the parameters of the run, and
it records no host name, no user name, no absolute path, no wall clock time and
nothing else about the machine. None of that is enforced by a machine yet, so
every sentence in this paragraph is a claim rather than a measurement, and today
there is no code for it to be a claim about.
[What leaves the host](docs/what-leaves-the-host.md) is the full inventory, with
the field list and the gap.

See [NOTICE.md](NOTICE.md) for the intended-use notice.

## License

AGPL-3.0, copyright 2026 Nils Lehnen.

The full text is in [LICENSE](LICENSE).
