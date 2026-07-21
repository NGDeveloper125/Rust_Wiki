---
title: "#[target_feature(...)] / #[instruction_set(...)]"
kind: attribute
embedded_support: full
groups: ["FFI & Linkage", "Memory & Unsafe"]
related_concepts: ["Unsafe Rust"]
related_syntax: [unsafe]
see_also: []
---

## Explanation

Both attributes let an individual function opt into CPU-specific
compilation settings that differ from the rest of the crate, rather than
requiring the whole crate be compiled for a narrower target than
necessary.

`#[target_feature(enable = "...")]` compiles one specific function
assuming a CPU feature is available — `"avx2"`, `"sse4.1"`, `"neon"`, and
similar target-specific feature names — even when the crate as a whole is
compiled for a baseline CPU that can't be assumed to have it. This is
what lets a crate ship a single binary with both a portable, baseline
implementation of a computation and a hand-vectorized version that uses
wider SIMD registers, choosing between them at runtime after checking
`is_x86_feature_detected!` (or the equivalent for the target
architecture).

A function marked `#[target_feature(enable = "...")]` must also be an
`unsafe fn` (or be called from inside an `unsafe` block via a function
pointer, on newer Rust versions that relax this for some cases) — calling
it on a CPU that doesn't actually have the named feature is undefined
behavior, typically manifesting as an illegal-instruction crash. The
compiler cannot verify the caller checked for the feature at runtime,
which is exactly the class of un-checkable precondition `unsafe` exists
to mark; the caller is responsible for gating the call behind an actual
runtime feature check.

`#[instruction_set(...)]` addresses a narrower, different problem:
architectures where a single CPU can execute more than one instruction
encoding — most notably ARM, which can switch between the 32-bit ARM
instruction set and the compact Thumb instruction set. `#[instruction_set(arm::a32)]`
or `#[instruction_set(arm::t32)]` pins a specific function to be compiled
for one or the other, which matters when interworking with code compiled
for a specific mode (a vendor bootloader, an interrupt vector expecting a
specific mode) rather than whatever the crate's default is.

## Basic usage example

```
#[target_feature(enable = "avx2")] // <- compiles this function assuming AVX2 is available
unsafe fn sum_avx2(data: &[f32]) -> f32 {
    data.iter().sum() // a real implementation would use AVX2 intrinsics directly
}
```

## Best practices & deeper information

### Scenario: Numeric computation

A signal-processing library sums large buffers of samples far more often
than anything else it does, so it ships both a portable baseline
implementation and an AVX2-accelerated one, selecting between them once
at runtime rather than assuming AVX2 is always present.

```
fn sum_samples(data: &[f32]) -> f32 {
    if is_x86_feature_detected!("avx2") {
        unsafe { sum_avx2(data) } // <- unsafe: caller must have verified the feature first
    } else {
        sum_baseline(data) // portable fallback, no target_feature needed
    }
}

#[target_feature(enable = "avx2")] // <- this function alone assumes AVX2, not the whole crate
unsafe fn sum_avx2(data: &[f32]) -> f32 {
    data.iter().sum() // stand-in for a real AVX2 intrinsic-based reduction
}

fn sum_baseline(data: &[f32]) -> f32 {
    data.iter().sum()
}
```

**Why this way:** `is_x86_feature_detected!` performs the runtime CPU
check that discharges the `unsafe` contract `#[target_feature]` imposes —
calling `sum_avx2` without first confirming AVX2 is present is undefined
behavior on a CPU lacking it, which the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/codegen.html#the-target_feature-attribute)
and `std::arch` documentation both specify as the required pattern:
gate every `target_feature`-enabled call behind an explicit runtime
detection, never an assumption based on the compilation target alone.

## Embedded Rust Notes

**Full support**, and `#[instruction_set(...)]` in particular is far more
relevant to embedded/bare-metal ARM targets than to hosted code — ARM
Cortex-M cores commonly interwork between ARM and Thumb mode, and
firmware linking against externally-compiled routines occasionally needs
to pin a function to match. `#[target_feature(...)]` behaves identically
without `std`, though runtime feature detection macros like
`is_x86_feature_detected!` depend on `std`; a `#![no_std]` crate targeting
a fixed microcontroller more commonly knows its CPU features at compile
time via the target spec itself, rather than detecting them at runtime.
