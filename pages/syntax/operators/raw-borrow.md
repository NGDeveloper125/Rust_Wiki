---
title: "&raw const / &raw mut"
kind: operator
embedded_support: full
groups: ["Ownership & Borrowing", "Memory & Unsafe"]
related_concepts: ["Borrowing (shared references)", "Mutable borrowing"]
related_syntax: ["&", mut, "*"]
see_also: ["&", "*"]
---

## Explanation

`&raw const expr` and `&raw mut expr` build a raw pointer (`*const T` /
`*mut T` respectively) directly from a place expression, without ever
constructing an intermediate reference to that place. `raw` is a **weak
keyword**: it has no special meaning anywhere else in the language and is
recognized only in this exact position, immediately after `&` and
immediately before `const` or `mut` — everywhere else, `raw` is a perfectly
ordinary identifier (a variable, field, or function can still be named
`raw`). The operators were stabilized in Rust 1.82.

The point of contrast is the older, more common idiom:
`&expr as *const T` / `&mut expr as *mut T`. That's a two-step operation —
first `&`/`&mut` creates a genuine reference to `expr` (which must satisfy
every validity requirement a reference has: proper alignment, no
dereferencing through invalid or uninitialized memory), and only afterward
is that reference cast to a raw pointer. `&raw const`/`&raw mut` collapses
this into one step that skips reference creation entirely. That matters
because it lets you obtain a pointer to a place where forming a reference at
all would be unsound — most commonly, a field of a `#[repr(packed)]`
struct, which the compiler cannot guarantee is aligned enough to satisfy a
reference's requirements. Taking `&packed.field` in that situation is
undefined behavior (and modern rustc denies or lints it outright);
`&raw const packed.field` sidesteps the question entirely by never forming
that reference.

`expr` must be a place expression — a variable, field access, dereference,
or index expression with an actual memory location — the same requirement
`&`/`&mut` already impose on their operand; it cannot be applied directly to
a value expression like a literal. **Forming** either raw-borrow expression
is always safe and needs no `unsafe` block by itself; what still requires
`unsafe` is *dereferencing* the resulting pointer (`unsafe { *ptr }` or a
pointer method like `read_unaligned`), exactly as with any other raw
pointer, and if the place expression itself involves dereferencing a raw
pointer (`&raw const (*ptr).field`), that dereference is unsafe and must be
inside an `unsafe` block even though the outer `&raw const` is not.

Be honest about how rarely this is needed: the overwhelming majority of
Rust code never writes `&raw const`/`&raw mut` at all. It matters almost
exclusively in systems/FFI-adjacent code dealing with externally-defined
memory layouts — packed structs, memory-mapped hardware registers, data
handed over from C — where an intermediate reference could itself be
invalid before you ever get the chance to read through it.

## Usage examples

### Reading an unaligned field of a packed struct

```
#[repr(packed)]
struct SensorFrame {
    tag: u8,
    reading: u32, // <- may end up unaligned inside a packed struct
}

let frame = SensorFrame { tag: 1, reading: 4_200 };
let reading_ptr: *const u32 = &raw const frame.reading; // <- pointer to the field, no `&u32` ever formed
let value = unsafe { reading_ptr.read_unaligned() };
println!("{value}");
```

### Designing a public API

A public wrapper around a packed protocol frame exposes safe getters that
read each field through `&raw const` and `read_unaligned`, rather than ever
forming a reference to a field that might not be properly aligned.

```
#[repr(packed)]
pub struct FrameHeader {
    version: u8,
    length: u32,
}

impl FrameHeader {
    pub fn length(&self) -> u32 {
        let len_ptr: *const u32 = &raw const self.length; // <- pointer to a possibly-unaligned field
        unsafe { len_ptr.read_unaligned() }
    }

    pub fn version(&self) -> u8 {
        self.version // ordinary field access is fine here: u8 has no alignment requirement to violate
    }
}
```

