---
title: "FFI (foreign function interface)"
area: "Memory & Unsafe"
embedded_support: full
groups: ["Memory & Unsafe", "Systems / Low-Level Programming", "Interfacing with C / Other Languages", "FFI / Interop", "Unique to Rust", "Coming from C / C++"]
related_syntax: [unsafe, extern, "*", "#[repr(...)]", "#[no_mangle] / #[link(...)] / #[link_name] / #[link_ordinal] / #[link_section] / #[no_link] / #[export_name]"]
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

## Explanation (Embedded)

Linking against a vendor's C SDK is one of the most common reasons an
embedded Rust project needs FFI at all — chip vendors have decades of C
driver code (ST's STM32Cube HAL, Nordic's `nrfx`, TI's DriverLib, a
vendor RTOS like FreeRTOS or ThreadX) and no Rust equivalent for a given
peripheral or protocol stack, so the pragmatic path is binding the
existing C library instead of reimplementing it. The mechanics are
identical to hosted FFI — an `unsafe extern "C"` block declares the
vendor functions' signatures, `#[repr(C)]` on every struct that crosses
the boundary keeps its layout byte-for-byte compatible with the vendor's
C header, and each call is `unsafe` because the compiler has no way to
check what the vendor's compiled object file actually does — but the
*shape* of what gets bound differs from a typical hosted library. A
vendor HAL init function commonly takes a pointer to a configuration
struct with a dozen fields (clock source, GPIO mode, alternate function
number, pull-up/down, speed) that must match the C header's `struct`
field-for-field, which is exactly why that struct needs `#[repr(C)]` on
the Rust side — see [Memory layout & repr](memory-layout-and-repr.md)
for why `repr(Rust)`'s default reordering would silently break this.
Linking itself typically means pulling in a prebuilt vendor `.a` archive
through a `build.rs` script (`cc`/`bindgen`-generated bindings, or
hand-written `extern` declarations for a small enough surface) rather
than finding a system library on the default linker search path, since
there is no OS package manager providing the vendor SDK.

## Basic usage example (Embedded)

```
#[repr(C)] // <- must match the vendor header's GPIO_InitTypeDef layout exactly
pub struct GpioInitTypeDef {
    pin: u32,
    mode: u32,
    pull: u32,
    speed: u32,
}

unsafe extern "C" {
    fn HAL_GPIO_Init(gpiox: *mut core::ffi::c_void, init: *mut GpioInitTypeDef); // <- vendor HAL function
}
```

## Best practices & deeper information (Embedded)

### Scenario: Crossing an FFI boundary

Initializing a GPIO pin through ST's HAL means building a config struct
whose layout the vendor's C library will read directly, then handing it
a raw pointer.

```
#[repr(C)] // <- layout must match STM32Cube's GPIO_InitTypeDef byte-for-byte
pub struct GpioInitTypeDef {
    pin: u32,
    mode: u32,
    pull: u32,
    speed: u32,
}

unsafe extern "C" {
    fn HAL_GPIO_Init(gpiox: *mut core::ffi::c_void, init: *mut GpioInitTypeDef);
}

pub fn init_led_pin(gpio_port: *mut core::ffi::c_void) {
    let mut config = GpioInitTypeDef { pin: 1 << 5, mode: 0x01, pull: 0, speed: 0x02 };
    unsafe {
        // SAFETY: `gpio_port` is a valid GPIOx base address (checked by
        // the caller), and `config` outlives this call as a stack local.
        HAL_GPIO_Init(gpio_port, &mut config); // <- crossing into the vendor's prebuilt C HAL
    }
}
```

**Why this way:** without `#[repr(C)]`, the compiler could reorder
`pin`/`mode`/`pull`/`speed` in memory while `HAL_GPIO_Init` keeps reading
them at the C header's fixed offsets, silently feeding the wrong value
into each field — the [Rustonomicon's FFI
chapter](https://doc.rust-lang.org/nomicon/ffi.html) treats
layout-matching as a precondition for any struct crossing an
`extern "C"` boundary, embedded or not.

### Scenario: Handling and propagating errors

Vendor HALs report failure the C way — an integer status code, not a
`Result` — so an idiomatic wrapper translates the vendor's convention at
the boundary instead of leaking `HAL_OK`/`HAL_ERROR` integers into
application code.

```
unsafe extern "C" {
    fn HAL_UART_Transmit(huart: *mut core::ffi::c_void, data: *const u8, len: u16, timeout: u32) -> i32; // 0 = HAL_OK
}

#[derive(Debug)]
pub struct UartError(i32);

pub fn send(uart_handle: *mut core::ffi::c_void, bytes: &[u8]) -> Result<(), UartError> {
    let status = unsafe {
        // SAFETY: `uart_handle` was initialized by the vendor SDK before
        // this call, and `bytes` is a valid slice for its full length.
        HAL_UART_Transmit(uart_handle, bytes.as_ptr(), bytes.len() as u16, 100)
    };
    if status == 0 { Ok(()) } else { Err(UartError(status)) } // <- vendor's int code becomes an idiomatic Result
}
```

**Why this way:** absorbing the vendor's `HAL_StatusTypeDef`-style
integer codes right at the call site means every caller further up the
firmware gets ordinary `?`-based error handling instead of having to
remember which vendor constant means what — the same
boundary-absorption idiom the
[Rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html) recommends
for any foreign error convention.

### Scenario: Designing a public API

A crate wrapping a vendor SDK should expose only safe, typed Rust
methods; every `unsafe extern "C"` declaration and `#[repr(C)]` struct
stays inside a private `sys` module.

```
mod sys {
    #[repr(C)]
    pub struct GpioInitTypeDef {
        pub pin: u32,
        pub mode: u32,
    }

    unsafe extern "C" {
        pub fn HAL_GPIO_Init(gpiox: *mut core::ffi::c_void, init: *mut GpioInitTypeDef); // <- raw vendor FFI surface, not exported
    }
}

pub struct Led {
    port: *mut core::ffi::c_void,
}

impl Led {
    pub fn new(port: *mut core::ffi::c_void, pin: u32) -> Self {
        let mut config = sys::GpioInitTypeDef { pin, mode: 0x01 };
        unsafe {
            // SAFETY: `port` is a valid GPIOx base address, validated by
            // this constructor's caller per its documented contract.
            sys::HAL_GPIO_Init(port, &mut config); // <- unsafe FFI call contained inside a safe constructor
        }
        Self { port }
    }
}
```

**Why this way:** keeping `sys` private means the rest of the crate — and
every downstream user of `Led` — never writes `unsafe` or sees the
vendor's raw C types, which is the same "contain unsafety in small
modules" idiom the
[Rustonomicon](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
recommends, applied to a vendor HAL instead of a hand-rolled data
structure.
