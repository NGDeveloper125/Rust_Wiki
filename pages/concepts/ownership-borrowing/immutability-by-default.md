---
title: "Immutability by default"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Functional Programming", "Coming from Python / JavaScript"]
related_syntax: [let, mut]
see_also: ["Ownership", "Mutable borrowing"]
---

## Explanation

Every binding introduced with `let` is immutable unless explicitly marked
`mut`. This is the opposite default from most mainstream languages (Java,
Python, JavaScript, C), where variables are mutable unless specially
declared `const`/`final`.

The practical effect is that immutability becomes the norm you reach for,
and mutability becomes something you opt into deliberately at each
binding site — a small, local signal to a reader that *this* particular
variable is expected to change, which makes the ones that don't stand out
as safe to reason about without tracking their value over time.

This default also interacts with the borrow checker directly: a shared
reference (`&T`) can never be used to mutate through it, precisely
because immutability-by-default is the baseline the whole borrowing model
is built on top of — mutability is the special case that needs an
explicit `&mut` to unlock, not the other way around. This is a large part
of why data races are ruled out at compile time: you cannot have two
simultaneous mutable accesses to the same data without the compiler
seeing an explicit `&mut` for it.

## Basic usage example

```
let x = 5;
// x = 6; // would fail to compile: x is immutable by default

let mut y = 5;
y = 6; // <- `mut` explicitly opts this binding into reassignment
println!("{x} {y}");
```

## Best practices & deeper information

### Scenario: Creating a new object

Constructing a fully-formed `Order` in one expression, rather than
creating a default and mutating fields into place afterward, means the
binding never needs `mut` at all.

```
struct Order {
    id: u64,
    total_cents: u64,
    status: &'static str,
}

// AVOID: default-then-mutate needs `mut` and leaves a window where `order` is incompletely formed
// let mut order = Order { id: 0, total_cents: 0, status: "" };
// order.id = 42;
// order.total_cents = 1999;
// order.status = "pending";

// PREFER: build the finished value in one immutable binding
let order = Order { id: 42, total_cents: 1999, status: "pending" }; // <- no `mut` needed: fully formed up front
```

**Why this way:** constructing a value complete at its point of creation
means there's no intermediate state where the struct exists but is only
half-initialized — the
[Rust Book](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
frames immutable-by-default as nudging code toward exactly this shape,
reserving `mut` for values that genuinely change over their lifetime.

### Scenario: Designing a public API

A public config type that hands back a new, independent value from an
"update" — instead of exposing mutable setters — means no caller has to
worry about a value changing out from under them after they've stored it.

```
#[derive(Clone)]
struct RetryPolicy {
    max_attempts: u32,
    backoff_ms: u32,
}

impl RetryPolicy {
    fn with_max_attempts(&self, max_attempts: u32) -> Self { // <- PREFER: returns a new, independent value
        Self { max_attempts, ..self.clone() }
    }
}

// AVOID: a public `set_max_attempts(&mut self, ...)` lets any holder mutate a policy others may rely on

let default_policy = RetryPolicy { max_attempts: 3, backoff_ms: 100 };
let aggressive = default_policy.with_max_attempts(10); // default_policy is untouched
```

**Why this way:** an API built around producing new immutable values
instead of mutating shared ones means no caller has to track whether a
`RetryPolicy` they're holding might change later — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/predictability.html)
favor predictable, non-surprising behavior, and an immutable-by-default
value type is the simplest way to guarantee it.

## Embedded Rust Notes

**Full support.** No `std`/allocator dependency — the immutable-by-default
rule is enforced identically on every target.
