---
title: "rstest"
version: "0.28.0"
publisher: "Michele d'Amico (la10736)"
publisher_url: "https://crates.io/users/la10736"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-28"
summary: "Fixtures and table-driven cases for Rust's own test harness: declare a test's dependencies as parameters, list the inputs as attributes, and get one real `#[test]` per case."
domain: "Testing & benchmarking"
categories: ["development-tools::testing"]
repository: "https://github.com/la10736/rstest"
docs: "https://docs.rs/rstest/0.28.0/rstest/"
---

## Overview

Rust's test harness takes no arguments, so two patterns show up in every
codebase: a helper function called at the top of every test to build the same
scaffolding, and a block of near-identical tests that differ only in a value.
`rstest` removes both.

```
use rstest::rstest;

fn square(n: u32) -> u32 {
    n * n
}

// One function, three separate tests.
#[rstest]
#[case(1, 1)]
#[case(2, 4)]
#[case(3, 9)]
fn squares_correctly(#[case] input: u32, #[case] expected: u32) {
    assert_eq!(square(input), expected);
}
```

That generates three real `#[test]` functions named
`squares_correctly::case_1`, `case_2` and `case_3`. They run in parallel, fail
independently, and `cargo test case_2` filters to one — because they *are*
ordinary tests. Nothing about the harness changes, which is why `rstest` drops
into an existing suite with no ceremony.

The other half is fixtures: a function marked `#[fixture]` becomes available as
a parameter by name.

```
use rstest::{fixture, rstest};

#[fixture]
fn numbers() -> Vec<u32> {
    vec![1, 2, 3]
}

// The parameter name matches the fixture, so it is supplied.
#[rstest]
fn uses_the_fixture(numbers: Vec<u32>) {
    assert_eq!(numbers.len(), 3);
}
```

**The thing that makes fixtures worth more than a helper call** is that they
compose: a fixture can itself take fixtures as parameters, and `rstest`
resolves the graph. So a `database` fixture depending on a `config` fixture
depending on a `tempdir` fixture is three small functions rather than one
nested builder, and a test asks only for what it needs.

**Where it earns its place most clearly** is the matrix. `#[values(..)]` on
several parameters generates the cross product, so testing four codecs against
three input sizes is twelve tests from one function — and each is named for its
combination, so a failure tells you which pair broke without reading the body.

**What it is not.** It does not generate inputs for you — that is
[`proptest`](proptest.md), and the two answer different questions: `rstest`
runs the cases *you* chose, property testing searches for ones you did not.
It also does not replace a test framework, because there is nothing to replace;
it is a code generator over `#[test]`.

The cost is a proc-macro dependency and attribute syntax to learn, and the
usual proc-macro consequence: when you get the attributes wrong the error can
point at the macro rather than your mistake. Note the **MSRV is 1.85**, and
async tests need an executor attribute from `tokio` or `async-std` alongside.

## When to use it

### Use case: Replacing a block of near-identical tests

The table-driven case, which is most of why people adopt it.

```
use rstest::rstest;

fn parse_port(text: &str) -> Result<u16, String> {
    let port: u16 = text.parse().map_err(|_| "not a number".to_string())?;
    if port < 1024 {
        return Err("privileged".to_string());
    }
    Ok(port)
}

#[rstest]
// Named cases, so a failure reports which one.
#[case::valid("8080", Ok(8080))]
#[case::privileged("80", Err("privileged".to_string()))]
#[case::not_a_number("http", Err("not a number".to_string()))]
#[case::empty("", Err("not a number".to_string()))]
fn parses_ports(#[case] input: &str, #[case] expected: Result<u16, String>) {
    assert_eq!(parse_port(input), expected);
}
```

**Why it fits:** written by hand this is four functions sharing one body, and
the fourth one is where someone copies the third and forgets to change the
assertion. Here the body exists once. `#[case::name(..)]` is worth using over
bare `#[case]` — it names the generated test `parses_ports::valid` instead of
`case_1`, so the output says what broke rather than where it was in the list.

