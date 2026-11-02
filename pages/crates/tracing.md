---
title: "tracing"
version: "0.1.44"
publisher: "Carl Lerche (carllerche), Eliza Weisman (hawkw), Publish tracing"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-11-02"
summary: "Structured, context-aware instrumentation. Events carry typed fields instead of a formatted sentence, and spans mark the operation they happened inside — so a log line from a shared helper still says which request it belongs to."
domain: "Logging, tracing & metrics"
categories: ["asynchronous", "development-tools::debugging", "development-tools::profiling"]
repository: "https://github.com/tokio-rs/tracing"
docs: "https://docs.rs/tracing/0.1.44/tracing/"
---

## Overview

A log line tells you something happened. In a server handling forty requests at
once, that is half the information you need — `cache miss` is useless unless you
know whose request missed. `tracing` adds the other half: a **span** marks a
period of work, and every event recorded while that span is active carries it.

```
fn handle(id: u32, path: &str) {
    let span = tracing::info_span!("request", id, path);
    let _guard = span.enter();

    tracing::info!("received");
    load(id);
    tracing::info!(status = 200, "responded");
}

// A helper that knows nothing about requests.
#[tracing::instrument]
fn load(id: u32) {
    tracing::warn!(cache = "miss", "falling back to disk");
}

fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .init();

    handle(7, "/users");
}

// INFO request{id=7 path="/users"}: received
// WARN request{id=7 path="/users"}:load{id=7}: falling back to disk cache="miss"
// INFO request{id=7 path="/users"}: responded status=200
```

Three things in that output are the whole pitch. `load` never mentions the
request, but its warning is stamped with one. The spans nest, so the line also
says *which* call to `load`. And `cache="miss"` and `status=200` are fields, not
prose — a collector can index them without parsing a sentence back apart.

**On its own the crate produces nothing.** `tracing` is the instrumentation
side; something has to be listening. With no subscriber installed, every macro
compiles to a near-free no-op:

```
use tracing::Level;

// No subscriber has been installed in this process.
assert!(!tracing::enabled!(Level::INFO));

let span = tracing::info_span!("nobody is watching");
assert!(span.is_disabled());

// Still legal, still does nothing.
let _guard = span.enter();
tracing::error!("goes nowhere");
```

That is the same split [`log`](log.md) uses, and the same rule follows from it:

- **A library depends on `tracing` only.** Where its output goes is the
  application's decision.
- **An application also installs a subscriber**, usually
  `tracing_subscriber::fmt::init()` early in `main`.

The ecosystem is four crates, and knowing which is which saves confusion:
`tracing` is what you call, `tracing-attributes` provides `#[instrument]` (on by
default, re-exported, you never name it), `tracing-core` is the small stable
contract that subscribers are written against, and `tracing-subscriber` has the
actual implementations — formatting, `RUST_LOG` filtering, JSON.

**Against `log`.** `log` is smaller, older and simpler, and for a CLI that
prints progress it is the better answer. Reach for `tracing` when output is
concurrent, when you want fields rather than sentences, or when you are already
in [`tokio`](tokio.md) — spans are what make async logs readable. The two
interoperate in both directions: `tracing`'s `log` feature turns events into
`log` records *when no subscriber is installed*, and `tracing-log` lets a
subscriber pick up records from `log`-using dependencies.

```
// With tracing's "log" feature and no tracing subscriber installed, this
// reaches an ordinary `log` logger:
//
//   LOG INFO hello user=7
//
// Install a tracing subscriber and the log record stops being emitted — the
// event goes to the subscriber instead. The `log-always` feature emits both.
tracing::info!(user = 7, "hello");
```

The cost is real but modest: `tracing-core` and `pin-project-lite` at runtime,
plus `syn`/`quote` at build time for `#[instrument]`. **MSRV is 1.65.**
`no_std` works with `default-features = false`, which also drops `attributes` —
so `#[instrument]` disappears unless you ask for it back. The
`max_level_*` and `release_max_level_*` features compile low-severity
instrumentation out of the binary entirely, so a `trace!` in a hot loop costs
nothing in a release build.

