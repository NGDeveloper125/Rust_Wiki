---
title: "Unsafe Rust"
area: "Memory & Unsafe"
embedded_support: full
groups: ["Memory & Unsafe", "Systems / Low-Level Programming", "Interfacing with C / Other Languages", "Unique to Rust", "Coming from C / C++"]
related_syntax: [unsafe, extern, "*const T", "*mut T"]
see_also: ["Raw pointers (*const T / *mut T)", "FFI (foreign function interface)", "Memory layout & repr", "The undefined-behavior boundary", "Smart pointers (Box<T>)", "Stack vs heap allocation"]
---

## Explanation

Unsafe Rust is the subset of the language where the compiler stops
checking a handful of specific guarantees and instead trusts the
programmer to uphold them by hand. It is not a different language, a
"turn off the borrow checker" switch, or a way to skip type checking —
ordinary Rust rules (types, lifetimes on bindings, most borrow rules)
still apply inside an `unsafe` block. What changes is narrow and
specific: only five extra operations become legal — dereferencing a raw
pointer, calling an `unsafe fn` or unsafe trait method, accessing or
mutating a mutable `static`, implementing an `unsafe trait`, and
accessing a union field.

Unsafe Rust exists because some operations are fundamentally impossible
for a compiler to verify at compile time, yet are still necessary:
talking to hardware registers, calling into a C library, building a data
structure whose safety invariant is more subtle than what the borrow
checker can express (like a doubly linked list or a custom `Vec`), or
implementing the very primitives — `Vec`, `Rc`, `Mutex` — that safe Rust
is built on top of. Someone has to write the unsafe code at the bottom of
the stack; Rust's design lets that code exist as an explicit, clearly
marked, opt-in exception rather than as an ambient possibility anywhere
in the language, which is what "safe by default, unsafe by exception"
means in practice.

The mental model worth keeping is a contract, not a compiler bypass:
every unsafe operation has an invariant a human must guarantee, and
`unsafe` is the keyword that says "I checked, trust me" at that one spot.
Violating the invariant doesn't produce a compiler error or even
necessarily a crash — it produces [undefined behavior](the-undefined-behavior-boundary.md),
which can manifest as anything from a wrong answer to a security hole,
often far away from the unsafe block that actually caused it. This is
why the idiomatic pattern is to keep unsafe code in small, heavily
documented modules and expose only a safe API to the rest of the program
— callers of the safe wrapper get all of Rust's usual guarantees back,
because the module's author has already discharged the unsafe contract
once, carefully, in one place.

Unsafe Rust is also the foundation the other pages in this group build
on: [raw pointers](raw-pointers.md) are the values unsafe code
dereferences, [FFI](ffi.md) is the single most common reason to write
`unsafe` at all, and [`repr`](memory-layout-and-repr.md) controls the
memory layout that unsafe code frequently has to reason about explicitly.

## Basic usage example

```
let mut num = 5;
let r = &mut num as *mut i32; // <- creating a raw pointer is safe...

unsafe {
    // SAFETY: `r` was just derived from a valid, live `&mut i32`, so it
    // points to a properly initialized, aligned i32 for the duration of
    // this block.
    *r += 1; // <- ...but dereferencing one requires `unsafe`
}
println!("{num}");
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

Calling a C library's logging function is the textbook reason to write
`unsafe` at all: the compiler has no way to verify what a foreign
function does with its arguments.

```
// Rustonomicon-style FFI example (std-only, no crate needed).
unsafe extern "C" {
    fn abs(input: i32) -> i32; // <- foreign function; the compiler cannot check its body
}

fn absolute_value(x: i32) -> i32 {
    unsafe {
        // SAFETY: `abs` from the C standard library is a pure function
        // with no preconditions on its i32 argument.
        abs(x) // <- calling into C requires unsafe: the compiler can't verify the FFI contract
    }
}

fn main() {
    println!("{}", absolute_value(-4));
}
```

**Why this way:** the compiler cannot check a foreign function's body, its
argument validity requirements, or its side effects, so `unsafe` marks the
exact point where the programmer is vouching for the call instead of the
compiler — the [Rustonomicon's FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html)
frames every `extern` call this way.

### Scenario: Designing a public API

The idiom "contain unsafety in small modules" means wrapping an unsafe
core in a safe, misuse-resistant API, so the rest of the codebase never
needs to write `unsafe` itself or reason about the invariant directly.

```
pub struct FixedBuffer {
    data: Vec<u8>,
    len: usize,
}

impl FixedBuffer {
    pub fn new(capacity: usize) -> Self {
        Self { data: vec![0; capacity], len: 0 }
    }

    pub fn push(&mut self, byte: u8) -> bool {
        if self.len == self.data.len() {
            return false; // <- safe wrapper enforces the invariant unsafe code below relies on
        }
        unsafe {
            // SAFETY: `self.len < self.data.len()` was just checked above,
            // so this index is in bounds and the write is aligned.
            *self.data.get_unchecked_mut(self.len) = byte;
        }
        self.len += 1;
        true
    }
}
```

**Why this way:** `get_unchecked_mut` skips the bounds check that
`Vec::push` normally performs, so the module keeps its own invariant
(`len <= data.len()`) enforced entirely by the safe `push` method — the
[Rustonomicon](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
recommends exactly this shape: a small unsafe core with a safe API around
it, so a caller can never observe or trigger the broken invariant.

## Embedded Rust Notes

**Full support.** Unsafe Rust is core-language and works identically in
`#![no_std]` — if anything, it is used more often in embedded code, since
memory-mapped registers, interrupt vector tables, and DMA buffers are
usually accessed through raw pointers or mutable statics that only
`unsafe` can touch. Hardware abstraction layer (HAL) crates typically
concentrate all of a project's `unsafe` blocks in one low-level module,
exposing a safe, typed API (`embedded-hal` traits) to application code —
the same "contain unsafety in small modules" idiom shown above, applied
to a microcontroller instead of a buffer.
