---
title: "wasm-bindgen-futures"
version: "0.4.79"
publisher: "daxpedda, Guy Bedford (guybedford), wasm-bindgen-publish"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-13"
summary: "Joins Rust's `async` to JavaScript's promises: `JsFuture` makes a promise awaitable, `spawn_local` runs a future on the browser's event loop, and `future_to_promise` hands one back to JS."
domain: "WebAssembly & JS interop"
categories: ["wasm"]
repository: "https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures"
docs: "https://docs.rs/wasm-bindgen-futures/0.4.79/wasm_bindgen_futures/"
---

## Overview

Rust and JavaScript both have `async`, and neither understands the other's
version of it. A `Promise` is a JS object that calls you back; a Rust `Future` is
an inert value that something must poll. `wasm-bindgen-futures` is the adapter
in both directions, and it is small enough to describe in three names:

```
use wasm_bindgen_futures::{future_to_promise, spawn_local, JsFuture};
use wasm_bindgen::prelude::*;

// JS promise -> Rust future, so .await works.
pub async fn awaited(promise: js_sys::Promise) -> Result<JsValue, JsValue> {
    JsFuture::from(promise).await
}

// Rust future -> JS promise, to hand back across the boundary.
pub fn exported() -> js_sys::Promise {
    future_to_promise(async { Ok(JsValue::from(1)) })
}

// Rust future -> run it, caring only about side effects.
pub fn detached() {
    spawn_local(async {
        let _ = awaited(js_sys::Promise::resolve(&JsValue::from(1))).await;
    });
}
```

**There is no runtime here, and that is the point.** On `wasm32-unknown-unknown`
there is one thread and the host already owns the event loop, so there is
nothing for a [`tokio`](tokio.md)-style executor to schedule. `spawn_local`
hands a future to the *browser's* loop via a microtask, which is why the crate
is a few hundred lines rather than a runtime — and why `tokio`'s timers, I/O and
thread pool are not available to you. Futures here make progress when the host
gives the module control, and not otherwise. A `loop {}` waiting on a flag
blocks the page and deadlocks.

In practice you rarely name the crate, because **an exported `async fn` is
converted for you**:

```
use wasm_bindgen::prelude::*;

// JS sees a function returning a Promise. No explicit conversion needed.
#[wasm_bindgen]
pub async fn fetch_count() -> Result<u32, JsValue> {
    Ok(7)
}
```

The explicit functions are for the cases the attribute does not cover: awaiting
a promise you were *given*, and starting work from a non-async context such as
an event handler.

One structural note on 0.4.79: the crate has become mostly a re-export, with
`JsFuture`, `spawn_local` and `future_to_promise` now implemented in
[`js-sys`](js-sys.md) under `js_sys::futures`. Importing them from here still
works and remains the documented path, so nothing needs changing — it is worth
knowing only if you wonder why the source is so short. It is released in
lockstep with [`wasm-bindgen`](wasm-bindgen.md), MSRV is 1.81, and `no_std`
works by turning off the default `std` feature.

## When to use it

### Use case: Awaiting a browser API that returns a promise

The common case, and the reason most projects add the crate.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Response};

pub async fn fetch_body(url: &str) -> Result<String, JsValue> {
    let window = window().ok_or("no window")?;

    // fetch returns a Promise; JsFuture makes it awaitable.
    let response: Response = JsFuture::from(window.fetch_with_str(url))
        .await?
        .dyn_into()?;

    // So does .text(), so there are two awaits in one request.
    let body = JsFuture::from(response.text()?).await?;
    Ok(body.as_string().unwrap_or_default())
}
```

**Why it fits:** nearly every interesting browser API is promise-based — `fetch`,
the clipboard, `IndexedDB`, WebUSB, permissions — so without this the code
becomes `Closure`s chained through `.then()`, each one a lifetime problem. With
it, the sequence reads top to bottom and `?` propagates the failures. Note that
each promise needs its own `JsFuture::from`: the type converts one promise, it
does not make the whole API async.

### Use case: Starting async work from a synchronous callback

Event handlers cannot be `async`, so something must bridge them.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

async fn save(value: String) -> Result<(), JsValue> {
    let _ = value;
    Ok(())
}

#[wasm_bindgen]
pub fn on_submit(value: String) {
    // The handler returns immediately; the future runs on the event loop.
    spawn_local(async move {
        if let Err(error) = save(value).await {
            web_sys::console::error_1(&error);
        }
    });
}
```

