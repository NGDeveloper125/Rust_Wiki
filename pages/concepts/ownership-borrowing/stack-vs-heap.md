---
title: "Stack vs heap allocation"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Boxing", "Systems / Low-Level Programming"]
related_syntax: []
see_also: ["Smart pointers (Box<T>)", "Memory layout & repr"]
---

## Explanation

The stack is a fast, automatically-managed region of memory for values
whose size is known at compile time and whose lifetime follows the call
stack exactly — local variables in a function are pushed on entry and
popped on return, with no allocation bookkeeping required. The heap is a
separately-managed region for values whose size isn't known until
runtime, or that need to outlive the specific function call that created
them; using it means an explicit allocation (and, in Rust, an equally
explicit deallocation when the owner is dropped).

Rust puts values on the stack by default and only moves data to the heap
when you ask for it — via `Box<T>`, `Vec<T>`, `String`, `Rc`/`Arc`, or any
other type that internally allocates. This is a deliberate design choice:
stack allocation is close to free, so the language doesn't hide heap
allocations behind implicit boxing the way some higher-level languages do
for every non-primitive value — in Rust, if a type allocates, that's
usually visible either in its name (`Box`, `Vec`, `String`) or documented
behavior, not a hidden cost baked invisibly into ordinary variable use.

Knowing which one a value lives on matters for two very different
reasons: performance (stack allocation and deallocation cost is
effectively zero; heap allocation goes through an allocator and costs
real, measurable time), and what's possible at all (a
[recursive type](../types-data-modeling/recursive-types-via-box.md) or a
[trait object](../traits-polymorphism/trait-objects-dynamic-dispatch.md)
*requires* indirection — a pointer of some kind (`Box`, `Rc`, or a plain
reference) — because their size isn't knowable at compile
time the way a stack allocation requires).

## Basic usage example

```
let a = 5;           // stack: fixed size, popped automatically at scope end
let b = Box::new(5); // <- heap: explicit allocation, freed when `b` is dropped
println!("{a} {b}");
```

## Best practices & deeper information

### Scenario: Boxing and heap allocation

A large, rarely-used buffer stored inline in an enum variant forces every
variant of that enum to reserve the biggest one's stack space — boxing
the large variant keeps the common ones cheap.

```
struct Board {
    cells: [[u8; 64]; 64], // 4 KiB — fine on its own, expensive to carry in every enum variant
}

enum GameState {
    Menu,
    Loading(u8),
    Playing(Box<Board>), // <- PREFER: heap-allocated; GameState's size is two words instead of ~4 KiB, even for `Menu`
}

let state = GameState::Playing(Box::new(Board { cells: [[0; 64]; 64] }));
```

