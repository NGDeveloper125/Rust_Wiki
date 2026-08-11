---
title: "bitflags"
version: "2.13.1"
publisher: "Ashley Mannix (KodrAus), libs"
no_std: "yes"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-09"
summary: "Turns a set of named bits into a real type. The `bitflags!` macro generates a struct with typed set operations, so a permissions mask stops being a bare `u32` that anything can be assigned to."
categories: ["data-structures", "macros", "no-std"]
repository: "https://github.com/bitflags/bitflags"
---

## Overview

A flags value is a single integer where each bit means something: bit 0 is
readable, bit 1 is writable, bit 2 is executable. You can write that with
constants and `|`, and people do — but then the type is `u32`, and a `u32`
accepts a file descriptor, a length, or another library's flags without
complaint.

`bitflags!` generates a struct instead. The bits are still a `u32` underneath
and the operations still compile to `|`, `&` and `^`, but the type is now
`Permissions`, its set operations are named, and `Debug` prints
`READ | WRITE` rather than `3`.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Permissions: u32 {
        const READ = 0b001;
        const WRITE = 0b010;
        const EXEC = 0b100;
    }
}

let p = Permissions::READ | Permissions::WRITE;
assert!(p.contains(Permissions::READ));
assert_eq!(format!("{p:?}"), "Permissions(READ | WRITE)");
```

Two things to know before you write your first one, both of which bite people
coming from version 1:

- **The generated type is a struct, not an enum.** `Permissions::READ` is an
  associated constant. That is what allows a value to hold several flags at
  once, and it means you can't `match` on it the way you would an enum.
- **You choose the derives.** Version 2 stopped deriving anything implicitly, so
  a type with no `#[derive]` inside the macro has no `Debug`, no `Copy` and no
  `PartialEq`. This is deliberate — it lets you derive `serde` or `bytemuck`
  traits too — but the first compile error is usually a missing `Copy`.

The other thing worth deciding early is what you want to happen to **bits you
didn't name**. `from_bits` rejects them, `from_bits_truncate` drops them, and
`from_bits_retain` keeps them. For data arriving from a file, a syscall or the
network, that choice is a small piece of your validation policy, and the crate
deliberately refuses to pick for you.

It is a tiny, mature dependency: no runtime dependencies, `#![no_std]`, MSRV
1.56, and the operations are `const fn` so flags can be combined in constants.
The alternatives are worth knowing but rarely win — `enumflags2` enforces at
compile time that each variant is a single bit, at the cost of a proc-macro
dependency, and hand-rolled constants give up the type and the `Debug`
formatting for nothing saved.

## When to use it

### Use case: Options crossing an API boundary

A function taking `u32` of options invites the caller to pass the wrong `u32`.
Making it a flags type means the wrong value doesn't compile, and the set is
documented by its own definition.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OpenFlags: u8 {
        const READ   = 1 << 0;
        const WRITE  = 1 << 1;
        const CREATE = 1 << 2;
        const APPEND = 1 << 3;
    }
}

fn open(path: &str, flags: OpenFlags) -> String {
    let mut how = String::from(path);
    if flags.contains(OpenFlags::WRITE | OpenFlags::APPEND) {
        how.push_str(" (appending)");
    }
    how
}

let opened = open("log.txt", OpenFlags::WRITE | OpenFlags::APPEND);
assert_eq!(opened, "log.txt (appending)");
```

**Why it fits:** `open("log.txt", 3)` no longer compiles, and neither does
passing a different library's flags that happen to be `u8`. The combination is
still one register-wide value at runtime.

### Use case: Reading a flags field out of a binary format

A header byte from a file or a device is exactly a flags value, and it may
contain bits your version doesn't know about. Which of the three constructors
you use *is* the compatibility decision.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Header: u8 {
        const COMPRESSED = 0b0000_0001;
        const ENCRYPTED  = 0b0000_0010;
        const SIGNED     = 0b0000_0100;
    }
}

let byte = 0b0000_1011; // COMPRESSED | ENCRYPTED, plus an unknown bit 3

// Strict: refuse anything we don't understand.
assert_eq!(Header::from_bits(byte), None);

// Lenient: ignore what we don't understand.
let seen = Header::from_bits_truncate(byte);
assert!(seen.contains(Header::COMPRESSED | Header::ENCRYPTED));

// Faithful: keep the unknown bit so it survives a round trip.
let kept = Header::from_bits_retain(byte);
assert_eq!(kept.bits(), byte);
```

**Why it fits:** the three functions make an implicit decision explicit. A
strict reader rejects a file from a newer writer; a truncating one silently
discards information; a retaining one can write the file back unchanged.

### Use case: Accumulating state as a program runs

