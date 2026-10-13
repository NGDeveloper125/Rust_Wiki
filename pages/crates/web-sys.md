---
title: "web-sys"
version: "0.3.106"
publisher: "Ingvar Stepanyan (RReverser), daxpedda, Guy Bedford (guybedford), wasm-bindgen-publish"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-13"
summary: "Bindings for every browser API — the DOM, `fetch`, canvas, storage, events, WebGL — generated from the web platform's own WebIDL. Each type sits behind its own Cargo feature, and none are on by default."
domain: "WebAssembly & JS interop"
categories: ["wasm"]
repository: "https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys"
docs: "https://wasm-bindgen.github.io/wasm-bindgen/api/web_sys/"
---

## Overview

`web-sys` is the browser, as Rust types. `Window`, `Document`, `Element`,
`fetch`, `localStorage`, canvas, WebGL, WebRTC, the clipboard — all of it,
generated mechanically from the WebIDL files that specify the web platform. That
generation is the crate's defining fact, and it explains everything else about
how it feels to use.

**Start with the thing that catches everyone: every type is behind its own
Cargo feature, and nothing is enabled by default.** The crate declares over
1700 of them. A fresh dependency gives you an empty namespace, and the error
reads as though the crate were broken:

```
// With `features = ["Window", "Document", "Element"]` in Cargo.toml:
use web_sys::{window, Document, Element};

// Without them, this same line is:
//   error[E0432]: unresolved import `web_sys::Document`
//   no `Document` in the root
```

The gating exists because enabling everything would mean compiling bindings for
the entire web platform on every build. So the workflow is: write the code,
read the compiler error, add the feature, repeat. Features are named exactly
after the WebIDL interface — `HtmlInputElement`, `CanvasRenderingContext2d`,
`console` — and docs.rs lists the required feature on every item's page.

Three consequences of being generated, all worth expecting before you meet them:

- **Nullable becomes `Option`, throwing becomes `Result`.** `window()` returns
  `Option<Window>` because there is no window in a worker. `document()` is
  `Option` too. So reaching the body is a chain of fallible steps, which is
  honest about a platform where any of them can be absent.
- **Overloads become suffixed method names.** JavaScript's one `fetch` taking
  different argument shapes becomes `fetch_with_str`,
  `fetch_with_request_and_init` and so on. The names are long and the right one
  is found by reading the list.
- **Everything needs casting.** `get_element_by_id` returns `Option<Element>`,
  because that is what the IDL says, and `Element` has no `.value()`. Reaching
  an input's text means a `dyn_into::<HtmlInputElement>()`. This is constant,
  and `wasm_bindgen::JsCast` must be in scope for it.

The crate is not ergonomic and does not try to be — it is a complete, correct,
low-level surface that higher-level crates build on. If you want something that
feels like Rust, a framework such as `leptos`, `yew` or `dioxus` sits on top of
this. Use `web-sys` directly when you want a specific browser API, not an
application framework.

It is part of the [`wasm-bindgen`](wasm-bindgen.md) family and released in
lockstep with it; [`js-sys`](js-sys.md) covers the JavaScript language where
this covers the browser. MSRV is 1.81.

## When to use it

### Use case: Reading from and writing to the DOM

The baseline: find an element, change it, add one.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Document, HtmlInputElement};

