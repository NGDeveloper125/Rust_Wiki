---
title: "Attribute-like macros"
area: "Macros & Metaprogramming"
embedded_support: full
groups: ["Macros & Metaprogramming", "Declarative / Metaprogramming", "Generating Code / Metaprogramming", "Macros & Code Generation"]
related_syntax: ["#[proc_macro] / #[proc_macro_derive(...)] / #[proc_macro_attribute]", "#[meta] / #![meta]"]
see_also: ["Declarative macros", "Procedural macros", "Derive macros", "Function-like macros"]
---

## Explanation

An attribute-like macro is a function marked `#[proc_macro_attribute]`
with the signature `fn(TokenStream, TokenStream) -> TokenStream`. The
first argument is whatever was written inside the attribute itself
(`GET, "/orders"` in `#[route(GET, "/orders")]`); the second is the full
item the attribute is attached to — a function, a struct, a module.
Unlike a [derive macro](derive-macros.md), which can only add code
alongside an item, an attribute-like macro's return value entirely
*replaces* the annotated item — return an empty token stream and the
item vanishes from the compiled output.

This is the mechanism behind DSL-flavored annotations that frameworks
build on top of ordinary Rust syntax: a routing attribute that turns a
plain `async fn` into a registered HTTP handler, `#[tokio::test]`
turning an `async fn` into something the ordinary `#[test]` harness can
run, or a state-machine attribute that wraps guard code around a
function body. Anything an attribute-like macro expresses could, in
principle, also be expressed by taking a closure or a builder call —
but the attribute form keeps the call site reading like plain,
undecorated Rust (a function, a struct), with the transformation
declared once at the top rather than threaded through every call site.

An attribute-like macro sits between the other two procedural kinds:
[derive macros](derive-macros.md) only add code and never take their own
arguments; [function-like macros](function-like-macros.md) don't attach
to an existing item at all, only to a bare `name!(...)` expression.
Attribute-like macros are the only one of the three that both receive
their own arguments *and* get to rewrite an entire existing item, which
makes them the most powerful — and the easiest to misuse — of the three.

Like every procedural macro (see [Procedural macros](procedural-macros.md)
for the shared crate-splitting rule), a `#[proc_macro_attribute]`
function must be defined in its own `proc-macro = true` crate, separate
from every crate that applies it. Because the macro can return literally
anything, a bug in one can silently delete the function it was attached
to (by returning nothing) or duplicate it — testing what the macro
actually produces matters more here than for the less powerful macro
kinds.

## Basic usage example

The attribute-like macro's definition, in its own `proc-macro = true`
crate:

```
use proc_macro::TokenStream;

#[proc_macro_attribute] // <- takes the attribute's own arguments AND the item it annotates, as two streams
pub fn traced(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item // a real implementation would parse `item` (e.g. with `syn`) and splice in new code
}
```

and the consuming crate that applies it:

```
// Cargo.toml: traced_macro = { path = "../traced_macro" }
use traced_macro::traced;

#[traced] // <- the attribute macro replaces this function's tokens with whatever `traced` returns
fn poll_sensor() -> f64 {
    21.5
}

fn main() {
    println!("{}", poll_sensor());
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A web framework attaches routing metadata directly to a handler
function with an attribute macro, so the route table lives next to the
code it dispatches to instead of in a separate registration file.

```
// crate "webkit" defines `route` as a #[proc_macro_attribute]
use webkit::{route, Router};

#[route(GET, "/sensors/:id")] // <- replaces this fn's tokens with a registered handler + metadata
async fn get_sensor(id: u32) -> String {
    format!("sensor {id}")
}

#[route(POST, "/sensors")] // <- same macro, different metadata: registers a second handler at compile time
async fn create_sensor(payload: String) -> String {
    format!("created: {payload}")
}

fn app() -> Router {
    Router::new() // <- both #[route]-annotated functions are already registered by the time this runs
}
```

**Why this way:** attaching the method and path as compile-time metadata
on the handler itself keeps the router's source of truth in one place
per endpoint rather than a separately maintained table that can drift
out of sync — the same shape actix-web's and axum-adjacent routing
attribute macros use, which
[Effective Rust](https://effective-rust.com/) frames as the strongest
case for reaching for an attribute macro over a plain function call.

### Scenario: Testing

A parameterized-test attribute macro expands one annotated function
into several ordinary `#[test]` functions, one per listed case, instead
of requiring a `macro_rules!` invocation per case.

