---
title: "Declarative macros (macro_rules!)"
area: "Macros & Metaprogramming"
embedded_support: full
groups: ["Macros & Metaprogramming", "Declarative / Metaprogramming", "Generating Code / Metaprogramming", "Macros & Code Generation"]
related_syntax: ["macro_rules!", "!", "$ident"]
see_also: ["Procedural macros", "Derive macros", "Attribute-like macros", "Function-like macros"]
---

## Explanation

A declarative macro is defined with `macro_rules!` and generates code by
matching the tokens of a macro call against a set of patterns and
substituting the matched pieces into a template — a compile-time
find-and-replace over syntax, not a function that runs at runtime.
Unlike the other four macro kinds on this wiki, `macro_rules!` is built
directly into the compiler: there's no separate crate, no special
`Cargo.toml` flag, and no token-stream-manipulating Rust code involved.

It exists because functions and generics only operate on values and
types *after* the language has already been parsed, and some repetition
can only be eliminated *before* that point — accepting a variable number
of arguments, generating several items (structs, impls, whole test
functions) from a single invocation, or building something like
`println!` or `vec!` that no ordinary function signature could express.
`macro_rules!` fills exactly that gap using pattern matching over token
trees, nothing more.

The mental model is a small match expression over syntax: each arm
pairs a pattern of metavariables (`$name:expr`, `$ty:ty`, `$($rest:tt),*`)
with a template, and invoking the macro — always with a trailing `!` —
matches the call's tokens against the arms in order, captures the
matched fragments, and substitutes them into the winning arm's template.
The compiler then parses that expansion exactly as if the programmer had
typed it directly. The `$(...)*`/`$(...)+` repetition syntax is how a
single arm accepts a variable-length list of arguments, which is what
makes macros like `vec!` work for any number of elements.

`macro_rules!` is one of five ways Rust generates code from other code;
the other four — [Procedural macros](procedural-macros.md),
[Derive macros](derive-macros.md),
[Attribute-like macros](attribute-like-macros.md), and
[Function-like macros](function-like-macros.md) — are all "procedural":
ordinary compiled Rust functions operating on `proc_macro::TokenStream`
that must live in their own crate. `macro_rules!` avoids both of those
costs (same-crate definition and use, no extra crate flag), and it's
hygienic by default — identifiers it introduces don't accidentally
collide with names at the call site — without the author writing any
code to make that happen. The tradeoff is the ceiling: pattern matching
over token shapes can't inspect a struct's actual field names, which is
exactly the limitation that pushes authors toward a derive macro instead.

## Basic usage example

```
macro_rules! max_of { // <- defines a declarative macro via pattern matching on token trees
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {
        std::cmp::max($x, max_of!($($rest),+))
    };
}

fn main() {
    let biggest = max_of!(3, 7, 2, 9, 4); // <- expands recursively into nested std::cmp::max calls
    println!("{biggest}");
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A crate with several domain ID newtypes (`OrderId`, `UserId`, `SensorId`)
needs the same `From<u64>` conversion on each one — a `macro_rules!`
macro turns three hand-written impls into one macro definition plus one
invocation.

```
struct OrderId(u64);
struct UserId(u64);
struct SensorId(u64);

macro_rules! impl_id_from_u64 { // <- one macro definition stands in for three hand-written impls
    ($($id:ident),+ $(,)?) => {
        $(
            impl From<u64> for $id {
                fn from(value: u64) -> Self {
                    $id(value)
                }
            }
        )+
    };
}

impl_id_from_u64!(OrderId, UserId, SensorId); // <- expands to three separate `impl From<u64>` blocks

let order: OrderId = 42.into();
```

**Why this way:** a family of near-identical trait impls is exactly the
boilerplate `macro_rules!` is meant to eliminate, and doing it with a
declarative macro keeps the definition and its uses in the same crate
with no extra dependency — [Effective Rust](https://effective-rust.com/)
recommends reaching for a macro once the same impl shape is copy-pasted
across more than a couple of types.

### Scenario: Testing

A parsing function for sensor temperature readings needs several
similar test cases; a `macro_rules!` macro turns each case into a
one-line invocation that expands into a full `#[test]` function.

```
fn parse_temperature(input: &str) -> Option<f64> {
    input.strip_suffix('C')?.trim().parse().ok()
}

macro_rules! temperature_test { // <- one macro call generates one #[test] function per case
    ($name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(parse_temperature($input), $expected);
        }
    };
}

temperature_test!(parses_whole_degree, "21C", Some(21.0));
temperature_test!(parses_with_space, "21 C", Some(21.0));
temperature_test!(rejects_missing_unit, "21", None);
```

