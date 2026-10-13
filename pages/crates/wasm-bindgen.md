---
title: "wasm-bindgen"
version: "0.2.129"
publisher: "Ingvar Stepanyan (RReverser), daxpedda, Guy Bedford (guybedford), wasm-bindgen-publish"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-13"
summary: "The bridge between Rust compiled to WebAssembly and JavaScript. One attribute exports Rust functions, structs and methods to JS, and declares the JS functions Rust may call."
domain: "WebAssembly & JS interop"
categories: ["wasm"]
repository: "https://github.com/wasm-bindgen/wasm-bindgen"
docs: "https://docs.rs/wasm-bindgen/0.2.129/wasm_bindgen/"
---

## Overview

WebAssembly's own interface is almost unusably narrow: a module can pass
integers and floats across the boundary, and that is all. No strings, no
objects, no closures, no return of anything structured. `wasm-bindgen` is the
layer that hides this, so Rust and JavaScript can exchange the types they
actually use.

It is one attribute, pointing in two directions:

```
use wasm_bindgen::prelude::*;

// Outward: Rust that JavaScript may call.
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Inward: JavaScript that Rust may call.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(message: &str);
}
```

On an exported function the attribute generates glue that unpacks JS values
into Rust ones; on an `extern` block it generates the Rust side of a call into
JS. `String`, `&str`, `Vec<T>`, `Option<T>`, `Result<T, E>`, structs, methods
and closures all cross the boundary, and none of them could without it.

**The crate alone does not produce something a browser can load.** The compiler
emits a `.wasm` file containing placeholder imports; a separate tool reads the
metadata the macro embedded and writes the matching JavaScript. So the workflow
is always two steps:

- `wasm-pack build`, which runs the compiler and the generator and writes an
  npm-ready package. This is the normal choice.
- or `cargo build --target wasm32-unknown-unknown` followed by
  `wasm-bindgen --target web ...` from the `wasm-bindgen-cli` crate, when you
  want control over the output.

Either way the crate needs `crate-type = ["cdylib"]` in its `[lib]` section, and
**the CLI version must match the library version exactly** — a mismatch is the
single most common failure, and it reports itself as a confusing schema error
rather than a version complaint.

**It is the foundation of a stack, not a standalone library.** `wasm-bindgen`
defines the boundary; [`js-sys`](js-sys.md) uses it to declare JavaScript's
built-in types, [`web-sys`](web-sys.md) to declare every browser API, and
[`wasm-bindgen-futures`](wasm-bindgen-futures.md) to join Rust's `async` to JS
promises. You will nearly always have more than one of them.

It targets the browser and Node. For WebAssembly outside a JavaScript host —
WASI, plugin runtimes, the component model — this is the wrong crate, and
`wit-bindgen` is the one that fits. MSRV is 1.81, and `no_std` works by turning
off the default `std` feature. One note on features: `serde-serialize` and the
`JsValue::from_serde`/`into_serde` methods it enables are **deprecated** because
they force a `serde_json` round-trip; use `serde-wasm-bindgen` instead, which
converts directly.

## When to use it

### Use case: Exporting a Rust implementation to a JavaScript front end

The common shape: heavy work in Rust, interface in JS.

```
use wasm_bindgen::prelude::*;

// Each exported function becomes a named export of the generated JS module.
#[wasm_bindgen]
pub fn checksum(data: &[u8]) -> u32 {
    data.iter().fold(0u32, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u32))
}

#[wasm_bindgen]
pub fn normalise(input: &str) -> String {
    input.trim().to_lowercase()
}

// Option becomes null; Vec becomes an Array.
#[wasm_bindgen]
pub fn first_word(input: &str) -> Option<String> {
    input.split_whitespace().next().map(str::to_string)
}
```

**Why it fits:** these signatures are ordinary Rust — the macro writes the
marshalling, and the function body never mentions JavaScript. That is what makes
the pattern worth it: the Rust stays testable as Rust, with a native `cargo
test` covering the logic, and only the thin exported surface needs a browser to
exercise. Borrowed parameters (`&str`, `&[u8]`) copy from JS memory into the
wasm heap for the duration of the call, so a large buffer passed repeatedly is
worth measuring.

### Use case: Calling JavaScript APIs that have no Rust equivalent

Declaring the other side of the boundary.

