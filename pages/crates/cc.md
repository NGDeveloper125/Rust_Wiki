---
title: "cc"
version: "1.4.5"
publisher: "rust-lang-owner"
publisher_url: "https://crates.io/users/rust-lang-owner"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-04"
summary: "Compiles C, C++ and assembly from a build script and links the result into your crate. The standard way a Rust crate wraps existing native code — and the reason that crate needs a C compiler to build."
categories: ["build-scripts", "ffi", "build-utils"]
repository: "https://github.com/rust-lang/cc-rs"
---

## Overview

Some things only exist as C. A codec, a database engine, a vendor's SDK — the
implementation is there and rewriting it is not the job. `cc` is how a Rust
crate compiles that source and links it in, from a `build.rs` that runs before
your own code:

```
// build.rs — this function is the body of its `main`.
fn build() {
    cc::Build::new()
        .file("csrc/hash.c")
        .file("csrc/util.c")
        .include("csrc/include")
        .define("HASH_FAST", "1")
        .compile("hash"); // <- produces libhash.a and links it
}
```

It finds the platform's compiler, picks the flags that match the target profile,
compiles each file, archives them into a static library, and prints the
`cargo:rustc-link-lib` and `cargo:rustc-link-search` lines that tell rustc where
the result is. You declare the source files; it handles the rest.

**The cost is the important part, and it is paid by everyone downstream.** A
crate depending on `cc` needs a working C toolchain on the machine building
it — `cc` or `clang` on Unix, MSVC Build Tools on Windows. That is fine for a
service you deploy yourself and a real problem for a widely-used library: it
breaks builds on machines without a compiler, complicates cross-compilation,
slows cold builds, and gives you a second language's undefined behaviour inside
a Rust project.

So the question before reaching for it is whether the native code is necessary.
A pure-Rust implementation, if one exists, avoids all of that — the ecosystem's
move from `openssl` to `rustls` is that trade being made deliberately. When the
C really is required, `cc` is the right tool and is maintained under the
rust-lang organisation.

**It is a build dependency, not a dependency.** It belongs under
`[build-dependencies]` in `Cargo.toml`, is used only from `build.rs`, and no
part of it ends up in your binary. Nothing on this page is callable from
ordinary code.

Two neighbours are worth knowing: `bindgen` generates the Rust declarations for
a C header, so the two pair — `cc` builds the library, `bindgen` describes it —
and the `cmake` crate drives a project that already has its own CMake build
rather than compiling files one by one. It requires Rust 1.65.

## When to use it

### Use case: Wrapping a small C library

The common shape: a few `.c` files vendored into the repository, compiled and
linked, with a hand-written `extern "C"` block describing them.

```
// build.rs
fn build() {
    cc::Build::new()
        .file("vendor/crc32.c")
        .include("vendor")
        .opt_level(2)
        .warnings(false) // <- vendored code is not yours to clean up
        .compile("crc32");
}

// src/lib.rs — the Rust side.
unsafe extern "C" {
    fn crc32_compute(data: *const u8, len: usize) -> u32;
}

pub fn crc32(data: &[u8]) -> u32 {
    // SAFETY: the pointer and length describe the same slice.
    unsafe { crc32_compute(data.as_ptr(), data.len()) }
}
```

**Why it fits:** the build script is three lines and the linking is automatic.
Turning warnings off for vendored code is the pragmatic choice — you did not
write it, cannot fix it, and a wall of warnings on every build trains people to
ignore them.

### Use case: Compiling C++ with a specific standard

C++ needs the language selected and usually a standard pinned, because the
default varies by compiler and version.

```
// build.rs
fn build() {
    cc::Build::new()
        .cpp(true) // <- selects the C++ compiler and links its stdlib
        .std("c++17")
        .file("src/engine.cpp")
        .flag_if_supported("-fno-exceptions")
        .compile("engine");
}
```

**Why it fits:** `cpp(true)` does two things that are easy to miss separately —
it invokes the C++ driver and arranges for the standard library to be linked.
`flag_if_supported` is the right form for a flag MSVC would reject, since the
same build script has to work on every platform.

### Use case: Detecting whether a flag is available

Portable build scripts often need to know what the compiler supports rather than
assuming.

```
fn probe() -> String {
    let build = cc::Build::new();

    // Returns Result: outside a build script there is no target to ask about.
    match build.is_flag_supported("-Wall") {
        Ok(true) => "supported".to_string(),
        Ok(false) => "unsupported".to_string(),
        Err(e) => format!("could not check: {e}"),
    }
}

// Running outside build.rs, this reports the missing environment rather than
// guessing — the same information a build script would get for free.
assert!(!probe().is_empty());
```

