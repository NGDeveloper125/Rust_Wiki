//! The crate directory's type taxonomy — the buckets a crate page is filed
//! under, in the sidebar and as sections on the index.
//!
//! The list mirrors the editorial classification the maintainer already uses
//! to sort the coverage backlog, with one deliberate difference: that list's
//! single "GUI, graphics & games" bucket is split three ways here, because
//! someone choosing a desktop app framework, someone choosing a game engine
//! and someone choosing an image library are not the same reader.
//!
//! A page names its bucket with `domain:` in its frontmatter, matched against
//! `label` case-insensitively. It is a closed list on purpose: a typo should
//! be a build warning, not a section of one crate nobody meant to create.

use super::Crate;

/// One bucket in the taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Domain {
    /// Display name, and the value a page writes in `domain:`.
    pub label: &'static str,
    /// URL fragment for the index section, e.g. `error-handling`.
    pub slug: &'static str,
    /// One line under the section heading, saying what belongs here.
    pub blurb: &'static str,
}

/// Every bucket, in the order they appear in the sidebar and on the index.
///
/// The order is editorial rather than alphabetical: the things most people
/// come looking for first (a runtime, a web framework, serialization, error
/// handling) sit at the top, and the deeper infrastructure a crate reaches
/// for rather than an application settles toward the bottom.
pub const DOMAINS: &[Domain] = &[
    Domain {
        label: "Async runtimes & concurrency",
        slug: "async-runtimes-concurrency",
        blurb: "Executors, task spawning, synchronisation primitives and the machinery under them.",
    },
    Domain {
        label: "HTTP, web & RPC",
        slug: "http-web-rpc",
        blurb: "Web frameworks, HTTP clients and servers, and the protocols services talk over.",
    },
    Domain {
        label: "Serialization & data formats",
        slug: "serialization-data-formats",
        blurb: "Turning values into bytes and back — JSON, TOML, binary formats, and the framework they share.",
    },
    Domain {
        label: "Error handling",
        slug: "error-handling",
        blurb: "Error types, context chains, and the derives that save writing them by hand.",
    },
    Domain {
        label: "CLI & terminal",
        slug: "cli-terminal",
        blurb: "Argument parsing, prompts, progress bars, colour, and full terminal user interfaces.",
    },
    Domain {
        label: "GUI & app frameworks",
        slug: "gui-app-frameworks",
        blurb: "Desktop and web application frameworks — windows, widgets, and the event loops behind them.",
    },
    Domain {
        label: "Game engines",
        slug: "game-engines",
        blurb: "Engines, and the systems games are built out of.",
    },
    Domain {
        label: "Graphics & images",
        slug: "graphics-images",
        blurb: "Rendering, rasterisation, image decoding, and plotting.",
    },
    Domain {
        label: "Databases & storage",
        slug: "databases-storage",
        blurb: "Database drivers, query builders, migrations, and embedded stores.",
    },
    Domain {
        label: "Text, parsing & Unicode",
        slug: "text-parsing-unicode",
        blurb: "Parsers, regular expressions, case conversion, and the rules text actually follows.",
    },
    Domain {
        label: "Collections & data structures",
        slug: "collections-data-structures",
        blurb: "Maps, sets, iterator adaptors, and containers with different trade-offs to the standard library's.",
    },
    Domain {
        label: "Numerics & math",
        slug: "numerics-math",
        blurb: "Linear algebra, arbitrary precision, statistics, and numeric traits.",
    },
    Domain {
        label: "Randomness & IDs",
        slug: "randomness-ids",
        blurb: "Random number generation, distributions, and unique identifiers.",
    },
    Domain {
        label: "Dates & time",
        slug: "dates-time",
        blurb: "Calendars, time zones, durations, and parsing the formats they are written in.",
    },
    Domain {
        label: "Crypto, hashing & TLS",
        slug: "crypto-hashing-tls",
        blurb: "Hashes, ciphers, signatures, password hashing, and transport security.",
    },
    Domain {
        label: "Compression & archives",
        slug: "compression-archives",
        blurb: "Compression codecs, and the archive formats built on them.",
    },
    Domain {
        label: "Files, paths & OS",
        slug: "files-paths-os",
        blurb: "The filesystem, processes, signals, and the platform APIs underneath.",
    },
    Domain {
        label: "Bytes, memory & layout",
        slug: "bytes-memory-layout",
        blurb: "Buffers, zero-copy casts, allocation control, and how a type is laid out.",
    },
    Domain {
        label: "FFI, wasm & bindings",
        slug: "ffi-wasm-bindings",
        blurb: "Calling other languages, being called by them, and running in the browser.",
    },
    Domain {
        label: "Macros, derive & codegen",
        slug: "macros-derive-codegen",
        blurb: "Procedural macro tooling, and the crates that generate code from a description.",
    },
    Domain {
        label: "Logging, tracing & metrics",
        slug: "logging-tracing-metrics",
        blurb: "Recording what a program did, and shipping it somewhere it can be read.",
    },
    Domain {
        label: "Testing & benchmarking",
        slug: "testing-benchmarking",
        blurb: "Assertions, fixtures, property testing, fuzzing, and measuring how fast it runs.",
    },
    Domain {
        label: "Config & environment",
        slug: "config-environment",
        blurb: "Layering configuration from files, the environment, and the command line.",
    },
    Domain {
        label: "Build scripts & tooling",
        slug: "build-scripts-tooling",
        blurb: "Build-time code: compiling native sources, finding system libraries, reading versions.",
    },
];

/// The bucket named by a `domain:` frontmatter value, or `None` if the value
/// isn't one of ours. Matching ignores case and surrounding whitespace, since
/// the label is prose a contributor retypes rather than an identifier.
pub fn lookup(name: &str) -> Option<&'static Domain> {
    let want = name.trim();
    DOMAINS.iter().find(|d| d.label.eq_ignore_ascii_case(want))
}

/// The buckets that actually have a page, in `DOMAINS` order.
///
/// Empty buckets are dropped rather than rendered as a heading with nothing
/// under it: a section that exists only to say "nothing here yet" advertises
/// a gap instead of helping anyone find a crate.
pub fn occupied(crates: &[Crate]) -> Vec<&'static Domain> {
    DOMAINS
        .iter()
        .filter(|d| crates.iter().any(|c| c.domain == Some(*d)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_unique_and_url_safe() {
        let mut seen: Vec<&str> = Vec::new();
        for d in DOMAINS {
            assert!(!seen.contains(&d.slug), "duplicate slug {}", d.slug);
            assert!(
                d.slug
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "slug {} is not url-safe",
                d.slug
            );
            seen.push(d.slug);
        }
    }

    #[test]
    fn lookup_is_case_and_space_insensitive() {
        assert_eq!(lookup("Error handling").map(|d| d.slug), Some("error-handling"));
        assert_eq!(lookup("  error HANDLING ").map(|d| d.slug), Some("error-handling"));
        assert!(lookup("Error Handling & friends").is_none());
        assert!(lookup("").is_none());
    }
}