### Use case: Sharing expensive setup between tests

Fixtures, including the compose-and-cache cases.

```
use rstest::{fixture, rstest};

struct Config {
    retries: u8,
}

struct Client {
    config: Config,
    calls: usize,
}

#[fixture]
fn config() -> Config {
    Config { retries: 3 }
}

// A fixture taking a fixture: rstest resolves the graph.
#[fixture]
fn client(config: Config) -> Client {
    Client { config, calls: 0 }
}

#[rstest]
fn client_starts_idle(client: Client) {
    assert_eq!(client.calls, 0);
    assert_eq!(client.config.retries, 3);
}

// A test asks only for what it needs.
#[rstest]
fn config_has_retries(config: Config) {
    assert_eq!(config.retries, 3);
}
```

**Why it fits:** the dependency is declared in the signature, so a test's needs
are visible without reading its body, and adding a layer of setup does not mean
editing every test that already existed. Fixtures run fresh per test by
default, which is what you want for isolation; `#[once]` on a fixture makes it
run a single time and hand out a shared reference, for something genuinely
expensive and genuinely immutable like a parsed data file.

### Use case: Covering a combination of conditions

The matrix, where hand-writing the cases is impractical.

```
use rstest::rstest;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Codec {
    Raw,
    Gzip,
}

fn encoded_len(codec: Codec, input_len: usize) -> usize {
    match codec {
        Codec::Raw => input_len,
        Codec::Gzip => input_len / 2 + 18,
    }
}

// Two codecs times three sizes: six generated tests.
#[rstest]
fn encoding_never_loses_everything(
    #[values(Codec::Raw, Codec::Gzip)] codec: Codec,
    #[values(0usize, 64, 4096)] size: usize,
) {
    let encoded = encoded_len(codec, size);
    assert!(encoded > 0 || size == 0);
}
```

**Why it fits:** the combination is where bugs live — the codec that works at
4096 bytes and divides by zero at 0 — and writing six tests by hand means
someone eventually adds a codec and updates three of them. The generated names
include both values, so a failure names the exact pair. Combinations multiply
fast, so keep the value lists short; `#[values]` and `#[case]` can be mixed,
which gives a matrix over each listed case.

## API map

Two attributes, a handful of parameter annotations, and the async support.

### Test and case attributes

#### `#[rstest]`

Marks a function as a test rstest should expand.

```
use rstest::rstest;

// With no parameters it is simply a #[test].
#[rstest]
fn plain_test() {
    assert_eq!(2 + 2, 4);
}

// The std attributes still apply.
#[rstest]
#[should_panic(expected = "boom")]
fn panics_as_expected() {
    panic!("boom");
}
```

**When to use it:** on any test using fixtures or cases, and harmlessly on ones
that use neither. It replaces `#[test]` rather than accompanying it — writing
both is an error. `#[should_panic]`, `#[ignore]` and `#[timeout]` work as you
would expect, and are applied to every generated case.

#### `#[case]` and `#[case::name]`

One generated test per listed tuple.

```
use rstest::rstest;

#[rstest]
#[case(0, "zero")]
#[case::one(1, "one")]
#[case::many(5, "many")]
fn describes(#[case] n: u32, #[case] expected: &str) {
    let actual = match n {
        0 => "zero",
        1 => "one",
        _ => "many",
    };
    assert_eq!(actual, expected);
}
```

**When to use it:** whenever several tests differ only in data. Each `#[case]`
supplies the parameters marked `#[case]`, positionally and in order — so the
count must match, and a mismatch is the commonest mistake. Prefer the named
form: `describes::many` in the test output beats `describes::case_3`, and the
name survives reordering the list.

#### `#[values]`

The cross product.

```
use rstest::rstest;

#[rstest]
fn matrix(
    #[values(1u32, 2, 3)] a: u32,
    #[values(10u32, 20)] b: u32,
) {
    // Six tests: every a with every b.
    assert!(a < b);
}
```

