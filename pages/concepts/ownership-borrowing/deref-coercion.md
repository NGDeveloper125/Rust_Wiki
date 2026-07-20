---
title: "Deref & DerefMut coercion"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing"]
related_syntax: ["*"]
see_also: ["Smart pointers (Box<T>)", "Shared ownership (Rc & Arc)"]
---

## Explanation

`Deref` and `DerefMut` let a smart pointer type transparently behave like
a reference to whatever it wraps. A type implementing `Deref<Target = T>`
can be used almost anywhere a `&T` is expected — most visibly, calling a
method defined on `T` directly on the smart pointer (`my_box.method()`
instead of `(*my_box).method()`), because the compiler automatically
inserts as many derefs as needed to find a matching method.

This coercion is what makes `Box<T>`, `Rc<T>`, `String` (whose
`Deref::Target` is `str`, so `&String` coerces to `&str`), and `Vec<T>`
(whose target is `[T]`, so `&Vec<T>` coerces to `&[T]`) feel ergonomic to use day
to day — you rarely need to think about the wrapper layer at all, because
method calls, and reference coercions in function-argument position, see
straight through it to the underlying type.

The tradeoff worth knowing about: overusing custom `Deref` impls purely
to fake inheritance-like "is-a" relationships between unrelated types is
a recognized anti-pattern in Rust (sometimes called "Deref polymorphism")
— `Deref` is meant to model "acts like a reference to," not "is a kind
of," and stretching it to the latter tends to produce confusing method
resolution rather than genuinely reusable abstraction.

## Basic usage example

```
fn greet(name: &str) {
    println!("hello, {name}");
}

let boxed = Box::new(String::from("world"));
greet(&boxed); // <- &Box<String> coerces through Box then String to &str
```

## Best practices & deeper information

### Scenario: Designing a public API

A function that only needs to read a string should accept `&str`, not
`&String` — deref coercion means callers holding either a `String` or a
string literal can call it without extra ceremony.

```
fn greet(name: &str) { // <- PREFER: &str accepts both &String (via coercion) and &str directly
    println!("hello, {name}");
}

// fn greet_narrow(name: &String) { ... } // AVOID: forces every caller to already own a String

let owned = String::from("Ada");
greet(&owned); // <- &String coerces to &str automatically
greet("Grace"); // a string literal is already &str; no coercion needed
```

**Why this way:** `&str` accepts everything `&String` does (deref
coercion covers that direction for free) plus string literals and other
`&str`-producing sources, so it's strictly the more widely callable
parameter type — the
[Rust Book](https://doc.rust-lang.org/book/ch04-03-slices.html#string-slices-as-parameters)
recommends `&str` over `&String` as the more general parameter type for
exactly this reason.

### Scenario: Writing generic code

A generic function bounded by `AsRef<str>` leans on the same "many
wrapper types act like a reference to the same target" idea deref
coercion embodies, letting one function body serve owned strings,
borrowed slices, and boxed strings alike.

```
fn shout<S: AsRef<str>>(value: S) -> String {
    value.as_ref().to_uppercase() // <- works whether `value` is String, &str, or Box<str>
}

let boxed: Box<str> = "quiet".into();
println!("{}", shout(&*boxed)); // <- Box<str> derefs to str, coercing to &str at the call boundary
println!("{}", shout(String::from("also quiet")));
```

**Why this way:** writing generic code against `AsRef<str>` lets one
function serve owned strings, borrowed slices, and smart-pointer-wrapped
strings without the caller manually unwrapping anything — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/flexibility.html#functions-minimize-assumptions-about-parameters-by-using-generics-c-generic)
recommend this for functions that only need read access to string-like
data.

## Embedded Rust Notes

**Full support.** `Deref`/`DerefMut` live in `core::ops` — no allocator
dependency (the mechanism works the same whether or not the smart pointer
being deref'd happens to need `alloc`, e.g. it applies just as well to
allocator-free wrapper types).
