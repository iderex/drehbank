//! What the test files share: the seed, the fixture loader, and the bijection
//! that is wrong on purpose.
//!
//! This is not a test target of its own. It is a module the files next to it
//! declare with `mod support;`.
//!
//! Each of those files compiles its own copy, so an item this file offers is
//! unused in every test binary that does not want it. That is what the allow
//! below is for, and it is the only reason: dead code here is a statement about
//! one binary rather than about the tree.
#![allow(dead_code)]

pub mod bracket_fixture;
pub mod detection_fixture;
pub mod exact;
pub mod gaussian;
pub mod lattice_fixture;
pub mod series_fixture;

use std::fs;
use std::path::{Path, PathBuf};

use drehbank_core::monomial::IndexError;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};

/// The seed every property run starts from.
///
/// A seed taken from the clock makes a failure that reproduces on the machine
/// that found it and nowhere else, which is the one property a counterexample
/// has to have. This is thirty-two bytes of readable text so that changing it
/// is a deliberate edit somebody can see in a diff rather than a number nobody
/// can tell from another number.
pub const SEED: [u8; 32] = *b"drehbank monomial index seed 001";

/// How many cases each property runs.
pub const CASES: u32 = 4096;

/// A runner that starts from [`SEED`] and writes nothing.
///
/// Failure persistence is off. The library's own regression file would put a
/// counterexample in a directory this repository does not track, where the next
/// run reads it and nothing else does. The route a counterexample takes here is
/// the fixture directory below, by hand, with a name and a reason, which is
/// what makes it a case somebody can read a year later.
pub fn fixed_seed_runner() -> TestRunner {
    fixed_seed_runner_with(CASES)
}

/// The same runner at a stated number of cases.
///
/// [`CASES`] is sized for a property whose case is one index computation. A
/// property whose case is a convolution over two graded arrays costs several
/// thousand of those, so it runs fewer of them and says how many rather than
/// taking the number for a property it is not.
pub fn fixed_seed_runner_with(cases: u32) -> TestRunner {
    let config = Config {
        cases,
        failure_persistence: None,
        ..Config::default()
    };
    TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &SEED))
}

/// The signature both the real forward map and the broken one are written to,
/// so a property can be run against either without being written twice.
pub type Forward = fn(&[u32], u32) -> Result<u64, IndexError>;

/// `C(n, k)`, in `u128` so the intermediate product cannot wrap.
///
/// Written here rather than reached for in the crate, because a fixture that
/// borrows the code it is meant to be independent of proves less.
fn binomial(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result: u128 = 1;
    for i in 0..k {
        result = result * u128::from(n - i) / u128::from(i + 1);
    }
    u64::try_from(result).expect("the fixture is only used at degrees that fit in 64 bits")
}

/// The rank of an exponent vector with the shift dropped, which is wrong.
///
/// `docs/decisions/0003-series-representation.md` ranks the partial sums after
/// pushing them apart into a strictly increasing tuple, `c_k = s_k + (k - 1)`.
/// This is the same loop with that shift left out, which is the mistake
/// somebody makes by reading the formula once. It is one term of one expression
/// and it costs injectivity: two different monomials get the same number, and
/// the coefficient one of them names is read for the other.
///
/// Held here as a fixture rather than described in a comment, because a test
/// that has never refused anything is a test nobody has checked.
pub fn index_of_without_the_shift(exponents: &[u32], degree: u32) -> Result<u64, IndexError> {
    if exponents.is_empty() {
        return Err(IndexError::NoVariables);
    }
    let total: u64 = exponents.iter().copied().map(u64::from).sum();
    if total != u64::from(degree) {
        return Err(IndexError::DegreeMismatch {
            stated: degree,
            actual: total,
        });
    }

    let mut partial: u64 = 0;
    let mut index: u64 = 0;
    for k in 1..exponents.len() {
        partial += u64::from(exponents[k - 1]);
        // The real one is `binomial(partial + k as u64 - 1, k as u64)`.
        index += binomial(partial, k as u64);
    }
    Ok(index)
}

/// One monomial in a regression case: the index and the exponent vector that
/// have to keep naming each other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monomial {
    pub index: u64,
    pub exponents: Vec<u32>,
}

/// A regression case read from the fixture directory.
#[derive(Debug, Clone)]
pub struct Case {
    /// The file it came from, so a failure names something somebody can open.
    pub name: String,
    pub variables: usize,
    pub degree: u32,
    pub monomials: Vec<Monomial>,
}

/// Where the cases live.
pub fn fixture_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("monomial-index")
}

/// Every case in `directory`, in filename order.
///
/// Panics rather than returning an error, and names the file and the line when
/// it does. A loader that skipped a file it could not parse would turn a broken
/// fixture into a suite that passes with one case fewer, which is the failure
/// this directory exists to prevent.
pub fn load_cases(directory: &Path) -> Vec<Case> {
    let mut paths: Vec<PathBuf> = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read the fixture directory {directory:?}: {error}"))
        .map(|entry| {
            entry
                .expect("cannot read an entry of the fixture directory")
                .path()
        })
        .filter(|path| path.extension().is_some_and(|extension| extension == "txt"))
        .collect();
    paths.sort();
    paths.iter().map(|path| load_case(path)).collect()
}

fn load_case(path: &Path) -> Case {
    let name = path
        .file_name()
        .expect("a fixture path always has a file name")
        .to_string_lossy()
        .into_owned();
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read the fixture {name}: {error}"));

    let mut variables: Option<usize> = None;
    let mut degree: Option<u32> = None;
    let mut monomials: Vec<Monomial> = Vec::new();
    let mut pending_index: Option<u64> = None;

    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = format!("{name}:{}", number + 1);
        let (key, value) = line
            .split_once(':')
            .unwrap_or_else(|| panic!("{at}: expected `key: value`, found {line:?}"));
        let value = value.trim();
        match key.trim() {
            "variables" => variables = Some(parse(&at, value)),
            "degree" => degree = Some(parse(&at, value)),
            "index" => {
                assert!(
                    pending_index.is_none(),
                    "{at}: an index line has no exponents line after it"
                );
                pending_index = Some(parse(&at, value));
            }
            "exponents" => {
                let index = pending_index.take().unwrap_or_else(|| {
                    panic!("{at}: an exponents line with no index line before it")
                });
                let exponents = value
                    .split_whitespace()
                    .map(|part| parse(&at, part))
                    .collect();
                monomials.push(Monomial { index, exponents });
            }
            other => panic!("{at}: unknown key {other:?}"),
        }
    }

    assert!(
        pending_index.is_none(),
        "{name}: the last index line has no exponents line after it"
    );
    let variables = variables.unwrap_or_else(|| panic!("{name}: no `variables` line"));
    let degree = degree.unwrap_or_else(|| panic!("{name}: no `degree` line"));
    assert!(!monomials.is_empty(), "{name}: no monomial in the case");
    for monomial in &monomials {
        assert_eq!(
            monomial.exponents.len(),
            variables,
            "{name}: an exponent vector of length {} in a case of {variables} variables",
            monomial.exponents.len()
        );
    }

    Case {
        name,
        variables,
        degree,
        monomials,
    }
}

fn parse<T: std::str::FromStr>(at: &str, value: &str) -> T {
    value
        .parse()
        .unwrap_or_else(|_| panic!("{at}: cannot read {value:?} as a number"))
}
