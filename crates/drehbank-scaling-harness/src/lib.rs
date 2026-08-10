//! What the harness needs, how it decides a host cannot carry it, and what it
//! writes down afterwards.
//!
//! The binary next to this file is the printing. Everything that decides
//! something is here, so that it can be read and tested without a large machine
//! and without hours.
//!
//! `docs/decisions/0012-headless-testability.md` is where the shape comes from.
//! The four conditions of that document keep the default suite runnable
//! anywhere, and the work that genuinely cannot meet them is not suppressed, it
//! is moved here and made to say what it needs. The property that matters most
//! is the one about a skip: a skipped case prints that it was skipped, why, and
//! what running it would have cost, because a skip that announces itself cannot
//! be read as a pass.

use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::num::NonZero;
use std::path::Path;
use std::time::{Duration, Instant};

use drehbank_core::Series;
use drehbank_core::monomial::dimension;
use drehbank_core::parallel::{self, Pool};

/// The width of one binary64 coefficient in bytes.
///
/// `docs/decisions/0002-coefficients.md` fixes it, and it is the number the
/// memory arithmetic below is evaluated with. The exact rational has no fixed
/// width, which is why that document says the arithmetic here cannot be
/// evaluated ahead of a group for it at all, and why no case in this harness is
/// declared over it.
pub const BINARY64_WIDTH: u64 = 8;

/// One run the harness knows how to make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Case {
    /// What it is called, in the record and on the command line.
    pub name: &'static str,
    /// Degrees of freedom, so `2 * freedoms` variables.
    pub freedoms: usize,
    /// The truncation order the product is taken at.
    pub order: u32,
}

/// The cases this harness holds today.
///
/// Every one of them is a graded product of two full series, which is the only
/// kernel in the tree. The long runs 0012 moves here, the Deprit triangle at
/// the target order and the falsifier over hours, are not among them because
/// neither exists yet: the triangle is issue #34 and the falsifier is issue
/// #47. Naming them here before they exist would put a case in the list that
/// cannot be run and would make the list a plan rather than a set of runs.
pub const CASES: &[Case] = &[
    Case {
        name: "product-3f-order-8",
        freedoms: 3,
        order: 8,
    },
    Case {
        name: "product-3f-order-12",
        freedoms: 3,
        order: 12,
    },
    Case {
        name: "product-6f-order-10",
        freedoms: 6,
        order: 10,
    },
    Case {
        name: "product-8f-order-14",
        freedoms: 8,
        order: 14,
    },
];

/// What a case needs before it is started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Requirements {
    /// The peak live set in bytes, computed from the case rather than quoted.
    pub peak_live_set: u64,
    /// How many coefficients one series of the case holds.
    pub coefficients: u64,
}

/// What a case needs, computed from the case in front of it.
///
/// A hardcoded number is true of one case on one day and goes stale in silence.
/// This is `C(order + m, m)` coefficients per series, from
/// `docs/decisions/0003-series-representation.md`, three series live at once,
/// which are the two operands and the destination, times the coefficient width
/// of 0002.
///
/// It is the peak of this kernel and not of the recursion. 0012 asks for the
/// peak live set of the case about to run and points at the formula in 0005 for
/// it, and 0005's engine does not exist yet, so what is computed here is the
/// product's own footprint. Where the memory ceiling of issue #50 lands, 0012
/// requires the ceiling and this to be one arithmetic rather than two.
pub fn requirements(case: Case) -> Result<Requirements, RequirementError> {
    let variables = case
        .freedoms
        .checked_mul(2)
        .ok_or(RequirementError::TooWide { case })?;
    let mut coefficients: u64 = 0;
    for degree in 0..=case.order {
        let at_degree =
            dimension(variables, degree).map_err(|_| RequirementError::TooWide { case })?;
        coefficients = coefficients
            .checked_add(at_degree)
            .ok_or(RequirementError::TooWide { case })?;
    }
    let peak_live_set = coefficients
        .checked_mul(BINARY64_WIDTH)
        .and_then(|bytes| bytes.checked_mul(3))
        .ok_or(RequirementError::TooWide { case })?;
    Ok(Requirements {
        peak_live_set,
        coefficients,
    })
}

/// A case whose own arithmetic does not fit in the width it is computed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequirementError {
    /// The case is wider or deeper than the index arithmetic can address.
    TooWide {
        /// The case that could not be sized.
        case: Case,
    },
}