#[wasm_bindgen]
pub fn greet_from_field() -> Result<String, JsValue> {
    // Each step is fallible, because in some contexts each is absent.
    let document: Document = window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    // Element has no .value(); HtmlInputElement does.
    let name = document
        .get_element_by_id("name")
        .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_else(|| "world".to_string());

    let greeting = document.create_element("p")?;
    greeting.set_text_content(Some(&format!("Hello, {name}!")));
    document.body().ok_or("no body")?.append_child(&greeting)?;

    Ok(name)
}
```

**Why it fits:** this is what the crate is for, and the shape of it is
representative — an `Option` chain to reach the document, a cast to get a
concrete element type, and `Result` on anything the DOM can refuse. Requires
features `Window`, `Document`, `Element`, `HtmlElement` and
`HtmlInputElement`. For a whole application this is a lot of ceremony per
interaction, which is the argument for a framework; for one targeted piece of
DOM work it is exactly right.

### Use case: Fetching over the network

`fetch` from Rust, awaited rather than chained.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Request, RequestInit, Response};

pub async fn get_text(url: &str) -> Result<String, JsValue> {
    let options = RequestInit::new();
    options.set_method("GET");
    let request = Request::new_with_str_and_init(url, &options)?;

    let response_value = JsFuture::from(
        window().ok_or("no window")?.fetch_with_request(&request),
    )
    .await?;

    // fetch resolves to a JsValue, so the concrete type needs a cast.
    let response: Response = response_value.dyn_into()?;

    // A 404 is a successful fetch: ok() is the check that matters.
    if !response.ok() {
        return Err(JsValue::from(format!("status {}", response.status())));
    }

    let text = JsFuture::from(response.text()?).await?;
    Ok(text.as_string().unwrap_or_default())
}
```

**Why it fits:** `fetch` is the one browser API nearly every wasm module needs,
and doing it through `web-sys` means no JS shim and no extra dependency beyond
[`wasm-bindgen-futures`](wasm-bindgen-futures.md) for the `.await`. The detail
worth internalising is the one on the third-to-last block: **`fetch` only
rejects on network failure**, so an HTTP error status arrives as a perfectly
successful response and `response.ok()` is what separates them. Requires
features `Window`, `Request`, `RequestInit` and `Response`.

### Use case: Responding to events

Attaching a Rust function as a DOM listener.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, MouseEvent};

#[wasm_bindgen]
pub fn wire_up() -> Result<(), JsValue> {
    let document = window().ok_or("no window")?.document().ok_or("no doc")?;
    let button = document.create_element("button")?;

    // The closure must outlive this function, since the DOM holds it.
    let on_click = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| {
        console::log_1(&JsValue::from(format!("clicked at {}", event.client_x())));
    });

    button.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;

    // Deliberately leaked: the listener lives as long as the page.
    on_click.forget();
    Ok(())
}
```

**Why it fits:** every interactive module needs this, and the memory question is
unavoidable rather than incidental. The DOM keeps a reference to the function
after the Rust scope has ended, so the `Closure` has to be kept alive — and
`forget` is the blunt way to do it. That is correct for a listener attached for
the lifetime of the page, and a leak for a listener on an element you will
remove, where the alternative is to store the `Closure` alongside the element
and drop both together. Requires features `Window`, `Document`, `Element`,
`EventTarget`, `MouseEvent` and `console`.

## API map

The crate is far too large to map. What follows is the handful of entry points
everything else is reached through, plus the conventions that let you find a
method you have not seen before.

### Entry points

#### `window` and `Document`

The root of everything in a page.

```
use wasm_bindgen::prelude::*;
use web_sys::{window, Document, Window};

#[wasm_bindgen]
pub fn page_info() -> Result<String, JsValue> {
    let win: Window = window().ok_or("no window")?;
    let doc: Document = win.document().ok_or("no document")?;

    let title = doc.title();
    let _url = doc.url()?;
    let _width = win.inner_width()?;

    Ok(title)
}
```

**When to use it:** first, in almost any browser code. `window()` is a free
function rather than a constant because it returns `Option` — in a web worker or
a service worker there is no `Window`, and that is the honest answer rather than
a panic. Cache it in a local if you need it repeatedly; each call is a JS
property lookup.

#### `console`

Logging, which `println!` cannot do here.

```
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn log_things() {
    console::log_1(&JsValue::from("a message"));
    console::warn_1(&JsValue::from("a warning"));
    console::error_1(&JsValue::from("a problem"));

    // The suffix is the argument count, matching console.log(a, b).
    console::log_2(&JsValue::from("value:"), &JsValue::from(42));
}
```

**When to use it:** for ad-hoc debugging. `println!` on
`wasm32-unknown-unknown` writes to a stdout that does not exist, so this is the
replacement, and the `_1`/`_2`/`_3` suffixes are the generator's way of
expressing a variadic function. For anything beyond debugging, wire the
[`log`](log.md) crate to a wasm backend so your logging statements are not
browser-specific. Install `console_error_panic_hook` too, or a Rust panic
reports only "unreachable executed".

### Conventions

#### Casting with `JsCast`

How to get from the IDL's type to the one with the method you want.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlInputElement};

#[wasm_bindgen]
pub fn read_input(id: &str) -> Option<String> {
    let document = window()?.document()?;
    let element = document.get_element_by_id(id)?;

    // Checked: None if the element is not actually an input.
    let input: HtmlInputElement = element.dyn_into().ok()?;
    Some(input.value())
}
```

