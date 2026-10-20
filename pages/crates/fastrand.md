---
title: "fastrand"
version: "2.5.0"
publisher: "Taiki Endo (taiki-e), admins"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-20"
summary: "Random numbers from a free function, with no setup and no dependencies — `fastrand::u32(1..=6)`. Fast and simple, and explicitly not cryptographically secure."
domain: "Randomness & IDs"
categories: ["algorithms"]
repository: "https://github.com/smol-rs/fastrand"
docs: "https://docs.rs/fastrand/2.5.0/fastrand/"
---

## Overview

`fastrand` answers one question: what if you just want a random number? There is
no generator to construct, no trait to import, no distribution to choose — the
functions are free functions on a thread-local generator, and the range goes in
the call.

```
// No setup, no imports.
let die = fastrand::u32(1..=6);
assert!((1..=6).contains(&die));

let index = fastrand::usize(..10);
assert!(index < 10);

let coin = fastrand::bool();
let _ = coin;

let mut hand = vec![1, 2, 3, 4, 5];
fastrand::shuffle(&mut hand);
assert_eq!(hand.len(), 5);
```

The range-in-the-call design is what distinguishes it. Where [`rand`](rand.md)
has `rng.gen_range(1..=6)` after obtaining an `Rng`, here the type is the
function name and the range is the argument, so there is nothing to set up and
nothing to pass around.

**It is not cryptographically secure, and its own documentation says so in
bold.** The algorithm is Wyrand — fast, small, statistically fine for
simulations and sampling, and entirely unsuitable for anything an adversary
cares about. Never use it for a session token, a password, a nonce, an API key
or a UUID that must be unguessable. For those, `rand`'s `OsRng` or `getrandom`
is the correct tool, and the distinction is not a formality: Wyrand's state is a
single `u64`, so a handful of outputs reveal it.

**Against `rand`,** which is the real decision. `rand` gives you a trait
hierarchy, a dozen distributions, seedable and cryptographically secure
generators, and a sizeable dependency tree. `fastrand` gives you none of that
and has **zero dependencies** — the whole generator is `struct Rng(u64)`. If you
need a normal distribution, a weighted choice, or a CSPRNG, use `rand`. If you
need a random index, a shuffle or a jittered retry delay, this is smaller,
faster to compile and enough. It is a `smol-rs` crate, and `tempfile` uses it to
generate filenames, which is the shape of its intended use.

**One genuine footgun, on WebAssembly.** The global generator seeds itself from
system entropy, which `wasm32-unknown-unknown` does not provide. Without the
`js` feature, the global RNG falls back to **a predefined seed** — so every run
of your program produces the identical sequence, silently. Enable `js` for
browser targets, or seed the generator yourself. On `no_std`, there is no global
generator at all and `Rng::with_seed` is the only constructor.

MSRV is 1.63, `no_std` works by turning off the default `std` feature, and
`fastrand-contrib` carries the niche algorithms this crate deliberately omits.

## When to use it

### Use case: A random choice in ordinary application code

Where reaching for `rand` would be more ceremony than the task deserves.

```
// Pick a backend to try first, so load spreads across them.
let backends = ["alpha", "beta", "gamma"];
let chosen = fastrand::choice(backends).unwrap();
assert!(backends.contains(&chosen));

// Jitter a retry delay, so clients do not reconnect in lockstep.
fn retry_delay_ms(attempt: u32) -> u64 {
    let base = 100u64 << attempt.min(6);
    base + fastrand::u64(..base / 2)
}

let delay = retry_delay_ms(3);
assert!((800..1200).contains(&delay));
```

**Why it fits:** neither of these needs a distribution or a reproducible stream,
and both are one line. The dependency cost is nil and the compile-time cost is
negligible, which matters when the alternative pulls in a tree to do something a
single function call covers. Retry jitter in particular is a case where
"statistically fine" is the entire requirement.

### Use case: Deterministic sequences for tests and benchmarks

Seeding so a failure can be reproduced.

