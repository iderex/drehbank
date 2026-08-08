//! The refusal that keeps a socket out of what this package ships.
//!
//! `docs/decisions/0013-data-stays-on-the-host.md` says personal data never
//! leaves the host, and the reason it holds today is that nothing in the shipped
//! graph can open a socket. A fact nothing checks is a fact that stops being
//! true without anybody noticing, so this is that fact as a refusal.
//!
//! It is a floor and not a proof. It refuses what is on the two lists below,
//! and a package that reaches a socket through a route nobody has listed walks
//! past it. `docs/what-leaves-the-host.md` says so in those words, and this
//! comment is not the place that softens it.
//!
//! Two readings, because a package can arrive at the capability two ways. By
//! name, from the resolved graph, which is what catches an ordinary client
//! crate. By declared link, from the package metadata, which is what catches a
//! thin wrapper over a system library whose own name says nothing.

use std::collections::BTreeSet;
use std::process::Command;

/// The members whose graphs are refused.
///
/// The scaling harness is deliberately not here. What it will do is drive long
/// runs on a real machine, and issue #51 leaves open how a result gets off that
/// machine, so a refusal written before that question is answered would be
/// refusing a decision rather than a capability. It is out of the default build
/// and the default suite as well, so nothing an operator installs carries it.
/// This exclusion is a scope choice and not an oversight, and it is the sentence
/// that says so.
const GUARDED: [&str; 2] = ["drehbank-core", "drehbank-cli"];

/// Package names that provide network access, and are refused anywhere in a
/// guarded graph at any depth.
///
/// Transports and sockets first, because they are the capability itself:
/// `socket2` and `mio` are the socket, and `tokio`, `async-std`, `smol` and
/// `hyper` carry one. Then clients, because they are how the capability
/// normally arrives: `reqwest`, `ureq`, `curl`, `isahc`, `attohttpc`, `surf`,
/// `tungstenite` and `quinn`. Then resolvers, `hickory-resolver` and
/// `trust-dns-resolver`, because a name lookup is a packet leaving the host
/// whatever else it is. Then the TLS crates, `rustls`, `native-tls` and
/// `openssl`: none of them opens a socket on its own, and each one appearing in
/// a graph that computes normal forms means something else in that graph does.
///
/// The list holds what has actually been seen arriving in Rust graphs. It does
/// not hold what nobody has written yet, and it never will.
const DENIED: [&str; 19] = [
    "socket2",
    "mio",
    "tokio",
    "async-std",
    "smol",
    "hyper",
    "h2",
    "reqwest",
    "ureq",
    "curl",
    "isahc",
    "attohttpc",
    "surf",
    "tungstenite",
    "quinn",
    "hickory-resolver",
    "trust-dns-resolver",
    "rustls",
    "native-tls",
];

/// System libraries whose `links` declaration is refused.
///
/// A `-sys` wrapper is named for its library and not for what the library does,
/// so a name list misses it. What it cannot hide is the `links` key, because
/// cargo requires it to be unique across the graph and reads it out of the
/// manifest into the package metadata.
const DENIED_LINKS: [&str; 6] = ["curl", "openssl", "ssl", "nghttp2", "ssh2", "zmq"];

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

fn run(arguments: &[&str]) -> String {
    let output = Command::new(cargo())
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("cannot run cargo {}: {error}", arguments.join(" ")));
    assert!(
        output.status.success(),
        "cargo {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("cargo writes utf-8")
}

/// One package in the graph, with the chain that reached it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Reached {
    name: String,
    path: Vec<String>,
}

/// The resolved graph of `member`, along the edges a caller links.
///
/// `--edges normal` is what makes this the shipped graph: a dev-dependency is
/// compiled by the suite and by nothing an operator runs, so refusing one would
/// be refusing a test. `--locked` refuses to move the lockfile, so this reads
/// the graph that will ship rather than one it resolved for itself.
fn resolved_graph(member: &str) -> Vec<Reached> {
    let text = run(&[
        "tree",
        "--package",
        member,
        "--edges",
        "normal",
        "--locked",
        "--offline",
        "--prefix",
        "depth",
        "--format",
        "{p}",
    ]);
    parse_tree(&text)
}

