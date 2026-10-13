---
title: "js-sys"
version: "0.3.106"
publisher: "Ingvar Stepanyan (RReverser), daxpedda, Guy Bedford (guybedford), wasm-bindgen-publish"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-13"
summary: "Rust bindings for every JavaScript standard built-in — `Array`, `Object`, `Map`, `Promise`, `JSON`, `Reflect`, the typed arrays. The ECMAScript library, declared once so you need not write the `extern` blocks yourself."
domain: "WebAssembly & JS interop"
categories: ["wasm"]
repository: "https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys"
docs: "https://docs.rs/js-sys/0.3.106/js_sys/"
---

## Overview

`js-sys` is one long `extern` block, written for you. Everything the ECMAScript
specification guarantees exists in any JavaScript host — `Array`, `Object`,
`Map`, `Set`, `Date`, `JSON`, `Math`, `Promise`, `Reflect`, `RegExp`, `Function`,
the typed arrays — is declared here as a Rust type with methods.

```
use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn build_list() -> Array {
    let items = Array::new();
    items.push(&JsValue::from("first"));
    items.push(&JsValue::from(2));

    assert_eq!(items.length(), 2);
    items
}
```

It sits directly on [`wasm-bindgen`](wasm-bindgen.md) and is purely
declarative: no logic, no algorithms, nothing but bindings. You could write
every one of them yourself in an `extern "C"` block — `js-sys` exists so that
nobody does, and so that everyone's `Array` is the same Rust type rather than
each crate inventing its own.

**The division of labour is worth getting straight.** `js-sys` covers the
*language*: things that exist in Node, in a browser, in a service worker, in any
engine. [`web-sys`](web-sys.md) covers the *browser* — `window`, `document`,
`fetch`, DOM elements, none of which the language specifies. If you are looking
for `console.log`, that is `web_sys::console`; if you are looking for
`Array.prototype.map`, it is here.

Two characteristics shape how the API feels. **Methods take `&self` and mutate**,
because that is what the underlying JS objects do — `Array::push` on a shared
reference is not a mistake. And **anything that can throw returns `Result`**,
which in JavaScript is a great deal: `JSON::parse`, `Reflect::get`, calling a
`Function`. There is no implicit propagation, so the `?` stays visible.

It is a thin binding layer, so the economics are simple: it only generates code
for what you call, MSRV is 1.81, and `no_std` works by turning off the default
`std` feature. The version tracks `wasm-bindgen`'s in lockstep — they are
released together from the same repository, and mixing versions does not work.

## When to use it

### Use case: Working with JavaScript values whose shape you do not control

Data arriving from JS, where the types are whatever the caller sent.

```
use js_sys::{Array, Reflect};
use wasm_bindgen::prelude::*;

// The caller passes an array of objects; pull one field out of each.
#[wasm_bindgen]
pub fn collect_names(records: &Array) -> Vec<String> {
    let key = JsValue::from("name");
    let mut names = Vec::new();

    for index in 0..records.length() {
        let record = records.get(index);
        if let Ok(value) = Reflect::get(&record, &key) {
            if let Some(name) = value.as_string() {
                names.push(name);
            }
        }
    }
    names
}
```

**Why it fits:** a JS caller will hand you an object with a shape no Rust type
describes, and `Reflect` is the general-purpose way to read it — property access
as a method call rather than a field. The `Result` and the `Option` are both
load-bearing: the property may be missing, and if present it may not be a
string. For data with a known shape, `serde-wasm-bindgen` deserialises straight
into a Rust struct and is much pleasanter; `Reflect` is for when there is no
known shape to deserialise into.

### Use case: Moving bulk binary data across the boundary

Typed arrays are how bytes cross without going through JS values one at a time.

```
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn invert(input: &Uint8Array) -> Uint8Array {
    // Copy into Rust memory, work there, copy back out.
    let mut bytes = vec![0u8; input.length() as usize];
    input.copy_to(&mut bytes);

    for byte in bytes.iter_mut() {
        *byte = 255 - *byte;
    }

    let output = Uint8Array::new_with_length(bytes.len() as u32);
    output.copy_from(&bytes);
    output
}
```

