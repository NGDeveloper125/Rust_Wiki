---
title: "*"
kind: operator
embedded_support: full
groups: [Arithmetic, Basics, "Ownership & Borrowing", "Memory & Unsafe / FFI"]
related_concepts: [Operator overloading, "Deref & DerefMut coercion", "Smart pointers (Box<T>)"]
related_syntax: ["*=", "&"]
see_also: ["&", "*="]
---

## Explanation

`*` has three unrelated meanings depending on position:

1. **Binary multiplication** (`a * b`) — overloadable via `std::ops::Mul`.
2. **Prefix dereference** (`*ref`) — follows a reference or smart pointer
   to the value it points to. Overloadable via `std::ops::Deref`
   (and `DerefMut` for `*ref = value`), which is exactly the mechanism
   that lets `Box<T>`, `Rc<T>`, etc. be dereferenced with plain `*`.
   Most of the time you don't need to write `*` explicitly, because
   **auto-deref** kicks in for method calls (`my_box.method()` inserts
   the derefs for you) — explicit `*` shows up mainly with operators,
   pattern matching, or assigning through a reference.
3. **Raw pointer type** (`*const T`, `*mut T`) — appears only in a type
   position, never as an expression on its own; dereferencing a raw
   pointer (`*ptr`) requires an `unsafe` block, unlike dereferencing `&T`.

Rust's grammar disambiguates these purely by position: `*` before an
expression with nothing on its left is prefix (dereference); `*` between
two expressions is binary (multiplication); `*const`/`*mut` immediately
followed by a type is the raw pointer type former.

## Usage examples

### Dereferencing a reference

```
let x = 5;
let r = &x;
let y = *r; // <- `*` dereferences `r`, reading the value it points to
```

### Sharing data with multiple references

Reading through a smart pointer with explicit `*` is mostly needed
outside method calls — for comparisons, formatting, or passing the
pointee itself somewhere a reference isn't wanted.

```
use std::rc::Rc;

let shared_config = Rc::new(String::from("production"));
let handle_a = Rc::clone(&shared_config);
let handle_b = Rc::clone(&shared_config);

if *handle_a == *handle_b { // <- `*` follows each handle to the String it points to
    println!("both handles agree: {}", *handle_a); // <- and again here, for formatting
}
```

`Deref` lets `Rc<T>`/`Box<T>` be read as if they were a
plain `&T`, and auto-deref already handles method calls (`handle_a.len()`
needs no `*`) — explicit `*` is reserved for the cases auto-deref doesn't
cover, per the [Book's Deref chapter](https://doc.rust-lang.org/book/ch15-02-deref.html).

### Mutating through a reference

Writing `*reference = value` (or a compound form like `*reference +=
value`) is how a function mutates the caller's data through a `&mut`
parameter, rather than rebinding its own local copy of the pointer.

```
fn apply_offset(reading: &mut f64, offset: f64) {
    *reading += offset; // <- `*` dereferences the &mut, writing through it
}

let mut temperature = 21.5;
apply_offset(&mut temperature, -0.3);
println!("calibrated: {temperature}");
```

Per the [Book's references chapter](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html),
writing `reading = ...` without the `*` is a type mismatch the compiler
rejects (an `f64` can't be assigned to a `&mut f64` place — rustc's
E0308 suggests adding the `*`) — `*reading = ...` is what makes the
write go through the reference to the value itself.

## Explanation (Embedded)

All three meanings carry over unchanged under `#![no_std]` — `Mul` and
`Deref`/`DerefMut` live in `core::ops`, so multiplication and reference/
smart-pointer dereference compile identically to hosted Rust. The genuinely
embedded-specific meaning is the raw-pointer case: dereferencing a
`*const T`/`*mut T` is how firmware reads and writes memory-mapped
hardware registers, and it's almost always wrapped in `unsafe`, since the
compiler has no way to know the address is valid or what side effects
touching it has. A plain `*addr = value` is also something the compiler is
free to reorder or eliminate if it looks like a dead store — for register
I/O, where the write's *side effect* is the entire point, code reaches for
`core::ptr::read_volatile`/`write_volatile` (or a HAL's volatile wrapper)
instead of a bare `*` for exactly that reason. Multiplication itself is
unremarkable in comparison — scaling a raw ADC sample by a calibration
factor is ordinary arithmetic, with the same overflow story as
[`+`](plus.md)/[`-`](minus.md).

## Usage examples (Embedded)

### Dereferencing a memory-mapped register

```
const GPIOA_ODR: *mut u32 = 0x4001_0C0C as *mut u32;

unsafe {
    *GPIOA_ODR = 1 << 5; // <- `*` dereferences the raw pointer, writing the register
}
```

A bare `*` write like this compiles, but is at the
mercy of the optimizer treating it like any other memory write; production
register access more often goes through `write_volatile` to guarantee the
write actually happens, in order, exactly once.

### Scaling a raw ADC reading by a calibration factor

```
fn to_millivolts(raw_sample: u16, calibration: f32) -> f32 {
    raw_sample as f32 * calibration // <- `*` scales the raw sample, same arithmetic as hosted code
}
```