## When to use it

### Use case: Giving every line the operation it belongs to

The reason to adopt it. A shared helper logs; the span says whose work it was.

```
fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .init();

    for (id, kind) in [(1u32, "import"), (2, "export")] {
        // One span per job, entered for the duration of the work.
        let span = tracing::info_span!("job", id, kind);
        let _guard = span.enter();

        tracing::info!("starting");
        checksum(&[1, 2, 3]);
        tracing::info!("finished");
    }
}

// Called from both jobs, and from a dozen other places.
fn checksum(bytes: &[u8]) -> u8 {
    let sum = bytes.iter().fold(0u8, |a, b| a.wrapping_add(*b));
    tracing::debug!(len = bytes.len(), sum, "checksummed");
    sum
}

// INFO job{id=1 kind="import"}: starting
// INFO job{id=1 kind="import"}: finished
// INFO job{id=2 kind="export"}: starting
// INFO job{id=2 kind="export"}: finished
```

**Why it fits:** `checksum` has no idea what a job is, and should not. With
`log` you would thread an id through every signature just so the message could
mention it, or give up and accept lines you cannot attribute. The span is
ambient instead — set once at the top of the work, carried by everything
underneath. (`checksum`'s `debug!` is absent above only because `fmt()` defaults
to `INFO`; raise the level and it appears inside the right job.)

### Use case: Instrumenting async code without the span leaking

**The one mistake everybody makes.** A span guard held across an `.await` stays
entered while the task is suspended, so the *next* task to run is recorded
inside it.

```
use std::sync::{Arc, Mutex};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

// A subscriber that records nothing but how deeply spans are nested.
#[derive(Clone, Default)]
struct Depth(Arc<Mutex<(usize, usize)>>);

impl Subscriber for Depth {
    fn enabled(&self, _: &Metadata<'_>) -> bool { true }
    fn new_span(&self, _: &Attributes<'_>) -> Id { Id::from_u64(1) }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, _: &Event<'_>) {}
    fn enter(&self, _: &Id) {
        let mut g = self.0.lock().unwrap();
        g.0 += 1;
        g.1 = g.1.max(g.0); // <- deepest nesting seen
    }
    fn exit(&self, _: &Id) { self.0.lock().unwrap().0 -= 1; }
}

// Wrong: the guard outlives the suspension point.
async fn leaky(tag: &'static str) {
    let span = tracing::info_span!("task", tag);
    let _guard = span.enter();
    tokio::task::yield_now().await;
    tracing::info!("done");
}

// Right: the span is attached to the future, so it is exited on every yield.
#[tracing::instrument]
async fn contained(tag: &'static str) {
    tokio::task::yield_now().await;
    tracing::info!("done");
}

fn deepest(fut: impl std::future::Future<Output = ()>) -> usize {
    let depth = Depth::default();
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    tracing::subscriber::with_default(depth.clone(), || rt.block_on(fut));
    depth.0.lock().unwrap().1
}

fn main() {
    // Two unrelated tasks, and one ends up inside the other.
    assert_eq!(deepest(async { tokio::join!(leaky("a"), leaky("b")); }), 2);

    // Each task sees only its own span.
    assert_eq!(deepest(async { tokio::join!(contained("a"), contained("b")); }), 1);
}
```

**Why it fits:** with `tracing-subscriber`'s formatter the leaky version prints
`task{tag="a"}:task{tag="b"}: done` — task *b*'s line claiming it happened
inside task *a*. Nothing errors, nothing warns, and the log is quietly wrong in
exactly the situation you reached for spans to fix. The fix is to never hand a
future a guard: `#[instrument]` on an `async fn`, or
`fut.instrument(span).await`, attach the span to the future so the runtime
enters and exits it around each poll. `Span::entered()` is a partial
safety net — it returns a `!Send` guard, so holding *it* across an `.await`
inside `tokio::spawn` fails to compile with "future cannot be sent between
threads safely". `enter()`'s guard is `Send`, so it compiles and misbehaves.

