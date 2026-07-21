---
title: "#[ignore]"
kind: attribute
embedded_support: partial
groups: ["Testing", "Testing & Tooling"]
related_concepts: ["Unit tests"]
related_syntax: ["#[test]", "#[should_panic]"]
see_also: ["#[test]"]
---

## Explanation

`#[ignore]` is placed alongside `#[test]` on a test function and excludes
it from the default `cargo test` run — the test still compiles and still
exists, but `cargo test` skips executing it and reports it as `ignored`
rather than passed or failed, unless the run is invoked with
`cargo test -- --include-ignored`, which runs ignored tests too.

The optional form `#[ignore = "reason"]` attaches a string explaining
*why* the test is skipped by default, shown alongside the test's name in
`cargo test`'s output — `#[ignore = "requires a live database connection"]`
tells the next person (or the same person, months later) why this test
isn't part of the everyday run without them having to go read the test
body or dig through commit history to find out.

This is the tool for a test that is correct and worth keeping, but
unsuitable for every ordinary `cargo test` invocation — most often because
it's slow (an end-to-end test hitting real infrastructure), flaky
(intermittent failures under CI load, pending an actual fix), or requires
an environment not guaranteed to be present (a specific piece of hardware,
a network resource). It is deliberately different from deleting the test
or commenting it out: an ignored test still compiles, so it can't silently
rot into code that no longer even builds, and it still shows up (as
skipped) in the test summary as a visible reminder that it exists and is
excluded on purpose.

## Usage examples

### Skipping a test and documenting why

```
#[test]
#[ignore = "hits a live network endpoint; run explicitly with --include-ignored"] // <- reason shown in test output
fn fetches_remote_config() {
    // network call omitted for this example
}
```

### Testing

A test suite has one test that's genuinely slow — it exercises a full
retry-with-backoff path against a simulated flaky connection — and
running it on every `cargo test` invocation would make the fast, everyday
loop noticeably slower for no benefit during normal development.

```
fn retry_with_backoff(max_attempts: u32) -> Result<(), &'static str> {
    if max_attempts == 0 {
        Err("exhausted retries")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn succeeds_with_available_retries() {
        assert!(retry_with_backoff(3).is_ok());
    }

    #[test]
    #[ignore = "exercises real backoff timing; slow, run explicitly before releases"] // <- documents WHY
    fn retries_across_simulated_backoff_delays() {
        // a full run of this test sleeps for real backoff intervals
        assert!(retry_with_backoff(3).is_ok());
    }
}
```

Naming the reason directly in the attribute — rather
than leaving a bare `#[ignore]` and a separate comment, or no explanation
at all — means anyone looking at `cargo test`'s output already knows
whether a given skipped test is worth running with `--include-ignored`
right now, without switching to the source file first; the
[Rust Book](https://doc.rust-lang.org/book/ch11-02-running-tests.html#ignoring-some-tests-unless-specifically-requested)
recommends the `= "reason"` form for exactly this reason.

## Explanation (Embedded)

`#[ignore]` is a modifier on `#[test]`, and `#[test]`'s harness itself
needs `std` — see [`#[test]`](test-attribute.md)'s Embedded Rust Notes for
the full host-vs-target testing story: hardware-independent embedded
logic is normally tested with plain `cargo test` on the host, behind a
`#[cfg(test)]` split, rather than on the target. Within that host-tested
slice, `#[ignore]` behaves exactly as it does anywhere else — nothing
about the code belonging to a `#![no_std]` crate changes what
`#[ignore]`/`#[ignore = "reason"]` do once the test is actually compiled
and run on the host. It's also a reasonable way to mark a host test that's
only meaningful with real hardware attached over `probe-rs` (an
integration-style test that flashes a board and reads a real response
back) as excluded from the default host-only run, since that scenario is
common enough in embedded projects to be worth naming directly.

## Usage examples (Embedded)

### Marking a hardware-in-the-loop host test as ignored by default

```
fn parse_frame(bytes: &[u8]) -> Result<u8, &'static str> {
    bytes.first().copied().ok_or("empty frame")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_frame() {
        assert_eq!(parse_frame(&[0x42]), Ok(0x42));
    }

    #[test]
    #[ignore = "requires a board attached over probe-rs; run explicitly, not on every host build"] // <- reason shown in output
    fn round_trips_a_frame_through_real_hardware() {
        // a full run of this test flashes a target board and reads the response back over the probe
    }
}
```