**Why it fits:** hard-coding a flag that one compiler rejects turns a build
failure into a support ticket. Note what the `Err` says: `cc` reads the target,
host and optimisation level from the environment cargo sets for a build script,
so outside one it cannot answer.

## API map

Everything is a method on `cc::Build`, a builder you configure and then finish
with `compile`. The examples below are written as functions rather than as a
`build.rs` `main`, so they can be type-checked here — in a real build script the
body goes straight into `main`.

### Naming the source

#### `Build::new`

An empty build, configured by chaining.

```
let mut build = cc::Build::new();
build.file("a.c").file("b.c");

let files: Vec<String> = build.get_files().map(|p| p.display().to_string()).collect();
assert_eq!(files, ["a.c", "b.c"]);
```

**When to use it:** once per library you produce. A build script that compiles
two separate archives creates two `Build`s, because `compile` consumes the whole
configured set into one output.

#### `file` and `files`

Adds source files, one at a time or from an iterator.

```
let mut build = cc::Build::new();
build.file("src/one.c");
build.files(["src/two.c", "src/three.c"]);

assert_eq!(build.get_files().count(), 3);
```

**When to use it:** `file` for a handful you list explicitly; `files` when the
set comes from a glob or a directory walk. Paths are relative to the crate root,
which is where cargo runs the build script.

#### `include` and `define`

Header search paths, and preprocessor definitions.

```
let mut build = cc::Build::new();
build
    .include("vendor/include")
    .include("vendor/compat")
    .define("HAVE_STDINT_H", "1")
    .define("DEBUG", None); // <- a bare -DDEBUG with no value

assert_eq!(build.get_files().count(), 0); // configuration only, so far
```

**When to use it:** `include` for every directory the sources `#include` from —
missing one is the usual first build failure. `define` with `None` produces a
valueless define, which is what most feature macros expect.

### Choosing the language

#### `cpp`

Compiles as C++ rather than C, and links the C++ standard library.

```
let mut build = cc::Build::new();
build.cpp(true).file("src/engine.cpp");

assert_eq!(build.get_files().count(), 1);
```

**When to use it:** any C++ source. Forgetting it produces link errors about
mangled symbols that look far stranger than the cause. `cpp_link_stdlib` is the
override for choosing between `libstdc++` and `libc++` explicitly.

#### `std`

Pins the language standard.

```
let mut build = cc::Build::new();
build.file("src/a.c").std("c11");

let mut cxx = cc::Build::new();
cxx.cpp(true).file("src/b.cpp").std("c++17");

assert_eq!(build.get_files().count(), 1);
assert_eq!(cxx.get_files().count(), 1);
```

**When to use it:** whenever the source needs a specific standard, which is
most non-trivial C++. The default is whatever the installed compiler picks, so
leaving it unset means the build depends on which compiler version a user has.

### Flags and profile

#### `flag` and `flag_if_supported`

Passes a raw flag, unconditionally or only where accepted.

```
let mut build = cc::Build::new();
build
    .file("src/a.c")
    .flag("-DNDEBUG")                  // <- fails the build if unsupported
    .flag_if_supported("-fno-strict-aliasing"); // <- skipped if unknown

assert_eq!(build.get_files().count(), 1);
```

**When to use it:** `flag` for something the build genuinely requires, so a
missing one is a loud failure. `flag_if_supported` for a hardening or warning
flag that is nice to have — and for anything GCC-specific, since MSVC rejects
the whole syntax.

#### `warnings` and `warnings_into_errors`

Controls the warning flags `cc` adds by default.

```
let mut vendored = cc::Build::new();
vendored.file("vendor/legacy.c").warnings(false);

let mut own = cc::Build::new();
own.file("src/mine.c").warnings(true).warnings_into_errors(true);

assert_eq!(vendored.get_files().count(), 1);
assert_eq!(own.get_files().count(), 1);
```

**When to use it:** off for vendored third-party sources you cannot fix; on, and
escalated to errors, for C you maintain yourself. Don't set
`warnings_into_errors` on code you don't control — a new compiler version adds
warnings and breaks your users' builds.

#### `opt_level` and `debug`

Overrides the optimisation and debug settings `cc` infers from the cargo
profile.

