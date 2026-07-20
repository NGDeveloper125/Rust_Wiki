---
title: "Higher-order functions"
area: "Functions & Closures"
embedded_support: full
groups: ["Functions & Closures", "Functional Programming"]
related_syntax: [fn, "|...| closures", "->"]
see_also: ["Closures & capturing", "Fn / FnMut / FnOnce", "Function pointers (fn types)", "Generics"]
---

## Explanation

A higher-order function is simply a function that takes another function
(or [closure](closures-and-capturing.md)) as a parameter, returns one, or
both — the same idea as `map` in Haskell or a callback parameter in
JavaScript. Nothing about the language changes to make this possible;
functions and closures are already values in Rust, so passing or
returning one is just passing or returning a value like any other.

Higher-order functions exist to separate "what varies" from "what stays
the same." A loop that walks every element of a collection and does
*something* to each one is the same loop regardless of what that
something is — the standard library bakes this directly into iterator
adaptors like `map` and `filter`, which are higher-order functions that
take the per-element behavior as a closure or plain
[function](functions.md) argument, so the traversal itself never needs to
be rewritten.

There are two ways to accept a function-shaped parameter: a generic
parameter bounded by [`Fn`/`FnMut`/`FnOnce`](fn-fnmut-fnonce.md) (or the
equivalent `impl Fn(...)` shorthand), which is monomorphized per call site
and accepts both closures and [function pointers](function-pointers.md)
at zero runtime cost, or a `dyn Fn`/`Box<dyn Fn>` trait object, needed
when the function has to be stored, returned without a concrete named
type, or held alongside other differently-shaped closures in one
collection.

The other direction — a function *returning* a function — usually returns
`impl Fn(...) -> ...` capturing whatever setup data went into building it;
this "factory function" shape is how you write a function parameterized
by some configuration once, instead of writing a slightly different
version of it per configuration value. Accepting a closure generically is
itself a specialization of writing [generic code](../types-data-modeling/generics.md):
the closure's trait bound is just another trait bound like any other.

## Basic usage example

```
fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 { // <- returns a function: this is a higher-order function
    move |n| n * factor
}

let triple = make_multiplier(3);
println!("{}", triple(7));
```

## Best practices & deeper information

### Scenario: Writing generic code

A validator "factory" builds a closure parameterized by a threshold,
generic over any orderable type — the higher-order function *returns* the
behavior rather than the caller writing it out each time.

```
fn threshold_validator<T: PartialOrd + Copy>(min: T) -> impl Fn(T) -> bool {
    // <- generic higher-order function: returns a closure specialized to `min`
    move |value| value >= min
}

let is_adult = threshold_validator(18);
let is_safe_temp = threshold_validator(-10.0);

assert!(is_adult(21));
assert!(is_safe_temp(5.0));
```

**Why this way:** returning `impl Fn(T) -> bool` instead of a boxed trait
object keeps the returned closure's type concrete and stack-allocated,
which the
[Book's closures chapter](https://doc.rust-lang.org/book/ch13-01-closures.html#returning-closures)
recommends whenever the closure's exact type doesn't need to be erased.

### Scenario: Working with collections

Filtering in-stock orders passes a plain, named function to `filter`
instead of an inline closure — `filter` itself is the higher-order
function here, and it doesn't care whether its argument is a closure or
a bare function.

```
struct Order {
    quantity: u32,
}

fn is_in_stock(order: &&Order) -> bool { // <- a plain function used as the higher-order argument
    order.quantity > 0
}

fn available_orders(orders: &[Order]) -> Vec<&Order> {
    orders.iter().filter(is_in_stock).collect()
    // <- `filter` is higher-order: it takes `is_in_stock` as its argument
}
```

**Why this way:** `Iterator::filter` is declared generic over any
`FnMut(&Self::Item) -> bool`, and a non-capturing function item satisfies
that bound automatically — the
[standard library docs for `filter`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter)
accept either form without a different call site.

### Scenario: Designing a public API

A processing pipeline accepts each step generically (so adding a step is
zero-cost at the call site) but stores the accumulated steps as trait
objects, since a `Vec` needs every element to share one concrete type.

```
struct Order {
    quantity: u32,
}

struct Pipeline {
    steps: Vec<Box<dyn Fn(&mut Order)>>, // <- stored as trait objects: steps come from different closures
}

impl Pipeline {
    fn new() -> Self {
        Pipeline { steps: Vec::new() }
    }

    fn add_step<F: Fn(&mut Order) + 'static>(&mut self, step: F) {
        // <- generic parameter here: no boxing cost at the caller's call site
        self.steps.push(Box::new(step));
    }

    fn run(&self, order: &mut Order) {
        for step in &self.steps {
            step(order);
        }
    }
}

let mut pipeline = Pipeline::new();
pipeline.add_step(|order: &mut Order| order.quantity += 1);
```

**Why this way:** accepting `impl Trait`/a generic bound at the API
boundary but storing the result as `Box<dyn Trait>` internally is a
standard combination when a function needs both an ergonomic, zero-cost
call site and heterogeneous storage — covered in the
[API Guidelines' flexibility guidance](https://rust-lang.github.io/api-guidelines/flexibility.html).

## Embedded Rust Notes

**Full support.** Passing or returning functions/closures generically is
resolved entirely at compile time through monomorphization, so it costs
nothing at runtime and needs no allocator — this works identically in
`#![no_std]`. Only the `dyn Fn`/`Box<dyn Fn>` trait-object form needs the
`alloc` crate; the generic, `impl Fn`-based form never does.
