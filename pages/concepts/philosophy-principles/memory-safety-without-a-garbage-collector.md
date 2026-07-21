---
title: "Memory safety without a garbage collector"
area: "Rust Philosophy & Design Principles"
embedded_support: full
groups: ["Rust Philosophy & Design Principles", "Unique to Rust"]
related_syntax: []
see_also: ["Ownership", "The borrow checker", "RAII & the Drop trait", "Lifetimes", "Smart pointers (Box<T>)", "Fearless concurrency"]
---

## Explanation

Rust guarantees memory safety — no use-after-free, no double-free, no
dangling references, no data races on shared memory — entirely through
compile-time checks, with zero runtime tracing, no collection pauses, and
no reference-counting overhead paid by default. This places it outside the
two camps memory management has historically been split between.
Garbage-collected languages (Java, Go, Python, JavaScript, C#) trade away
manual memory management for a runtime that tracks liveness and reclaims
memory automatically, at the cost of unpredictable pause times, extra
memory overhead for bookkeeping, and CPU cycles spent on collection rather
than on the program's own work. Manually-managed languages (C, C++) keep
that runtime cost at zero but hand the entire responsibility of calling
`free`/`delete` correctly to the programmer — and getting that wrong is
consistently cited as the largest single source of serious security
vulnerabilities in audited C/C++ codebases.

The mechanism is [ownership](../ownership-borrowing/ownership.md) plus
[the borrow checker](../ownership-borrowing/borrow-checker.md) plus
[lifetimes](../ownership-borrowing/lifetimes.md), working together to
*prove*, at compile time, that every reference is valid for exactly as
long as it's used and that a value is never freed while something still
refers to it. By the time a program compiles, these bug classes are
eliminated for the safe subset of the language — not merely made less
likely, the way a linter reduces but doesn't remove a class of bug.
[RAII and the `Drop` trait](../ownership-borrowing/raii-and-drop.md) supply
the other half: deterministic cleanup the instant a value's single owner
goes out of scope, which is the same discipline C++ programmers already
practice by hand with guard types — Rust just makes the compiler enforce
it universally, for every type, rather than leaving it as an opt-in
convention a programmer can forget to follow.

None of this is free to design around, and it's worth being honest about
where the guarantee's edges are. It covers *safe* Rust; [unsafe Rust](../memory-unsafe/unsafe-rust.md)
is an explicit, opt-in escape hatch for the rare patterns the borrow
checker's conservative rules can't verify even though they're actually
sound, and code inside an `unsafe` block is back to manual discipline, the
same as C. The borrow checker is also deliberately conservative — it
rejects some programs that are genuinely memory-safe but that it simply
can't prove sound with the rules it has. This is exactly why
[`Rc`/`Arc`](../ownership-borrowing/shared-ownership-rc-arc.md) and
[interior mutability](../ownership-borrowing/interior-mutability.md)
(`Cell`/`RefCell`) exist: they move a small, explicit, still-checked amount
of runtime bookkeeping (a reference count, a runtime borrow check) back
into a specific value, opted into only where a single static owner
genuinely isn't expressive enough — rather than paying that bookkeeping
cost globally, for every allocation in the program, the way a tracing
garbage collector does regardless of whether any particular allocation
needs it.

What this buys in practice is substantial: memory-safety bugs are
repeatedly cited by browser and OS vendors as the cause of the large
majority of their most serious security vulnerabilities in C/C++
codebases, and Rust's compile-time guarantee removes that entire class
structurally, in the safe subset of the language, without paying a
garbage collector's runtime cost to do it. That combination is precisely
why Rust has been adopted for Linux kernel modules, browser-engine
components, and firmware — contexts where a GC's pause times or memory
overhead were disqualifying, but where hand-rolled C-style memory
management kept leaving the same categories of exploitable bug behind.

It's also worth keeping this guarantee's scope narrow in your head:
"memory-safe" is a distinct claim from "bug-free," and a distinct claim
from data-race-free in general async or distributed logic — see
[Fearless concurrency](fearless-concurrency.md) for the closely related
but separate promise around threads specifically. And like every guarantee
covered on this page, it's delivered at effectively no runtime cost, which
is the same [zero-cost abstractions](zero-cost-abstractions.md) principle
showing up again: the safety proof is entirely a compile-time artifact,
leaving nothing behind in the compiled binary.

## Basic usage example

```
fn make_reading() -> i32 { // <- returns an owned value: nothing here can outlive its data
    let raw = 5;
    raw
}

// fn dangling() -> &'static i32 {
//     let raw = 5;
//     &raw // would fail to compile: `raw` is dropped here, so this reference would dangle
// }
```

## Best practices & deeper information

### Scenario: Managing resources (RAII)

A batch job that writes to a scratch directory needs that directory
cleaned up no matter how the job exits — tying the cleanup to a value's
scope means it happens deterministically, the instant the owner goes out
of scope, rather than "eventually, whenever a collector gets to it."

```
struct TempWorkspace { path: std::path::PathBuf }

impl Drop for TempWorkspace {
    fn drop(&mut self) { // <- runs synchronously the moment the owner's scope ends
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn run_batch_job() {
    let workspace = TempWorkspace { path: "build/tmp-job".into() };
    // ... write intermediate files under workspace.path ...
} // <- workspace.drop() runs here, before this function returns — no GC pause, no finalizer queue
```

**Why this way:** the
[Rust Book's `Drop` chapter](https://doc.rust-lang.org/book/ch15-03-drop.html)
covers this deterministic, scope-tied cleanup as RAII's central guarantee
— unlike a garbage-collected language's finalizers, which run at an
unspecified time (or not at all, if the process exits first), `Drop` here
runs at a precise, known point.

### Scenario: Boxing and heap allocation

A large in-memory frame buffer needs to live on the heap rather than the
stack, and be reclaimed the instant it's no longer needed — `Box` gives
both without any tracing pass over the heap to find and free it.

```
struct FrameBuffer { pixels: [u8; 1_920 * 1_080 * 4] } // <- large: kept off the stack entirely

fn capture_frame() -> Box<FrameBuffer> {
    Box::new(FrameBuffer { pixels: [0; 1_920 * 1_080 * 4] }) // <- one heap allocation, one deterministic owner
} // <- freed the instant its Box is dropped: no GC scan needed to reclaim the ~8 MB

let frame = capture_frame();
println!("captured {} bytes", frame.pixels.len());
```

**Why this way:** the
[`std::boxed::Box` docs](https://doc.rust-lang.org/std/boxed/struct.Box.html)
describe `Box` as the plain, single-owner heap allocation with no
reference counting; freeing it is a direct deallocation the moment its
owner's scope ends, rather than something a collector has to discover is
now garbage before it can reclaim it.

## Embedded Rust Notes

**Full support** — and one of the clearest real-world proofs of this
principle. Embedded and firmware targets typically have no room for a
tracing garbage collector's memory overhead or unpredictable pause times,
which historically left C as the only realistic systems choice and its
manual-memory bugs (buffer overflows, use-after-free on interrupt-shared
buffers) as a recurring source of firmware vulnerabilities. Rust's
compile-time guarantee applies identically under `#![no_std]` at zero
runtime cost, which is precisely why it has been adopted for kernel
modules and bare-metal firmware; `unsafe` still exists for the raw
register and DMA-buffer access embedded code genuinely needs, and
allocator-free alternatives like `heapless` cover the cases where even
`Box`'s single heap allocation isn't available.
