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

## Embedded Rust Notes

**Full support.** `#[cold]` is a pure codegen hint with no allocator or
OS dependency, so it behaves identically in `#![no_std]` — if anything,
it can matter more on resource-constrained targets, where keeping a
rarely-taken error or panic-formatting path out of a hot interrupt
handler's inlined code has a more noticeable effect on that handler's
size and instruction-cache footprint than on a desktop CPU.