Flags are a compact way to track which of several independent things have
happened, with set operations reading better than a pile of booleans.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Checks: u8 {
        const PARSED    = 1 << 0;
        const VALIDATED = 1 << 1;
        const RESOLVED  = 1 << 2;

        // A named combination, built from the others.
        const READY = Self::PARSED.bits() | Self::VALIDATED.bits() | Self::RESOLVED.bits();
    }
}

let mut done = Checks::empty();
done.insert(Checks::PARSED);
done.insert(Checks::VALIDATED);

assert!(!done.contains(Checks::READY));
let missing = Checks::READY.difference(done);
assert_eq!(missing, Checks::RESOLVED); // <- exactly what is left to do
```

**Why it fits:** `difference` answers "what is still outstanding" in one
operation, where separate booleans would need enumerating by hand. The `READY`
constant shows the composition idiom — inside the macro you combine with
`.bits()`, because the constants are being defined as you go.

## API map

The `bitflags!` macro generates inherent methods on your type, so the entries
below are called as `MyFlags::empty()` and `value.contains(..)` with no import
beyond the macro. A few extras live on the `Flags` trait, which has to be
imported to be used; those say so.

### Defining a type

#### `bitflags!`

The macro. It takes a struct definition whose "fields" are the named bits, and
generates the type, its constants and its operations.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Style: u16 {
        const BOLD      = 1 << 0;
        const ITALIC    = 1 << 1;
        const UNDERLINE = 1 << 2;
    }
}

assert_eq!(Style::BOLD.bits(), 1);
```

**When to use it:** any time a set of independent on/off options travels
together. Write the derives yourself — `Debug, Clone, Copy, PartialEq, Eq` is
the usual set, and without `Copy` every use moves the value. Several types can
be declared in one macro invocation.

### Constructing values

#### `empty` and `all`

The two extremes: no bits set, and every *named* bit set.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 1;
        const B = 2;
    }
}

assert_eq!(F::empty().bits(), 0);
assert_eq!(F::all().bits(), 0b11);
assert!(F::empty().is_empty());
assert!(F::all().is_all());
```

**When to use it:** `empty()` as the starting point for accumulation and as a
`Default`-like value; `all()` for a permissive default or to mask off unknown
bits. Both are `const`, so they work in constant definitions.

#### `from_bits`

Builds a value from a raw integer, returning `None` if any bit isn't named.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 0b01;
        const B = 0b10;
    }
}

assert_eq!(F::from_bits(0b11), Some(F::A | F::B));
assert_eq!(F::from_bits(0b100), None); // <- bit 2 is not defined
```

**When to use it:** validating input where an unknown bit means the data is
wrong or from an incompatible version, and you'd rather fail than guess. The
strictest of the three constructors, and the right default for anything
untrusted.

#### `from_bits_truncate`

Keeps the bits it knows and discards the rest.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 0b01;
        const B = 0b10;
    }
}

let value = F::from_bits_truncate(0b111);
assert_eq!(value, F::A | F::B); // <- bit 2 dropped
```

**When to use it:** forward compatibility, where a newer writer may set bits
this build doesn't know and ignoring them is correct. Be deliberate: the
discarded bit may have meant "this record is encrypted".

#### `from_bits_retain`

Keeps every bit, named or not.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 0b01;
    }
}

let value = F::from_bits_retain(0b101);
assert_eq!(value.bits(), 0b101);
assert!(value.contains(F::A));
```

**When to use it:** round-tripping data you must not damage — reading a header,
modifying one flag, writing it back. The value then holds bits with no name, so
`all()` and `is_all()` stop meaning what you'd assume; `Flags::truncate` drops
them again when you're done.

#### `from_name`

Looks a flag up by the name it was declared with.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const READ = 1;
        const WRITE = 2;
    }
}

assert_eq!(F::from_name("WRITE"), Some(F::WRITE));
assert_eq!(F::from_name("write"), None); // <- exact, case-sensitive
```

**When to use it:** mapping configuration or CLI strings onto flags. It matches
one name only; for a whole `"READ | WRITE"` string use the `FromStr` impl.

### Testing and reading

#### `contains`

True when *every* bit of the argument is present.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 1;
        const B = 2;
        const C = 4;
    }
}

let value = F::A | F::B;
assert!(value.contains(F::A));
assert!(value.contains(F::A | F::B)); // <- all of them
assert!(!value.contains(F::A | F::C));
```

**When to use it:** the standard "is this option on" test, and for requiring a
combination in one call. Note it is *all*, not *any* — that's `intersects`, and
mixing them up is the most common bug with this crate.

#### `intersects`

True when *any* bit overlaps.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 1;
        const B = 2;
        const C = 4;
    }
}

let value = F::A;
assert!(value.intersects(F::A | F::C)); // <- A is enough
assert!(!value.intersects(F::B | F::C));
```

**When to use it:** "does this have any of these" — checking whether a change
touched any watched field, or whether a request needs any privileged
permission.

#### `bits`

The raw integer, for handing to a syscall, writing to a file, or comparing.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u32 {
        const A = 0b01;
        const B = 0b10;
    }
}

let raw: u32 = (F::A | F::B).bits();
assert_eq!(raw, 3);
```

