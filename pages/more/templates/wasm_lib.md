<!--
  THE wasm_lib TEMPLATE'S OWN PAGE.

  What this template lays out and why it is arranged that way. Background on
  WebAssembly itself belongs in the WebAssembly section, not here, and the
  general case for templates belongs in pages/more/_templates.md.

  Same two rules as those files: no code fences, and links are relative paths
  to built pages (../../wasm/, ../../concepts/….html).
-->

A [WebAssembly](../../wasm/) module compiled from Rust has a boundary that
only carries numbers, so anything richer has to be marshalled across it.
`wasm-bindgen` generates that marshalling from an attribute, and the crate it
generates it for is an adapter: it converts between JavaScript's types and the
library's, and does nothing else.

The work therefore sits in a separate library crate, which the module crate
depends on by path. That split is what lets `cargo test` run the tests on the
host, in milliseconds, with no browser and no wasm target involved. A test of
the binding itself would need `wasm-bindgen-test` and a headless browser; the
adapter is kept thin enough that there is nothing in it worth testing that
way.

The module crate declares `crate-type = ["cdylib"]` and nothing else. An
`rlib` would be needed only if another Rust crate linked this one, or if it
carried [integration tests](../../concepts/testing-tooling/integration-tests.html),
which link a crate the way an outside dependant does. Unit tests inside
`src/` compile against a `cdylib` perfectly well.

Two prerequisites beyond cargo-generate, since this template targets wasm
rather than the host: the `wasm32-unknown-unknown` target, added with
`rustup target add wasm32-unknown-unknown`, and `wasm-pack`, installed with
`cargo install wasm-pack`. The generated README repeats both.

`wasm-pack build crates/<name>_wasm --target web --out-dir ../../pkg` writes
`pkg/` at the project root — the `.wasm`, the JavaScript that loads it, and
TypeScript definitions. The `web` target means no bundler and no Node: the
`index.html` beside `pkg/` imports the generated JavaScript directly and calls
the export.

Serving that page is not optional. It loads an ES module, and the module
fetches the `.wasm` next to it; opening the file from disk fails on both
counts. Any local HTTP server will do.

One deliberate omission: the panic hook that turns a Rust panic into a
readable console message is bound to `console.error` in eight lines rather
than taken from `console_error_panic_hook`. That crate is the usual choice and
is downloaded millions of times a month, but it has not been released since
2021 and sits in the archived rustwasm organisation. Binding it directly keeps
the dependency list to `wasm-bindgen` alone.