**Why it fits:** image pixels, audio samples, a file the user dropped — the
quantity makes per-value conversion untenable, and `copy_to`/`copy_from` are a
single `memcpy` each. `Uint8Array` and its siblings (`Float32Array`,
`Int32Array`, and the rest) are the common currency for this, and most browser
APIs that produce or consume buffers are typed in terms of them. Both copies
are real, so for very large buffers it is worth knowing you can instead view
wasm memory directly — at the cost of the view being invalidated whenever the
wasm heap grows.

### Use case: Receiving and returning promises

`Promise` is the type JS async APIs speak.

```
use js_sys::Promise;
use wasm_bindgen::prelude::*;

// A value a JS caller can await.
#[wasm_bindgen]
pub fn ready() -> Promise {
    Promise::resolve(&JsValue::from("done"))
}

#[wasm_bindgen]
pub fn failed() -> Promise {
    Promise::reject(&JsValue::from("nope"))
}
```

**Why it fits:** an exported function returning a `Promise` is awaitable from
JS, which is what any asynchronous interface needs. What `js-sys` does *not*
give you is a way to `.await` one from Rust — a `Promise` is an opaque JS
object here, not a `Future`. That conversion is
[`wasm-bindgen-futures`](wasm-bindgen-futures.md)'s job, and in practice you
will want both crates together.

## API map

The crate mirrors the ECMAScript built-ins, so the names are the JavaScript
names and the reference is MDN. The entries below cover the types that come up
constantly and the two or three places the Rust shape differs from what you
would guess.

### Collections

#### `Array`

JavaScript's array.

```
use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn arrays() -> Array {
    // Built up, or constructed from a fixed number of values.
    let dynamic = Array::new();
    dynamic.push(&JsValue::from(1));

    let fixed = Array::of3(&JsValue::from(1), &JsValue::from(2), &JsValue::from(3));

    let _text = fixed.join(",");
    let _doubled = fixed.map(&mut |value, _index, _array| {
        JsValue::from(value.as_f64().unwrap_or(0.0) * 2.0)
    });
    let _first = fixed.get(0);

    fixed
}
```

**When to use it:** whenever a JS caller passes a list or expects one. Note
`push` takes `&self` — the JS array is mutated through a shared reference,
because the Rust value is only a handle. `of1` through `of5` exist because a
variadic constructor cannot be expressed; past five, build it with `new` and
`push`. The adaptor methods (`map`, `filter`, `for_each`) take `&mut` closures
and run in JS, so they are rarely worth it over iterating in Rust.

#### `Object` and `Reflect`

Property access on arbitrary values.

```
use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn build_record() -> Result<Object, JsValue> {
    let record = Object::new();

    Reflect::set(&record, &JsValue::from("id"), &JsValue::from(7))?;
    Reflect::set(&record, &JsValue::from("label"), &JsValue::from("seven"))?;

    let has_id = Reflect::has(&record, &JsValue::from("id"))?;
    assert!(has_id);

    let _keys = Object::keys(&record);
    let _pairs = Object::entries(&record);

    Ok(record)
}
```

**When to use it:** building an object to hand back to JS, or reading one whose
fields you only know by name. `Reflect`'s methods are associated functions
taking the target first, mirroring the JS `Reflect` namespace, and every one
returns `Result` because a proxy can throw on property access. `Object::keys`
and `entries` give you an `Array` to walk when you do not know the field names
at all.

#### `Map` and `Set`

The keyed collections, when a plain object will not do.

```
use js_sys::{Map, Set};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn collections() -> u32 {
    let map = Map::new();
    map.set(&JsValue::from("a"), &JsValue::from(1));
    map.set(&JsValue::from("b"), &JsValue::from(2));

    let set = Set::new(&JsValue::UNDEFINED);
    set.add(&JsValue::from("x"));
    set.add(&JsValue::from("x")); // a duplicate, so size stays 1

    map.size() + set.size()
}
```

**When to use it:** when keys are not strings, when insertion order must be
preserved, or when you are handed one. For data that stays inside Rust, a
`HashMap` is faster and better typed — these are for the boundary, not for
storage. `Set::new` takes an argument because the JS constructor accepts an
iterable; pass `UNDEFINED` for an empty one.

### Language services

#### `JSON`

Parsing and serialising, as JS does it.

```
use js_sys::JSON;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn reformat(text: &str) -> Result<String, JsValue> {
    // parse throws on malformed input, hence the Result.
    let value = JSON::parse(text)?;
    let normalised = JSON::stringify(&value)?;
    Ok(String::from(normalised))
}
```

