//! The command line an operator runs.
//!
//! The verbs, their arguments, their streams and their exit codes are issue #60.
//! Until they exist this binary says so rather than exiting zero, because a
//! placeholder that succeeds silently is indistinguishable from a run that did
//! the work.

use std::process::ExitCode;

fn main() -> ExitCode { eprintln!("drehbank: no verbs yet. The command line is issue #60."); ExitCode::FAILURE }