### Use case: Recording fields a machine can read

Values as fields rather than interpolated text, including ones you only learn
later.

```
use tracing::field::Empty;

fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .init();

    // `status` has no value yet, but must be declared now.
    let span = tracing::info_span!("request", path = "/users", status = Empty);
    let _guard = span.enter();

    tracing::info!("received");

    let status = serve();
    span.record("status", status);
    tracing::info!("responded");

    // Declared-then-recorded: present.
    assert!(span.has_field("status"));
    // Never declared: the record above it would be dropped in silence.
    span.record("elapsed_ms", 4);
    assert!(!span.has_field("elapsed_ms"));
}

fn serve() -> u16 {
    200
}

// INFO request{path="/users"}: received
// INFO request{path="/users" status=200}: responded
```

**Why it fits:** a span's field *names* are fixed when it is created, because
they are part of the static metadata that makes a disabled span free. So the
response status — known only at the end — has to be reserved with
`field::Empty` up front. Get that wrong and `record` does nothing at all: no
panic, no warning, just a field that never appears. It is the second most
common `tracing` bug after the `await` one, and `has_field` is how you catch it.
The payoff is downstream: a JSON subscriber emits `"status": 200` as a number
you can filter on, rather than a sentence someone has to regex.

## API map

Event macros, span macros, and the trait that decides where it all goes. Most
code touches only the first two groups.

### Recording events

#### `info!`, `warn!`, `error!`, `debug!` and `trace!`

One macro per level, each taking fields and a message.

```
fn main() {
    tracing_subscriber::fmt().without_time().with_target(false).init();

    let host = "db-1";
    let attempts = 3;

    // Fields first, then an optional format-string message.
    tracing::info!(host, attempts, "connected");
    // `?` records with Debug, `%` with Display.
    tracing::warn!(addr = ?std::net::Ipv4Addr::LOCALHOST, "unexpected peer");
    tracing::error!(code = 7, "giving up after {attempts} attempts");
    // A message on its own is fine.
    tracing::debug!("cache warm");
}

// INFO connected host="db-1" attempts=3
// WARN unexpected peer addr=127.0.0.1
// ERROR giving up after 3 attempts code=7
```

**When to use it:** the same severity judgement as any logger — `error!` for a
failure someone must act on, `warn!` for degraded but continuing, `info!` for
what an operator watches, `debug!` and `trace!` for diagnosis. What is new is
the field list: prefer `tracing::info!(user_id, "login failed")` over
`info!("login failed for {user_id}")`, because the first is queryable and the
second is a string. `host` on its own is shorthand for `host = host`; `?x` and
`%x` wrap a value in `Debug` or `Display` for types that are not primitives.

#### `event!`

The general form the level macros expand to.

```
use tracing::{event, Level};

fn main() {
    tracing_subscriber::fmt().with_target(true).without_time().init();

    // A target other than the module path, which is what RUST_LOG filters on.
    event!(target: "metrics", Level::INFO, requests = 42);

    // An event with no parent, escaping whatever span is current.
    let span = tracing::info_span!("job");
    let _guard = span.enter();
    event!(parent: None, Level::INFO, "process-wide, not job-wide");

    // The level is baked into static metadata, so it cannot be a variable.
    // One call per arm is the way out.
    let severe = true;
    match severe {
        true => event!(Level::WARN, "threshold crossed"),
        false => event!(Level::INFO, "threshold crossed"),
    }
}

// INFO metrics: requests=42
// INFO tracing_demo: process-wide, not job-wide
// WARN job: tracing_demo: threshold crossed
```

**When to use it:** when you need `target:` or `parent:`, which the level macros
also accept but are easier to read without. `target:` is the lever for filtering
by subsystem — `RUST_LOG=metrics=info` matches on exactly this, so grouping
cross-cutting events under a hand-written target is how you let an operator
enable one area. `parent: None` makes an event a root, for something that is
genuinely not part of the current operation — note it escapes the `job` span
above while the `WARN` below it does not.

