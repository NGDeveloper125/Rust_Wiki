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

## Usage examples

### Enabling a CPU feature for a single function

```
#[target_feature(enable = "avx2")] // <- compiles this function assuming AVX2 is available
unsafe fn sum_avx2(data: &[f32]) -> f32 {
    data.iter().sum() // a real implementation would use AVX2 intrinsics directly
}
```

### Numeric computation

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

`is_x86_feature_detected!` performs the runtime CPU
check that discharges the `unsafe` contract `#[target_feature]` imposes —
calling `sum_avx2` without first confirming AVX2 is present is undefined
behavior on a CPU lacking it, which the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/codegen.html#the-target_feature-attribute)
and `std::arch` documentation both specify as the required pattern:
gate every `target_feature`-enabled call behind an explicit runtime
detection, never an assumption based on the compilation target alone.

## Explanation (Embedded)

CPU capability varies far more across the embedded target space than it
does between one desktop or server chip and another. Cortex-M0/M0+ cores
(ARMv6-M) have no hardware divide and no DSP extension at all; Cortex-M3
(ARMv7-M) adds hardware divide but still no DSP extension; Cortex-M4 and
M7 (ARMv7E-M) add the DSP extension — extra SIMD-like saturating and
packed-arithmetic instructions — plus an optional single/double-precision
FPU; and Cortex-M33/M55 (ARMv8-M) add DSP and FPU as options again,
alongside TrustZone. A signal-processing routine that would benefit from
the DSP extension's saturating-arithmetic instructions genuinely cannot
assume they exist just because the target is "some Cortex-M" — it has to
be gated the same way an AVX2 routine is gated behind a feature check on
x86.

Where the story diverges from x86 is *how* that gating happens. On a
hosted target, one binary is routinely shipped to run on any of a wide,
unknown range of CPUs, so `is_x86_feature_detected!` checks the feature
at runtime, once, the first time the accelerated path might run.
Embedded firmware is built the opposite way: a given binary is compiled
for one specific, known microcontroller variant, chosen at build time —
by the `--target`/target-cpu selection and the HAL crate's own chip-
specific Cargo feature (`stm32f405`, `nrf52840`, and similar) — so the
CPU's features are already fully known at compile time. Rather than a
runtime check, embedded code more commonly branches with
`#[cfg(target_feature = "dsp")]`, selecting an entire implementation at
compile time and never emitting the alternative path (or the runtime
check) into the binary at all. `is_x86_feature_detected!`-style runtime
detection also depends on `std`, unavailable under `#![no_std]` for the
x86-specific macro itself, which reinforces compile-time `cfg` gating as
the natural embedded idiom rather than a runtime workaround.

## Usage examples (Embedded)

### Enabling the DSP extension for a saturating-arithmetic routine

```
#[target_feature(enable = "dsp")] // <- assumes the ARMv7E-M DSP extension (Cortex-M4/M7), not M0/M3
unsafe fn scale_saturating(samples: &mut [i16], factor: i16) {
    for sample in samples {
        *sample = sample.saturating_mul(factor); // a real implementation would use DSP intrinsics directly
    }
}
```

### Selecting an implementation at compile time instead of runtime

```
#[cfg(target_feature = "dsp")] // <- compiled in only when building for a DSP-capable Cortex-M variant
fn scale_samples(samples: &mut [i16], factor: i16) {
    for sample in samples {
        *sample = sample.saturating_mul(factor); // DSP-accelerated path
    }
}

#[cfg(not(target_feature = "dsp"))] // <- compiled in for Cortex-M0/M3 targets instead
fn scale_samples(samples: &mut [i16], factor: i16) {
    for sample in samples {
        *sample = (*sample).saturating_mul(factor); // portable fallback, identical result
    }
}
```

Unlike the `is_x86_feature_detected!` pattern, there's no runtime branch
here at all — the build is already committed to one specific
microcontroller variant, so `cfg(target_feature = ...)` resolves entirely
at compile time and only one of the two functions above ever exists in
the compiled firmware.