**When to use it:** for independent dimensions — a feature flag crossed with an
input size, an enum variant crossed with a boundary value. Note the difference
from `#[case]`: cases are a list of *tuples you chose*, values are a list per
parameter *combined automatically*. Reach for `#[case]` when only certain
combinations make sense, since a matrix will happily generate the nonsensical
ones too.

### Fixtures

#### `#[fixture]`

A function that supplies a test's input.

```
use rstest::{fixture, rstest};

#[fixture]
fn empty_vec() -> Vec<String> {
    Vec::new()
}

#[fixture]
fn two_items(mut empty_vec: Vec<String>) -> Vec<String> {
    empty_vec.push("a".into());
    empty_vec.push("b".into());
    empty_vec
}

#[rstest]
fn starts_empty(empty_vec: Vec<String>) {
    assert!(empty_vec.is_empty());
}

#[rstest]
fn composed(two_items: Vec<String>) {
    assert_eq!(two_items.len(), 2);
}
```

**When to use it:** for any setup two or more tests share. Matching is **by
parameter name**, which is the one surprising rule — rename the parameter and
the fixture is no longer found, and the error says the name is unknown rather
than suggesting the fixture. A fixture is re-run for every test that asks for
it, so there is no cross-test contamination to reason about.

#### `#[default]` and `#[with]`

Parameterising a fixture.

```
use rstest::{fixture, rstest};

// A fixture with its own arguments and defaults.
#[fixture]
fn repeated(#[default(3)] count: usize, #[default("x")] text: &str) -> String {
    text.repeat(count)
}

#[rstest]
fn uses_defaults(repeated: String) {
    assert_eq!(repeated, "xxx");
}

// #[with] overrides them for one test.
#[rstest]
fn overrides(#[with(2, "ab")] repeated: String) {
    assert_eq!(repeated, "abab");
}
```

**When to use it:** when most tests want the standard setup but a few need a
variant — a client with a short timeout, a database with a particular schema.
It is the alternative to writing five near-identical fixtures, and keeps the
variation visible at the call site rather than buried in a fixture name like
`client_with_short_timeout_and_no_retries`.

#### `#[once]`

A fixture evaluated a single time.

```
use rstest::{fixture, rstest};

// Runs once for the whole test binary; tests get a shared reference.
#[fixture]
#[once]
fn expensive_table() -> Vec<u32> {
    (0..1000).collect()
}

#[rstest]
fn reads_the_table(expensive_table: &Vec<u32>) {
    assert_eq!(expensive_table.len(), 1000);
}
```

**When to use it:** for setup that is expensive and provably immutable — a
parsed fixture file, a compiled regex, a loaded model. The signature changes to
a reference, which is the compiler enforcing that you cannot mutate shared
state from a test. Do not reach for it to speed up a mutable fixture: tests run
in parallel, and sharing mutable setup between them is how you get failures
that depend on ordering.

### Async

#### Async tests

Fixtures and cases with `.await`.

```
use rstest::{fixture, rstest};

#[fixture]
async fn value() -> u32 {
    42
}

// The executor comes from tokio or async-std, not from rstest.
#[rstest]
#[tokio::test]
async fn awaits_the_fixture(#[future] value: u32) {
    assert_eq!(value.await, 42);
}

#[rstest]
#[case(1u32)]
#[case(2u32)]
#[tokio::test]
async fn async_cases(#[case] n: u32) {
    assert!(n > 0);
}
```

**When to use it:** for anything async, which in practice means most integration
tests of a service. Two details matter: `rstest` supplies no runtime, so the
executor attribute (`#[tokio::test]`, `#[async_std::test]`) must be present
alongside; and an async fixture arrives as a future, marked `#[future]` at the
use site and awaited in the body. `#[future(awt)]` or a top-level `#[awt]`
awaits it for you if the `.await` noise bothers you.