**What it will not do is take a computed level.** Every callsite compiles to a
`static` metadata item holding its level, so `event!(level, ...)` where `level`
is a variable fails with `error[E0435]: attempt to use a non-constant value in a
constant` and a suggestion about `static` that has nothing to do with your code.
A `const` is accepted; a `match` with one call per arm is the usual answer, at
the cost of one callsite per level.

#### Fields and the `message` field

What a field can hold, and the one field you did not declare.

```
fn main() {
    tracing_subscriber::fmt().without_time().with_target(false).init();

    let items = vec![1, 2, 3];
    let path = std::path::Path::new("/tmp/x");

    tracing::info!(
        count = items.len(),        // integers, bools, &str, f64 directly
        ratio = 0.5,
        retry = false,
        items = ?items,             // anything Debug
        path = %path.display(),     // anything Display
        "indexed",
    );
    // Dotted names are allowed and conventional for namespacing.
    tracing::info!(http.status = 200, http.method = "GET", "served");
}

// INFO indexed count=3 ratio=0.5 retry=false items=[1, 2, 3] path=/tmp/x
// INFO served http.status=200 http.method="GET"
```

**When to use it:** whenever a value matters more than a sentence does. Two
details worth knowing. The message is itself a field, named `message`, which is
why a subscriber can treat it as just another key — and why it arrives *first*
in the field order even though you write it last. And a field's value must
implement `Value`, which is done for primitives and `&str` only; everything else
needs `?` or `%`, and forgetting that is the commonest compile error on the
crate. The `valuable` feature adds structured (nested) values, but is still
behind a `--cfg tracing_unstable` flag.

### Spans

#### `info_span!` and the span macros

A span is a value you create, then enter.

```
use tracing::{span, Level};

fn main() {
    tracing_subscriber::fmt().without_time().with_target(false).init();

    // The named-level forms, one per level.
    let a = tracing::info_span!("named", kind = "info");
    let b = tracing::debug_span!("quieter");

    // The general form, which also takes `target:` and `parent:`.
    let c = span!(Level::INFO, "general", attempt = 1);
    let d = span!(target: "db", Level::INFO, "query");

    // Creating a span does not enter it, and never emits anything.
    let _ = (&b, &c, &d);
    let _guard = a.enter();
    tracing::info!("inside a");
}

// INFO named{kind="info"}: inside a
```

**When to use it:** once per unit of work with an identity — a request, a job, a
transaction, a database query. The name should be a noun describing the work,
and the fields should be what identifies *this* instance of it. Note what
creation does and does not do: it allocates nothing a subscriber does not ask
for, and a span whose level is filtered out is a no-op you can create in a hot
loop. Don't wrap every function in one by hand — that is `#[instrument]`'s job.

#### `Span::enter` and `Span::entered`

The two guards, and why the choice matters.

```
fn main() {
    tracing_subscriber::fmt().without_time().with_target(false).init();

    // enter(): borrows the span, so the span outlives the guard.
    let span = tracing::info_span!("borrowed");
    {
        let _guard = span.enter();
        tracing::info!("inside");
    }
    // The span is still here, and can be entered again or recorded into.
    assert!(!span.is_disabled());

    // entered(): consumes the span and returns an owned guard.
    let guard = tracing::info_span!("owned").entered();
    tracing::info!("also inside");
    let span = guard.exit(); // <- hands the span back
    assert!(!span.is_disabled());
}

// INFO borrowed: inside
// INFO owned: also inside
```

**When to use it:** `enter()` for synchronous work where you also want to
`record` into the span afterwards. `entered()` when you want one expression
rather than two, or in async-adjacent code where its `!Send` guard is a feature:
holding it across an `.await` in a `tokio::spawn`ed future is a compile error,
where `enter()`'s guard silently compiles and misattributes. **Neither belongs
across an `.await`** — see `#[instrument]` and `Instrument` below. Both exit on
drop, so a guard bound to `_` rather than `_guard` exits immediately and does
nothing, which is a quiet and nasty typo.