```
// crate "testkit" defines `cases` as a #[proc_macro_attribute]
use testkit::cases;

#[cases(20.0, 21.5, -5.0)] // <- expands into one #[test] fn per listed value
fn parses_valid_reading(input: f64) {
    assert!(parse_reading(input).is_some());
}

fn parse_reading(celsius: f64) -> Option<f64> {
    (-40.0..=125.0).contains(&celsius).then_some(celsius)
}
```

**Why this way:** each value the macro generates a test for is still an
ordinary `#[test]` the harness discovers on its own, matching the
[Rust Book's testing conventions](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
— the attribute only removes the copy-pasted case-by-case boilerplate,
not the tests themselves.

## Explanation (Embedded)

An attribute-like macro's transformation happens entirely on the host
during compilation, the same host/target split every procedural macro
follows (see [Procedural macros](procedural-macros.md)) — but embedded
Rust leans on this specific macro kind for something genuinely
load-bearing: turning an ordinary function or module into the exact code
the hardware's startup and interrupt-dispatch conventions require.
`cortex-m-rt`'s `#[entry]` rewrites a plain `fn main() -> !` into the
function the reset handler actually calls, enforcing at compile time
that there is exactly one entry point and that it never returns;
`#[interrupt]` does the analogous rewrite for a specific interrupt
vector, wiring the annotated function into the vector table under the
compiler's control rather than the programmer's. RTIC's `#[app]` goes
further still, expanding an entire annotated module into the scheduling,
resource-locking (`#[shared]`/`#[local]`), and task-dispatch code an RTIC
application runs on — code that would be substantial, error-prone
boilerplate to hand-write per project. In every case the expansion is
ordinary, already-`no_std`-compatible Rust; nothing about the macro
itself needs a heap or `std`, and the transformation is exactly as total
as the classic Explanation describes — return the wrong tokens and the
annotated function or module simply isn't there anymore, which is why
these crates test their macro output carefully rather than trusting it
by inspection alone.

## Basic usage example (Embedded)

```
#![no_std]
#![no_main]

use cortex_m_rt::entry;

#[entry] // <- rewrites this function into the shape the reset handler calls; enforces exactly one entry point
fn main() -> ! {
    loop {}
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A firmware project with several interrupt-driven tasks and shared state
uses RTIC's `#[app]` to get resource-locking and scheduling generated
from a declarative module, instead of hand-writing a critical-section-
guarded dispatcher.

```
#[rtic::app(device = pac)] // <- expands this whole module into scheduling + resource-locking code
mod app {
    #[shared]
    struct Shared {
        counter: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        (Shared { counter: 0 }, Local {})
    }

    #[task(binds = TIM2, shared = [counter])]
    fn on_timer(mut cx: on_timer::Context) {
        cx.shared.counter.lock(|c| *c += 1); // <- lock/scheduling code is macro-generated, not hand-written
    }
}
```

**Why this way:** the critical-section-guarded dispatcher this expands
into is exactly the code the [RTIC book](https://rtic.rs/) argues is
harder to get right, and harder to review, by hand than the declarative
`#[shared]`/`#[task]` module the macro expands from — the attribute
macro's job is precisely to keep that generated dispatcher consistent as
tasks and resources are added.

### Scenario: Testing

An `#[entry]`-annotated function is awkward to unit test directly — it
never returns and assumes real hardware — so firmware logic is kept in
ordinary functions the macro-generated entry point calls into.

```
#![no_std]
#![no_main]

use cortex_m_rt::entry;

fn compute_next_state(count: u32) -> u32 { // <- plain function: testable with ordinary #[test] on the host
    count.wrapping_add(1)
}

#[entry] // <- stays a thin wrapper; the logic worth testing lives outside it
fn main() -> ! {
    let mut count = 0;
    loop {
        count = compute_next_state(count);
    }
}

#[cfg(test)]
mod tests {
    use super::compute_next_state;

    #[test]
    fn wraps_on_overflow() {
        assert_eq!(compute_next_state(u32::MAX), 0);
    }
}
```

**Why this way:** the attribute macro's rewritten entry point only ever
runs on target hardware, so the
[Rust Book's testing guidance](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
can only reach it if the logic worth checking is factored into plain
functions the host's own test harness can call directly.