**Why this way:** an enum's stack size is the size of its largest
variant (plus a tag, unless niche optimization absorbs it) — inlining a
large `Board` directly would force `Menu` and
`Loading` to reserve the same 4 KiB on the stack even though they never
use it. Boxing the large variant is the standard fix, covered in the
[Rust Book](https://doc.rust-lang.org/book/ch15-01-box.html) as one of
`Box`'s core use cases; see
[Smart pointers (Box<T>)](smart-pointers-box.md) for the fuller
treatment.

## Explanation (Embedded)

Stack allocation needs no allocator, no runtime, and no `std` — it's
available on every target, including the smallest microcontrollers, which
is exactly why embedded Rust leans on it (and on `static`/BSS memory) far
more heavily than hosted code typically does. The heap, by contrast,
isn't just "less preferred" in embedded code — on many targets it doesn't
exist at all unless a project deliberately configures one: `#![no_std]`
alone gives you `core`, not `alloc`, and even a project that pulls in
`alloc` still has to provide a `#[global_allocator]` pointing at real
memory before a single `Box::new` or `Vec::push` will work.

Even where a heap *is* available, embedded code has a reliability reason
to avoid it that hosted code rarely worries about: **uptime**. A desktop
or server process that leaks or fragments its heap can be restarted — by
a user, a supervisor, an orchestrator — usually within seconds to minutes.
Firmware on a deployed device (a sensor node, a medical device, an
appliance controller) is routinely expected to run for months or years
between power cycles, with no supervisor to restart it if the allocator's
free list slowly fragments into uselessness. A fixed-size allocation
pattern that would be a non-issue on a server — repeatedly allocating and
freeing similarly-sized-but-not-identical buffers — can, over a long
enough uptime, fail an allocation that would have trivially succeeded
right after boot. Stack and `static` allocations don't have this failure
mode at all: their sizes are fixed at compile time, so if the program
links and the stack doesn't overflow, the memory layout never changes for
the rest of the device's uptime.

This is why so much embedded Rust code — including entire projects that
never pull in `alloc` — is written to size everything at compile time:
fixed-capacity buffers (`heapless::Vec<T, N>` instead of `Vec<T>`),
`static` storage for anything that must outlive a function call instead
of a heap allocation, and stack-based recursion limits kept shallow and
bounded rather than open-ended. The tradeoff moves in the other direction
from hosted code: instead of "avoid the stack for anything large or
long-lived," embedded Rust's default is "avoid the heap unless the data's
size genuinely can't be known until runtime."

## Basic usage example (Embedded)

```
let sample: i32 = 5;                  // stack: works identically to hosted Rust, no allocator needed
static CALIBRATION: [u8; 4] = [0; 4]; // <- static/BSS: fixed address, known at link time, no heap involved
```

## Best practices & deeper information (Embedded)

### Scenario: Boxing and heap allocation

A large, rarely-used state variant that would justify `Box` on a hosted
target needs a different fix on a project with no allocator configured at
all — moving the big buffer out of the enum and into its own `static`
instead of boxing it.

```
struct Board {
    cells: [[u8; 64]; 64], // 4 KiB
}

// AVOID (no allocator configured): inlining the large variant bloats every GameState to ~4 KiB
enum GameStateInline {
    Menu,
    Playing(Board),
}

// PREFER: the large buffer lives once in `static` storage; GameState only ever holds a flag/index
static mut BOARD: Board = Board { cells: [[0; 64]; 64] }; // <- fixed address, sized at compile time, no heap

enum GameState {
    Menu,
    Playing, // <- refers to the static BOARD rather than owning a copy
}
```

**Why this way:** with no allocator, `Box<Board>` isn't an option at all,
and inlining `Board` directly forces every `GameState` value — even
`Menu` — to reserve the same 4 KiB on the stack; giving the large buffer
its own fixed `static` location sidesteps both problems at the cost of a
single global instance instead of one per value, which is the standard
embedded trade when a type's size, not its heap-vs-stack placement, is
the actual problem; see
[Smart pointers (Box<T>)](smart-pointers-box.md) for when `Box` remains
the right call once `alloc` is already configured.

### Scenario: Creating a new object

A DMA transfer needs a buffer whose address is fixed and known well
before the transfer starts, and that stays put for the transfer's entire
duration — a `static` buffer satisfies this directly; a heap allocation
would need extra work to make the same promise.

```
const FRAME_LEN: usize = 256;

static mut DMA_BUFFER: [u8; FRAME_LEN] = [0; FRAME_LEN]; // <- fixed address at link time, never moves

fn start_transfer() -> *mut u8 {
    unsafe { DMA_BUFFER.as_mut_ptr() } // <- safe to hand to a DMA controller: this address is stable forever
}
```

**Why this way:** DMA hardware is given a raw address and writes to it
independently of the CPU, so the buffer must never move for as long as
the transfer is in flight — a `static` buffer's address is fixed at link
time and never changes, while a heap-allocated buffer's address is stable
only until something reallocates or moves it, making `static` (or a
stack buffer whose scope provably outlives the transfer) the safer
default for hardware that holds onto a bare pointer.
