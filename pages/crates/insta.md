---
title: "insta"
version: "1.49.0"
publisher: "Armin Ronacher (mitsuhiko), Maximilian Roos (max-sixty)"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-28"
summary: "Snapshot testing: assert that output matches a stored copy, and when it changes, review the diff and accept it. Removes the work of writing out expected values by hand."
domain: "Testing & benchmarking"
categories: ["development-tools::testing"]
repository: "https://github.com/mitsuhiko/insta"
docs: "https://docs.rs/insta/1.49.0/insta/"
---

## Overview

Writing the expected value by hand is the tedious part of testing anything with
large output — a rendered template, a parsed syntax tree, a formatted error, a
serialised document. Snapshot testing inverts it: run the code, look at what it
produced, and if it is right, store it as the expectation.

```
use insta::assert_snapshot;

fn render(name: &str, items: &[&str]) -> String {
    let mut out = format!("# {name}\n");
    for item in items {
        out.push_str(&format!("- {item}\n"));
    }
    out
}

// The expected value is written inline, after the @.
assert_snapshot!(render("List", &["one", "two"]), @r"
# List
- one
- two
");
```

**The workflow is the product, more than the macros are.** The first time a
snapshot assertion runs with nothing stored, it **fails** and writes a `.snap.new`
file beside the test. You then run `cargo insta review`, which shows a coloured
diff of old against new and asks per snapshot: accept, reject, or skip.
Accepting writes the `.snap` file, which you commit. On every later run the
assertion compares against it, and a change in output fails with a diff rather
than a "left != right" dump of two thousand characters.

That first failure surprises people, so it is worth stating plainly: a new
snapshot test does not pass until you have accepted it. `cargo insta test
--accept` does both in one step, and `INSTA_UPDATE=always` in the environment
accepts without review — useful in a tight loop, dangerous as a habit.

**The real risk is accepting without reading.** A snapshot test is only as good
as your attention when the diff appeared; press `a` on autopilot and you have
recorded the bug as the expectation. This is the honest cost of the technique,
and it is why snapshots suit output whose correctness you can *see* — text,
structure, formatting — and suit numerical or security-relevant results much
less well.

**Two forms, and the choice matters.** Inline snapshots (`@"..."`) live in the
source, so the test is self-contained and reviewing the diff in a pull request
shows the change in place; they get unwieldy past a few dozen lines. File
snapshots live in `snapshots/` next to the test and handle any size, at the cost
of the expectation being a file away from the assertion. Start inline, move out
when it gets long.

**Non-determinism has to be dealt with** or every run fails: timestamps, UUIDs,
temp paths, hash-map ordering. Redactions replace a matched field with a
placeholder, and `Settings` can apply a filter by regex. If output cannot be
made deterministic, it is not a snapshot candidate.

**Most of the interesting features are opt-in**, and the errors when they are
missing are abrupt rather than helpful. `redactions` is needed for the
`{".field" => "[redacted]"}` blocks, and without it the macro fails to compile
with "insta was compiled without redactions support". `filters` is needed for
`Settings::add_filter`, and without it the method simply does not exist. Neither
is implied by `yaml` or `json`, which only pull in `serde`. A dependency line
covering everything on this page is
`insta = { version = "1.49", features = ["yaml", "json", "filters", "redactions"] }`.

Pairs with `cargo-insta`, the companion CLI that provides `review` — the library
works without it, but the workflow does not, so install both.
`assert_debug_snapshot!` needs no features at all and covers the common case.
MSRV is 1.66.

## When to use it

### Use case: Testing formatted or rendered output

Where the expected value is too big to type.

```
use insta::assert_snapshot;

fn format_table(rows: &[(&str, u32)]) -> String {
    let mut out = String::new();
    for (name, count) in rows {
        out.push_str(&format!("{name:<10} {count:>5}\n"));
    }
    out
}

#[test]
fn renders_the_table() {
    let rows = [("alpha", 1), ("beta", 22), ("gamma", 333)];

    assert_snapshot!(format_table(&rows), @r"
    alpha          1
    beta          22
    gamma        333
    ");
}
```

