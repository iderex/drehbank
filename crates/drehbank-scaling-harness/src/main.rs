//! Scaling and measurement runs.
//!
//! This member is outside the default build and the default suite, because the
//! runs it will hold need tens of gigabytes, hours of wall clock, and on some
//! hosts more privilege than a gate has. A harness like that hidden inside the
//! ordinary suite fails on every laptop and is deleted after the third report.
//!
//! What it requires, how it checks the host against those requirements, how it
//! announces a skip and how it records the machine a number came from is issue
//! #51. It measures nothing today and says so.

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!(
        "drehbank-scaling-harness: no runs yet, so nothing was measured. The harness is issue #51."
    );
    ExitCode::FAILURE
}
