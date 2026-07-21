---
title: "String vs &str"
area: "Collections & Strings"
embedded_support: partial
groups: ["Collections & Strings", "Working with Collections", "String Handling"]
related_syntax: ["&"]
see_also: ["String formatting (Display, Debug, format!)", "Vec<T>", "Slices"]
---

## Explanation

`String` is an owned, growable, heap-allocated buffer of UTF-8 bytes;
`&str` is a borrowed view into UTF-8 bytes, sitting somewhere else —
inside a `String`, in the compiled binary (a `&'static str` literal), or
inside another `&str`. The relationship between them mirrors
[`Vec<T>` and its slice](vec.md): `String` owns and can grow its buffer
(`.push_str()`, `.push()`), while `&str` is a fat pointer — address plus
byte length — into data it doesn't own, and derefs from `String`
automatically wherever a `&str` is expected.

Both guarantee their contents are valid UTF-8 at all times, which is why
neither supports indexing by character position directly (`s[2]` doesn't
compile) — a byte offset can land in the middle of a multi-byte
character, so the API instead offers `.chars()`, `.char_indices()`, and
byte-range slicing (`&s[0..4]`), which panics if the requested range
doesn't fall on a character boundary rather than silently producing
invalid UTF-8.

The practical split is almost always about ownership and lifetime: use
`String` when a value needs to be built up, stored past the current
scope, or returned as new data the caller now owns; use `&str` when
looking at text that already lives somewhere else and only reading it
is needed. A function that just inspects text should almost always take
`&str`, the same way a function that just reads a sequence takes `&[T]`
rather than `&Vec<T>` — accepting the narrower, borrowed type lets the
caller pass a `String`, a literal, or a substring of either without
having to convert anything first.

Converting between them is cheap in one direction and not in the other:
borrowing a `&str` from a `String` (`&my_string` or `my_string.as_str()`)
is free, since it's just reading the existing buffer through a
narrower view, while turning a `&str` into a `String`
(`.to_string()`, `.to_owned()`, `String::from()`) always copies the
bytes into a new heap allocation, because the `&str` might not outlive
the `String` that would otherwise need to borrow it.

## Basic usage example

```
let name: &str = "Priya";      // <- borrowed view into a string literal baked into the binary
let mut greeting = String::from("Hello, "); // <- owned, growable buffer
greeting.push_str(name); // <- &str borrowed here, its bytes copied into greeting's buffer

println!("{greeting}"); // Hello, Priya
```

**Restriction:** slicing a `String`/`&str` at a byte index that isn't a
character boundary (`&greeting[0..1]` when the first character is
multi-byte) panics at runtime rather than silently truncating a
character.

## Best practices & deeper information

### Scenario: Designing a public API

A function that only reads text should accept `&str`, never `&String`
— the narrower borrowed type accepts a `String`, a literal, or any
substring of either, while `&String` only accepts an actual `String`.

```
fn greet(name: &str) -> String { // <- &str: accepts a String, a literal, or a substring, all for free
    format!("Hello, {name}!")
}

let owned = String::from("Priya");
let literal = "Sam";

println!("{}", greet(&owned));   // &String derefs to &str automatically
println!("{}", greet(literal));  // already a &str
```

