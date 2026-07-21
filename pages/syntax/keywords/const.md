---
title: "const"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Const generics, "Zero-cost abstractions"]
related_syntax: [static, let]
see_also: [static]
---

## Explanation

`const` declares a compile-time constant, as in
`const MAX_POINTS: u32 = 100_000;`.

A `const` must have an explicit type annotation (unlike `let`, type
inference alone is not enough) and its value must be computable entirely
at compile time — the initializer runs through Rust's const evaluator, not
at runtime. There is no fixed memory address for a `const`: every place it
is used, the compiler is free to inline its value directly, the same way
it would inline a literal.

`const` items can be declared at module scope, inside a function body,
inside a `trait`/`impl` block (an *associated const*), and inside a
`struct`/`enum` definition's generic parameter list — an entirely
different use, introducing a **const generic** parameter
(`struct Buffer<const N: usize>`), which parameterizes a type by a value
rather than by another type.

Naming convention is `SCREAMING_SNAKE_CASE`. `const` bindings are always
implicitly immutable — `const mut` does not exist.

## Usage examples

### Declaring a compile-time constant

```
const MAX_POINTS: u32 = 100_000; // <- `const` declares a compile-time constant
```

### Numeric computation

A buffer size that's a fixed multiple of a record size is exactly the
kind of arithmetic `const` is for — it's computed once, at compile time,
instead of being recomputed (or hand-typed as a literal) wherever it's
needed.

```
const MAX_RETRY_COUNT: u8 = 5;
const BUFFER_CAPACITY: usize = 64 * 4; // <- `const` requires a value computable entirely at compile time

fn should_retry(attempt: u8) -> bool {
    attempt < MAX_RETRY_COUNT
}
```

The initializer expression runs through the compiler's
const evaluator rather than at runtime, so `BUFFER_CAPACITY` costs nothing
beyond the final value — the
[Reference's constant items](https://doc.rust-lang.org/reference/items/constant-items.html)
page confirms the initializer must itself be a const expression, which is
exactly what makes this guarantee possible.

### Designing a public API

A library that wants callers to know its default timeout, and to be able
to reference it by name instead of a magic number, exposes the value as a
public `const`.

```
/// Default timeout applied when a caller doesn't specify one.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30; // <- `const`, public, SCREAMING_SNAKE_CASE per convention

pub fn connect_with_default_timeout() {
    connect(DEFAULT_TIMEOUT_SECS)
}

fn connect(_timeout_secs: u64) {}
```

`SCREAMING_SNAKE_CASE` for constants is codified in the
[API Guidelines' casing conventions](https://rust-lang.github.io/api-guidelines/naming.html#casing-conforms-to-rfc-430-c-case),
and publishing the constant lets downstream code refer to
`DEFAULT_TIMEOUT_SECS` by name instead of duplicating the literal `30` at
every call site.

## Explanation (Embedded)

`const` means exactly the same thing under `#![no_std]` as on a hosted
target — a value computed entirely at compile time and inlined at every
use site, with no fixed memory address of its own. This is, if
anything, more consequential in embedded code than in hosted code,
because so much of a firmware crate's numeric vocabulary — peripheral
register addresses, bitmasks for individual fields inside a register,
fixed buffer/queue capacities, timing constants derived from a clock
frequency — is known completely at compile time and never needs the
indirection of a runtime-computed value or the storage a `static` sets
aside.

The distinction from [`static`](static.md) matters more here than almost
anywhere else, because the two are reached for in genuinely different
embedded situations, not just as a style preference. A `const` has no
address: `const GPIOA_BASE: u32 = 0x4001_0800;` is inlined as the literal
`0x4001_0800` everywhere it's written, the same as if the number had
been typed by hand at each call site — exactly right for a register
address, since nothing about "the address of GPIOA" needs to be an
addressable object in memory; the addressed object is the *peripheral*,
not the constant naming it. A `static`, by contrast, is the right tool
the moment something needs its own fixed storage that more than one part
of the program points at the *same* location of — a lookup table shared
by every caller, or state an interrupt handler and `main` both need to
reach through one shared address. For a raw scalar like an address or a
bitmask, `const` is the strictly correct choice: no embedded codebase
gains anything by giving `0x4001_0800` its own RAM/flash address.

One genuine embedded-specific wrinkle: a `const` whose value is an
*aggregate* (an array, a struct) is inlined at *every* use site, not
shared — referencing the same array-valued `const` from several
different functions can genuinely materialize several separate copies of
that array in the compiled binary, one per call site, unlike a `static`
array, which is placed once. For a small address or bitmask this never
matters; for a sizable lookup table it's exactly why the CRC-table
example on the [`static`](static.md) page uses `static`, not `const`,
despite the table itself being computed at compile time by a `const fn`.

## Usage examples (Embedded)

### Defining register addresses and bit-field masks as compile-time constants

```
const GPIOA_BASE: u32 = 0x4001_0800; // <- `const`: inlined at every use site, no storage of its own
const ODR_OFFSET: u32 = 0x14;
const PIN5_MASK: u32 = 1 << 5;

fn set_pa5() {
    let odr = (GPIOA_BASE + ODR_OFFSET) as *mut u32;
    unsafe { odr.write_volatile(odr.read_volatile() | PIN5_MASK) } // <- both consts inlined directly here
}
```

### Sizing a fixed sample buffer at compile time

```
const SAMPLE_RATE_HZ: u32 = 8_000;
const WINDOW_MS: u32 = 100;
const BUFFER_LEN: usize = (SAMPLE_RATE_HZ * WINDOW_MS / 1000) as usize; // <- `const`: computed entirely at compile time

fn window_sum(samples: &[u16; BUFFER_LEN]) -> u32 {
    samples.iter().map(|&s| s as u32).sum()
}
```
