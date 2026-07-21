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

## Basic usage example

```
let x = 5;
let r = &x;
let y = *r; // <- `*` dereferences `r`, reading the value it points to
```

## Best practices & deeper information

### Scenario: Sharing data with multiple references

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

**Why this way:** `Deref` lets `Rc<T>`/`Box<T>` be read as if they were a
plain `&T`, and auto-deref already handles method calls (`handle_a.len()`
needs no `*`) — explicit `*` is reserved for the cases auto-deref doesn't
cover, per the [Book's Deref chapter](https://doc.rust-lang.org/book/ch15-02-deref.html).

### Scenario: Mutating through a reference

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

**Why this way:** per the [Book's references chapter](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html),
writing `reading = ...` without the `*` is a type mismatch the compiler
rejects (an `f64` can't be assigned to a `&mut f64` place — rustc's
E0308 suggests adding the `*`) — `*reading = ...` is what makes the
write go through the reference to the value itself.

## Embedded Rust Notes

**Full support** for all three meanings. `Mul`/`Deref` live in `core::ops`,
and raw-pointer dereference is exactly how embedded code reads/writes
memory-mapped hardware registers via `unsafe { *addr }` or, more often, a
volatile wrapper around it.