Forming a direct reference to a misaligned field of a
`#[repr(packed)]` struct is undefined behavior, which is exactly what the
[Rust Reference's raw borrow operators section](https://doc.rust-lang.org/reference/expressions/operator-expr.html#raw-borrow-operators)
documents `&raw const`/`&raw mut` as existing to avoid — the getter never
creates the invalid intermediate reference the old `&self.length as *const
u32` idiom would.

### Crossing an FFI boundary

A C library hands back a pointer to a tightly packed struct matching an
external wire format; reading one field for logging shouldn't risk
constructing an invalid reference if the incoming pointer turns out to only
be byte-aligned.

```
#[repr(C, packed)]
struct DeviceStatus {
    code: u16,
    flags: u32,
}

extern "C" {
    fn poll_device() -> *const DeviceStatus;
}

fn log_flags() {
    unsafe {
        let status_ptr = poll_device();
        let flags_ptr: *const u32 = &raw const (*status_ptr).flags; // <- pointer to a field of an externally-owned struct
        let flags = flags_ptr.read_unaligned();
        println!("flags: {flags:#x}");
    }
}
```

The
[Rustonomicon's guidance on working with externally-defined layouts](https://doc.rust-lang.org/nomicon/other-reprs.html)
favors raw pointers over references whenever a layout's alignment and
validity can't be locally guaranteed — `&raw const` is the tool that keeps
this idiomatic instead of relying on `&(*status_ptr).flags as *const u32`,
which briefly forms a reference the incoming layout may not actually
satisfy.

## Explanation (Embedded)

`&raw const`/`&raw mut` carry over into `#![no_std]` completely
unchanged — they're core-language and allocator-free — but bare-metal
firmware is where they earn their keep beyond the packed-struct/FFI case
covered above. A `static mut` shared between `main` and an interrupt
handler is a textbook case: taking `&mut TICK_COUNT` to get a plain
mutable reference requires proving, for as long as that reference lives,
that nothing else can access `TICK_COUNT` — a guarantee an interrupt that
can fire at any moment genuinely breaks, which is exactly why current
Rust increasingly treats `&mut` to a `static mut` as dangerous or rejects
it outright. `&raw mut TICK_COUNT` sidesteps the problem: it produces a
`*mut T` pointing at the static without ever asserting exclusive access,
so forming the pointer stays sound even though the static is genuinely
shared; only the later dereference needs its own `unsafe` block and its
own reasoning about why that particular read or write is safe (typically:
interrupts disabled around it, or the access is a single properly-aligned
load/store the target guarantees is atomic). The same reasoning is why a
`#[repr(C)]` register-block struct benefits from the same operator:
taking a pointer to one field with `&raw mut regs.cr1` doesn't require
asserting a reference to the *entire* struct is simultaneously exclusive,
which matters when a peripheral's register block is reachable from more
than one place — an interrupt handler and the main loop both holding a
pointer into the same MMIO block, for instance.

## Usage examples (Embedded)

### Taking a raw pointer to a `static mut` shared with an interrupt handler

```
static mut TICK_COUNT: u32 = 0;

// Called only from the timer interrupt handler.
unsafe fn increment_tick_count() {
    let ptr: *mut u32 = &raw mut TICK_COUNT; // <- forms the pointer without asserting exclusive access to the static
    unsafe { *ptr += 1; } // the write itself is its own, separately-justified unsafe operation
}

// Called only from the main loop, with interrupts briefly disabled around the read.
fn read_tick_count() -> u32 {
    let ptr: *const u32 = &raw const TICK_COUNT; // <- same static, read-only pointer, still no `&u32` ever formed
    unsafe { ptr.read_volatile() }
}
```

### Pointing at one register of a memory-mapped peripheral block

```
#[repr(C)]
struct Usart {
    cr1: u32,
    cr2: u32,
    sr: u32,
}

const USART1: *mut Usart = 0x4001_3800 as *mut Usart;

fn enable_usart() {
    unsafe {
        let cr1_ptr: *mut u32 = &raw mut (*USART1).cr1; // <- pointer to one register, no reference to the whole peripheral block
        cr1_ptr.write_volatile(cr1_ptr.read_volatile() | 0x1); // set the enable bit
    }
}
```