**Why it fits:** a DOM listener has signature `FnMut(Event)` and the browser
will not await it, so any async work it starts has to be detached.
`spawn_local` is how, and the shape above shows the obligation that comes with
it: **nothing is watching the result**, so the future must handle its own
errors. A `spawn_local` whose body returns `Result` and ignores it is a silent
failure waiting to happen — hence the explicit `if let Err`.

### Use case: Giving JavaScript something to await

Exposing Rust async work across the boundary.

```
use wasm_bindgen::prelude::*;

// The attribute does the conversion: JS gets a Promise.
#[wasm_bindgen]
pub async fn compute(input: u32) -> Result<u32, JsValue> {
    if input == 0 {
        return Err(JsValue::from("input must be non-zero"));
    }
    Ok(input * 2)
}

// The explicit form, for a future you already have.
#[wasm_bindgen]
pub fn compute_promise(input: u32) -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move {
        Ok(JsValue::from(input * 2))
    })
}
```

**Why it fits:** the `async fn` export is what you want almost always — it is
shorter, it keeps real Rust types in the signature rather than `JsValue`, and
`Err` arrives as a rejected promise. `future_to_promise` is for when the future
comes from somewhere else, or when you need the `Promise` as a value to store
or pass on rather than return. Its signature is fixed at
`Result<JsValue, JsValue>`, which is the reason to prefer the attribute.

## API map

Four items, and the constraint they all operate under.

### Crossing in

#### `JsFuture`

Wraps a `Promise` as a `Future`.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub async fn resolve_and_reject() -> Result<JsValue, JsValue> {
    // A resolved promise yields its value...
    let value = JsFuture::from(js_sys::Promise::resolve(&JsValue::from(1))).await?;

    // ...and a rejected one becomes Err, carrying whatever JS threw.
    let failure = JsFuture::from(js_sys::Promise::reject(&JsValue::from("no"))).await;
    assert!(failure.is_err());

    Ok(value)
}
```

**When to use it:** on every promise you receive. The output is always
`Result<JsValue, JsValue>` — `Ok` for resolve, `Err` for reject — which means
two things in practice: the success value needs a cast or an `as_string` to
become a Rust type, and the error is whatever the JS side threw, with no type
guarantee at all. Awaiting the same `JsFuture` twice is not possible, since
`await` consumes it; wrap the promise again if you need to.

### Crossing out

#### `spawn_local`

Runs a future to completion on the host's event loop.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub fn start_work() {
    spawn_local(async {
        let result = Work::run().await;
        // Nothing receives this, so log rather than return.
        web_sys::console::log_1(&JsValue::from(result));
    });
}

struct Work;

impl Work {
    async fn run() -> u32 {
        42
    }
}
```

**When to use it:** from any synchronous context that needs to start async work
— an event handler, `#[wasm_bindgen(start)]`, a `Drop`. The future must be
`'static`, so anything it borrows has to be moved in, which is what `async move`
is for. It returns nothing: there is no join handle, no cancellation, and a
panic inside aborts the module. If you need the result, `future_to_promise` and
let JS await it instead.

#### `future_to_promise`

Converts a future into a `Promise`.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen]
pub fn deferred(fail: bool) -> js_sys::Promise {
    future_to_promise(async move {
        if fail {
            // Becomes a rejected promise.
            Err(JsValue::from("rejected"))
        } else {
            Ok(JsValue::from("resolved"))
        }
    })
}
```

**When to use it:** when you need the `Promise` itself — to store it, to hand it
to a JS API expecting one, or to return from a function that is not `async`.
The future must resolve to `Result<JsValue, JsValue>`, so Rust types get
converted at the end rather than in the signature. For a plain export, the
`async fn` attribute is better on every count. `future_to_promise_typed` is the
variant that preserves a concrete JS type in the generated TypeScript.

### The constraint

#### Single-threaded, host-driven execution

What is *not* available, which matters more than what is.

```
use wasm_bindgen_futures::spawn_local;

pub fn concurrency() {
    // Two futures can be in flight: both are handed to the event loop.
    spawn_local(async { let _ = 1; });
    spawn_local(async { let _ = 2; });

    // What cannot happen: parallelism, blocking, or a thread.
    // No tokio::time::sleep, no std::thread::spawn, no block_on.
}
```

**When to use it:** as the mental model for everything above. Concurrency works
— many futures can be awaiting promises at once — but parallelism does not,
because there is one thread, and blocking it stops the page. The practical
consequences: for a delay use `setTimeout` through
[`web-sys`](web-sys.md) rather than a `tokio` timer; for CPU work that would
block, a Web Worker is the only answer; and a library pulling in a `tokio`
runtime will not work here regardless of what its feature flags claim. The
crate does expose an experimental `tokio` module built on JSPI, but it is
explicitly not ready to depend on.