**When to use it:** when the JSON is going to or coming from JS anyway, so
`JSON::parse` hands you a `JsValue` the rest of your JS interop already speaks.
If the data is destined for Rust types, [`serde_json`](serde_json.md) is the
better tool — it parses straight into your struct without a trip through the JS
heap. `stringify` returns a `JsString`, which `String::from` brings into Rust.

#### `Function` and `Reflect::apply`

Calling a JavaScript function you were given.

```
use js_sys::Function;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn invoke(callback: &Function, value: i32) -> Result<JsValue, JsValue> {
    // call0 through call3 cover the common arities; `this` comes first.
    callback.call1(&JsValue::NULL, &JsValue::from(value))
}
```

**When to use it:** for a callback parameter, which is how a great deal of JS
API surface is shaped. `call0`/`call1`/`call2`/`call3` cover fixed arities and
`apply` takes an `Array` for anything longer. The first argument is `this` —
pass `JsValue::NULL` when it does not matter, which is usually. The `Result`
reflects that the function may throw, and it is a real possibility for code you
did not write.

#### `JsString` and `Number`

The primitive wrappers, and getting back to Rust types.

```
use js_sys::{JsString, Number};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn primitives() -> String {
    let text = JsString::from("Hello");
    let _upper = text.to_upper_case();
    let _utf16_len = text.length();

    let number = Number::from(3.14159);
    let _rounded = number.to_fixed(2);

    // JsString -> String copies and converts UTF-16 to UTF-8.
    String::from(text)
}
```

**When to use it:** rarely by choice — a `&str` parameter or `String` return on
an exported function converts automatically, which is almost always what you
want. Reach for `JsString` when you are holding a string as a `JsValue` and want
JS string methods on it, or when you want to avoid the UTF-16 to UTF-8
conversion that crossing the boundary costs. `length` is a UTF-16 code unit
count, not a character count and not a byte count.

### Binary data and async

#### Typed arrays

`Uint8Array` and friends, for bulk numeric data.

```
use js_sys::{Float32Array, Uint8Array};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn buffers(input: &Uint8Array) -> Uint8Array {
    let mut bytes = vec![0u8; input.length() as usize];
    input.copy_to(&mut bytes);

    let output = Uint8Array::new_with_length(bytes.len() as u32);
    output.copy_from(&bytes);

    // from copies a Rust slice in one step, allocating a new JS array.
    let _floats = Float32Array::from(&[1.0f32, 2.0, 3.0][..]);

    output
}
```

**When to use it:** any time more than a handful of numbers crosses the
boundary. `copy_to` and `copy_from` are the workhorses and each is one bulk
copy; `from` is the convenience that allocates. There is one trap worth
remembering: a typed array constructed as a *view* onto wasm memory is
invalidated when the wasm heap grows, so never hold one across a call that might
allocate.

#### `Promise`

The JS async primitive.

```
use js_sys::Promise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn resolved() -> Promise {
    Promise::resolve(&JsValue::from(42))
}

#[wasm_bindgen]
pub fn rejected() -> Promise {
    Promise::reject(&JsValue::from("failed"))
}
```

**When to use it:** as the return type of an export JS will `await`, and as the
parameter type when JS hands you one. `then`, `catch` and `finally` exist, but
chaining them from Rust means building `Closure`s and managing their lifetimes —
converting the promise into a Rust `Future` with
[`wasm-bindgen-futures`](wasm-bindgen-futures.md) and using `.await` is both
shorter and harder to get wrong.

#### `Date` and `Math`

The remaining globals worth naming.

```
use js_sys::{Date, Math};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn now_and_then() -> f64 {
    let date = Date::new_0();
    let _year = date.get_full_year();

    let _random = Math::random();

    // Milliseconds since the epoch, which is the one you usually want.
    Date::now()
}
```

**When to use it:** `Date::now` for a timestamp, because `std::time::SystemTime`
is unavailable on `wasm32-unknown-unknown` — this is the way to read the clock.
`Math::random` likewise, when you do not want to pull in a full RNG; note it is
not seedable and not suitable for anything security-sensitive. For real date
arithmetic, do it in Rust with a date crate rather than through these bindings.
