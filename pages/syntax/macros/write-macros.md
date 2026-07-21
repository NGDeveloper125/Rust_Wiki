---
title: "write! / writeln!"
kind: macro
embedded_support: partial
groups: ["Output & Formatting", "Macros & Metaprogramming"]
related_concepts: ["String formatting (Display, Debug, format!)"]
related_syntax: ["format!", "println! / print! / eprintln! / eprint!"]
see_also: ["format!", "String formatting (Display, Debug, format!)"]
---

## Explanation

`write!`/`writeln!` use the exact same format-string grammar as
[`format!`](format-macro.md), but instead of allocating and returning a
fresh `String`, they write the formatted text into a destination the
caller already has — the first argument to either macro is that
destination, followed by the format string and its arguments:
`write!(destination, "...", args...)`. `writeln!` additionally appends a
trailing newline, the same relationship `println!` has to `print!`.

The destination can implement either of two different, similarly-named
but distinct `Write` traits, and which one it is changes what "writing"
actually means: `std::fmt::Write` (also available as `core::fmt::Write`,
needing no allocator) is implemented by `String`, and it's what a custom
[`Display`](../../concepts/collections-strings/string-formatting.md)
impl's `fmt` method writes into via its `&mut Formatter` argument;
`std::io::Write` is implemented by files, TCP sockets, and locked
standard output/error, and represents writing bytes somewhere an actual
I/O operation might fail.

That difference shows up directly in the return type: both macros return
a `Result`, but for an `fmt::Write` target the error case (`fmt::Error`)
is essentially unreachable in ordinary code — the operation only fails if
the underlying `Write` impl itself reports failure, which `String`'s
never does — so it's common, and generally fine, to `.unwrap()` it. For
an `io::Write` target, though, the `Result` represents a real I/O
failure (a full disk, a broken pipe, a closed socket), and production
code should propagate it with `?` rather than unwrap, exactly the way any
other fallible I/O call is handled.

## Basic usage example

```
use std::fmt::Write as _;

fn build_summary(item_count: u32, total: f64) -> String {
    let mut summary = String::new();
    write!(summary, "{item_count} items, ${total:.2} total").unwrap(); // <- fmt::Write for String never actually fails
    summary
}

let summary = build_summary(3, 47.5);
```

## Best practices & deeper information

### Scenario: Working with text

A custom `Display` impl for a duration-like type formats its value piece
by piece using `write!`, avoiding an intermediate allocation just to
combine the pieces.

```
use std::fmt;

struct Elapsed { seconds: u64 }

impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let minutes = self.seconds / 60;
        let remaining = self.seconds % 60;
        write!(f, "{minutes}m {remaining:02}s") // <- writes straight into the Formatter, no intermediate String
    }
}

let elapsed = Elapsed { seconds: 125 };
println!("{elapsed}"); // 2m 05s
```

**Why this way:** the
[std fmt docs](https://doc.rust-lang.org/std/fmt/trait.Display.html)
show writing directly into the `&mut Formatter` that `Display::fmt` is
handed as the idiomatic implementation shape; building an intermediate
`String` with `format!` just to immediately write it out again is an
unnecessary allocation on every call.

### Scenario: Working with text

Appending log lines to an open file propagates the `io::Write` `Result`
with `?` rather than unwrapping it, since the destination is a real OS
resource whose writes can genuinely fail.

```
use std::fs::File;
use std::io::{self, Write};

fn log_reading(file: &mut File, sensor_id: u32, value: f64) -> io::Result<()> {
    writeln!(file, "sensor {sensor_id}: {value:.2}")?; // <- io::Write target: a real I/O error must be propagated, not unwrapped
    Ok(())
}
```

**Why this way:** an `io::Write` destination can genuinely fail (disk
full, broken pipe), so production code propagates the `Result` with `?`
the same way any other fallible I/O call is handled — the
[Book's error-handling chapter](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
treats unwrapping a fallible I/O result as acceptable only in throwaway
code, unlike the `fmt::Write`-into-a-`String` case above where
`.unwrap()` is standard practice.

## Embedded Rust Notes

**Split by target (partial overall).** Writing into a `core::fmt::Write`
destination — a `heapless::String`, a fixed `[u8; N]` buffer wrapper, a
UART peripheral implementing the trait — has **full** support and needs
no allocator at all, which is exactly why `write!` is the standard
replacement for `println!`/`format!` in `#![no_std]` code. Writing into a
`std::io::Write` destination (a file, a socket) has **no** support under
`#![no_std]`, since `std::io` itself needs an OS. The two look like the
same macro call at the source level, but which trait the destination
implements decides which of these applies.