**Why it fits:** the alignment *is* the thing under test, and expressing it as a
hand-written string literal with counted spaces is miserable to write and worse
to maintain. With a snapshot you check the rendering once by eye and thereafter
the test guards it. This is the strongest case for the technique: CLI help text,
pretty-printed errors, generated code, templated documents.

### Use case: Pinning the shape of a parsed structure

`assert_debug_snapshot!`, which needs only `Debug`.

```
use insta::assert_debug_snapshot;

#[derive(Debug)]
struct Config {
    name: String,
    retries: u8,
    features: Vec<String>,
}

fn parse(text: &str) -> Config {
    let mut config = Config {
        name: String::new(),
        retries: 0,
        features: Vec::new(),
    };
    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "name" => config.name = value.trim().to_string(),
                "retries" => config.retries = value.trim().parse().unwrap_or(0),
                "features" => {
                    config.features =
                        value.split(',').map(|f| f.trim().to_string()).collect()
                }
                _ => {}
            }
        }
    }
    config
}

#[test]
fn parses_the_whole_structure() {
    let parsed = parse("name = svc\nretries = 3\nfeatures = a, b");

    // One assertion covers every field, including ones added later.
    assert_debug_snapshot!(parsed, @r#"
    Config {
        name: "svc",
        retries: 3,
        features: [
            "a",
            "b",
        ],
    }
    "#);
}
```

**Why it fits:** field-by-field `assert_eq!` means a new field is silently
untested until someone remembers to assert it; a snapshot covers the whole value,
so adding a field makes the test fail and demands a decision. That is usually
what you want from a parser test. The flip side is that any change anywhere in
the structure fails this test, which is noise if the structure is volatile — so
snapshot the part you care about rather than a whole application state.

### Use case: Handling values that change every run

Redactions and filters, for the non-deterministic parts.

```
use insta::{assert_snapshot, Settings};

fn log_line(id: u64, message: &str) -> String {
    format!("[{id}] {message}")
}

#[test]
fn ignores_the_identifier() {
    let mut settings = Settings::clone_current();

    // Replace anything matching the pattern before comparing.
    settings.add_filter(r"\[\d+\]", "[ID]");

    settings.bind(|| {
        assert_snapshot!(log_line(918_273, "started"), @"[ID] started");
    });
}
```

**Why it fits:** a snapshot containing today's date or a random id fails
tomorrow, and the usual reaction — deleting the test — loses real coverage.
A filter normalises the varying part so the rest stays under test. `add_filter`
takes a regex and a replacement and is the general tool; for serialised
structures, `assert_yaml_snapshot!` with redaction paths is more precise, since
it targets a named field rather than matching text. Either way the principle is
to pin what you control and blank what you do not.

## API map

A family of assertion macros that differ only in how they format the value,
plus `Settings` for everything configurable.

### Assertions

#### `assert_snapshot!`

For anything that is already a string.

```
use insta::assert_snapshot;

#[test]
fn strings() {
    // Inline: the expectation follows @.
    assert_snapshot!("hello", @"hello");

    // Multi-line, with r" " to avoid escaping.
    assert_snapshot!("line one\nline two", @r"
    line one
    line two
    ");

    // Omitting @ makes it a file snapshot, stored under snapshots/.
    // assert_snapshot!(some_long_output);
}
```

**When to use it:** whenever the value is a `String` or `&str`, which after a
`format!` or a `Display` is most of the time. It is also the one to use for
anything you have already serialised yourself. Leading indentation in the inline
form is stripped, which is why the expected text can be indented to match the
surrounding code.

#### `assert_debug_snapshot!`

For any type implementing `Debug`.