#### `Span::in_scope`

Enter for the duration of a closure, and get its value back.

```
fn main() {
    tracing_subscriber::fmt().without_time().with_target(false).init();

    let span = tracing::info_span!("parse");

    let total: u32 = span.in_scope(|| {
        tracing::info!("summing");
        [1, 2, 3].iter().sum()
    });

    assert_eq!(total, 6);
}

// INFO parse: summing
```

**When to use it:** when the scope you want is an expression rather than a
block — a closure passed to `map`, a short computation whose result you are
assigning. It is exactly `enter()` with the guard's lifetime pinned to the
closure, which also makes it impossible to forget to drop. Do not reach for it
with an `async` block inside: the closure returns the future without polling it,
so the span is exited before any of the work happens.

#### `Span::record` and `field::Empty`

Filling in a field whose value arrives later.

```
use tracing::field::Empty;

fn main() {
    tracing_subscriber::fmt().without_time().with_target(false).init();

    let span = tracing::info_span!("upload", name = "big.bin", bytes = Empty);
    let _guard = span.enter();

    let bytes = 4096;
    span.record("bytes", bytes);

    // From inside an instrumented function, reach the span this way.
    assert!(tracing::Span::current().has_field("bytes"));

    tracing::info!("stored");
}

// INFO upload{name="big.bin" bytes=4096}: stored
```

**When to use it:** for the outcome of the work the span describes — a status
code, a row count, an elapsed time. The rule that catches people is that the
*name* must exist at creation, hence the `Empty` placeholder; `record` on a name
that was never declared is silently ignored, and `has_field` is the only way to
notice. Inside a function annotated `#[instrument(fields(x = Empty))]` the
idiom is `Span::current().record("x", value)`, since you have no handle on the
span the macro made.

#### `Span::current`

The span that is active right now.

```
fn main() {
    tracing_subscriber::fmt().without_time().with_target(false).init();

    // Outside any span it is the empty span.
    assert!(tracing::Span::current().is_none());

    let span = tracing::info_span!("outer");
    let _guard = span.enter();

    let current = tracing::Span::current();
    assert!(!current.is_none());
    assert_eq!(current.id(), span.id());
}
```

**When to use it:** to record into the ambient span when you did not create it,
and to capture context you are about to hand to another thread or task — a
`Span::current()` cloned into a `std::thread::spawn` closure keeps the
relationship, which nothing else will do for you. `Instrument::in_current_span`
is the async equivalent and reads better there. `Span::or_current()` is the
related combinator: it replaces a *disabled* span with the current one, which is
what you want when spawning under a level filter that might have switched your
new span off.

#### `Span::follows_from`

A causal link between spans that do not nest.

```
fn main() {
    tracing_subscriber::fmt().without_time().with_target(false).init();

    let enqueue = tracing::info_span!("enqueue", msg = 1);
    let _g = enqueue.enter();
    tracing::info!("queued");
    drop(_g);

    // Later, elsewhere: the work caused by that message.
    let consume = tracing::info_span!("consume", msg = 1);
    consume.follows_from(enqueue.id());
    let _g = consume.enter();
    tracing::info!("handled");
}
```

**When to use it:** for queues, retries and anything fan-out — the consumer is
*caused by* the producer but does not run inside it, so a parent/child
relationship would be a lie. Be aware that most subscribers, `fmt` included,
ignore the link entirely; it earns its keep with a trace collector that draws
graphs, such as `tracing-opentelemetry`. If you only ever read formatted logs,
skip it.

### Carrying a span through async code

#### `#[instrument]`

Wraps a function in a span, and handles `async fn` correctly.