/// Depth-prefixed `cargo tree` output into packages and the chain to each.
///
/// The depth is what carries the path: a line at depth `n` was reached by the
/// nearest line above it at depth `n - 1`. A refusal that named only the
/// package would leave the reader to work out which direct dependency dragged
/// it in, and that is the whole question when one arrives.
fn parse_tree(text: &str) -> Vec<Reached> {
    let mut stack: Vec<String> = Vec::new();
    let mut reached = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let digits: String = line.chars().take_while(char::is_ascii_digit).collect();
        let depth: usize = digits
            .parse()
            .unwrap_or_else(|_| panic!("a cargo tree line with no depth prefix: {line:?}"));
        let rest = &line[digits.len()..];
        // `{p}` is `name version` and, for a workspace member, a path after it.
        // A deduplicated subtree is marked with a trailing `(*)`, which is a
        // repeat of a package already listed and carries no new capability.
        let name = rest
            .split_whitespace()
            .next()
            .unwrap_or_else(|| panic!("a cargo tree line with no package: {line:?}"))
            .to_owned();
        stack.truncate(depth);
        stack.push(name.clone());
        reached.push(Reached {
            name,
            path: stack.clone(),
        });
    }
    reached
}

/// Every `(name, links)` pair the resolved metadata declares.
///
/// Read with a scanner rather than with a serialisation crate. The whole
/// question is two string fields per package, and `docs/dependencies.md` asks
/// what removing a dependency would cost before one arrives: here it is a
/// brace counter, which is cheaper than the entry would be.
fn declared_links(text: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for object in objects_at_depth_two(text) {
        let name = string_field(object, "name");
        let links = string_field(object, "links");
        if let (Some(name), Some(links)) = (name, links) {
            pairs.push((name, links));
        }
    }
    pairs
}