impl fmt::Display for RequirementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequirementError::TooWide { case } => write!(
                formatter,
                "{}: order {} in {} degree(s) of freedom is beyond what the index \
                 arithmetic addresses, so its cost cannot be computed",
                case.name, case.order, case.freedoms
            ),
        }
    }
}

impl core::error::Error for RequirementError {}

/// What the harness was told about the host it is on.
///
/// The memory ceiling is given by the caller and is not read from the machine.
/// That is deliberate twice over. `docs/decisions/0009-parallelism-and-memory.md`
/// makes the ceiling the caller's to set, and reading a total from the operating
/// system portably needs either a dependency or a privileged call, neither of
/// which this harness is willing to take on for a number the caller already
/// knows. Where no ceiling is given, every case is skipped and says so, which is
/// a measurement that was not made rather than a default nobody chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Host {
    /// The parallelism the runtime reported, or `None` where it could not say.
    pub parallelism: Option<usize>,
    /// The pool size the kernel runs on.
    ///
    /// A [`NonZero`] because a pool of no threads is not a case with an error
    /// message, and because the command line already refuses one.
    pub pool: NonZero<usize>,
    /// The memory ceiling in bytes, where the caller gave one.
    pub ceiling: Option<u64>,
}

/// Why a case is not being run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Skip {
    /// No memory ceiling was given, so nothing can be checked against one.
    NoCeiling,
    /// The case needs more than the ceiling allows.
    BeyondCeiling {
        /// What the case needs, in bytes.
        needs: u64,
        /// What the caller allowed, in bytes.
        ceiling: u64,
    },
    /// The case was not asked for.
    NotSelected,
}

impl fmt::Display for Skip {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Skip::NoCeiling => write!(
                formatter,
                "no memory ceiling was given, and this harness does not read the \
                 host's memory, so nothing could be checked against one"
            ),
            Skip::BeyondCeiling { needs, ceiling } => write!(
                formatter,
                "the case needs {needs} byte(s) and the ceiling given is \
                 {ceiling} byte(s), short by {} byte(s)",
                needs.saturating_sub(*ceiling)
            ),
            Skip::NotSelected => write!(
                formatter,
                "the case was not named on the command line, and this harness \
                 runs only what it is asked for"
            ),
        }
    }
}

/// Whether a case may run on this host, and why not when it may not.
///
/// The refusal happens before anything is allocated, which is the reason 0009
/// gives for checking a ceiling ahead of a group rather than at the allocation:
/// a refusal before the work says which case cannot be afforded and by how much,
/// and a kill during the work says nothing at all.
pub fn admit(requirements: Requirements, host: Host) -> Result<(), Skip> {
    let Some(ceiling) = host.ceiling else {
        return Err(Skip::NoCeiling);
    };
    if requirements.peak_live_set > ceiling {
        return Err(Skip::BeyondCeiling {
            needs: requirements.peak_live_set,
            ceiling,
        });
    }
    Ok(())
}

/// What a run produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    /// The case that was run.
    pub case: Case,
    /// How long the kernel took.
    pub elapsed: Duration,
    /// The pool size the kernel ran on.
    pub pool: NonZero<usize>,
    /// The chunk target the output was partitioned with.
    pub chunk: usize,
    /// The peak live set the case was admitted against.
    pub peak_live_set: u64,
}

/// Run one case: build two full series and take their graded product across the
/// pool.
///
/// The series are filled from a fixed integer pattern rather than from a random
/// one, so that the same case measures the same arithmetic on every host. A
/// generator seeded from anything that moves would make two runs of one case
/// two different measurements wearing one name.
///
/// The kernel is `drehbank_core::parallel::product`, so the pool size is what
/// the run was made on and not a number nothing read. What it returns does not
/// depend on that size, which is the property `tests/parallel.rs` in the core
/// holds, so a speedup curve measured by varying the pool is a curve over one
/// answer rather than over several.
pub fn run(case: Case, host: Host) -> Result<Measurement, drehbank_core::Error> {
    let mut left: Series<f64> = Series::zero(case.freedoms, case.order)?;
    let mut right: Series<f64> = Series::zero(case.freedoms, case.order)?;
    let variables = case.freedoms * 2;
    for degree in 0..=case.order {
        let width = dimension(variables, degree)?;
        for index in 0..width {
            let value = f64::from((index % 7) as u32) - 3.0;
            left.set_coefficient(degree, index, value)?;
            right.set_coefficient(degree, index, 1.0 - value)?;
        }
    }
    let started = Instant::now();
    let product = parallel::product(&left, &right, Pool::of(host.pool))?;
    let elapsed = started.elapsed();
    // Read one coefficient out so that nothing in the compilation can conclude
    // the product was never wanted. A measurement of a computation that was
    // optimised away is not a measurement of anything.
    let witness = product.coefficient(0, 0)?;
    debug_assert!(witness.is_finite() || !witness.is_finite());
    Ok(Measurement {
        case,
        elapsed,
        pool: host.pool,
        chunk: Pool::of(host.pool).chunk(),
        peak_live_set: requirements(case)
            .map(|requirements| requirements.peak_live_set)
            .unwrap_or_default(),
    })
}