**Why this way:** each generated function is still an ordinary `#[test]`
the harness discovers normally, so the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
conventions still apply — the macro only removes the copy-pasted
`#[test] fn ... { assert_eq!(...) }` scaffolding around each case, not
the tests themselves.

## Explanation (Embedded)

Everything about `macro_rules!` matching and expansion happens purely at
compile time, before the compiler has generated any code at all, so it
behaves identically in a `#![no_std]` binary and a hosted one — there is
no runtime component to a declarative macro on any target. What changes
in embedded code isn't the mechanism but how often reaching for it is the
right call: peripheral and register access is one of the most repetitive
corners of embedded Rust, with dozens of GPIO pins, UART instances, or
timer channels that each need a near-identical accessor function or
wrapper `impl`, differing only in a base address, bit position, or type
name. Hand-writing each one invites the exact class of bug a compiler
can't catch on its own — a copy-pasted accessor that reads the wrong
register because the address wasn't updated — while a single
`macro_rules!` definition expanded once per peripheral guarantees every
instance has the same shape by construction. This is precisely the
technique tools like `svd2rust` are built around: generating an entire
chip's register-access API from its hardware description by expanding
one macro-shaped code generator once per register, rather than by hand.

The token-level mechanics of writing such a macro — capturing names and
addresses as metavariables, repeating a template over a list of pins —
are covered on the syntax pages for [`$ident`](../../syntax/macros/metavariable.md),
[`$ident:kind`](../../syntax/macros/fragment-specifier.md), and
[`$(...)…`](../../syntax/macros/repetition.md). This page is about the
design decision sitting above those mechanics: reaching for `macro_rules!`
specifically, ahead of a proc-macro or hand-written code, whenever the
pattern is "many near-identical items differing only in a few
substituted tokens" — which describes an unusually large fraction of
register- and peripheral-facing embedded code.

## Basic usage example (Embedded)

```
macro_rules! gpio_output_pin { // <- one macro definition stands in for many hand-written pin wrappers
    ($name:ident, $bit:expr) => {
        struct $name;
        impl $name {
            fn set_high(&self, odr: &mut u32) {
                *odr |= 1 << $bit;
            }
        }
    };
}

gpio_output_pin!(Pa5, 5); // <- expands into a full `Pa5` type with its own `set_high`
gpio_output_pin!(Pa6, 6);
```

## Best practices & deeper information (Embedded)

### Scenario: Bit manipulation and flags

A driver needs a `set`/`clear` pair of bit-level accessors for every GPIO
pin on a port, each pin differing only in which bit of the output-data
register it touches.

```
macro_rules! gpio_bit { // <- generates one set/clear pair per pin, differing only in $bit
    ($set_fn:ident, $clear_fn:ident, $bit:expr) => {
        fn $set_fn(odr: &mut u32) {
            *odr |= 1 << $bit; // <- $bit is the only thing that changes between pins
        }
        fn $clear_fn(odr: &mut u32) {
            *odr &= !(1 << $bit);
        }
    };
}

gpio_bit!(set_pa5, clear_pa5, 5);
gpio_bit!(set_pa6, clear_pa6, 6);
```

**Why this way:** writing the mask/shift logic once and substituting only
the bit position keeps every pin's accessor byte-for-byte identical in
shape, which is what makes a copy-paste-introduced wrong bit position
impossible rather than merely unlikely — the same discipline the [Rust
Embedded Book's registers chapter](https://docs.rust-embedded.org/book/start/registers.html)
describes for register access, whether written by hand or generated.

### Scenario: Designing a public API

A small HAL exposes a consistent constructor-plus-accessor shape for
several peripheral instances (UART1, UART2, UART3) sitting at different
base addresses; the macro is the single source of truth for that shape.

```
macro_rules! uart_peripheral { // <- one definition; the API shape can't drift between instances
    ($ty:ident, $base:expr) => {
        pub struct $ty;
        impl $ty {
            pub fn new() -> Self { $ty }
            pub fn status(&self) -> u32 {
                unsafe { core::ptr::read_volatile(($base + 0x00) as *const u32) }
            }
        }
    };
}

uart_peripheral!(Uart1, 0x4001_3800);
uart_peripheral!(Uart2, 0x4000_4400);
```

**Why this way:** expanding every instance from one macro guarantees
`Uart1::new().status()` and `Uart2::new().status()` share the exact same
method surface, which matters more in a public HAL crate than in
application code — [Effective Rust](https://effective-rust.com/)
recommends a macro precisely when the alternative is several hand-written
impls that must be kept in lockstep.