/// The objects one level inside the top-level object's arrays.
///
/// `cargo metadata` puts every package object at that depth, and nothing else
/// this reads is nested deeper, so a brace counter that is aware of strings and
/// their escapes is enough. It is aware of both, because a package description
/// containing a brace inside a quoted string is not hypothetical.
fn objects_at_depth_two(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut objects = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (position, byte) in bytes.iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => {
                depth += 1;
                if depth == 2 {
                    start = position;
                }
            }
            b'}' => {
                if depth == 2 {
                    objects.push(&text[start..=position]);
                }
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    objects
}

/// The value of a top-level string field of `object`, or `None` when it is
/// absent or null.
fn string_field(object: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let mut from = 0usize;
    while let Some(position) = object[from..].find(&needle) {
        let after = from + position + needle.len();
        let rest = object[after..].trim_start();
        if let Some(value) = rest.strip_prefix('"') {
            let mut out = String::new();
            let mut characters = value.chars();
            while let Some(character) = characters.next() {
                match character {
                    '\\' => out.push(characters.next().unwrap_or('\\')),
                    '"' => return Some(out),
                    other => out.push(other),
                }
            }
            return Some(out);
        }
        from = after;
    }
    None
}

/// What the graph of `member` is refused for, by package name.
///
/// A function rather than the body of a test, so that the refusal itself can be
/// run against a graph that carries a client. Otherwise the only thing anyone
/// could check is that it stays quiet on a tree it has never had to refuse.
fn refusals_by_name(member: &str, graph: &[Reached]) -> Vec<String> {
    let denied: BTreeSet<&str> = DENIED.into_iter().collect();
    graph
        .iter()
        .filter(|package| denied.contains(package.name.as_str()))
        .map(|package| {
            format!(
                "{member}: {} reached by {}",
                package.name,
                package.path.join(" -> ")
            )
        })
        .collect()
}

/// What the resolved metadata is refused for, by declared link.
fn refusals_by_link(metadata: &str) -> Vec<String> {
    let denied: BTreeSet<&str> = DENIED_LINKS.into_iter().collect();
    declared_links(metadata)
        .into_iter()
        .filter(|(_, links)| denied.contains(links.as_str()))
        .map(|(name, links)| format!("{name} declares links = {links:?}"))
        .collect()
}

#[test]
fn no_package_in_a_guarded_graph_is_on_the_denied_list() {
    let refusals: Vec<String> = GUARDED
        .into_iter()
        .flat_map(|member| refusals_by_name(member, &resolved_graph(member)))
        .collect();
    assert!(
        refusals.is_empty(),
        "network-capable package(s) in the shipped graph:\n{}",
        refusals.join("\n")
    );
}

#[test]
fn no_package_in_the_resolved_metadata_links_a_networking_library() {
    let metadata = run(&["metadata", "--format-version", "1", "--locked", "--offline"]);
    let refusals = refusals_by_link(&metadata);
    assert!(
        refusals.is_empty(),
        "package(s) linking a system networking library:\n{}",
        refusals.join("\n")
    );
}

/// The graph is read, not assumed.
///
/// Both tests above pass on an empty result as happily as on a correct one, so
/// this refuses the case where `cargo tree` returned nothing and the refusal
/// above was green because it examined no packages.
#[test]
fn every_guarded_graph_was_actually_read() {
    for member in GUARDED {
        let graph = resolved_graph(member);
        assert!(
            !graph.is_empty(),
            "{member}: the resolved graph came back empty"
        );
        assert_eq!(
            graph[0].name, member,
            "{member}: the graph does not start at the member it was asked for"
        );
    }
}

#[cfg(test)]
mod parsing {
    use super::{
        DENIED, DENIED_LINKS, declared_links, parse_tree, refusals_by_link, refusals_by_name,
        string_field,
    };

    /// The shape `cargo tree --prefix depth --format {p}` produces, with a
    /// client three levels down, which is where one actually arrives.
    const TREE: &str = "\
0drehbank-cli v0.0.0 (/w/crates/drehbank-cli)
1drehbank-core v0.0.0 (/w/crates/drehbank-core)
2fancy-oracle v2.1.0
3reqwest v0.12.9
4hyper v1.5.1
2num-traits v0.2.19
";

    #[test]
    fn the_depth_prefix_reconstructs_the_chain_that_reached_a_package() {
        let reached = parse_tree(TREE);
        let names: Vec<&str> = reached.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "drehbank-cli",
                "drehbank-core",
                "fancy-oracle",
                "reqwest",
                "hyper",
                "num-traits"
            ]
        );
        let client = reached
            .iter()
            .find(|entry| entry.name == "reqwest")
            .expect("the fixture has a client in it");
        assert_eq!(
            client.path,
            ["drehbank-cli", "drehbank-core", "fancy-oracle", "reqwest"],
            "the path is what tells a reader which direct dependency dragged it in"
        );
        // Leaving the depth back out at 2 has to pop the chain rather than
        // extend it, which is the mistake that reports a plausible wrong path.
        let sibling = reached.last().expect("the fixture ends at depth 2");
        assert_eq!(
            sibling.path,
            ["drehbank-cli", "drehbank-core", "num-traits"]
        );
    }

    #[test]
    fn the_denied_names_in_the_fixture_are_found() {
        let found: Vec<String> = parse_tree(TREE)
            .into_iter()
            .filter(|entry| DENIED.contains(&entry.name.as_str()))
            .map(|entry| entry.name)
            .collect();
        assert_eq!(found, ["reqwest", "hyper"]);
    }

    /// The refusal itself, run against a graph that carries a client.
    ///
    /// The two tests over the real tree are green because the real tree has
    /// nothing in it to refuse, which says nothing about whether they could
    /// refuse anything. This is the same function on a graph that has to be
    /// refused, and what it has to produce is the chain rather than the name.
    #[test]
    fn the_refusal_names_the_package_and_the_chain_that_reached_it() {
        let refusals = refusals_by_name("drehbank-cli", &parse_tree(TREE));
        assert_eq!(
            refusals,
            [
                "drehbank-cli: reqwest reached by drehbank-cli -> drehbank-core -> fancy-oracle -> reqwest",
                "drehbank-cli: hyper reached by drehbank-cli -> drehbank-core -> fancy-oracle -> reqwest -> hyper",
            ]
        );
    }

    /// A description carrying a brace inside a quoted string, which is what a
    /// counter that does not know about strings gets wrong.
    const METADATA: &str = r#"{"packages":[
{"name":"drehbank-core","version":"0.0.0","links":null,"description":"a { brace } in prose"},
{"name":"openssl-sys","version":"0.9.104","links":"openssl","description":"FFI"},
{"name":"num-traits","version":"0.2.19","description":"no links key at all"}
],"workspace_root":"/w"}"#;

    #[test]
    fn a_declared_link_is_read_and_a_null_or_absent_one_is_not() {
        assert_eq!(
            declared_links(METADATA),
            [("openssl-sys".to_owned(), "openssl".to_owned())]
        );
        assert!(DENIED_LINKS.contains(&"openssl"));
    }

    /// The link refusal, run against metadata that has to be refused.
    ///
    /// This is the half a graph carrying an ordinary client does not exercise:
    /// a pure-Rust client pulls in no `-sys` wrapper, so the tree has to be one
    /// with a declared link in it, and here that is the fixture.
    #[test]
    fn a_wrapper_declaring_a_networking_library_is_refused_by_its_link() {
        assert_eq!(
            refusals_by_link(METADATA),
            [r#"openssl-sys declares links = "openssl""#]
        );
    }

    #[test]
    fn a_brace_inside_a_string_does_not_end_the_object() {
        let objects = super::objects_at_depth_two(METADATA);
        assert_eq!(objects.len(), 3, "three packages, not more");
        assert_eq!(
            string_field(objects[0], "description").as_deref(),
            Some("a { brace } in prose")
        );
    }

    #[test]
    fn an_escaped_quote_inside_a_string_does_not_end_it() {
        let text = r#"{"packages":[{"name":"odd","description":"a \" quote","links":"ssl"}]}"#;
        assert_eq!(declared_links(text), [("odd".to_owned(), "ssl".to_owned())]);
    }
}