```
#[derive(Debug)]
struct Blob(Vec<u8>);

// Every argument becomes a field, so every argument must be Debug.
#[tracing::instrument]
fn connect(host: &str, port: u16) {}

// skip the big ones, and add fields of your own.
#[tracing::instrument(skip(blob), fields(size = blob.0.len()))]
fn store(blob: &Blob, key: &str) {}

// A quieter level, a different span name, the return value, the error.
#[tracing::instrument(level = "debug", name = "lookup", skip_all, ret, err)]
fn find(id: u32) -> Result<u32, String> {
    if id == 0 { Err("no such id".to_string()) } else { Ok(id * 2) }
}

// On an async fn the span is attached to the future, not entered around it.
#[tracing::instrument]
async fn fetch(url: &str) {
    tokio::task::yield_now().await;
    tracing::info!("fetched");
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .without_time()
        .with_target(false)
        .init();

    connect("db-1", 5432);
    store(&Blob(vec![0; 9]), "k");
    assert_eq!(find(3), Ok(6));
    assert!(find(0).is_err());

    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    rt.block_on(fetch("/x"));
}

// DEBUG lookup: return=6
// ERROR lookup: error=no such id
// INFO fetch{url="/x"}: fetched
```

**When to use it:** on anything you would otherwise open with a hand-written
span, and on **every** `async fn` worth tracing — this is the safe route past
the guard-across-`await` problem, because the macro attaches the span to the
future rather than entering it. The arguments worth remembering: `skip` and
`skip_all` for values that are huge, secret, or not `Debug`; `fields(...)` to
add or reserve your own (`field::Empty` plus `Span::current().record` for a
value computed in the body); `ret` and `err` to emit the outcome, where `err`
logs at `ERROR` and fires only on the `Err` branch. Note `ret` and `err` are
exclusive in practice — a call that fails emits the error event, not the return
one.

#### `Instrument::instrument` and `in_current_span`

Attaching a span to a future you did not write.

```
use tracing::Instrument;

async fn work(n: u32) -> u32 {
    tokio::task::yield_now().await;
    tracing::info!("working");
    n * 2
}

fn main() {
    tracing_subscriber::fmt().without_time().with_target(false).init();

    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();

    rt.block_on(async {
        // A span of your choosing, around someone else's future.
        let doubled = work(21).instrument(tracing::info_span!("job", id = 1)).await;
        assert_eq!(doubled, 42);

        // Carry the current span into a spawned task.
        let outer = tracing::info_span!("outer");
        let _g = outer.enter();
        tokio::spawn(work(1).in_current_span());
    });
}

// INFO job{id=1}: working
```

**When to use it:** whenever the future is not a function you can annotate — a
block, a combinator chain, something from a library. `.instrument(span)` is the
general tool; `.in_current_span()` is the one you want at a `tokio::spawn`,
because a spawned task starts with *no* span otherwise and its output becomes
unattributable. Pair it with `Span::or_current()`
(`.instrument(debug_span!("x").or_current())`) when the new span's level might
be filtered out, so the task falls back to the caller's context rather than to
nothing.

### Levels, filtering and subscribers

#### `enabled!`

Whether anything would be recorded, before you do the work.

```
use tracing::Level;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .without_time()
        .init();

    assert!(tracing::enabled!(Level::INFO));
    assert!(!tracing::enabled!(Level::DEBUG));

    // Per-target, matching what RUST_LOG filters on.
    let _ = tracing::enabled!(target: "metrics", Level::INFO);

    if tracing::enabled!(Level::DEBUG) {
        tracing::debug!(dump = expensive(), "state");
    }
}

fn expensive() -> String {
    (0..1000).map(|n| n.to_string()).collect()
}
```

**When to use it:** only to guard a computation that exists purely to be logged
— a serialisation, an extra query, a `Debug` of something enormous. The macros
already skip their own formatting when disabled, so wrapping ordinary calls in
it buys nothing and costs readability. `event_enabled!` and `span_enabled!` are
the narrower forms, for when a subscriber answers differently for the two.

#### `Level`, `LevelFilter` and `STATIC_MAX_LEVEL`