```
use insta::assert_debug_snapshot;

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn debug_output() {
    assert_debug_snapshot!(Point { x: 1, y: 2 }, @r"
    Point {
        x: 1,
        y: 2,
    }
    ");

    assert_debug_snapshot!(vec![1, 2, 3], @"[1, 2, 3]");
}
```

**When to use it:** as the default for structured data, because it needs no
dependency beyond `Debug` and `Debug` is derived everywhere. The caveat is that
`Debug` output is explicitly not a stable format — a standard-library change to
how a `HashMap` or a `Duration` prints will break the snapshot. For data you
need stability on, serialise it instead.

#### `assert_yaml_snapshot!` and `assert_json_snapshot!`

For `Serialize` types, with redaction support.

```
use insta::assert_yaml_snapshot;
use serde::Serialize;

#[derive(Serialize)]
struct Response {
    id: u64,
    status: String,
}

#[test]
fn serialised() {
    let response = Response { id: 7, status: "ok".into() };

    // Redactions replace a field by path, leaving the rest under test.
    assert_yaml_snapshot!(response, {
        ".id" => "[id]"
    }, @r#"
    id: "[id]"
    status: ok
    "#);
}
```

**When to use it:** for API responses and anything else with a `Serialize` impl,
where YAML reads better in a diff than `Debug` does. The redaction block is the
real reason to prefer these over `assert_debug_snapshot!`: `".id" => "[id]"`
blanks one field by path, which is far more precise than a regex over the whole
rendered text. Requires the `yaml` or `json` feature, **and `redactions` for
the redaction block** — which is not implied by either.

### Configuration

#### `Settings`

Scoped configuration for a block of assertions.

```
use insta::{assert_snapshot, Settings};

#[test]
fn configured() {
    let mut settings = Settings::clone_current();

    // A suffix distinguishes snapshots of the same test across variants.
    settings.set_snapshot_suffix("linux");
    // Drop the "Expression: ..." line from the stored file.
    settings.set_omit_expression(true);
    // Normalise anything that varies.
    settings.add_filter(r"\d{4}-\d{2}-\d{2}", "[DATE]");

    settings.bind(|| {
        assert_snapshot!("built on 2026-10-28", @"built on [DATE]");
    });
}
```

**When to use it:** `set_snapshot_suffix` when one test runs under several
configurations and each needs its own stored output — per platform, per feature
flag. `add_filter` for non-determinism, as above. `bind` scopes the settings to
the closure, which matters because the alternative, mutating global state,
breaks when tests run in parallel. `Settings::clone_current()` rather than
`new()` keeps whatever the surrounding configuration already set. Note that
`add_filter` exists only with the **`filters`** feature enabled.

#### `with_settings!`

The same thing, as a macro.

```
use insta::{assert_snapshot, with_settings};

#[test]
fn inline_settings() {
    with_settings!({filters => vec![(r"\d+", "[N]")]}, {
        assert_snapshot!("saw 42 items", @"saw [N] items");
    });
}
```

**When to use it:** for a one-off adjustment, where building a `Settings` value
is more ceremony than the change deserves. It takes the same options under
shorthand names and scopes them to the block. For settings shared across a whole
test module, a `Settings` value bound once reads better.

### Glob testing

#### `glob!`

Running one assertion over a directory of input files.

```
// no-compile — needs fixture files on disk to iterate
use insta::{assert_snapshot, glob};

#[test]
fn every_fixture_parses() {
    glob!("fixtures/*.txt", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        // One snapshot per input file, named after it.
        assert_snapshot!(input.trim().to_uppercase());
    });
}
```

**When to use it:** for a parser, formatter or compiler with a corpus of test
inputs. Adding a case becomes dropping a file in a directory rather than writing
a test, and each file gets its own snapshot named after it — so a regression
names the input that broke. It is the natural fit for the "hundreds of small
example files" shape that language tooling accumulates.