**When to use it:** constantly. The IDL gives each method the most general
return type it can, so `get_element_by_id` yields `Element` and every concrete
capability is one cast away. Use `dyn_into`/`dyn_ref`, which check and return
`Result`/`Option`; `unchecked_into` skips the check and is only right when the
value provably has that type — as with a `Closure` being handed to
`add_event_listener_with_callback`, where the target type is a JS function by
construction.

#### Suffixed methods for overloads

Reading the long names.

```
use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit};

#[wasm_bindgen]
pub fn build_request(url: &str) -> Result<Request, JsValue> {
    // new_with_str: the overload taking just a URL string.
    let simple = Request::new_with_str(url)?;
    let _ = simple;

    // new_with_str_and_init: the one taking a URL and an options object.
    let options = RequestInit::new();
    options.set_method("POST");
    Request::new_with_str_and_init(url, &options)
}
```

**When to use it:** whenever the obvious short name does not exist. The rule is
mechanical — the suffix lists the argument types in order, so JavaScript's
`new Request(url, init)` becomes `new_with_str_and_init`. Scanning the docs.rs
method list for the suffix matching your arguments is faster than guessing.
Note that `RequestInit` and the other dictionary types take `&self` setters in
current versions, having previously been chained builders; older examples will
not compile.

#### `Storage`

Persisting data in the browser.

```
use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen]
pub fn remember(key: &str, value: &str) -> Result<Option<String>, JsValue> {
    // Two Results deep: local_storage() can throw, and may be None.
    let storage = window()
        .ok_or("no window")?
        .local_storage()?
        .ok_or("storage unavailable")?;

    storage.set_item(key, value)?;
    storage.get_item(key)
}
```

**When to use it:** small amounts of durable state — a preference, a session
token, a draft. The double fallibility is not over-engineering: `local_storage`
throws when a page's cookies are blocked, and `set_item` throws when the quota
is exceeded, both of which happen in real browsers. Values are strings only, so
anything structured goes through [`serde_json`](serde_json.md) first. Requires
features `Window` and `Storage`.

#### Canvas and `CanvasRenderingContext2d`

Drawing, as a representative of the larger APIs.

```
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub fn draw(canvas_id: &str) -> Result<(), JsValue> {
    let document = window().ok_or("no window")?.document().ok_or("no doc")?;

    let canvas: HtmlCanvasElement = document
        .get_element_by_id(canvas_id)
        .ok_or("no canvas")?
        .dyn_into()?;

    // get_context returns Option<Object>: the context type may be unsupported.
    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or("no 2d context")?
        .dyn_into()?;

    context.set_fill_style_str("#336699");
    context.fill_rect(10.0, 10.0, 100.0, 50.0);
    Ok(())
}
```

**When to use it:** for custom rendering, and as a template for how the bigger
APIs look — two casts, an `Option` for a capability that may be missing, and
`f64` everywhere because that is what JS numbers are. WebGL, WebAudio and the
rest follow the same pattern at greater length. Requires features `Window`,
`Document`, `HtmlCanvasElement` and `CanvasRenderingContext2d`.