```
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // A global function.
    fn alert(message: &str);

    // Renamed, and reached through an object.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(message: &str);

    // A function that may throw: catch turns the exception into Err.
    #[wasm_bindgen(catch, js_namespace = JSON, js_name = parse)]
    fn json_parse(text: &str) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
pub fn describe(text: &str) -> String {
    match json_parse(text) {
        Ok(_) => "valid JSON".to_string(),
        Err(_) => "not JSON".to_string(),
    }
}
```

**Why it fits:** the declaration is a promise you are making to the compiler,
and nothing checks it — an `extern` block naming a function that does not exist
compiles cleanly and fails at run time with a JS error. `catch` is the
important attribute: without it a JS exception unwinds through wasm as a trap
that Rust cannot handle, so any import that can throw wants it. For the APIs
that already exist, prefer [`js-sys`](js-sys.md) and
[`web-sys`](web-sys.md) — they have declared them for you, correctly.

### Use case: Exposing a stateful object as a JavaScript class

A struct with methods becomes something JS can `new`.

```
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Tokenizer {
    separator: String,
    count: usize,
}

#[wasm_bindgen]
impl Tokenizer {
    #[wasm_bindgen(constructor)]
    pub fn new(separator: &str) -> Tokenizer {
        Tokenizer { separator: separator.to_string(), count: 0 }
    }

    // A getter, so JS reads it as a property rather than a method call.
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> usize {
        self.count
    }

    pub fn split(&mut self, input: &str) -> Vec<String> {
        let parts: Vec<String> =
            input.split(&self.separator).map(str::to_string).collect();
        self.count += parts.len();
        parts
    }
}
```

**Why it fits:** state that lives across several calls has to live somewhere,
and keeping it in Rust avoids serialising it on every call. The generated JS
class holds a pointer into the wasm heap, which has one consequence worth
knowing: **the JS side must call `.free()` when done**, because nothing else
drops the Rust value. Forgetting it leaks for the lifetime of the page, and it
is the usual explanation for a wasm module's memory climbing.

## API map

Most of the crate *is* the attribute, so the entries below are mostly its
arguments. The runtime surface is small: `JsValue` and the traits for moving
between it and Rust types.

### Exporting to JavaScript

#### `#[wasm_bindgen]` on functions and the `start` hook

The plain export, and the one that runs on load.

```
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn double(n: i32) -> i32 {
    n * 2
}

// Renamed on the JS side, where snake_case is unidiomatic.
#[wasm_bindgen(js_name = toUpperCase)]
pub fn to_upper_case(s: &str) -> String {
    s.to_uppercase()
}

// Runs once when the module is instantiated, before anything else.
#[wasm_bindgen(start)]
fn main() {}
```

**When to use it:** on anything JS calls. `js_name` matters more than it looks —
the generated TypeScript definitions use it, so a crate exporting `snake_case`
reads as foreign in the consuming project. `start` is where one-time setup
belongs: installing a panic hook, initialising a logger. Keep it short and
non-failing, since a panic there leaves the module unusable.

#### `constructor`, `getter` and `setter`

Making an exported struct feel like a JS class.

```
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Config {
    retries: u32,
}

#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Config {
        Config { retries: 3 }
    }

    #[wasm_bindgen(getter)]
    pub fn retries(&self) -> u32 {
        self.retries
    }

    #[wasm_bindgen(setter)]
    pub fn set_retries(&mut self, value: u32) {
        self.retries = value;
    }
}
```

**When to use it:** whenever the JS side would rather write `config.retries = 5`
than `config.set_retries(5)`. A public field on an exported struct generates a
getter and setter automatically if its type can cross the boundary, so the
explicit pair is for computed or validated properties. A `&mut self` method is
fine, but note the generated class has no way to express Rust's borrow rules —
two live references from JS are prevented at run time by a panic, not at compile
time.

#### Returning errors with `JsError`

Making a Rust `Err` arrive as a JS exception.

```
use wasm_bindgen::prelude::*;

// Err becomes a thrown Error on the JS side; Ok becomes the return value.
#[wasm_bindgen]
pub fn parse_port(input: &str) -> Result<u16, JsError> {
    let port: u16 = input
        .parse()
        .map_err(|_| JsError::new("port must be a number"))?;

    if port < 1024 {
        return Err(JsError::new("port must be 1024 or above"));
    }
    Ok(port)
}
```

