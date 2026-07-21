---
title: "println! / print! / eprintln! / eprint!"
kind: macro
embedded_support: none
groups: ["Macros & Metaprogramming"]
related_concepts: ["String formatting (Display, Debug, format!)"]
related_syntax: ["format!", "write! / writeln!"]
see_also: ["format!", "write! / writeln!"]
---

## Explanation

Four macros split along two independent axes: which stream they write to,
and whether they append a trailing newline. `println!`/`print!` write to
standard output; `eprintln!`/`eprint!` write the identical formatted text
to standard error instead. `println!`/`eprintln!` append a `\n` after the
formatted text; `print!`/`eprint!` don't, leaving the cursor on the same
line for further output.

All four parse the same format-string mini-language as `format!` and
`write!` — `{}` for a value's `Display` output, `{:?}` for `Debug`,
positional (`{0}`) and named (`{name}`) arguments — this page doesn't
re-derive that grammar; see [`format!`](format-macro.md) for the full
syntax and
[String formatting](../../concepts/collections-strings/string-formatting.md)
for the `Display`/`Debug` distinction itself.

Two calling-convention points worth knowing: called with zero arguments,
`println!()`/`print!()` are legal and simply emit a bare newline (or
nothing at all); and standard output is line-buffered when connected to a
terminal but block-buffered when redirected to a file or pipe, so `print!`
output without a following `println!` may not appear until the buffer
flushes or the process exits — an explicit `std::io::stdout().flush()`
fixes that on the rare occasion it matters (a prompt printed with `print!`
right before reading input, for instance).

## Basic usage example

```
let temperature = 21.5;
println!("Reading: {temperature}");    // <- prints to stdout, with a trailing newline
eprintln!("warning: sensor is stale"); // <- prints to stderr instead, same newline behavior
```

## Best practices & deeper information

### Scenario: Working with text

A CLI tool prints normal progress to stdout, so it can be piped or
captured, and problems to stderr, so they show up even when stdout is
redirected elsewhere.

```
fn run_backup(files_done: u32, files_total: u32, failed: &[String]) {
    println!("backed up {files_done}/{files_total} files"); // <- progress goes to stdout, safe to pipe/redirect
    for name in failed {
        eprintln!("failed to back up {name}"); // <- errors go to stderr, so they survive `prog > log.txt`
    }
}

run_backup(8, 10, &["archive.zip".to_string()]);
```

**Why this way:** keeping errors on stderr and normal output on stdout
follows a long-standing Unix convention the
[std docs](https://doc.rust-lang.org/std/macro.eprintln.html) build
`println!`/`eprintln!` around specifically so that redirecting stdout
(`prog > out.txt`) doesn't silently swallow warnings along with it.

### Scenario: Documenting an API

A doc comment for a public function includes a runnable example that
prints the function's output for illustration, with `assert!` doing the
actual verification.

```
/// Converts a Celsius reading to Fahrenheit.
///
/// # Examples
///
/// ```
/// let f = to_fahrenheit(21.5);
/// println!("{f:.1}"); // <- shown for illustration; the doctest is really enforced by the assert below
/// assert!((f - 70.7).abs() < 0.05);
/// ```
pub fn to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}
```

**Why this way:** the [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
treats every fenced code block in a doc comment as a runnable test by
default, so `println!` inside one doubles as both documentation and a
compiled, executed example — but it's the `assert!` that actually catches
a regression, since a doctest only fails on a panic or compile error, not
on printed output going unchecked.

## Embedded Rust Notes

**No support.** `println!`/`print!`/`eprintln!`/`eprint!` are defined in
terms of `std::io::stdout()`/`stderr()`, which assume a hosted OS
providing file descriptors — none of that exists under `#![no_std]`, so
the toggle is off for a real reason, not merely a caveat. Embedded code
reaches for a different mechanism entirely: a `defmt`/`rtt-target`-style
logging macro that ships formatted output over a debug probe, or a
hand-rolled `write!` into a UART peripheral implementing
`core::fmt::Write` (see [`write!` / `writeln!`](write-macros.md)) — same
formatting grammar, a different sink.
