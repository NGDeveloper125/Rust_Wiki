---
title: "libc"
version: "0.2.189"
publisher: "Huon Wilson (huonw), Josh Triplett (joshtriplett), gnzlbg, Yuki Okushi (JohnTitor), rust-lang-owner, libs"
no_std: "yes"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-09"
summary: "Raw declarations of the platform's C library — types, constants, structs and functions, exactly as C sees them. The bottom layer under every crate that talks to the operating system, and `unsafe` all the way down."
categories: ["ffi", "os", "no-std"]
repository: "https://github.com/rust-lang/libc"
---

## Overview

`libc` is a set of `extern "C"` declarations. It implements nothing: it tells
the compiler that `getpid` exists and returns a `pid_t`, that `O_RDONLY` is
`0`, that `struct stat` has these fields in this order — and then the linker
points those at the C library already on the machine.

That makes it the bottom of the stack. It appears near the top of the download
charts because almost everything reaches the operating system eventually, not
because many people write `use libc` themselves.

**And mostly you shouldn't.** Every function here is `unsafe`, returns errors as
`-1`, takes raw pointers and lengths with no relationship the compiler can
check, and has a slightly different signature on each platform. There are two
better answers for ordinary code:

- **`std`** covers files, sockets, threads, processes and time already, safely
  and portably. Reach past it only for something it genuinely doesn't expose.