Severity, thresholds, and what the compiler removed.

```
use tracing::level_filters::{LevelFilter, STATIC_MAX_LEVEL};
use tracing::Level;

fn main() {
    // Ordered by severity, most severe *smallest* — as in the `log` crate.
    assert!(Level::ERROR < Level::WARN);
    assert!(Level::DEBUG > Level::INFO);

    // A filter is a level plus OFF.
    let filter = LevelFilter::from_level(Level::WARN);
    assert_eq!(filter, LevelFilter::WARN);
    assert!(Level::ERROR <= filter);
    assert!(!(Level::INFO <= filter));

    // What the max_level_* features left in the binary.
    assert_eq!(STATIC_MAX_LEVEL, LevelFilter::TRACE);
}
```

**When to use it:** comparing severities, and parsing a threshold out of
configuration. The ordering is the thing that trips people: `ERROR` is the
*smallest* value, so "would this be recorded" reads as
`record_level <= filter`, backwards from how it sounds. `STATIC_MAX_LEVEL` is
the compile-time ceiling set by the `max_level_*` features — instrumentation
above it is not merely disabled, it is absent from the binary, which is how a
`trace!` costs nothing in release.

#### The `Subscriber` trait

What receives spans and events.

```
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

// Seven required methods; this one just counts.
struct Counting(std::sync::atomic::AtomicUsize);

impl Subscriber for Counting {
    // Called once per callsite per level check.
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        *metadata.level() <= Level::INFO
    }
    // Must return an id; the subscriber owns the numbering.
    fn new_span(&self, _span: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }
    fn record(&self, _span: &Id, _values: &Record<'_>) {}
    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}
    fn event(&self, _event: &Event<'_>) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    fn enter(&self, _span: &Id) {}
    fn exit(&self, _span: &Id) {}
}

fn main() {
    let sub = Counting(std::sync::atomic::AtomicUsize::new(0));
    tracing::subscriber::with_default(sub, || {
        tracing::info!("one");
        tracing::debug!("filtered out");
        tracing::warn!("two");
    });
}
```

**When to use it:** almost never — `tracing-subscriber` composes layers for
filtering, formatting, JSON and OpenTelemetry export, and a `Layer` is the right
unit of custom behaviour. Implementing the trait directly is for a test double
that captures what was emitted, and for understanding what the macros actually
call. Two traps if you do: `enabled` is yours to honour, since `tracing` does
not filter on your behalf before calling `event`; and the default
`current_span()` returns "unknown", so `Span::current()` comes back empty until
you track a span stack yourself.

#### Installing a subscriber

Globally for the process, or scoped to a block.

```
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

struct Noop;
impl Subscriber for Noop {
    fn enabled(&self, _: &Metadata<'_>) -> bool { true }
    fn new_span(&self, _: &Attributes<'_>) -> Id { Id::from_u64(1) }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, _: &Event<'_>) {}
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
}

fn main() {
    // Scoped to a closure, on this thread only.
    tracing::subscriber::with_default(Noop, || {
        assert!(tracing::enabled!(tracing::Level::INFO));
    });

    // Scoped to a guard's lifetime, on this thread only.
    {
        let _guard = tracing::subscriber::set_default(Noop);
        assert!(tracing::enabled!(tracing::Level::INFO));
    }

    // The process-wide default, settable exactly once.
    tracing::subscriber::set_global_default(Noop).unwrap();
    assert!(tracing::subscriber::set_global_default(Noop).is_err());
}
```

**When to use it:** `set_global_default` once, early in `main` — though in an
application you call `tracing_subscriber::fmt::init()` instead, which does this
plus `RUST_LOG` parsing. `with_default` and `set_default` are **per-thread**,
which makes them right for tests (each test installs its own capturing
subscriber without fighting the others) and wrong for an application, since a
spawned thread would see nothing. The second `set_global_default` failing is
deliberate, for the same reason as in `log`: a library that installed one would
silently override the binary's choice.
