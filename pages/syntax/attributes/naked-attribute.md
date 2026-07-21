---
title: "#[naked]"
kind: attribute
embedded_support: full
groups: ["Memory & Unsafe"]
related_concepts: ["Unsafe Rust", "FFI (foreign function interface)"]
related_syntax: [unsafe, "#[no_mangle] / #[link(...)] / #[link_name] / #[link_ordinal] / #[link_section] / #[no_link] / #[export_name]"]
see_also: []
---

## Explanation

`#[naked]` is placed on a `fn` item and removes the compiler-generated
function prologue and epilogue entirely — the stack-frame setup, callee-
saved register spills, and matching teardown that an ordinary function
gets automatically. A naked function's body must consist of nothing but
a single inline assembly block (`core::arch::naked_asm!` — the
`asm!`-based version specifically meant for naked functions), because the
compiler generates none of the usual scaffolding that would let ordinary
Rust statements execute correctly: there is no guaranteed stack frame for
local variables to live in, and the calling convention's register-saving
contract is left entirely to the assembly the function's body writes by
hand.

This exists for exactly one category of use case: code that must control
the machine's exact register and stack state at a boundary the compiler's
normal codegen can't be trusted to preserve — the very first instructions
of an interrupt or exception handler (before any register is known to be
saved), or a context-switch routine that manually swaps a CPU's register
state between two tasks. In both cases, an ordinary function's
compiler-generated prologue would already have clobbered or saved
registers in a way the routine's own hand-written logic needs to control
directly.

**This is a rare, advanced, and still-evolving feature.** Naked functions
stabilized on `x86_64`/`aarch64`/similar targets only in fairly recent
Rust releases, and the exact rules (what a naked function's body may
legally contain, how it interacts with `#[unsafe(no_mangle)]` and calling
conventions) are stricter and narrower than for an ordinary `unsafe fn` —
consult the current [Rust Reference entry for `#[naked]`](https://doc.rust-lang.org/reference/attributes/codegen.html#the-naked-attribute)
for the exact, target-specific constraints before writing one, since they
are more likely to have shifted between Rust versions than most of the
language.

## Basic usage example

```
#[unsafe(naked)] // <- no compiler-generated prologue/epilogue: the body is pure asm
pub extern "C" fn identity_trampoline() {
    core::arch::naked_asm!("ret");
}
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

An interrupt controller on a microcontroller jumps directly to a handler
address with no compiler-managed calling convention already in place —
the very first instructions must save the interrupted context by hand
before anything resembling an ordinary function call is safe, which is
exactly what a naked trampoline provides.

```
#[unsafe(naked)] // <- interrupt entry: no assumption about register state can be made yet
pub extern "C" fn interrupt_trampoline() {
    core::arch::naked_asm!(
        "push {{r4-r11, lr}}",   // manually save the registers a normal prologue would have saved
        "bl   {handler}",        // now safe to call an ordinary Rust function
        "pop  {{r4-r11, pc}}",
        handler = sym interrupt_handler,
    );
}

extern "C" fn interrupt_handler() {
    // ordinary Rust code, safe to run once registers are saved
}
```

**Why this way:** an ordinary (non-naked) function assumes the calling
convention's register-saving contract is already in effect, which is not
guaranteed true at the exact instruction an interrupt vector jumps to —
`#[naked]` exists specifically so this hand-off can be written by hand
before handing control to ordinary Rust code; the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/codegen.html#the-naked-attribute)
documents naked functions as intended for exactly this class of
low-level trampoline, and the exact register set and asm syntax are
target-specific rather than portable — this snippet's precise form
compiles only for the ARM-family target it names.

## Embedded Rust Notes

**Full support**, and this is essentially an embedded-only attribute —
`#[naked]` has almost no legitimate use case in hosted `std` code, since
hosted programs never need to write their own interrupt entry or
context-switch trampolines. Even within embedded Rust it is reached for
rarely: most projects get equivalent behavior for free from a hardware
support crate's `#[interrupt]` attribute, which generates an ordinary
(non-naked) handler function on top of a runtime crate's own naked entry
trampoline, so application code essentially never writes `#[naked]`
itself.