**When to use it:** at the boundary where the typed value has to become a
number again — FFI, serialisation, a wire format. Inside your own code, keep
the typed value; converting early throws away everything the crate bought you.

#### `iter` and `iter_names`

Walk the individual flags in a value; `iter_names` yields the name alongside.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const READ = 1;
        const WRITE = 2;
        const EXEC = 4;
    }
}

let value = F::READ | F::EXEC;
let names: Vec<&str> = value.iter_names().map(|(name, _)| name).collect();

assert_eq!(names, ["READ", "EXEC"]);
assert_eq!(value.iter().count(), 2);
```

**When to use it:** logging, error messages and building human-readable output —
"missing permissions: WRITE". `iter` yields each set flag as a single-bit value
of the same type, so it composes with the rest of the iterator toolkit.

### Combining and modifying

#### `union` and `|`

Every bit from either side. The operator and the method are the same thing.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 1;
        const B = 2;
    }
}

assert_eq!(F::A | F::B, F::A.union(F::B));
```

**When to use it:** the operator in ordinary code; the method when you need
`const` in a context where operators aren't allowed, or when chaining reads
better.

#### `intersection` and `difference`

The bits in both, and the bits in the first but not the second.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 1;
        const B = 2;
        const C = 4;
    }
}

let granted = F::A | F::B;
let required = F::B | F::C;

assert_eq!(granted.intersection(required), F::B);
assert_eq!(required.difference(granted), F::C); // <- what's missing
```

**When to use it:** `difference` is the one that earns its keep — "which
required permissions are absent" in a single operation, ready to print with
`iter_names`. `symmetric_difference` (`^`) gives what changed between two
states.

#### `insert`, `remove` and `toggle`

In-place mutation of a flags value.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 1;
        const B = 2;
    }
}

let mut value = F::empty();
value.insert(F::A);
value.toggle(F::B);
assert_eq!(value, F::A | F::B);

value.remove(F::A);
assert_eq!(value, F::B);
```

**When to use it:** building a value up over several steps. `set(flag, bool)`
is the form to reach for when a condition decides it —
`value.set(F::A, config.enabled)` beats an `if`/`else` around insert and remove.

#### `complement` and `!`

Every named bit that isn't set.

```
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 1;
        const B = 2;
    }
}

assert_eq!(!F::A, F::B);
assert_eq!(F::A.complement(), F::B);
```

**When to use it:** inverting a mask. It only covers *named* bits — the result
never contains an undefined bit — which is why `!F::A` is `F::B` here and not
`0b1111_1110`.

### Text and traits

#### `parser::to_writer` and `parser::from_str`

Text conversion, as `READ | WRITE`. Note that the macro does **not** implement
`Display` or `FromStr` on your type — these free functions are the API, and you
write the trait impls yourself if you want them.

```
use bitflags::{bitflags, parser};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const READ = 1;
        const WRITE = 2;
    }
}

let value = F::READ | F::WRITE;

let mut text = String::new();
parser::to_writer(&value, &mut text).unwrap();
assert_eq!(text, "READ | WRITE");

let parsed: F = parser::from_str("READ | WRITE").unwrap();
assert_eq!(parsed, value);

// Unknown bits survive as hex rather than being silently lost.
let mut odd = String::new();
parser::to_writer(&F::from_bits_retain(0b101), &mut odd).unwrap();
assert_eq!(odd, "READ | 0x4");
```

**When to use it:** configuration files, CLI arguments and logs, where a name
beats a number. Wrap them in your own `Display` and `FromStr` impls if the type
is part of your public API — the crate leaves that to you deliberately, so it
doesn't dictate the text format of your type. `from_str_strict` and
`to_writer_strict` are the variants that refuse unknown bits.

#### The `Flags` trait

The trait behind the generated methods. Importing it adds a few operations the
inherent methods don't expose, and lets you write code generic over any flags
type.

```
use bitflags::{bitflags, Flags};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct F: u8 {
        const A = 1;
        const B = 2;
    }
}

// Generic over any flags type, not just this one.
fn describe<T: Flags>(value: &T) -> Vec<&'static str> {
    value.iter_names().map(|(name, _)| name).collect()
}

assert_eq!(describe(&(F::A | F::B)), ["A", "B"]);

// Trait-only: drop any bits that aren't named.
let mut loose = F::from_bits_retain(0b111);
loose.truncate();
assert_eq!(loose, F::A | F::B);
```

**When to use it:** writing a helper that works across several flags types, and
for `truncate` / `clear`, which exist only here. In ordinary single-type code the
inherent methods are enough and need no import.