- **[`rustix`](https://crates.io/crates/rustix) or
  [`nix`](https://crates.io/crates/nix)** wrap the syscalls in safe Rust with
  `Result` and real types. `nix` builds on `libc`; `rustix` can bypass it
  entirely on Linux. Either is what you want for `mmap`, `fcntl` or `signalfd`.

`libc` is the right choice when you're binding to a C library that isn't
wrapped, implementing one of those wrapper crates, or reaching a syscall nobody
has covered yet — and when you need the C types themselves (`c_int`, `size_t`)
to describe an FFI boundary.

**The thing that surprises people is how platform-specific it is.** This is not
a portability layer. Items exist only on the targets that have them, so `libc::
fork` is absent on Windows and won't fail until you build for it; `struct stat`
has different fields on Linux and macOS; and the Windows build exposes only the
MSVC C runtime, which is a small fraction of the Unix surface. Code using it
non-trivially ends up behind `#[cfg(unix)]`, and that is normal rather than a
sign you're doing it wrong.

It has no dependencies, requires Rust 1.65, and works without `std` once you
turn off the default `std` feature — which enables no code of its own, only the
`cfg` the crate keys a few `impl`s off. It lives under the rust-lang
organisation alongside the compiler, and is the same crate `std` itself is
built on.

## When to use it

### Use case: A syscall the standard library doesn't expose

`std` has no `getrlimit`. Reading a process's file-descriptor limit means
declaring the struct, calling the function, and checking the C-style return
value yourself.

```
#[cfg(unix)]
fn fd_limit() -> std::io::Result<(u64, u64)> {
    // SAFETY: getrlimit writes a fully-initialised rlimit through the pointer
    // when it returns 0; we only read it on that path.
    let mut limits = std::mem::MaybeUninit::<libc::rlimit>::uninit();
    let rc = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, limits.as_mut_ptr()) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()); // <- -1 means "check errno"
    }
    let limits = unsafe { limits.assume_init() };
    Ok((limits.rlim_cur as u64, limits.rlim_max as u64))
}

#[cfg(unix)]
fn main() {
    let (soft, hard) = fd_limit().unwrap();
    assert!(soft <= hard);
}

#[cfg(not(unix))]
fn main() {}
```

**Why it fits:** nothing above `libc` offers `getrlimit`, so the alternative is
doing without. Note the shape of the work: a `MaybeUninit` for the out-param, a
`-1` check turned into a real `io::Error`, and the whole thing behind `cfg(unix)`
so other targets still build.

### Use case: Describing a C library's ABI

When you bind to a C library, the signatures have to be written in C's types,
not Rust's. `libc` supplies them, so `c_int` is whatever `int` is on this
target rather than a guess.

```
use libc::{c_char, c_int, size_t};

// The C library we're binding to:
//   int compress(char *dst, size_t *dst_len, const char *src, size_t src_len);
unsafe extern "C" {
    fn compress(dst: *mut c_char, dst_len: *mut size_t,
                src: *const c_char, src_len: size_t) -> c_int;
}

// A safe wrapper is the whole point of writing the binding.
pub fn compressed_len(src: &[u8]) -> Option<usize> {
    let mut out = vec![0u8; src.len() * 2];
    let mut out_len: size_t = out.len();
    // SAFETY: both pointers are valid for the lengths passed alongside them.
    let rc = unsafe {
        compress(out.as_mut_ptr().cast(), &mut out_len,
                 src.as_ptr().cast(), src.len())
    };
    (rc == 0).then_some(out_len)
}
```

**Why it fits:** `c_int` is `i32` on every target Rust supports today, but
`c_char` is signed on x86 and *unsigned* on ARM, and `size_t` follows the
pointer width. Writing `i8` and `u64` by hand is a bug waiting for a different
machine.

### Use case: Reading `errno` correctly

C reports failure by returning `-1` and setting a thread-local `errno`. Getting
at it portably is fiddly — and you almost never should, because `std` already
does it.

```
#[cfg(unix)]
fn main() {
    // Provoke a failure: closing a file descriptor that was never open.
    let rc = unsafe { libc::close(-1) };
    assert_eq!(rc, -1);

    // The right way to read errno: std wraps the platform differences.
    let err = std::io::Error::last_os_error();
    assert_eq!(err.raw_os_error(), Some(libc::EBADF));
    assert!(!err.to_string().is_empty()); // <- "Bad file descriptor"
}

#[cfg(not(unix))]
fn main() {}
```

**Why it fits:** `errno` is a macro in C, not a variable, and each platform
reaches it differently (`__errno_location` on glibc, `__error` on macOS).
`std::io::Error::last_os_error()` already knows all of that, and gives you a
type that prints properly. Read `errno` immediately after the failing call —
anything in between can overwrite it.

## API map

Everything here is either `unsafe` to call or a plain type or constant. The
groups below cover what a binding actually needs; the crate itself has tens of
thousands of items, almost all target-specific, and there is no useful way to
enumerate them — `grep` the source for your platform, or read the C man page and
trust the name to match.

### C type aliases

#### `c_int`, `c_uint`, `c_long`

Rust names for C's integer types, sized per target.

```
use libc::{c_int, c_long, c_uint};

let status: c_int = -1;
let flags: c_uint = 0;
let offset: c_long = 4096;

assert_eq!(std::mem::size_of::<c_int>(), 4);
let _ = (status, flags, offset);
```

**When to use it:** in every `extern "C"` signature and every struct crossing
the boundary. `c_long` is the one that matters — 64-bit on Linux, 32-bit on
Windows — so writing `i64` for a C `long` is wrong on half your targets. These
are also in `std::ffi`, which is preferable when you need nothing else from
`libc`.

#### `c_char` and `c_void`

The character and opaque-pointer types.

```
use libc::{c_char, c_void};

let text = c"hello"; // <- a CStr literal, NUL-terminated
let ptr: *const c_char = text.as_ptr();
let opaque: *mut c_void = std::ptr::null_mut();

assert_eq!(unsafe { libc::strlen(ptr) }, 5);
let _ = opaque;
```

**When to use it:** `*const c_char` for C strings — pair it with `CStr`/`CString`
from `std::ffi` rather than building them by hand — and `*mut c_void` for
handles a C library gives you and you only pass back. `c_char`'s signedness
varies by architecture, so cast through it rather than through `i8`.

#### `size_t` and `ssize_t`

Pointer-width unsigned and signed integers, as C uses for lengths and counts.

```
use libc::{size_t, ssize_t};

let length: size_t = 4096;
let result: ssize_t = -1; // <- read/write return a count, or -1

assert_eq!(std::mem::size_of::<size_t>(), std::mem::size_of::<usize>());
let _ = (length, result);
```

**When to use it:** any C parameter documented as `size_t`. `ssize_t` is the
return type of `read` and `write`, and the reason they can report failure at
all — a signed count with `-1` reserved for the error.

### Calling functions

#### `libc::getpid`

A representative no-argument call: the current process id.

```
#[cfg(unix)]
fn main() {
    // SAFETY: getpid takes no arguments and cannot fail.
    let pid = unsafe { libc::getpid() };
    assert!(pid > 0);
}

#[cfg(not(unix))]
fn main() {}
```

**When to use it:** as the shape of every call here — `unsafe`, a C return type,
and a `cfg` guard if the target set is wider than the function. `std::process::id`
already covers this particular one, which is the general lesson: check `std`
first.

#### `libc::open`, `read` and `close`

The file trio, showing the raw file-descriptor lifecycle.

```
#[cfg(unix)]
fn read_first_byte(path: &std::ffi::CStr) -> std::io::Result<Option<u8>> {
    // SAFETY: path is a valid NUL-terminated string for the duration of the call.
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut byte = 0u8;
    // SAFETY: the buffer is one byte and we pass a length of one.
    let n = unsafe { libc::read(fd, (&raw mut byte).cast(), 1) };
    // Close before inspecting n, so the descriptor leaks on no path.
    unsafe { libc::close(fd) };
    match n {
        -1 => Err(std::io::Error::last_os_error()),
        0 => Ok(None),
        _ => Ok(Some(byte)),
    }
}

#[cfg(unix)]
fn main() {
    let _ = read_first_byte(c"/etc/hostname");
}

#[cfg(not(unix))]
fn main() {}
```

**When to use it:** essentially never — `std::fs::File` does this safely and
closes on drop. It is here because it shows what you take on: a descriptor with
no destructor, an error check per call, and a `close` you must not forget on any
path, including the error ones.

#### `libc::malloc` and `free`

C's allocator, for memory that a C library will take ownership of.

```
use libc::{c_void, size_t};

fn main() {
    // SAFETY: malloc returns either null or a block of the requested size.
    let block = unsafe { libc::malloc(64 as size_t) };
    assert!(!block.is_null(), "allocation failed");

    // SAFETY: writing within the 64 bytes we just asked for.
    unsafe { libc::memset(block, 0, 64) };

    // SAFETY: block came from malloc and is freed exactly once.
    unsafe { libc::free(block as *mut c_void) };
}
```

**When to use it:** only when a C API will `free` the pointer itself, or hands
you one you must `free`. Never mix allocators — memory from Rust's `Box` must
not reach `free`, and `malloc`ed memory must not reach Rust's deallocator. For
ordinary buffers use `Vec`.

### Structs and constants

#### `libc::timespec`

A C struct laid out exactly as the platform defines it.

```
#[cfg(unix)]
fn main() {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: clock_gettime writes through a valid pointer to timespec.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };

    assert_eq!(rc, 0);
    assert!(ts.tv_sec > 0 || ts.tv_nsec > 0);
}

#[cfg(not(unix))]
fn main() {}
```

**When to use it:** whenever a call needs a C struct. Build them field by field
as here, or with `MaybeUninit` when the callee fills them in. Don't assume the
field set — `timespec` is small and stable, but `stat` and `sigaction` differ
substantially between Linux, macOS and the BSDs.

#### Error and flag constants

`errno` values, open flags, signal numbers — the integers C headers define.

```
#[cfg(unix)]
fn main() {
    // errno values, for comparing against raw_os_error()
    assert_ne!(libc::ENOENT, libc::EACCES);

    // open(2) flags, combined with |
    let flags = libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC;
    assert_ne!(flags, 0);

    // signal numbers
    assert_eq!(libc::SIGKILL, 9);
}

#[cfg(not(unix))]
fn main() {}
```

**When to use it:** comparing against `raw_os_error()`, and building flag
arguments. Their *values* differ by platform even when the names don't — `SIGKILL`
is 9 nearly everywhere but `ENOENT` and the `O_*` flags are not, which is exactly
why you use the constant rather than the number you saw in a header once.

### Working safely on top

#### `std::io::Error::last_os_error`

Not from `libc`, but the function you should pair with almost every call here.

```
#[cfg(unix)]
fn main() {
    let rc = unsafe { libc::close(-1) };
    assert_eq!(rc, -1);

    let err = std::io::Error::last_os_error();
    assert_eq!(err.raw_os_error(), Some(libc::EBADF));
    // Prints the system's message, not a number.
    assert!(!err.to_string().is_empty());
}

#[cfg(not(unix))]
fn main() {}
```

**When to use it:** immediately after any call that returned `-1`. It reads
`errno` the way this platform requires and produces an error that composes with
`?` and prints properly. Reading `errno` yourself is a portability bug in
waiting.

#### `libc::c_int` return checking

The convention every C function follows, and the discipline that makes a binding
safe.

```
use libc::c_int;

/// Turn C's -1-means-failure into a Rust Result.
fn check(rc: c_int) -> std::io::Result<c_int> {
    if rc < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(rc)
    }
}

fn main() {
    assert!(check(0).is_ok());
    assert!(check(7).is_ok());
    assert!(check(-1).is_err());
}
```

**When to use it:** write it once per binding crate and route every call through
it. The bugs in hand-written FFI are overwhelmingly unchecked return values, not
exotic aliasing — a helper like this removes the chance to forget. Functions
returning a pointer signal failure with null instead, and `ssize_t` returners
with `-1`.