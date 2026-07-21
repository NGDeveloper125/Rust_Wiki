---
title: "let"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Ownership, Immutability by default, Destructuring]
related_syntax: [mut, const, static, if, else]
see_also: [mut, const]
---

## Explanation

`let` introduces a new variable binding in the current scope. It binds a
name to the value produced by an expression.

This is a declaration, not an assignment in the C sense — `let` always
creates a *new* binding, even if a variable of the same name already
exists. Using `let` again with a name already in scope **shadows** the
previous binding rather than mutating it; the old value still exists (and
may still be borrowed elsewhere) until it goes out of scope or is dropped.

Bindings introduced by `let` are immutable unless the pattern includes
`mut` (see the [`mut`](mut.md) page) — `let` itself does not imply
mutability. A `let` can:

- carry an explicit type annotation: `let x: i32 = 5;`
- destructure a pattern: `let (a, b) = pair;`, `let Point { x, y } = p;`
- be refutable when paired with `else` (`let Some(x) = opt else { return };`)
  — the pattern must match or the `else` block runs and must diverge
- appear with no initializer at all (`let x;`), deferring assignment,
  as long as the compiler can prove it's assigned before first use

`let` is a **statement**, not an expression — it has no value of its own
and cannot be used where an expression is required.

## Usage examples

### Introducing a new binding

```
let x = 5; // <- `let` introduces a new binding named `x`
```

### Creating a new object

A freshly constructed value is typically bound with `let` right where it
is created, via a `new` constructor rather than a bare struct literal at
the call site.

```
struct Order {
    id: u32,
    total: f64,
}

impl Order {
    fn new(id: u32, total: f64) -> Self {
        Order { id, total }
    }
}

let order = Order::new(1042, 59.99); // <- `let` binds the freshly constructed value to `order`
```

An inherent `new` associated function is the
conventional constructor shape per the
[API Guidelines' C-CTOR](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor),
and `let` binding its result immediately keeps construction and naming in
one place.

### Sharing data with multiple references

Several `let` bindings can each hold a shared reference to the same
value at once — none of them take ownership, so the original stays
usable too.

```
let config = String::from("production");

let a = &config; // <- `let` binds a shared reference; `config` isn't moved
let b = &config; // multiple shared references can coexist
println!("{a} {b} {config}");
```

Any number of `&T` references can be live
simultaneously as long as no `&mut T` exists at the same time — the
[Book's chapter on references and borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
is the canonical statement of this rule.

### Handling and propagating errors

Parsing a value that might fail and immediately propagating the error
with `?` is one of the most common shapes a `let` binding takes in
fallible code.

```
fn read_timeout(raw: &str) -> Result<u64, std::num::ParseIntError> {
    let timeout = raw.trim().parse::<u64>()?; // <- `let` binds the value `?` unwraps on success
    Ok(timeout * 1000)
}
```

The `?` operator returns the `Err` variant to the
caller immediately on failure, so by the time `let` finishes binding
`timeout`, the rest of the function can treat it as certainly valid — see
the
[Book's section on the `?` operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator).

## Embedded Rust Notes

**Full support.** `let` is core language grammar with no dependency on
`std` — bindings work identically in `#![no_std]` firmware, on the stack,
exactly as on a hosted target.
