---
title: "await"
kind: keyword
embedded_support: partial
groups: ["Concurrency & Async"]
related_concepts: ["Async/await", "Futures"]
related_syntax: ["async"]
see_also: ["async"]
---

## Explanation

`await` is a keyword, but it is not written like one: `expr.await` places
it after a `.`, in postfix/method-call position, rather than before the
expression it acts on the way `if`, `for`, `return`, and almost every
other keyword in Rust are written. There is no `await expr` form. This
unusual position exists so `.await` chains fluently the same way method
calls do — `fetch_order(id).await?.total_cents()` reads left to right —
and it's why `.await` is often described as a "postfix keyword-operator"
rather than an ordinary keyword or an ordinary method. It is not sugar
for a real method call, either: `await` is a full reserved keyword (since
the 2018 edition), so no type can define an actual method named `await`;
the compiler recognizes `.await` as its own grammatical form, distinct
from `.method_name()` syntax.

`expr.await` is only legal directly inside the body of an `async fn` or
an `async` / `async move` block (see [`async`](async.md)) — writing it
anywhere else (a plain `fn`, an `impl Iterator::next`, module-level code)
is a compile error: "await is only allowed inside async functions and
blocks." `expr` must be a value whose type implements `Future`, or, since
Rust 1.64, `IntoFuture` — the standard library blanket-implements
`IntoFuture` for every `Future`, so ordinary futures still work
unchanged.

Mechanically, `expr.await` polls the future `expr` evaluates to. If that
poll returns `Poll::Ready(value)`, the whole `expr.await` expression
evaluates to `value` and execution continues on the next line, exactly as
if it were an ordinary synchronous call. If the poll returns
`Poll::Pending`, the enclosing `async fn`/block suspends right there —
control returns to whatever is driving it, one level up — and that driver
is free to make progress on other work in the meantime. Crucially, this
suspension only ever gives up the *task*, never the OS thread: the thread
that was running the suspended task is immediately free to run other
tasks, and the suspended one resumes from exactly this `.await` point once
it's woken and polled again. See [Futures](../../concepts/concurrency-async/futures.md)
for what `poll`/`Future` actually are, and
[Async/await](../../concepts/concurrency-async/async-await.md) for when
reaching for this suspension model is the right call in the first place.

## Usage examples

### Awaiting a future with `.await`

```
async fn fetch_greeting() -> String {
    String::from("hello")
}

async fn greet() {
    let greeting = fetch_greeting().await; // <- `.await`: postfix position, suspends `greet` here until ready
    println!("{greeting}");
}
```

### Async tasks

Fetching a product's price and its stock count from two independent
services shouldn't be written as two back-to-back `.await`s when the
calls don't depend on each other — each `.await` there pays its own
latency in full, one after the other, when both requests could be in
flight at once.

```
// [dependencies] tokio = { version = "1", features = ["full"] }
use std::time::Duration;

async fn fetch_price_cents(sku: &str) -> u32 {
    tokio::time::sleep(Duration::from_millis(30)).await; // <- suspends this call only, thread stays free
    let _ = sku;
    2499
}

async fn fetch_stock_count(sku: &str) -> u32 {
    tokio::time::sleep(Duration::from_millis(20)).await;
    let _ = sku;
    42
}

#[tokio::main]
async fn main() {
    // AVOID: two sequential `.await`s serialize latencies that don't depend on each other
    let price = fetch_price_cents("sku-1").await; // <- .await #1 finishes fully before #2 even starts
    let stock = fetch_stock_count("sku-1").await; // <- .await #2

    // PREFER: run both futures concurrently, awaiting them together
    let (price2, stock2) = tokio::join!(fetch_price_cents("sku-1"), fetch_stock_count("sku-1"));
    println!("{price} {stock} {price2} {stock2}");
}
```

Each `.await` only suspends until its own future
resolves, so writing two of them back-to-back for independent work
serializes their latencies; `tokio::join!` polls both futures under one
concurrent point instead, finishing in roughly the slower call's time —
see [Futures](../../concepts/concurrency-async/futures.md) and the
[Tokio tutorial](https://tokio.rs/tokio/tutorial/select#join) for
`join!`'s own mechanics, which this page doesn't re-explain.

## Explanation (Embedded)

**Partial support.** `.await`'s grammar and its polling/suspension
semantics are core-language, resting only on `core::future::Future` —
none of that requires `std`. What's missing under `#![no_std]` is
anything to actually drive the poll loop: there's no `tokio` reactor
waking tasks on I/O readiness or timer expiry. The idiomatic embedded
substitute is an executor like `embassy`, whose own timer and peripheral
futures — `embassy_time::Timer::after` being the simplest example — are
exactly the kind of `Future` that `.await` suspends on. The postfix
`.await` syntax itself doesn't change at all; only what's on the other
end of it does.

## Usage examples (Embedded)

### Awaiting an embassy timer inside a task

```
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
async fn sample_sensor() {
    loop {
        // ... trigger an ADC conversion here
        Timer::after(Duration::from_millis(100)).await; // <- `.await`: suspends this task only; embassy's executor runs other tasks meanwhile
        // ... read the conversion result here
    }
}
```

`Timer::after(...).await` plays the same role
`tokio::time::sleep(...).await` played in the classic example — a
future that resolves once its duration has elapsed — just backed by
embassy's own timer queue instead of an OS-provided timer.