```
// A local generator, seeded, so the sequence is fixed.
let mut rng = fastrand::Rng::with_seed(42);
let first: Vec<u64> = (0..5).map(|_| rng.u64(..100)).collect();

let mut again = fastrand::Rng::with_seed(42);
let second: Vec<u64> = (0..5).map(|_| again.u64(..100)).collect();

assert_eq!(first, second);

// The global generator can be seeded too, which affects the whole thread.
fastrand::seed(7);
let a = fastrand::u32(..);
fastrand::seed(7);
assert_eq!(fastrand::u32(..), a);
```

**Why it fits:** a property test or a randomised benchmark that cannot be
replayed is much less useful, and `with_seed` makes the run reproducible from a
number you can print in the failure message. Prefer a local `Rng` over
`fastrand::seed` in tests: the global one is thread-local, so seeding it from one
test perturbs every other test on that thread and makes the suite
order-dependent. Note the sequence for a given seed is not guaranteed stable
across crate versions, so do not assert on literal values.

### Use case: Generating random identifiers and filler data

Strings and bytes, for fixtures and scratch files.

```
use std::iter::repeat_with;

// A random alphanumeric name, the way tempfile builds one.
let name: String = repeat_with(fastrand::alphanumeric).take(12).collect();
assert_eq!(name.len(), 12);
assert!(name.chars().all(|c| c.is_ascii_alphanumeric()));

// Bulk bytes in one call, rather than one at a time.
let mut buffer = [0u8; 32];
fastrand::fill(&mut buffer);
assert!(buffer.iter().any(|&b| b != 0));

// Hex digits, for a short id.
let id: String = repeat_with(|| fastrand::digit(16)).take(8).collect();
assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
```

**Why it fits:** `repeat_with` plus a character function is the whole idiom, and
`fill` covers bulk bytes without a loop. The warning bears repeating here
because this is exactly where it gets ignored: a name for a scratch file is a
fine use, and **a session identifier is not** — if guessing the value is a
security problem, this is the wrong generator regardless of how convenient the
call site is.

## API map

The free functions and the `Rng` methods are the same set, so the entries below
name both forms once.

### Numbers

#### Integer functions

One function per integer type, taking a range.

```
// The function name is the type; the argument is the range.
let anywhere = fastrand::i32(..);
let _ = anywhere;

assert!((1..=6).contains(&fastrand::u8(1..=6)));
assert!(fastrand::usize(..10) < 10);
assert!((-5..5).contains(&fastrand::i64(-5..5)));

// Every primitive integer type has one.
let _ = (fastrand::u16(..), fastrand::i128(..), fastrand::isize(..));
```

**When to use it:** for any bounded integer. The range accepts anything
implementing `RangeBounds`, so `..`, `..n`, `a..b` and `a..=b` all work, and `..`
means the type's full range. An empty range panics rather than returning
nonsense — `fastrand::u32(5..5)` is a bug, and it is reported as one.

#### Floats and booleans

The remaining scalar generators.

```
// Floats land in [0, 1).
let f = fastrand::f64();
assert!((0.0..1.0).contains(&f));

// Or [0, 1] if the endpoint matters.
assert!(fastrand::f64_inclusive() <= 1.0);

// f32 equivalents exist too.
let _ = (fastrand::f32(), fastrand::f32_inclusive());

// A fair coin.
let _ = fastrand::bool();
```

**When to use it:** `f64()` scaled by hand is how you get an arbitrary float
range, since there is no `f64(range)` form — multiply and add. The `_inclusive`
variants exist because whether 1.0 is reachable genuinely matters for some
sampling, and leaving it ambiguous would be worse. There is no weighted `bool`;
compare an `f64` against a probability instead.

### Collections

#### `shuffle`, `choice` and `choose_multiple`

Operating on sequences.

```
// Shuffle in place: a permutation, so the contents are unchanged.
let mut values = vec![1, 2, 3, 4, 5];
fastrand::shuffle(&mut values);
values.sort();
assert_eq!(values, vec![1, 2, 3, 4, 5]);

// One element, or None when there is nothing to choose from.
assert!(fastrand::choice([10, 20, 30]).is_some());
assert_eq!(fastrand::choice(Vec::<u8>::new()), None);

// Several distinct elements, from any iterator.
let picked = fastrand::choose_multiple(0..20, 5);
assert_eq!(picked.len(), 5);

// Asking for more than exists yields everything, not an error.
assert_eq!(fastrand::choose_multiple(0..3, 10).len(), 3);
```

