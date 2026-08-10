---
title: "rand"
version: "0.10.2"
publisher: "Diggory Hardy (dhardy), libs"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-09"
summary: "Random number generation: a thread-local generator that needs no setup, seedable generators for reproducible runs, and the distributions and sequence operations — shuffle, choose, weighted pick — built on top."
categories: ["algorithms", "randomness", "no-std"]
repository: "https://github.com/rust-random/rand"
---

## Overview

Rust's standard library has no random number generator, so `rand` is where
everyone goes. It covers three jobs that are easy to conflate:

- **Generating values** — `rand::random()` for a `u32`, a `bool`, an `f64`;
  `random_range(1..=6)` for a die roll.
- **Choosing a generator** — thread-local, seeded, fast-but-insecure, or
  straight from the operating system.
- **Sampling and shuffling** — distributions, weighted choice, `shuffle`,
  `choose`.

Two decisions decide whether your use of it is correct, and both are easy to get
wrong quietly.

**Is it a security decision?** `ThreadRng` (what `rand::random()` uses) is a
cryptographically secure generator seeded from the OS, so it is safe for tokens
and nonces. `SmallRng` is *not* — it is fast and predictable from its output, so
a session id built with it is guessable. The type system won't stop you: both
have the same methods. `CryptoRng` is the marker that separates them, and taking
`impl CryptoRng` in a function that generates secrets is how you make the
compiler check it. For key material specifically, a dedicated crypto crate is
still the better answer than assembling it from random bytes yourself.

**Do you need the same numbers next time?** A seeded `StdRng` reproduces a run
on the same machine and the same `rand` version — but `StdRng`'s algorithm is
explicitly allowed to change in a new release, so committing a test's expected
output against it will break on upgrade. When the values must be stable
forever — a golden test, a procedurally generated world, a published
simulation — name the algorithm: `ChaCha20Rng` or `Xoshiro256PlusPlus`, which
carry a value-stability guarantee.