/// The machine a number came from.
///
/// A number without its machine is not a measurement of anything, so every one
/// of these fields is written next to every recorded result. Where a field
/// could not be read it says so rather than being left out, because an absent
/// field and an unreadable one are different statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machine {
    /// What the runtime reported for available parallelism.
    pub parallelism: String,
    /// The processor as the operating system names it, or that it was not read.
    pub processor: String,
    /// The operating system and architecture this binary was built for.
    pub platform: String,
    /// The compiler that built this binary, from the build script.
    pub toolchain: String,
}

impl Machine {
    /// Read what can be read without a dependency and without a privilege.
    pub fn read() -> Self {
        Machine {
            parallelism: match std::thread::available_parallelism() {
                Ok(count) => count.to_string(),
                Err(error) => format!("not read ({error})"),
            },
            processor: read_processor(),
            platform: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            toolchain: env!("DREHBANK_TOOLCHAIN").to_string(),
        }
    }
}

impl fmt::Display for Machine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "parallelism {}, processor {}, platform {}, toolchain {}",
            self.parallelism, self.processor, self.platform, self.toolchain
        )
    }
}

/// The processor string, from whichever place this platform keeps it.
///
/// Windows puts one in the environment and Linux puts one in `/proc/cpuinfo`.
/// Where neither answers, this says the field was not read. It never guesses,
/// because a guessed processor in a recorded measurement is worse than a blank
/// one: the blank is visible.
fn read_processor() -> String {
    if let Ok(identifier) = std::env::var("PROCESSOR_IDENTIFIER") {
        return identifier;
    }
    if let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in text.lines() {
            if let Some((key, value)) = line.split_once(':')
                && key.trim() == "model name"
            {
                return value.trim().to_string();
            }
        }
    }
    "not read".to_string()
}

/// The measurements that need a privileged hardware counter.
///
/// None of them is made. 0012 moves them here rather than into the suite, and
/// then says what happens when the counter is unavailable: the harness prints
/// that the measurement was **not made**, and does not substitute an estimate
/// or put a derived number in the column a measured one would have occupied.
/// This harness never asks for the privilege either, so the answer is the same
/// on every host and no run of it can raise a consent prompt on somebody's
/// machine.
pub const COUNTER_MEASUREMENTS: &[&str] = &[
    "cache misses per output coefficient",
    "instructions retired per output coefficient",
];

/// One line per recorded run, appended to the record.
///
/// Appended and never rewritten, so that a regression is a change over time in
/// a file rather than a number somebody remembers being different. Nothing in
/// this tree refuses an edit to what has already been written; the file says so
/// at its head and this function is the only thing here that writes it.
pub fn append_record(path: &Path, line: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")
}

/// The record line for a measurement.
///
/// The pool size is the number of threads the kernel ran on. Until the pool was
/// consumed it was a number the caller gave and nothing read, and the line said
/// that in place of the count; lines in that shape are already in the record and
/// still parse, which the last test in this file covers.
///
/// The chunk target is written beside it because 0009 asks for the partition to
/// be reconstructible and the series type has nowhere to carry it. A wall clock
/// recorded without the partition it was measured under is not comparable
/// against one measured under another.
pub fn record_line(measurement: &Measurement, machine: &Machine) -> String {
    format!(
        "case {} | freedoms {} | order {} | pool {} thread(s), chunk {} | \
         peak-live-set {} bytes | wall-clock {:?} | {}",
        measurement.case.name,
        measurement.case.freedoms,
        measurement.case.order,
        measurement.pool,
        measurement.chunk,
        measurement.peak_live_set,
        measurement.elapsed,
        machine
    )
}

