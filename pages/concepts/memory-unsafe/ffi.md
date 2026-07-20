---
title: "FFI (foreign function interface)"
area: "Memory & Unsafe"
embedded_support: full
groups: ["Memory & Unsafe", "Systems / Low-Level Programming", "Interfacing with C / Other Languages", "FFI / Interop", "Unique to Rust", "Coming from C / C++"]
related_syntax: [unsafe, extern, "*const T", "*mut T", "#[repr(...)]", "#[no_mangle]"]
see_also: ["Unsafe Rust", "Raw pointers (*const T / *mut T)", "Memory layout & repr", "The undefined-behavior boundary"]
---

## Explanation

FFI, the foreign function interface, is the mechanism Rust uses to call
functions written in another language — almost always C, or a language
that exposes a C-compatible interface — and to expose Rust functions so
that other languages can call them back. Rust has no runtime and no
special calling convention of its own to protect; it can talk directly to
the C ABI, which is the closest thing systems programming has to a
universal language boundary, and that's exactly what `extern` blocks and
`extern "C"` functions plug into.

An `extern "C"` block declares functions that live in a foreign library,
telling the compiler their signature without giving it the body — the
compiler trusts the declaration completely, which is why calling one of
these functions is `unsafe`: nothing in Rust verifies that the C
implementation actually behaves the way the signature promises. Going the
other direction, marking a Rust function `extern "C"` and `#[no_mangle]`
makes it callable from C: `extern "C"` picks the C calling convention
instead of Rust's unstable internal one, and `#[no_mangle]` keeps the
function's name intact in the compiled binary instead of letting the
compiler rename it, so a C caller can link against it by its plain name.

Because the two languages agree on almost nothing beyond raw bytes and
calling convention, FFI signatures are built out of [raw
pointers](raw-pointers.md) and `#[repr(C)]`-controlled data — see [Memory
layout & repr](memory-layout-and-repr.md) for why a Rust struct needs an
explicit, stable layout before it can be handed across this boundary at
all. FFI is also the single most common real-world reason ordinary Rust
code ends up needing [`unsafe`](unsafe-rust.md): the compiler simply has
no way to check what happens on the other side of the boundary, so the
programmer has to vouch for it explicitly at every call site.

The safety burden at an FFI boundary is larger than a typical unsafe
block: it includes everything the foreign side assumes about pointer
validity, buffer lengths, thread-safety, and even who is responsible for
freeing memory. Idiomatic Rust FFI code concentrates all of this in a
thin, carefully audited layer — often called `sys` or `-sys` bindings —
with a safe, idiomatic Rust API layered on top, so the rest of the
program never touches `extern "C"` directly. See [The undefined-behavior
boundary](the-undefined-behavior-boundary.md) for what specifically must
never go wrong at this edge.

## Basic usage example

```
unsafe extern "C" {
    fn abs(input: i32) -> i32; // <- extern: declares a foreign C function
}

fn main() {
    // SAFETY: `abs` is the C standard library's `abs`, a pure function
    // with no preconditions on its i32 argument.
    let result = unsafe { abs(-7) }; // <- calling across the FFI boundary requires unsafe
    println!("{result}");
}
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

A sensor driver crate typically needs both directions of FFI: calling a
vendor-supplied C library to read hardware, and exposing a Rust callback
that the same C library can invoke when new data arrives.

```
// Rustonomicon-style FFI example (std-only, no crate needed for a
// basic extern "C" declaration — linking against a real C library
// would additionally require a build script).
unsafe extern "C" {
    fn sensor_read_raw(channel: u8) -> i32; // <- declared in the vendor's C header, implemented in their library
}

pub fn read_channel(channel: u8) -> i32 {
    unsafe {
        // SAFETY: `sensor_read_raw` is documented by the vendor as safe
        // to call from any thread with a channel in 0..=7.
        sensor_read_raw(channel) // <- crossing INTO the foreign library
    }
}

#[unsafe(no_mangle)] // <- keeps the symbol name stable so C can link against it
pub extern "C" fn on_sensor_alarm(code: i32) {
    // <- crossing FROM C back into Rust: the vendor library calls this by name
    eprintln!("sensor alarm, code {code}");
}
```

**Why this way:** `extern "C"` on both the declaration and the exported
function selects the platform's C calling convention on each side of the
boundary, and `#[unsafe(no_mangle)]` is required on the Rust-side function
or the compiler's name-mangling would make it unfindable by the linker —
edition 2024 requires wrapping `no_mangle` in `unsafe(...)` since it
affects global symbol linkage the compiler can't verify, per the
[Rust Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html);
the [Rustonomicon's FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html)
covers both directions as the two halves of the same boundary.

### Scenario: Handling and propagating errors

C libraries report failure through sentinel return values or error codes,
never through `Result` — an idiomatic Rust wrapper translates that
convention at the boundary instead of leaking C-style codes into the rest
of the program.

```
unsafe extern "C" {
    fn sensor_init(channel: u8) -> i32; // <- returns 0 on success, a negative errno-style code on failure
}

#[derive(Debug)]
pub struct SensorError(i32);

pub fn init_channel(channel: u8) -> Result<(), SensorError> {
    let code = unsafe {
        // SAFETY: `channel` is a plain integer with no aliasing or
        // lifetime concerns; the vendor library validates it internally.
        sensor_init(channel)
    };
    if code == 0 {
        Ok(())
    } else {
        Err(SensorError(code)) // <- C's error convention converted into an idiomatic Result at the boundary
    }
}
```

**Why this way:** translating error codes into `Result` right at the FFI
call site means every caller further up the stack gets ordinary `?`-based
error handling instead of having to remember which integer means what —
the [Rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html) treats the
boundary itself as the right place to absorb a foreign calling
convention, C-style codes included.

### Scenario: Designing a public API

A crate wrapping a C SDK should expose only safe, typed Rust functions;
every `unsafe extern "C"` declaration and raw pointer stays private to a
small internal module.

```
mod sys {
    unsafe extern "C" {
        pub fn hal_gpio_write(pin: u8, level: u8); // <- raw FFI surface, not exported from the crate
    }
}

pub struct GpioPin {
    number: u8,
}

impl GpioPin {
    pub fn set_high(&self) {
        unsafe {
            // SAFETY: `self.number` was validated as a real GPIO pin when
            // this `GpioPin` was constructed; the HAL requires 0 or 1 for level.
            sys::hal_gpio_write(self.number, 1); // <- unsafe call contained inside the safe method
        }
    }
}
```

**Why this way:** keeping `sys` private and exposing only `GpioPin`
methods means callers of the crate never write `unsafe` or see a raw
pointer themselves — the
[Rustonomicon's guidance on containing unsafety](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
applies directly to FFI wrappers, which are one of the most common places
this idiom shows up in real crates.

## Embedded Rust Notes

**Full support.** FFI works identically without `std`, and is arguably
*more* common in embedded Rust than in hosted Rust: instead of calling a
general-purpose OS library, embedded FFI usually means linking against a
vendor-supplied C HAL or SDK (an ST, Nordic, or Espressif peripheral
library) that predates any Rust support for that chip. The same
`extern "C"` declarations and `#[repr(C)]` structs apply; the main
practical difference is that linking typically pulls in a vendor `.a`
archive via a `build.rs` script rather than a system library found by the
linker's default search path.
