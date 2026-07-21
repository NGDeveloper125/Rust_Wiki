---
title: "#[cold]"
kind: attribute
embedded_support: full
groups: ["Compiler Hints & Limits", "Memory & Unsafe"]
related_concepts: []
related_syntax: [fn]
see_also: []
---

## Explanation

`#[cold]` is placed on a `fn` item and hints to the compiler's optimizer
that the function is unlikely to be called in practice — an error path,
a panic-formatting helper, a rarely-hit fallback branch. It carries no
semantic effect whatsoever: a `#[cold]` function still runs exactly the
same code and returns exactly the same values as it would without the
attribute. It only changes optimization and code-layout decisions.

Concretely, the compiler uses this hint to keep "cold" code out of the
way of "hot" code: it's less likely to inline a `#[cold]` function into
its callers (inlining a rarely-taken branch just bloats the common,
hot-path instruction stream with cold bytes that hurt instruction-cache
locality for no benefit), and it tends to place the generated machine
code for `#[cold]` functions away from the hot path in the final binary,
so the CPU's instruction cache and branch predictor aren't spending
their limited resources on code that almost never runs. This is a pure
hint, not a directive the compiler is obligated to follow exactly — like
`#[inline]`, the final decision is the optimizer's.

`#[cold]` is most often reached for on small helper functions that a hot
function calls only on its failure path — factoring the "what to do when
this fails" logic into a separate, `#[cold]`-marked function keeps the
success path's compiled code small and branch-predictor-friendly, since
the compiler no longer has to interleave cold error-formatting logic
into the function actually being optimized for the common case.

## Usage examples

### Marking a rarely-called function as cold

```
#[cold] // <- hints that this function is rarely called; keeps it out of the hot path's inlining
fn report_out_of_range(value: i32) -> ! {
    panic!("value {value} out of range");
}

fn clamp_index(value: i32, len: usize) -> usize {
    if value < 0 || value as usize >= len {
        report_out_of_range(value); // rarely taken
    }
    value as usize
}
```

### Handling and propagating errors

A request-validation function has a fast, common success path and a rare
failure path that builds a detailed error message — splitting the error
construction into a separate `#[cold]` function keeps the success path
lean, since the compiler no longer needs to consider inlining the
message-formatting logic into it.

```
#[derive(Debug)]
struct ValidationError(String);

#[cold] // <- marks the failure path as unlikely, keeping it out of the hot path's codegen
fn invalid_order_total(total_cents: i64) -> ValidationError {
    ValidationError(format!("order total {total_cents} is negative"))
}

fn validate_order_total(total_cents: i64) -> Result<(), ValidationError> {
    if total_cents < 0 {
        return Err(invalid_order_total(total_cents)); // <- rare branch, delegated to a #[cold] fn
    }
    Ok(())
}
```

`#[cold]` communicates a likelihood the compiler can't
infer purely from control flow — that this particular branch is expected
to run rarely — so the optimizer can prioritize the success path's
instruction density and branch prediction over the failure path's; the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/codegen.html#the-cold-attribute)
documents `#[cold]` as exactly this kind of optimization hint, commonly
paired with error-construction helpers for this reason.

## Explanation (Embedded)

`#[cold]` behaves identically under `#![no_std]` — it's a pure codegen
hint with no allocator or OS dependency — but it tends to matter more on
a constrained core than on a desktop CPU. Many Cortex-M0/M0+ parts have
no instruction cache at all, and even parts that do have one have far
less of it than a desktop chip; every extra byte of rarely-taken code
inlined into a hot path is extra flash the core has to fetch through,
often across more wait states than an equivalent SRAM access would cost.
That effect is sharpest inside an interrupt service routine, where the
hot path's size and predictability directly bound how quickly the ISR can
finish and how much interrupt latency the rest of the system sees —
keeping a rare fault or overrun branch `#[cold]` and out of the ISR's
inlined body is a small, concrete way to protect that latency budget
instead of a general-purpose style preference.

## Usage examples (Embedded)

### Keeping an ADC interrupt handler's fault path cold

```
#[cold] // <- rare fault path stays out of the ISR's hot, latency-sensitive body
fn handle_adc_overrun() {
    // log or count the overrun; a real handler might set a fault flag
}

#[unsafe(no_mangle)]
pub extern "C" fn ADC1_2() {
    let overrun = false; // stand-in for a real overrun-flag register read
    if overrun {
        handle_adc_overrun(); // rarely taken
    }
    // ordinary sample read continues here, uninterrupted by the fault-handling code above
}
```

### Marking an allocator's out-of-memory path cold

```
#[cold] // <- keeps the common allocation path lean; OOM is the rare branch
fn report_alloc_failure() -> ! {
    panic!("heap exhausted");
}

fn alloc_or_halt(remaining: usize, requested: usize) -> usize {
    if requested > remaining {
        report_alloc_failure();
    }
    remaining - requested
}
```

Both examples split a rare, larger branch into its own `#[cold]` function
for the same underlying reason as the classic case — the optimizer stops
considering it for inlining into the path that matters — but on firmware
the payoff is measured directly in cycles of interrupt latency and bytes
of flash, not just instruction-cache locality in the abstract.