/// The most recent wall clock recorded for a case, or `None` where the record
/// holds no run of it.
///
/// This is what lets the plan print the expected cost of a case instead of
/// saying it was never measured. It reads the record rather than a constant,
/// so a case that has never been run says so for as long as that is true.
pub fn recorded_cost(record: &str, case: Case) -> Option<String> {
    let needle = format!("case {} |", case.name);
    record
        .lines()
        .filter(|line| line.starts_with(&needle))
        .filter_map(|line| {
            line.split('|')
                .map(str::trim)
                .find_map(|field| field.strip_prefix("wall-clock "))
        })
        .next_back()
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::{BINARY64_WIDTH, Case, Host, NonZero, Skip, admit, recorded_cost, requirements};

    fn pool(threads: usize) -> NonZero<usize> {
        NonZero::new(threads).expect("the counts in these tests are all above zero")
    }

    /// The cost of a case is the arithmetic of 0003 and not a constant.
    ///
    /// `C(8 + 6, 6)` is 3003, which is the cumulative count 0003 tabulates for
    /// six variables up to degree eight:
    ///
    ///     $ python -c "from math import comb; print(comb(14,6))"
    ///     3003
    #[test]
    fn the_cost_of_a_case_is_computed_from_the_case() {
        let case = Case {
            name: "product-3f-order-8",
            freedoms: 3,
            order: 8,
        };
        let requirements = requirements(case).expect("this case is addressable");
        assert_eq!(requirements.coefficients, 3003);
        assert_eq!(requirements.peak_live_set, 3003 * BINARY64_WIDTH * 3);
    }

    /// A case beyond the ceiling is refused before anything is allocated, and
    /// the refusal carries both numbers.
    #[test]
    fn a_case_beyond_the_ceiling_is_refused_with_both_numbers() {
        let case = Case {
            name: "product-3f-order-8",
            freedoms: 3,
            order: 8,
        };
        let requirements = requirements(case).expect("this case is addressable");
        let host = Host {
            parallelism: Some(4),
            pool: pool(4),
            ceiling: Some(1000),
        };
        assert_eq!(
            admit(requirements, host),
            Err(Skip::BeyondCeiling {
                needs: 3003 * BINARY64_WIDTH * 3,
                ceiling: 1000
            })
        );
    }

    /// With no ceiling given, nothing is admitted and the reason says the
    /// number was never read rather than that the case is too large.
    ///
    /// The two are different statements and only one of them is true here.
    #[test]
    fn no_ceiling_is_a_skip_and_not_a_refusal_for_size() {
        let case = Case {
            name: "product-3f-order-8",
            freedoms: 3,
            order: 8,
        };
        let requirements = requirements(case).expect("this case is addressable");
        let host = Host {
            parallelism: Some(4),
            pool: pool(4),
            ceiling: None,
        };
        assert_eq!(admit(requirements, host), Err(Skip::NoCeiling));
    }

    /// A case that fits is admitted.
    #[test]
    fn a_case_inside_the_ceiling_is_admitted() {
        let case = Case {
            name: "product-3f-order-8",
            freedoms: 3,
            order: 8,
        };
        let requirements = requirements(case).expect("this case is addressable");
        let host = Host {
            parallelism: Some(4),
            pool: pool(4),
            ceiling: Some(1 << 30),
        };
        assert_eq!(admit(requirements, host), Ok(()));
    }

    /// The expected cost comes out of the record, and a case with no run in it
    /// has none.
    ///
    /// The first two lines are in the shape the harness wrote before the pool
    /// was consumed and the third is the shape it writes now. Both are here
    /// because the record is append-only and the older lines stay in it: a
    /// reader that understood only the current shape would report a case that
    /// has been run as one that was never measured.
    #[test]
    fn the_expected_cost_is_read_from_the_record_and_is_absent_when_nothing_ran() {
        let record = "\
case product-3f-order-8 | freedoms 3 | order 8 | pool 4 requested, kernel sequential | peak-live-set 72072 bytes | wall-clock 1.5ms | parallelism 4
case product-3f-order-8 | freedoms 3 | order 8 | pool 4 requested, kernel sequential | peak-live-set 72072 bytes | wall-clock 2.5ms | parallelism 4
case product-3f-order-8 | freedoms 3 | order 8 | pool 4 thread(s), chunk 256 | peak-live-set 72072 bytes | wall-clock 3.5ms | parallelism 4
";
        let ran = Case {
            name: "product-3f-order-8",
            freedoms: 3,
            order: 8,
        };
        let never = Case {
            name: "product-6f-order-10",
            freedoms: 6,
            order: 10,
        };
        assert_eq!(recorded_cost(record, ran).as_deref(), Some("3.5ms"));
        assert_eq!(recorded_cost(record, never), None);
    }
}