**Why this way:** the API Guidelines'
[C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html#functions-minimize-assumptions-about-parameters-by-using-generic-types-c-generic)
guidance to minimize assumptions about parameters singles out `&str`
over `&String` as the standard example — the narrower borrow costs
nothing and accepts strictly more callers.

### Scenario: Working with text

Building a piece of text incrementally — from parts that arrive one at a
time, or from a loop — calls for an owned, growable `String`, since a
`&str` can't be extended in place; reading it back afterward can go
straight through a `&str` view.

```
let items = ["order #42", "order #17", "order #90"];

let mut summary = String::new(); // <- owned buffer that will be grown in the loop below
for (i, item) in items.iter().enumerate() {
    if i > 0 {
        summary.push_str(", "); // <- &str literal borrowed and appended into summary's buffer
    }
    summary.push_str(item);
}

let preview: &str = &summary[..9]; // <- borrowed view into the finished String, no copy
println!("{preview}...");
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch08-02-strings.html) frames
`String` as the type to reach for whenever text is being assembled or
modified, with `&str` views taken afterward for anything that only needs
to read the result.

### Scenario: Converting between types

Accepting `AsRef<str>` instead of committing to `&str` or `String`
widens a function to take either one, plus any other type that can cheaply
produce a `&str` view of itself, without an explicit conversion at the
call site.

```
fn log_event(message: impl AsRef<str>) { // <- accepts &str, String, or anything else AsRef<str>
    println!("[event] {}", message.as_ref());
}

log_event("order placed");                 // &str
log_event(String::from("order shipped"));  // String, borrowed via AsRef for the call
```

**Why this way:** the API Guidelines'
[C-CONV](https://rust-lang.github.io/api-guidelines/conversions.html)
recommend `AsRef` for functions that only need a borrowed view, since it
lets `&str` and `String` (and other string-like types) all satisfy the
same signature without the caller converting first.

## Explanation (Embedded)

The split this page draws between owned and borrowed text maps onto a
second split under `#![no_std]`: `&str` needs nothing beyond `core`,
while `String` needs `alloc`. A `&'static str` literal is compiled
straight into the binary's read-only data — reading it back is the same
single operation on a hosted target or a microcontroller with no heap at
all, since nothing is ever allocated to produce it. `String`, by
contrast, is defined in `alloc` and requires `extern crate alloc` plus a
configured `#[global_allocator]` before `String::new()`/`.push_str()`
compile — the same requirement `Vec<T>` carries, since `String` is
`Vec<u8>` with a UTF-8 invariant layered on top.

Where an owned, mutable string is genuinely needed but no heap is
available, `heapless::String<N>` is the substitute: a fixed capacity `N`
chosen at the type level, backed by an inline byte array instead of a
heap allocation. It offers the same "build it up as you go" shape
`String` does — `.push_str()`, `write!` support via `core::fmt::Write` —
but bounded, so an append that would exceed `N` returns an error rather
than growing. The practical embedded question this page's split becomes:
is this text always one of a fixed, known-at-compile-time set of
messages (`&'static str`, no heap ever touched), or does it get built or
mutated at runtime (an owned buffer — `heapless::String<N>` without an
allocator, `String` with one)?

## Basic usage example (Embedded)

```
use core::fmt::Write as _;
use heapless::String;

const DEVICE_NAME: &str = "sensor-07"; // <- &'static str: baked into the binary, no heap involved

let mut status: String<32> = String::new(); // <- owned, growable-within-32-bytes buffer, no allocator
write!(status, "{DEVICE_NAME} ready").unwrap();

let view: &str = &status; // <- borrows the heapless::String, same as &String deref to &str
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

The classic page's guidance — accept `&str`, not an owned type, for
anything that only reads text — holds unchanged here: a function
written against `&str` accepts a `&'static str` literal or a borrow of a
`heapless::String<N>` equally, since `heapless::String<N>` also derefs
to `&str`.

```
fn log_event(message: &str) { // <- &str: accepts a &'static str literal or a &heapless::String<N> alike
    // ... hand `message`'s bytes to a UART/logging peripheral
}

const READY: &str = "device ready"; // <- &'static str, no heap
let mut status: heapless::String<32> = heapless::String::new();
core::fmt::Write::write_str(&mut status, "calibrating").unwrap();

log_event(READY);   // &'static str
log_event(&status); // heapless::String<32> derefs to &str, same call site
```

**Why this way:** the API Guidelines' preference for the narrowest
borrowed type an embedded function's signature stays valid regardless of
whether the caller's storage needs a heap at all — the function itself
never has to know or care.

### Scenario: Working with text

Building a status line from values only known at runtime — a reading, an
error count, an uptime counter — needs owned, mutable storage the same
way the classic page's log-building example does; on a target with no
heap, that owned storage is a `heapless::String<N>` instead of `String`.

```
use core::fmt::Write as _;
use heapless::String;

fn build_status(errors: u8, uptime_s: u32) -> String<64> {
    let mut line: String<64> = String::new(); // <- fixed 64-byte capacity, no heap
    write!(line, "errors={errors} uptime={uptime_s}s").unwrap(); // <- fails only if the text would exceed 64 bytes
    line
}
```

**Why this way:** the capacity bound has to live somewhere once there's
no allocator to grow into — putting it in the type (`String<64>`) makes
the limit visible at every call site, rather than discovering it only
when a runtime append happens to fail.