**When to use it:** for any fallible export. `JsError` is the right return type
because it produces a real JS `Error` with a message and a stack, which a
`JsValue` holding a string does not. Any Rust error implementing
`std::error::Error` converts into it, so `?` works in an exported function. Do
not reach for panics instead: a panic aborts the module rather than throwing
something catchable.

### Importing from JavaScript

#### `js_namespace`, `js_name` and `catch`

The attributes that describe where an imported function lives.

```
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Math.random()
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;

    // console.warn(), under a Rust-idiomatic name.
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn console_warn(message: &str);

    // decodeURIComponent, which throws on malformed input.
    #[wasm_bindgen(catch, js_name = decodeURIComponent)]
    fn decode(encoded: &str) -> Result<String, JsValue>;
}
```

**When to use it:** `js_namespace` for anything reached through an object,
`js_name` when the JS name is not a legal or idiomatic Rust one, and `catch` on
anything that can throw — which, in JavaScript, is more than you would expect.
The cost of `catch` is a `Result` at every call site, which is why it is opt-in,
but an uncaught JS exception crossing wasm is unrecoverable.

#### `module` and `inline_js`

Importing from your own JavaScript rather than the globals.

```
use wasm_bindgen::prelude::*;

// The JS is embedded in the attribute, which the generator emits as a module.
#[wasm_bindgen(inline_js = "export function triple(x) { return x * 3; }")]
extern "C" {
    fn triple(x: i32) -> i32;
}

#[wasm_bindgen]
pub fn nine() -> i32 {
    triple(3)
}
```

**When to use it:** `inline_js` for a few lines that belong with the Rust that
calls them — a shim around an awkward API, a polyfill. For anything longer,
`#[wasm_bindgen(module = "/js/helpers.js")]` points at a real file, but note
that the path is **resolved at compile time**: a missing file is a build error,
not a run-time one, which is a useful property and an occasional surprise.

### Values crossing the boundary

#### `JsValue`

The opaque handle to anything JavaScript.

```
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn classify(value: &JsValue) -> String {
    if value.is_null() {
        "null".to_string()
    } else if value.is_undefined() {
        "undefined".to_string()
    } else if let Some(s) = value.as_string() {
        format!("string of {} chars", s.len())
    } else if let Some(n) = value.as_f64() {
        format!("number {n}")
    } else {
        "object".to_string()
    }
}

#[wasm_bindgen]
pub fn make_values() -> JsValue {
    let _text = JsValue::from_str("hello");
    let _flag: JsValue = true.into();
    let _nothing = JsValue::NULL;
    JsValue::from(42.0)
}
```

**When to use it:** when the type genuinely is not known — a value from
`JSON.parse`, a user-supplied callback argument. Reach for it last rather than
first: a `JsValue` parameter gives up every check the boundary could have done,
and `js_sys`' and `web_sys`' concrete types exist so you do not need one.
`as_string` and `as_f64` return `Option` because the value may be anything,
which is the honest signature.

#### `JsCast`

Converting between JavaScript types.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
pub fn as_js_string(value: &JsValue) -> bool {
    // Checked: Some only if the value really is a JS string.
    value.dyn_ref::<js_sys::JsString>().is_some()
}
```

**When to use it:** on anything from `web-sys`, constantly — a DOM lookup
returns `Element`, and reading `.value` needs an `HtmlInputElement`.
`dyn_ref`/`dyn_into` check and return `Option`/`Result`; `unchecked_ref` and
`unchecked_into` skip the check, and are undefined behaviour in the JS sense if
you are wrong. Prefer the checked form until a profile says the check matters.

#### `Closure`

Handing a Rust function to JavaScript as a callback.

```
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn make_doubler() -> JsValue {
    let closure = Closure::<dyn Fn(i32) -> i32>::new(|x: i32| x * 2);

    let as_js_function = closure.as_ref().clone();

    // Without this the closure is dropped here and JS gets a dangling handle.
    closure.forget();

    as_js_function
}
```

**When to use it:** event listeners, timers, anything JS calls back into. The
lifetime problem is the whole difficulty: JS holds the function after the Rust
scope ends, so the `Closure` must outlive it. `forget` leaks it deliberately,
which is correct for a listener that lives as long as the page and a leak for
anything else — the alternative is to store the `Closure` in a struct whose
`Drop` deregisters the callback. Note the explicit `dyn Fn(..)` type: the macro
needs the signature spelled out, since a Rust closure's type does not name it.
