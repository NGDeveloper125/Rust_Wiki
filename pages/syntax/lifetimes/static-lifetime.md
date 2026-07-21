---
title: "'static"
kind: lifetime
embedded_support: full
groups: ["Ownership & Borrowing"]
related_concepts: ["Lifetimes", "Closures & capturing"]
related_syntax: ["'a (named lifetime)", "&"]
see_also: ["'a (named lifetime)"]
---

## Explanation

`'static` is a lifetime built into the language — unlike `'a`, it never
needs to be declared in a generic parameter list before use; it (along with
the elided placeholder `'_`) is simply always available. It appears in two
positions with related but distinct meanings.

As the lifetime of a **reference type**, `&'static T` means the reference is
valid for the entire remainder of the running program. String literals are
the most common example: a literal like `"engine-status"` is baked directly
into the compiled binary's read-only data and never deallocated, so its type
is `&'static str` automatically, with no annotation needed.

As a **bound** on a generic type parameter, `T: 'static`, the meaning is
different enough that it's the most commonly misread piece of syntax in
this area: it does **not** mean "values of `T` live for the whole program."
It means "`T` contains no borrowed data with a lifetime shorter than
`'static`" — every reference nested anywhere inside `T`, if there are any at
all, is itself valid for the program's duration. This is why fully owned
types — `String`, `i32`, `Vec<u8>`, or a struct made entirely of owned
fields — satisfy `T: 'static` trivially and unconditionally, no matter how
briefly a particular value of that type actually lives. A `String` created
and dropped a microsecond later still satisfies `String: 'static`, because
the bound describes the type's *structure* (can it possibly contain a
dangling short-lived borrow?), not any individual value's runtime lifespan.
A type like `&'a str`, or a struct holding one, only satisfies a `'static`
bound if `'a: 'static` — that is, if the reference it holds is itself
program-long-lived.

`T: 'static` is required wherever a consumer can't prove how long it will
keep hold of a value — most visibly, `thread::spawn`'s closure parameter is
bounded `F: FnOnce() -> T + Send + 'static`, because the compiler has no way
to know when (or whether) the spawned thread finishes relative to the
caller's stack frame, so it must rule out the closure holding any borrow
that frame's locals could invalidate. This is the place most Rust
programmers first run into `'static` as a bound and, misreading it as "the
closure must run forever," reach for `.clone()` or `Arc` before realizing an
already-owned value satisfies the bound for free (see
[Move semantics](../../concepts/ownership-borrowing/move-semantics.md) and
[`move`](../keywords/move.md) for how an owned capture gets there).

## Basic usage example

```
let name: &'static str = "engine-status"; // <- string literals are `&'static str` automatically
println!("{name}");
```

## Best practices & deeper information

### Scenario: Multi-threading

`thread::spawn`'s closure must be `'static`; moving an owned `String` into
it satisfies that bound trivially, since owned data can't contain a
dangling borrow of anything.

```
use std::thread;

fn start_worker(label: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // <- the closure passed to `spawn` must be `'static`; `label` is owned, so it qualifies for free
        println!("[{label}] worker running");
    })
}

let handle = start_worker(String::from("indexer"));
handle.join().unwrap();
```

**Why this way:** the spawned thread's lifetime isn't bounded by the
caller's stack frame, so nothing it captures may borrow from that frame —
the
[Book's concurrency chapter](https://doc.rust-lang.org/book/ch16-01-threads.html#using-move-closures-with-threads)
covers `move`-ing an owned value in as the standard way to satisfy
`thread::spawn`'s `'static` bound without cloning anything unnecessarily.

### Scenario: Designing a public API

A scheduler that stores heterogeneous jobs behind `Box<dyn Trait>` needs
those trait objects to be `'static` — not because a job runs forever, but
because the scheduler can't predict how long it will hold onto any given
job.

```
trait Job: Send {
    fn run(&self);
}

struct Scheduler {
    jobs: Vec<Box<dyn Job + 'static>>, // <- `'static` here means "no non-'static borrows inside," not "lives forever"
}

impl Scheduler {
    fn add(&mut self, job: impl Job + 'static) {
        self.jobs.push(Box::new(job));
    }
}
```

**Why this way:** `dyn Trait` defaults to a `'static` lifetime bound unless
a shorter one is spelled out explicitly, which the
[Rust Reference's trait-object lifetime-bounds section](https://doc.rust-lang.org/reference/types/trait-object.html#trait-object-lifetime-bounds)
documents as the reason a stored trait object needs the data behind it to
be fully owned or itself `'static`.

### Scenario: Sharing data with multiple references

Constants and literals baked into the binary are naturally `&'static`, so
many call sites can share the same reference without any explicit lifetime
parameter appearing anywhere.

```
const DEFAULT_REGION: &str = "us-east-1"; // <- elided, but the real type is `&'static str`

fn region_label(region: Option<&'static str>) -> &'static str {
    region.unwrap_or(DEFAULT_REGION)
}

println!("{}", region_label(Some("eu-west-1")));
println!("{}", region_label(None));
```

**Why this way:** the
[standard library's `str` documentation](https://doc.rust-lang.org/std/primitive.str.html)
notes that string literals are `&'static str` because they live directly in
the compiled binary and are never deallocated, which is what lets code pass
them around anywhere a `'static` reference is required with no lifetime
annotation to write.

## Embedded Rust Notes

**Full support.** `'static` is core-language and erased before codegen —
string literals are still `&'static str` with `#![no_std]`, and `T: 'static`
bounds show up constantly in embedded code for globally-shared driver
state (`static` items, interrupt-shared data) for exactly the same
structural reason described above, not because anything runs "forever."