**When to use it:** `shuffle` for randomising order — a deck, a test order, a
worklist. `choice` takes an `IntoIterator` and returns `Option`, which is the
honest signature for a possibly-empty input. `choose_multiple` is reservoir
sampling over an iterator, so it works on something you cannot index and does
not allocate the whole source; note it returns up to `amount`, silently, so
check the length if you need exactly that many.

#### `fill`

Bulk random bytes.

```
let mut buffer = [0u8; 64];
fastrand::fill(&mut buffer);

// One call rather than sixty-four.
assert!(buffer.iter().any(|&b| b != 0));
```

**When to use it:** filling a buffer with filler data, or generating many bytes
at once — it is markedly faster than calling `u8(..)` in a loop, since the
generator produces 64 bits at a time. The usual caveat is sharpest here: these
bytes are not suitable as key material or a nonce. Use `getrandom` when the
bytes must be unpredictable.

### Characters

#### `alphanumeric`, `digit` and the case functions

Single random characters.

```
assert!(fastrand::alphanumeric().is_ascii_alphanumeric());
assert!(fastrand::alphabetic().is_ascii_alphabetic());
assert!(fastrand::lowercase().is_ascii_lowercase());
assert!(fastrand::uppercase().is_ascii_uppercase());

// digit takes a base, so base 16 gives a hex digit.
assert!(fastrand::digit(16).is_ascii_hexdigit());
assert!(fastrand::digit(10).is_ascii_digit());

// Or a range of chars directly.
let letter = fastrand::char('a'..='z');
assert!(('a'..='z').contains(&letter));
```

**When to use it:** with `repeat_with(..).take(n).collect::<String>()`, which is
the idiom for a random string. These are all ASCII despite returning `char`;
`char(range)` is the one that will give you anything outside it, and a range
spanning surrogates or unassigned code points is your responsibility. `digit`
panics for a base above 36, there being no digits left to use.

### Generators

#### `Rng::new` and `Rng::with_seed`

A local generator instead of the thread-local one.

```
// Seeded from system entropy: two instances differ.
let mut a = fastrand::Rng::new();
let mut b = fastrand::Rng::new();
assert_ne!(a.u64(..), b.u64(..));

// Seeded explicitly: reproducible, and the seed is readable back.
let mut fixed = fastrand::Rng::with_seed(42);
assert_eq!(fastrand::Rng::with_seed(42).get_seed(), 42);
assert_eq!(fixed.u64(..), fastrand::Rng::with_seed(42).u64(..));
```

**When to use it:** whenever you generate more than a handful of values, since
every free-function call does a thread-local lookup that a local `Rng` skips —
the crate's own documentation recommends this for bulk generation. Also whenever
determinism matters, so that seeding does not leak into other code on the
thread. `with_seed` is the only constructor available under `no_std`, there being
no entropy source to read.

#### `fork`

A child generator derived from a parent.

```
let mut parent = fastrand::Rng::with_seed(1);

// Independent stream, derived deterministically from the parent's state.
let mut child = parent.fork();
assert_ne!(child.u64(..), parent.u64(..));
```

**When to use it:** giving each worker, task or test case its own generator from
one seed — the whole tree is then reproducible from that single number, while
the streams stay independent. It is the right answer for parallel work, where
sharing one `Rng` would need a lock and seeding each thread from entropy would
lose reproducibility.

#### `fastrand::seed` and the global generator

Seeding the thread-local generator.

```
// Affects every subsequent free-function call on this thread.
fastrand::seed(12345);
let first = fastrand::u32(..);

fastrand::seed(12345);
assert_eq!(fastrand::u32(..), first);

// get_seed reads the current global seed back.
let _ = fastrand::get_seed();
```

**When to use it:** sparingly. It is genuinely useful in a small binary that
wants a reproducible run from a `--seed` flag, and it is a hazard in a test suite
or a library, because it reaches every other user of the global generator on
that thread. A library should never call it: take an `Rng` or a seed from the
caller instead. Remember too that the global generator is seeded from a
**fixed** value on `wasm32-unknown-unknown` without the `js` feature, so on that
target it is deterministic whether you asked for that or not.
