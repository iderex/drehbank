//! Scaling and measurement runs.
//!
//! This member is outside the default build and the default suite, because the
//! runs it holds need a real machine and, for some of what it would like to
//! measure, more privilege than a gate has. A harness like that hidden inside
//! the ordinary suite fails on every laptop and is deleted after the third
//! report.
//!
//! Run with no arguments it measures nothing. It prints that it is not part of
//! the gate, what it found on this host, and what every case would cost, and it
//! stops there. A case runs only when it is named and only when a memory
//! ceiling has been given for it to be checked against.
//!
//!     drehbank-scaling-harness
//!     drehbank-scaling-harness --case product-3f-order-8 --memory 1073741824
//!
//! The decisions are all in the library beside this file, which is where they
//! can be read and tested without a large machine.

use std::num::NonZero;
use std::path::PathBuf;
use std::process::ExitCode;

use drehbank_core::parallel::Pool;

use drehbank_scaling_harness::{
    CASES, COUNTER_MEASUREMENTS, Case, Host, Machine, Skip, admit, append_record, record_line,
    recorded_cost, requirements, run,
};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let request = match Request::parse(&arguments) {
        Ok(request) => request,
        Err(complaint) => {
            eprintln!("drehbank-scaling-harness: {complaint}");
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let machine = Machine::read();
    let available = std::thread::available_parallelism().ok();
    let host = Host {
        parallelism: available.map(NonZero::get),
        pool: request.pool.or(available).unwrap_or(NonZero::<usize>::MIN),
        ceiling: request.memory,
    };

    println!("drehbank-scaling-harness is not part of the gate.");
    println!("A red line below is a measurement and not a failed check.");
    println!();
    println!("host: {machine}");
    match host.ceiling {
        Some(ceiling) => println!("memory ceiling: {ceiling} byte(s), as given by the caller"),
        None => println!(
            "memory ceiling: not given, and this harness does not read the host's \
             memory, so it was not measured"
        ),
    }
    println!(
        "pool size: {} thread(s), which is what the kernel runs on, partitioned in          chunks of {} coefficient(s)",
        host.pool,
        Pool::of(host.pool).chunk()
    );
    println!();
    for measurement in COUNTER_MEASUREMENTS {
        println!("{measurement}: not made, it needs a privileged hardware counter");
    }
    println!(
        "No privilege is requested for those, on any host, so the answer above is \
         the same everywhere and no run of this harness can raise a consent prompt."
    );
    println!();

    let record = std::fs::read_to_string(&request.record).unwrap_or_default();
    println!("the plan:");
    for case in CASES {
        let cost = match requirements(*case) {
            Ok(requirements) => format!("{} byte(s)", requirements.peak_live_set),
            Err(error) => format!("not computed: {error}"),
        };
        let expected = recorded_cost(&record, *case)
            .unwrap_or_else(|| "not measured, no run of it is in the record".to_string());
        println!(
            "  {}: peak live set {cost}, wall clock {expected}",
            case.name
        );
    }
    println!();

    let Some(wanted) = request.case else {
        for case in CASES {
            report_skip(case, Skip::NotSelected, &record);
        }
        println!();
        println!("nothing was measured, because no case was named.");
        return ExitCode::SUCCESS;
    };

    let Some(case) = CASES.iter().find(|case| case.name == wanted) else {
        eprintln!("drehbank-scaling-harness: no case is called {wanted:?}");
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };

    let requirements = match requirements(*case) {
        Ok(requirements) => requirements,
        Err(error) => {
            eprintln!("drehbank-scaling-harness: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(skip) = admit(requirements, host) {
        report_skip(case, skip, &record);
        for other in CASES.iter().filter(|other| other.name != case.name) {
            report_skip(other, Skip::NotSelected, &record);
        }
        return ExitCode::SUCCESS;
    }

    let measurement = match run(*case, host) {
        Ok(measurement) => measurement,
        Err(error) => {
            eprintln!(
                "drehbank-scaling-harness: {} could not run: {error}",
                case.name
            );
            return ExitCode::FAILURE;
        }
    };
    let line = record_line(&measurement, &machine);
    println!("measured: {line}");
    match append_record(&request.record, &line) {
        Ok(()) => println!("appended to {}", request.record.display()),
        Err(error) => {
            eprintln!(
                "drehbank-scaling-harness: the measurement was made and could not be \
                 recorded in {}: {error}",
                request.record.display()
            );
            return ExitCode::FAILURE;
        }
    }
    for other in CASES.iter().filter(|other| other.name != case.name) {
        report_skip(other, Skip::NotSelected, &record);
    }
    ExitCode::SUCCESS
}

/// The three lines a skip prints, in the shape 0012 sets out.
fn report_skip(case: &Case, skip: Skip, record: &str) {
    let cost = match requirements(*case) {
        Ok(requirements) => format!("{} byte(s)", requirements.peak_live_set),
        Err(error) => format!("not computed: {error}"),
    };
    let expected = recorded_cost(record, *case).unwrap_or_else(|| "not measured".to_string());
    println!("skipped: {}", case.name);
    println!("reason:  {skip}");
    println!("cost:    peak live set {cost}, wall clock {expected}");
}

const USAGE: &str = "\
usage: drehbank-scaling-harness [--case <name>] [--memory <bytes>] [--pool <n>] [--record <path>]

With no --case it prints what every case would cost and measures nothing.
A case runs only when --memory gives a ceiling for it to be checked against;
this harness does not read the host's memory and will not guess it.";

/// What the command line asked for.
struct Request {
    case: Option<String>,
    memory: Option<u64>,
    pool: Option<NonZero<usize>>,
    record: PathBuf,
}

impl Request {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut request = Request {
            case: None,
            memory: None,
            pool: None,
            record: PathBuf::from("docs/scaling-runs.md"),
        };
        let mut rest = arguments.iter();
        while let Some(flag) = rest.next() {
            let value = || {
                rest.clone()
                    .next()
                    .cloned()
                    .ok_or_else(|| format!("{flag} needs a value"))
            };
            match flag.as_str() {
                "--case" => {
                    request.case = Some(value()?);
                    rest.next();
                }
                "--memory" => {
                    let given = value()?;
                    request.memory =
                        Some(given.parse().map_err(|_| {
                            format!("--memory wants a number of bytes, not {given:?}")
                        })?);
                    rest.next();
                }
                "--pool" => {
                    let given = value()?;
                    let count: usize = given
                        .parse()
                        .map_err(|_| format!("--pool wants a count, not {given:?}"))?;
                    request.pool = Some(
                        NonZero::new(count)
                            .ok_or_else(|| "--pool wants at least one thread".to_string())?,
                    );
                    rest.next();
                }
                "--record" => {
                    request.record = PathBuf::from(value()?);
                    rest.next();
                }
                other => return Err(format!("unknown argument {other:?}")),
            }
        }
        Ok(request)
    }
}