```
let mut build = cc::Build::new();
build.file("src/hot.c").opt_level(3).debug(false);

assert_eq!(build.get_files().count(), 1);
```

**When to use it:** rarely — by default these follow your cargo profile, which
is usually right. Override when the native code needs optimising even in a debug
build, which is common for a codec or hash whose unoptimised form is unusably
slow in tests.

#### `pic` and `static_flag`

Position-independent code, and static linking of the produced library.

```
let mut build = cc::Build::new();
build.file("src/a.c").pic(true).static_flag(true);

assert_eq!(build.get_files().count(), 1);
```

**When to use it:** `pic` is on by default where it matters and worth setting
explicitly when producing a `cdylib`. These are the settings to reach for when a
link error mentions relocations, which is otherwise a confusing failure to
diagnose.

### Producing the library

#### `compile`

Runs the compiler and archiver, and emits the cargo directives that link the
result.

```
fn build() {
    cc::Build::new()
        .file("src/a.c")
        .compile("mylib"); // <- libmylib.a, plus the link directives
}
```

**When to use it:** as the last call in a build script. It panics on failure,
which is correct there — a build script has no way to recover, and the panic
message carries the compiler's own output. The name has no `lib` prefix or
extension; `cc` adds what the platform expects.

#### `try_compile`

The same, returning `Result` instead of panicking.

```
fn build() {
    let result = cc::Build::new()
        .file("src/optional.c")
        .try_compile("optional");

    if let Err(e) = result {
        // Fall back rather than failing the whole build.
        println!("cargo:warning=optional C support unavailable: {e}");
        println!("cargo:rustc-cfg=no_native_accel");
    }
}
```

**When to use it:** an optional acceleration path that should degrade to a Rust
implementation rather than break the build. Emitting a `cargo:warning` and a
`cfg` is the idiomatic way to let the rest of the crate adapt.

#### `try_get_compiler`

Reports which compiler would be used, without compiling anything.

```
let build = cc::Build::new();

// Err outside a build script: cc reads TARGET, HOST and OPT_LEVEL from the
// environment cargo provides.
match build.try_get_compiler() {
    Ok(tool) => assert!(!tool.path().as_os_str().is_empty()),
    Err(e) => assert!(!e.to_string().is_empty()),
}
```

**When to use it:** branching on the compiler family — MSVC needs different
flags from GCC, and asking is better than inferring from the target triple. The
`Tool` it returns exposes `is_like_msvc`, `is_like_clang` and `is_like_gnu` for
exactly that.

#### `get_files`

The source files configured so far.

```
let mut build = cc::Build::new();
build.files(["a.c", "b.c", "c.c"]);

let names: Vec<String> = build
    .get_files()
    .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
    .collect();

assert_eq!(names, ["a.c", "b.c", "c.c"]);
```

**When to use it:** checking what a conditional build actually assembled, and
emitting `cargo:rerun-if-changed` lines for each source so a `.c` edit triggers
a rebuild. Useful in tests of your own build logic, which is otherwise awkward
to verify.

### Cargo integration

#### `cargo_metadata`

Controls whether `cc` prints the `cargo:` directives that link the library.

```
let mut build = cc::Build::new();
build.file("src/a.c").cargo_metadata(false); // <- you emit the directives

assert_eq!(build.get_files().count(), 1);
```

**When to use it:** almost never — the automatic directives are the main
convenience. Turn it off when producing an object you link some other way, or
when a wrapper crate wants to control exactly what is emitted.

#### `target` and `host`

Overrides the triples `cc` reads from the environment.

```
let mut build = cc::Build::new();
build
    .file("src/a.c")
    .target("armv7-unknown-linux-gnueabihf")
    .host("x86_64-unknown-linux-gnu");

assert_eq!(build.get_files().count(), 1);
```

**When to use it:** when driving `cc` outside a cargo build script, or building
a second artifact for a different target within one script. In an ordinary build
script leave both alone — cargo sets them, and overriding is how a
cross-compilation setup quietly builds for the wrong machine.

#### `compiler` and `archiver`

Names the tools explicitly instead of detecting them.

```
let mut build = cc::Build::new();
build.file("src/a.c").compiler("clang-18").archiver("llvm-ar");

assert_eq!(build.get_files().count(), 1);
```

**When to use it:** pinning an exact toolchain for reproducibility, or reaching
a cross-compiler that detection misses. Prefer the `CC` and `AR` environment
variables where you can — they let a user redirect the build without editing
your script.
