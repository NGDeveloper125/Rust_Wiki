---
title: "Zero-sized types & PhantomData"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Designing Robust Data Models"]
related_syntax: []
see_also: ["Unit structs", "\"Make invalid states unrepresentable\""]
---

## Explanation

A zero-sized type occupies no memory at runtime at all — `size_of::<T>() == 0`
— while still existing fully as a type at compile time. The unit type
`()`, unit structs (`struct Marker;`), and a single-variant field-less
enum are the most common naturally-occurring examples (a *multi*-variant
field-less enum like `Ordering` isn't zero-sized — it still needs a
discriminant byte): the compiler doesn't need to store anything to
represent a value that carries no data, since there's only ever one
possible value of that type.

`PhantomData<T>` is a special zero-sized type used to tell the compiler
"pretend this struct owns/relates to a `T`" without actually storing a
`T` anywhere in the struct — needed when a generic parameter is used only
in a way the compiler can't see directly (for example, in raw pointers
inside an `unsafe` implementation), so that lifetime checking, variance,
and drop-check analysis still treat the struct as if it genuinely
contained a `T`.

Both are examples of a broader theme: using the type system to carry
information that has real compile-time meaning but zero runtime cost —
the compiler tracks and enforces it, and none of it survives into the
compiled binary as actual bytes.

## Basic usage example

```
use std::marker::PhantomData;

struct Typed<T> {
    value: u32,
    _marker: PhantomData<T>, // <- no T is stored, but the compiler treats this as "owning" a T
}

let x: Typed<f64> = Typed { value: 1, _marker: PhantomData };
```

**Restriction:** `PhantomData<T>` contributes zero bytes to the struct's
size, but it isn't purely decorative — it still affects variance and
drop-check analysis as if a real `T` were stored, which can change
whether certain lifetime or drop patterns compile.

## Best practices & deeper information

### Scenario: Writing generic code

A typestate-style marker parameter lets a generic type encode which
state a value is in — so operations invalid for that state simply don't
exist as callable methods — at zero bytes of runtime cost.

```
use std::marker::PhantomData;

struct Open;   // <- zero-sized marker types
struct Closed;

struct Connection<State> {
    socket_fd: i32,
    _state: PhantomData<State>, // <- costs 0 bytes; only tracked at compile time
}

impl Connection<Closed> {
    fn open(self) -> Connection<Open> { // <- consumes a Closed connection, returns an Open one
        Connection { socket_fd: self.socket_fd, _state: PhantomData }
    }
}

impl Connection<Open> {
    fn send(&self, data: &[u8]) { /* ... */ }
    // send() simply doesn't exist on Connection<Closed> -- calling it on a
    // closed connection is a compile error, not a runtime panic
}
```

**Why this way:** this typestate pattern, covered in the
[Embedded Rust Book's Typestate Programming chapter](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html),
moves a whole category of "used it in the wrong order" bugs from a
runtime check to a compile error, and `PhantomData<State>` is what makes
the state parameter free — it contributes nothing to `Connection`'s
runtime size.

## Embedded Rust Notes

**Full support.** No allocator dependency — `PhantomData`-based typestate
is a signature embedded HAL pattern (e.g. `Pin<MODE>` encoding a GPIO
pin's configuration in its type so misusing it is a compile error, with
zero runtime representation for the state itself).
