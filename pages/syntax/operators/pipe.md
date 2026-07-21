---
title: "|"
kind: operator
embedded_support: full
groups: [Bitwise, Basics, "Control Flow & Pattern Matching", "Functions & Closures"]
related_concepts: [Operator overloading, "Closures & capturing", "match expressions"]
related_syntax: ["||", "&", "|="]
see_also: ["||"]
---

## Explanation

`|` has three unrelated meanings depending on context:

1. **Binary: bitwise OR** between integers, overloadable via
   `std::ops::BitOr`: `a | b` (`BitOr` is also implemented for `bool`,
   making `a | b` a non-short-circuiting logical OR).
2. **Pattern alternatives:** `Some(1) | Some(2) => ...` inside a `match`
   arm (or any refutable pattern position) — "matches if any of these
   patterns match." Entirely unrelated to bitwise OR; no `BitOr` impl is
   involved.
3. **Closure parameter delimiters:** `|x, y| x + y` — opens and closes
   the closure's parameter list, standing in for the parentheses a
   function's parameter list would use.

The empty-parameter-list closure form uses `||` (its own token — see
[`||`](pipe-pipe.md)) rather than `| |` with a space; the two are not
interchangeable in the grammar.

## Usage examples

### Bitwise OR between two integers

```
let mask = 0b0100 | 0b0001; // <- `|` bitwise OR between two integers
```

### Bit manipulation and flags

Combining several flag bits into one value to pass around or compare
against is the bitwise-OR use of `|` — distinct from `|=`, which mutates
an existing binding rather than producing a new combined value.

```
const FLAG_READY: u8   = 0b0000_0001;
const FLAG_LOGGING: u8 = 0b0000_0100;

fn startup_flags(verbose: bool) -> u8 {
    if verbose {
        FLAG_READY | FLAG_LOGGING // <- `|` combines two bits into one value
    } else {
        FLAG_READY
    }
}

assert_eq!(startup_flags(true), 0b0000_0101);
```

Building the combined value with `|` and returning it
(rather than mutating a `mut` accumulator with `|=`) fits a function
that hands back a fresh flag set rather than modifying state in place —
see [`|=`](pipe-equals.md) for the in-place variant of the same bitwise
operation.

### Branching on data (pattern matching)

Inside a `match` arm, `|` separates alternative patterns that should all
take the same branch — a different meaning of the same token from
bitwise OR, resolved entirely by position (pattern position vs.
expression position).

```
enum HttpStatus {
    Ok,
    Created,
    NotFound,
    ServerError(u16),
}

fn is_client_error(status: &HttpStatus) -> bool {
    match status {
        HttpStatus::NotFound => true,
        HttpStatus::Ok | HttpStatus::Created => false, // <- `|`: matches either variant
        HttpStatus::ServerError(_) => false,
    }
}
```

An or-pattern collapses what would otherwise be two
near-identical match arms with the same body, which the
[Rust Reference](https://doc.rust-lang.org/reference/patterns.html#or-patterns)
documents as a first-class pattern form rather than sugar layered on top
of separate arms.

## Explanation (Embedded)

All three meanings of `|` are common in embedded code, though the mix
differs from hosted code's. Bitwise OR is used to assemble a register's
new value from several independent configuration bits *before* a
single write — combining bits in a local variable first and writing
the peripheral once, rather than once per bit, avoids extra volatile
writes to memory-mapped I/O, which matters whenever a peripheral's
write side-effects (triggering a state-machine transition, say)
shouldn't happen mid-configuration. Pattern alternatives are just as
common as in hosted code, for collapsing several related interrupt or
fault variants into a single `match` arm. The closure-delimiter meaning
has a distinctly embedded home too: HAL crates built on `cortex_m`
commonly expose a critical-section function that takes a closure with
one parameter — `cortex_m::interrupt::free(|cs| { ... })` — so `|cs|`
opening that closure's parameter list is boilerplate seen in almost any
interrupt-safe embedded driver.

## Usage examples (Embedded)

### Assembling a peripheral control register from several config bits

```
const CR1_UE: u32 = 1 << 13; // USART enable
const CR1_TE: u32 = 1 << 3;  // transmitter enable
const CR1_RE: u32 = 1 << 2;  // receiver enable

fn usart_cr1_value() -> u32 {
    CR1_UE | CR1_TE | CR1_RE // <- `|` combines three independent config bits into one write
}
```

### Matching several fault codes in one arm

```
enum FaultCode {
    Overcurrent,
    Overtemperature,
    UnderVoltage,
    Ok,
}

fn is_critical(fault: &FaultCode) -> bool {
    matches!(fault, FaultCode::Overcurrent | FaultCode::Overtemperature) // <- `|`: either variant is critical
}
```

### Opening a critical-section closure's parameter list

```
struct CriticalSection;

fn interrupt_free<F: FnOnce(&CriticalSection)>(f: F) {
    // in real firmware this disables interrupts, runs `f`, then restores them
    let cs = CriticalSection;
    f(&cs);
}

static mut SHARED_COUNTER: u32 = 0;

fn increment_shared_counter() {
    interrupt_free(|_cs| { // <- `|_cs|` opens this closure's one-parameter list
        unsafe { SHARED_COUNTER += 1; }
    });
}
```
