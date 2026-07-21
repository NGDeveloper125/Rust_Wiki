---
title: "#[naked]"
kind: attribute
embedded_support: full
groups: ["FFI & Linkage", "Memory & Unsafe"]
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

## Usage examples

### Writing a function with no compiler-generated prologue or epilogue

```
#[unsafe(naked)] // <- no compiler-generated prologue/epilogue: the body is pure asm
pub extern "C" fn identity_trampoline() {
    core::arch::naked_asm!("ret");
}
```

### Crossing an FFI boundary

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

An ordinary (non-naked) function assumes the calling
convention's register-saving contract is already in effect, which is not
guaranteed true at the exact instruction an interrupt vector jumps to —
`#[naked]` exists specifically so this hand-off can be written by hand
before handing control to ordinary Rust code; the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/codegen.html#the-naked-attribute)
documents naked functions as intended for exactly this class of
low-level trampoline, and the exact register set and asm syntax are
target-specific rather than portable — this snippet's precise form
compiles only for the ARM-family target it names.

## Explanation (Embedded)

`#[naked]` is one of the rare attributes that is genuinely embedded-only
in any practical sense — hosted programs never need to hand-control
register and stack state at a boundary the OS and calling convention
already guarantee, but bare-metal code hits that need at exactly two
places: the very first instructions after a hardware exception/interrupt
fires (before any register is known to be saved), and a preemptive
scheduler's context switch, where a routine must save one task's entire
register file, swap the stack pointer itself, and restore a different
task's register file — something an ordinary function's compiler-
generated prologue is neither written for nor able to do, since it
assumes it's the one establishing the stack frame, not tearing down and
rebuilding someone else's.

This second case — a hand-rolled real-time-OS context switch — is the
single most iconic real use of `#[naked]` in embedded Rust. On Cortex-M,
the switch is conventionally driven from the low-priority `PendSV`
exception specifically so it can be deferred until no higher-priority
interrupt needs the core, and the handler for it is written naked because
it must control the exact sequence of register pushes/pops and the stack-
pointer swap itself, with no compiler-inserted instruction anywhere in
between.

Most application-level embedded code never writes `#[naked]` directly.
A hardware support crate's `#[interrupt]` attribute (from `cortex-m-rt`)
generates an ordinary, non-naked handler function on top of a runtime
crate's own naked entry trampoline, so `#[naked]` mostly stays inside the
handful of low-level runtime and RTOS crates that need it, not
application code sitting on top of them.

## Usage examples (Embedded)

### Writing a Cortex-M PendSV context-switch handler

```
#[unsafe(no_mangle)]
#[unsafe(naked)] // <- must control the exact register save/restore and stack-pointer swap itself
pub extern "C" fn PendSV() {
    core::arch::naked_asm!(
        "mrs r0, psp",           // r0 = the interrupted task's stack pointer
        "stmdb r0!, {{r4-r11}}", // manually save the callee-saved registers a prologue would have saved
        "bl {switch}",           // pick the next task; returns its saved stack pointer in r0
        "ldmia r0!, {{r4-r11}}", // restore the next task's callee-saved registers
        "msr psp, r0",           // hand the CPU the next task's stack pointer
        "bx lr",                 // exception return: resumes the next task exactly where it left off
        switch = sym choose_next_task,
    );
}

extern "C" fn choose_next_task(saved_sp: u32) -> u32 {
    saved_sp // stand-in for real scheduler logic picking the next task's stack pointer
}
```

### Writing a naked reset handler that sets up the stack before any Rust runs

```
#[unsafe(no_mangle)]
#[unsafe(naked)] // <- runs before the stack pointer or .data/.bss are guaranteed set up
pub unsafe extern "C" fn Reset() -> ! {
    core::arch::naked_asm!(
        "ldr sp, =_stack_start", // set the stack pointer from a linker-script symbol
        "bl {init}",             // now safe to call ordinary Rust: a real stack exists
        init = sym init_runtime,
    );
}

extern "C" fn init_runtime() -> ! {
    loop {} // stand-in for zeroing .bss, copying .data, then calling the user's #[entry] fn
}
```

An ordinary function's prologue assumes a stack pointer and calling
convention are already valid, which is precisely what hasn't happened yet
at the exact instruction the reset vector jumps to — `#[naked]` is what
lets that hand-off be written explicitly before anything resembling
normal Rust runs. In practice, both examples above are exactly the kind
of code `cortex-m-rt` already provides, so most firmware never writes
either by hand.