Version 0.10 reshuffled the traits, so older examples won't compile:
[`Rng`](https://docs.rs/rand_core) is now the core trait from `rand_core`
(`next_u32`, `fill_bytes`), and `RngExt` carries the ergonomic methods
(`random`, `random_range`, `sample`). Importing `rand::prelude::*` gets both.
Earlier releases renamed a great deal too — `thread_rng()` became `rng()`,
`gen()` became `random()`, `gen_range()` became `random_range()` — so anything
written against 0.8 needs translating rather than skimming.

The default features pull in `getrandom` for OS entropy and `chacha20` for
`StdRng`. It builds `no_std` with `default-features = false`, where you supply
your own seed, and it requires Rust 1.85.

## When to use it

### Use case: Shuffling and dealing from a deck

Sequence operations live in extension traits rather than on the RNG, so the
call reads from the collection: `deck.shuffle(&mut rng)`, not
`rng.shuffle(&mut deck)`.

```
use rand::prelude::*;

let mut deck: Vec<u8> = (1..=52).collect();
let mut rng = rand::rng();

deck.shuffle(&mut rng);
let hand: Vec<u8> = deck.drain(..5).collect();

assert_eq!(hand.len(), 5);
assert_eq!(deck.len(), 47);

// Or pick without removing.
let cut = deck.choose(&mut rng);
assert!(cut.is_some());
```

**Why it fits:** `shuffle` is an in-place Fisher–Yates, and `choose` is one
indexed pick — both avoid the subtly biased `rng.random::<usize>() % len` that
hand-rolled versions reach for.

### Use case: A simulation you can re-run exactly

A bug that only appears one run in a thousand is untestable unless you can
replay that run. Seeding the generator from a value you record turns a flaky
failure into a fixed one.

```
use rand::prelude::*;
use rand::rngs::StdRng;

fn simulate(seed: u64) -> u32 {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..1000).map(|_| rng.random_range(1..=6)).sum()
}

// The same seed always gives the same run.
assert_eq!(simulate(42), simulate(42));

// A different seed almost certainly does not.
assert_ne!(simulate(42), simulate(43));
```

**Why it fits:** the seed is the whole reproduction case — log it on failure and
a colleague can replay the exact run. Note the limit: this holds for one `rand`
version, so a test asserting `simulate(42) == 3521` would be a hostage to your
next upgrade. Assert on properties, or pin `ChaCha20Rng`.

### Use case: Sampling the same distribution many times

Building a distribution once and reusing it moves the range setup out of the
loop, which matters when the loop is long.

```
use rand::distr::{Distribution, Uniform};

let mut rng = rand::rng();
let latency = Uniform::new(10u32, 250).unwrap(); // <- validated once

let samples: Vec<u32> = latency.sample_iter(&mut rng).take(1000).collect();

assert_eq!(samples.len(), 1000);
assert!(samples.iter().all(|&ms| (10..250).contains(&ms)));
```

**Why it fits:** `random_range` re-derives the sampling parameters on every call
and can't report a bad range except by panicking. `Uniform::new` returns a
`Result` you handle once, then samples cheaply forever after.

## API map

`rand::prelude::*` brings in the traits these entries need — `RngExt` for
`random`/`random_range`, the `seq` traits for `choose`/`shuffle`, and
`SeedableRng`. The methods are on traits, so without the import they simply
aren't found.

### Generating a value with no setup

#### `rand::random`

One random value of any type the standard distribution covers, from the
thread-local generator.

```
let flip: bool = rand::random();
let byte: u8 = rand::random();
let unit: f64 = rand::random(); // <- floats land in 0.0..1.0

assert!((0.0..1.0).contains(&unit));
let _ = (flip, byte);
```

**When to use it:** a one-off value where you don't hold a generator. In a loop,
bind `rand::rng()` once instead — each call otherwise re-fetches the
thread-local handle.

#### `rand::random_range`

A value from a range, uniformly and without modulo bias.

```
let die = rand::random_range(1..=6);
assert!((1..=6).contains(&die));

let index = rand::random_range(0..10usize);
assert!(index < 10);
```

**When to use it:** any bounded integer or float. It panics on an empty or
reversed range, which is a programming error rather than a runtime condition —
if the bounds come from input, validate them with `Uniform::new` instead.

#### `rand::rng`

A handle to the thread-local generator: seeded from the OS, periodically
reseeded, and cryptographically secure.

```
use rand::prelude::*;

let mut rng = rand::rng();
let a: u32 = rng.random();
let b = rng.random_range(0..100);

assert!(b < 100);
let _ = a;
```

**When to use it:** the default for anything not needing reproducibility. Bind
it once outside a loop. It is neither `Send` nor `Sync` — each thread gets its
own, which is what makes it fast.

### The generator traits

#### `RngExt::random`

The generic value method behind `rand::random`, on whichever generator you hold.

```
use rand::prelude::*;
use rand::rngs::SmallRng;

let mut rng = SmallRng::seed_from_u64(7);
let n: u64 = rng.random();
let pair: (u8, bool) = rng.random(); // <- tuples work too

let _ = (n, pair);
```

**When to use it:** wherever you have a generator. The type is chosen by
inference, so annotate the binding — `let n: u64` — rather than relying on a
default, because there isn't one.

#### `RngExt::random_bool`

A `bool` that is true with the given probability.

```
use rand::prelude::*;

let mut rng = rand::rng();
let mut heads = 0;
for _ in 0..1000 {
    if rng.random_bool(0.5) {
        heads += 1;
    }
}
assert!((300..700).contains(&heads)); // <- overwhelmingly likely
```

**When to use it:** modelling a chance of something happening — packet loss, a
retry, a drop table. `random_ratio(1, 6)` says the same thing in integers when
the odds are naturally a fraction.

#### `RngExt::fill`

Fills a slice with random values in one call.

```
use rand::prelude::*;

let mut key = [0u8; 32];
rand::rng().fill(&mut key);

assert_eq!(key.len(), 32);
assert!(key.iter().any(|&b| b != 0)); // <- practically certain
```

**When to use it:** generating a buffer of bytes — a nonce, a salt, a test
payload. Far faster than pushing values one at a time. For key material, prefer
a crypto crate's own key generation over rolling it from raw bytes.

#### `Rng` (the core trait)

The low-level trait from `rand_core` that every generator implements:
`next_u32`, `next_u64`, `fill_bytes`. `RngExt` is built on it.

```
use rand::Rng;

let mut rng = rand::rng();
let raw: u32 = rng.next_u32();
let wide: u64 = rng.next_u64();

let _ = (raw, wide);
```

**When to use it:** implementing a generator, or writing a function generic over
one — `fn shuffle<R: Rng>(rng: &mut R)`. In ordinary code prefer `RngExt`'s
methods, which handle ranges and types for you.

#### `CryptoRng`

The marker trait asserting a generator is cryptographically secure. It has no
methods; it exists to be a bound.

```
use rand::{CryptoRng, RngExt};

// Won't accept SmallRng: the compiler enforces the security decision.
fn new_token<R: CryptoRng + RngExt>(rng: &mut R) -> [u8; 16] {
    let mut token = [0u8; 16];
    rng.fill(&mut token);
    token
}

let token = new_token(&mut rand::rng());
assert_eq!(token.len(), 16);
```

**When to use it:** as a bound on every function that generates a secret. This
is the one mechanism that stops a fast generator being swapped in later by
someone who doesn't know what the bytes are for.

### Choosing a generator

#### `rngs::ThreadRng`

The thread-local generator returned by `rand::rng()`. Secure, automatically
seeded, and reseeded as it goes.

```
use rand::prelude::*;

fn roll(rng: &mut ThreadRng) -> u32 {
    rng.random_range(1..=20)
}

assert!((1..=20).contains(&roll(&mut rand::rng())));
```

**When to use it:** the default. Name the type only when storing or passing the
generator; otherwise `rand::rng()` at the call site reads better.

#### `rngs::StdRng`

A seedable cryptographically secure generator — currently ChaCha12.

```
use rand::prelude::*;
use rand::rngs::StdRng;

let mut rng = StdRng::seed_from_u64(2026);
let first: u32 = rng.random();

let mut again = StdRng::seed_from_u64(2026);
assert_eq!(first, again.random::<u32>()); // <- same seed, same stream
```

**When to use it:** when you need both reproducibility and security — seeded
tests of cryptographic code, or a server that must replay a session. Which
algorithm backs it may change between `rand` releases, so don't commit expected
values against it.

#### `rngs::SmallRng`

The fast, small, **non-cryptographic** generator.

```
use rand::prelude::*;
use rand::rngs::SmallRng;

let mut rng = SmallRng::seed_from_u64(1);
let noise: Vec<f32> = (0..8).map(|_| rng.random()).collect();

assert_eq!(noise.len(), 8);
```

**When to use it:** high-volume, non-secret randomness — particle effects,
procedural terrain, load generators, Monte Carlo runs. Never for tokens,
passwords, nonces or shuffles whose outcome someone could profit from
predicting; its state is recoverable from its output.

#### `rngs::SysRng`

Reads straight from the operating system's entropy source on every call, with no
userspace state. Asking the OS can fail, so it implements the *fallible*
`TryRng` rather than `Rng` — its methods return `Result`.

```
use rand::rand_core::{TryRng, UnwrapErr};
use rand::rngs::SysRng;
use rand::RngExt;

let mut seed = [0u8; 32];
SysRng.try_fill_bytes(&mut seed).expect("no OS entropy");

// UnwrapErr adapts it to the infallible Rng, panicking on failure,
// which is what lets it be used where an Rng is expected.
let mut rng = UnwrapErr(SysRng);
let n: u32 = rng.random();

assert_eq!(seed.len(), 32);
let _ = n;
```

**When to use it:** seeding another generator, or the rare case where you want no
cached state at all. It is markedly slower than `ThreadRng` — which is itself
OS-seeded — so it is the wrong default for bulk generation. The fallibility is
real on some targets: treat a failure as fatal rather than falling back to a
weaker source.

### Seeding

#### `SeedableRng::seed_from_u64`

Builds a generator from a single integer, spreading it across the full seed.

```
use rand::prelude::*;
use rand::rngs::StdRng;

let seed = 12345u64;
let mut rng = StdRng::seed_from_u64(seed);
let value: u16 = rng.random();

let _ = value; // reproducible for this seed and this rand version
```

**When to use it:** tests, simulations, and anywhere the seed is something you
log and replay. Not for security: a `u64` seed is guessable, so a secret
generated from one is only as strong as the number you picked.

#### `SeedableRng::from_seed`

Takes the generator's full seed type — 32 bytes for `StdRng`.

```
use rand::prelude::*;
use rand::rngs::StdRng;

let seed = [7u8; 32];
let mut rng = StdRng::from_seed(seed);
let value: u32 = rng.random();

let _ = value;
```

**When to use it:** when the seed comes from somewhere real — a key derivation,
a recorded fixture, a network peer — and you need every bit of it to count.

#### `SeedableRng::from_rng`

Seeds one generator from another.

```
use rand::prelude::*;
use rand::rngs::SmallRng;

// A fast per-task generator, seeded securely once.
let mut fast = SmallRng::from_rng(&mut rand::rng());
let n: u32 = fast.random();

let _ = n;
```

**When to use it:** giving each thread or task its own fast generator without
each one paying for OS entropy. The usual pattern is one secure seed, many cheap
generators.

### Sequences

#### `IndexedRandom::choose`

One uniformly chosen element, or `None` when empty.

```
use rand::prelude::*;

let colours = ["red", "amber", "green"];
let picked = colours.choose(&mut rand::rng()).unwrap();

assert!(colours.contains(picked));
assert!(Vec::<u8>::new().choose(&mut rand::rng()).is_none());
```

**When to use it:** picking from a slice, `Vec` or array. Use `choose_mut` when
you mean to modify what you picked, and `IteratorRandom::choose` when you only
have an iterator.

#### `IndexedRandom::choose_multiple`

Several distinct elements — sampling without replacement.

```
use rand::prelude::*;

let pool: Vec<u32> = (0..50).collect();
let sample: Vec<_> = pool.choose_multiple(&mut rand::rng(), 6).copied().collect();

assert_eq!(sample.len(), 6);
let mut sorted = sample.clone();
sorted.sort_unstable();
sorted.dedup();
assert_eq!(sorted.len(), 6); // <- no repeats
```

**When to use it:** lottery draws, test subsets, picking N of M. Calling
`choose` in a loop is the buggy version — it repeats. Order is not randomised;
`shuffle` the result if that matters.

#### `IndexedRandom::choose_weighted`

A pick where each element carries a weight.

```
use rand::prelude::*;

let loot = [("common", 80), ("rare", 15), ("legendary", 5)];
let drop = loot.choose_weighted(&mut rand::rng(), |item| item.1).unwrap();

assert!(loot.contains(drop));
```

**When to use it:** drop tables, weighted routing, anything where outcomes
aren't equally likely. It returns `Result` — zero total weight and negative
weights are errors, not panics. Build a `WeightedIndex` instead when you'll
sample the same weights repeatedly.

#### `SliceRandom::shuffle`

Shuffles in place, uniformly.

```
use rand::prelude::*;

let mut order: Vec<u32> = (0..10).collect();
order.shuffle(&mut rand::rng());

assert_eq!(order.len(), 10);
let mut sorted = order.clone();
sorted.sort_unstable();
assert_eq!(sorted, (0..10).collect::<Vec<_>>()); // <- same elements
```

**When to use it:** randomising order — a playlist, test execution, a deck.
`partial_shuffle` is the one to use when you only need the first few and the
collection is large.

#### `IteratorRandom::choose`

Picks from an iterator in one pass, without collecting it first.

```
use rand::prelude::*;

let evens = (0..1000).filter(|n| n % 2 == 0);
let picked = evens.choose(&mut rand::rng()).unwrap();

assert_eq!(picked % 2, 0);
```

**When to use it:** when the sequence is lazy, filtered, or too big to hold —
reservoir sampling, so memory stays constant. If you already have a slice,
`IndexedRandom::choose` is cheaper because it can index directly.

### Distributions

#### `Uniform`

A reusable, pre-validated uniform range.

```
use rand::distr::Uniform;
use rand::prelude::*;

let mut rng = rand::rng();
let percent = Uniform::new_inclusive(0u8, 100).unwrap();

let readings: Vec<u8> = (0..5).map(|_| percent.sample(&mut rng)).collect();
assert!(readings.iter().all(|&p| p <= 100));
```

**When to use it:** sampling one range many times, or when the bounds are
computed and might be invalid — `new` returns `Result` where `random_range`
panics. `new_inclusive` includes the upper bound.

#### `Distribution::sample_iter`

Turns a distribution into an iterator of samples.

```
use rand::distr::{Distribution, Uniform};

let rolls: Vec<u8> = Uniform::new_inclusive(1u8, 6)
    .unwrap()
    .sample_iter(rand::rng())
    .take(10)
    .collect();

assert_eq!(rolls.len(), 10);
assert!(rolls.iter().all(|&r| (1..=6).contains(&r)));
```

**When to use it:** generating a batch, where it composes with the rest of the
iterator toolkit — `take`, `filter`, `zip`. It takes the generator by value, so
pass `&mut rng` to keep using it afterwards.

#### `Alphanumeric`

Samples ASCII letters and digits, with a helper for building a `String`.

```
use rand::distr::{Alphanumeric, SampleString};

let id = Alphanumeric.sample_string(&mut rand::rng(), 12);

assert_eq!(id.len(), 12);
assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
```

**When to use it:** human-readable identifiers — test fixtures, temporary
filenames, request ids. For anything that must be unguessable, generate the
bytes with a `CryptoRng` and encode them, and prefer a UUID crate when you want
a standard format.

#### `Bernoulli`

A reusable weighted coin.

```
use rand::distr::{Bernoulli, Distribution};

let mut rng = rand::rng();
let lossy = Bernoulli::new(0.1).unwrap(); // <- 10% of the time

let dropped = (0..100).filter(|_| lossy.sample(&mut rng)).count();
assert!(dropped < 40); // <- overwhelmingly likely
```

**When to use it:** a fixed probability sampled repeatedly — simulated packet
loss, fault injection. `random_bool` is simpler for a one-off; `Bernoulli`
validates the probability once and returns `Result` rather than panicking.