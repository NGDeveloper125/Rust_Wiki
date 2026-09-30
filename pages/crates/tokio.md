---
title: "tokio"
version: "1.53.1"
publisher: "Carl Lerche (carllerche), Alice Ryhl (Darksonn), Core"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-30"
summary: "The async runtime the ecosystem settled on: an executor for tasks, non-blocking I/O, timers, and the channels and locks that work across `.await` points."
domain: "Async runtimes & concurrency"
categories: ["async", "concurrency", "networking"]
repository: "https://github.com/tokio-rs/tokio"
---

## Overview

Rust has `async` and `.await` in the language, and no runtime to execute them.
A `Future` does nothing until something polls it, and the standard library
provides nothing that does. `tokio` is that something, plus everything a real
program needs alongside it:

```
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Tasks run concurrently on the runtime's threads.
    let a = tokio::spawn(async { 21 * 2 });
    let b = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(5)).await;
        "done"
    });

    assert_eq!(a.await.unwrap(), 42);
    assert_eq!(b.await.unwrap(), "done");
}
```

It is a runtime, an async reimplementation of the parts of `std` that block
(`tokio::fs`, `tokio::net`, `tokio::io`), timers, and synchronisation
primitives that can be held across an `.await`. Under it sits
[`mio`](mio.md) for the OS readiness API.

**The rule that matters most: never block in an async task.** A task that calls
`std::fs::read`, `std::thread::sleep`, or spends a long time computing, stops
the whole worker thread — and with it every other task scheduled there. The
symptom is a server that becomes unresponsive under load for no visible reason.
The fixes are `tokio::fs` and friends for I/O, and `spawn_blocking` for
CPU-bound or unavoidably blocking work, which moves it to a separate pool.

**The second thing to internalise: dropping a future cancels it.** There is no
`Task::cancel` because dropping the `JoinHandle`'s future — or losing a `select!`
branch — simply stops polling, and the work stops wherever it was. That is
powerful and sharp: a `select!` that loses a branch mid-write can leave a
half-written message, so code that must not be interrupted needs to say so,
usually by moving it into its own `spawn`.

**Async is not automatically faster.** It wins when a program waits on many
things at once — thousands of connections, many concurrent requests. For
CPU-bound work it adds machinery and no speed, and for a handful of
simultaneous operations, threads are simpler and perform fine. Async also
colours your API: an `async fn` can only be called from async code, so adopting
it is a decision about the whole program rather than one function.

The alternatives are real but narrow. `smol` is much smaller and does the same
core job; `glommio` is thread-per-core for io_uring workloads. In practice the
ecosystem standardised on tokio — `axum`, `hyper`, `tonic`, `reqwest` and most
database drivers expect it — so choosing another runtime means checking that
everything you depend on still works.

Features are opt-in and `full` is the usual starting point: `rt-multi-thread`
for the threaded scheduler, `macros` for `#[tokio::main]`, `net`, `fs`, `time`,
`sync`. Trimming them matters for build time on a small service. It requires
Rust 1.71.

## When to use it

### Use case: Doing several slow things at once

The case async exists for. Three requests that each take 100 ms take 100 ms
together, not 300.

```
use std::time::Duration;

async fn fetch(name: &str, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await; // <- stands in for I/O
    format!("{name} ok")
}

#[tokio::main]
async fn main() {
    let started = std::time::Instant::now();

    // join! polls all three on this task, concurrently.
    let (a, b, c) = tokio::join!(
        fetch("users", 30),
        fetch("orders", 30),
        fetch("stock", 30),
    );

    assert_eq!((a.as_str(), b.as_str()), ("users ok", "orders ok"));
    assert_eq!(c, "stock ok");

    // Concurrent, so well under the 90ms a sequential version would take.
    assert!(started.elapsed() < Duration::from_millis(80));
}
```

**Why it fits:** `join!` runs the futures concurrently on one task, with no
spawning and no threads. Use `spawn` instead when the work should run in
*parallel* across threads or outlive the current scope; `join!` is the cheaper
answer when you simply need several awaits to overlap.

### Use case: A worker fed by a channel

The standard shape for background work: producers send, one task consumes, and
backpressure is built in.

```
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Capacity 8: send() waits once the queue is full, which is backpressure.
    let (tx, mut rx) = mpsc::channel::<u32>(8);

    let worker = tokio::spawn(async move {
        let mut total = 0;
        while let Some(job) = rx.recv().await {
            total += job;
        }
        total // <- recv returns None once every sender is dropped
    });

    for n in 1..=4 {
        tx.send(n).await.unwrap();
    }
    drop(tx); // <- closes the channel, ending the loop

    assert_eq!(worker.await.unwrap(), 10);
}
```

**Why it fits:** the bounded channel is the point. An unbounded queue turns a
slow consumer into unbounded memory growth; a bounded one makes the producer
wait, which pushes the pressure back to where it can be handled. Dropping the
sender is how the consumer learns to stop — not a sentinel message.

### Use case: Giving an operation a deadline

Anything over a network needs a time limit, and the caller is where it belongs.

```
use std::time::Duration;

async fn slow_call() -> &'static str {
    tokio::time::sleep(Duration::from_secs(30)).await;
    "eventually"
}

#[tokio::main]
async fn main() {
    // Err on timeout; the inner future is dropped, which cancels it.
    let result = tokio::time::timeout(Duration::from_millis(20), slow_call()).await;
    assert!(result.is_err());

    // Inside the limit, the value comes through.
    let quick = tokio::time::timeout(Duration::from_millis(50), async { 7 }).await;
    assert_eq!(quick.unwrap(), 7);
}
```

**Why it fits:** `timeout` wraps any future, so it works on a request, a lock
acquisition or a whole pipeline without those knowing about it. What it does on
expiry is *drop* the future — so anything half-done is abandoned, which is the
cancellation behaviour to keep in mind when the operation had side effects.

## API map

tokio is large, so this is the part you reach for daily: running the runtime,
spawning work, the synchronisation primitives, time, and the async I/O types.

### Starting the runtime

#### `#[tokio::main]`

Turns an async `main` into a sync one that builds a runtime and blocks on it.

```
#[tokio::main]
async fn main() {
    let value = async { 1 + 1 }.await;
    assert_eq!(value, 2);
}
```

**When to use it:** in binaries, once. It needs the `macros` and
`rt-multi-thread` features. `#[tokio::main(flavor = "current_thread")]` gives a
single-threaded runtime, which is lighter and the right choice for a CLI tool
that just happens to make a few async calls.

#### `Runtime::new` and `block_on`

Building a runtime by hand, for when an attribute will not do.

```
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let result = runtime.block_on(async {
        tokio::spawn(async { "from a task" }).await.unwrap()
    });

    assert_eq!(result, "from a task");
}
```

**When to use it:** a library exposing a sync API over async internals, a test
harness, or a program with a non-async `main` it does not control. `Builder`
configures worker count and thread names. Never call `block_on` from inside a
runtime — it panics, and it is the usual cause of "cannot block the current
thread from within a runtime".

#### `#[tokio::test]`

An async test, with a fresh runtime per test.

```
// In a real crate this carries #[tokio::test]; here it is called directly.
async fn adds_up() {
    let handle = tokio::spawn(async { 2 + 2 });
    assert_eq!(handle.await.unwrap(), 4);
}

#[tokio::main]
async fn main() {
    adds_up().await;
}
```

**When to use it:** every test touching async code. It defaults to a
current-thread runtime, which is usually what you want — deterministic, and it
surfaces a deadlock as a hang in one test rather than a flake.

### Tasks

#### `spawn`

Hands a future to the runtime to run independently.

```
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        (1..=10).sum::<u32>()
    });

    // The task is already running; await collects its result.
    assert_eq!(handle.await.unwrap(), 55);
}
```

**When to use it:** work that should proceed whether or not you are waiting, or
should run in parallel on another thread. The future must be `Send + 'static`,
which is what forces owned data into it — usually via `move` and a `clone` of
whatever it needs.

#### `JoinHandle`

The handle to a spawned task: await it for the result, drop it to detach, abort
it to cancel.

```
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async { 1 });
    assert_eq!(handle.await.unwrap(), 1);

    // A panicking task does not kill the runtime; it surfaces here.
    let bad = tokio::spawn(async { panic!("boom") });
    let err = bad.await.unwrap_err();
    assert!(err.is_panic());

    // abort stops a task; awaiting it afterwards reports cancellation.
    let long = tokio::spawn(async { std::future::pending::<()>().await });
    long.abort();
    assert!(long.await.unwrap_err().is_cancelled());
}
```

**When to use it:** whenever you need the result or the outcome. The
`unwrap_err().is_panic()` case is the one to know — a panic in a task is
*contained*, so a supervisor that never awaits its handles will never learn a
task died.

#### `spawn_blocking`

Moves blocking work off the async threads.

```
#[tokio::main]
async fn main() {
    // Pretend this is a CPU-heavy or blocking-IO call.
    let total = tokio::task::spawn_blocking(|| {
        (1..=1_000u64).sum::<u64>()
    })
    .await
    .unwrap();

    assert_eq!(total, 500_500);
}
```

**When to use it:** synchronous file APIs, CPU-bound computation, an FFI call
that blocks, a database driver with no async version. This is the fix for the
rule in the Overview — the closure runs on a separate pool sized for blocking,
so stalling it costs nothing to the async tasks.

### Synchronisation

#### `mpsc::channel`

A bounded multi-producer, single-consumer queue.

```
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<&str>(2);

    let tx2 = tx.clone(); // <- multi-producer
    tokio::spawn(async move { tx2.send("a").await.unwrap() });
    tokio::spawn(async move { tx.send("b").await.unwrap() });

    let mut got = vec![rx.recv().await.unwrap(), rx.recv().await.unwrap()];
    got.sort_unstable();
    assert_eq!(got, ["a", "b"]);

    // With every sender dropped, recv yields None.
    assert_eq!(rx.recv().await, None);
}
```

**When to use it:** the default channel — job queues, event pipelines, fan-in
from many producers. Choose the capacity deliberately: it is the amount of work
allowed to pile up before producers are slowed down. `unbounded_channel` exists
but removes exactly that protection.

#### `oneshot::channel`

A single value, sent once, from one place to one place.

```
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel::<u32>();

    tokio::spawn(async move {
        tx.send(99).unwrap(); // <- not async: it never blocks
    });

    assert_eq!(rx.await.unwrap(), 99);

    // A dropped sender is an error rather than a hang.
    let (tx2, rx2) = oneshot::channel::<u32>();
    drop(tx2);
    assert!(rx2.await.is_err());
}
```

**When to use it:** request/response between tasks — send a job carrying a
`oneshot::Sender` and await the reply. The dropped-sender error is the important
property: if the worker dies, the caller gets an error rather than waiting
forever.

#### `broadcast` and `watch`

Two fan-out channels with different semantics.

```
use tokio::sync::{broadcast, watch};

#[tokio::main]
async fn main() {
    // broadcast: every receiver sees every message.
    let (tx, mut rx1) = broadcast::channel::<u8>(4);
    let mut rx2 = tx.subscribe();
    tx.send(1).unwrap();
    assert_eq!(rx1.recv().await.unwrap(), 1);
    assert_eq!(rx2.recv().await.unwrap(), 1);

    // watch: receivers see only the latest value.
    let (wtx, mut wrx) = watch::channel("starting");
    wtx.send("ready").unwrap();
    wrx.changed().await.unwrap();
    assert_eq!(*wrx.borrow_and_update(), "ready");
}
```

**When to use it:** `broadcast` for events every subscriber must see, accepting
that a slow receiver can lag and miss messages. `watch` for current state —
configuration, a shutdown flag, a health status — where only the newest value
matters and missing intermediate ones is correct.

#### `Mutex` and `RwLock`

Locks that can be held across an `.await`.

```
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let counter = Arc::new(Mutex::new(0u32));

    let mut handles = Vec::new();
    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        handles.push(tokio::spawn(async move {
            let mut guard = counter.lock().await; // <- await, not block
            *guard += 1;
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    assert_eq!(*counter.lock().await, 4);
}
```

**When to use it:** only when the guard must survive an `.await`. Otherwise
`std::sync::Mutex` is faster and perfectly correct in async code, provided you
lock, use, and drop the guard without awaiting in between. Reaching for the
tokio one by default is a common and needless slowdown.

### Time

#### `sleep` and `interval`

Waiting, without blocking a thread.

```
use std::time::Duration;

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(start.elapsed() >= Duration::from_millis(10));

    // interval ticks repeatedly; the first tick is immediate.
    let mut ticker = tokio::time::interval(Duration::from_millis(5));
    ticker.tick().await;
    ticker.tick().await;
    assert!(start.elapsed() >= Duration::from_millis(15));
}
```

**When to use it:** retry backoff, polling loops, rate limiting. Never
`std::thread::sleep` in a task — it stops the worker thread and everything
scheduled on it. `interval`'s immediate first tick surprises people; it is
documented behaviour and usually what a polling loop wants.

#### `timeout`

A deadline around any future.

```
use std::time::Duration;

#[tokio::main]
async fn main() {
    let fast = tokio::time::timeout(Duration::from_millis(50), async { "ok" }).await;
    assert_eq!(fast.unwrap(), "ok");

    let slow = tokio::time::timeout(
        Duration::from_millis(10),
        tokio::time::sleep(Duration::from_secs(5)),
    )
    .await;
    assert!(slow.is_err());
}
```

**When to use it:** every network call, and anything that could hang. It returns
`Result<T, Elapsed>`, so the timeout is a value you handle rather than a
surprise. Remember the inner future is dropped on expiry — cancelled, not
allowed to finish in the background.

### Racing futures

#### `select!`

Waits on several futures and takes the first to finish.

```
use std::time::Duration;

#[tokio::main]
async fn main() {
    let slow = async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        "slow"
    };
    let fast = async { "fast" };

    let winner = tokio::select! {
        v = slow => v,
        v = fast => v,
    };

    assert_eq!(winner, "fast");
}
```

**When to use it:** a work loop that must also watch for shutdown, or a
first-response-wins race. The crucial caveat: **the losing branches are
dropped**, so a branch that was midway through a write is cancelled there. If a
branch must complete once started, `spawn` it and select on its `JoinHandle`
instead.

#### `join!` and `try_join!`

Waits for all of several futures.

```
#[tokio::main]
async fn main() {
    let (a, b) = tokio::join!(async { 1 }, async { 2 });
    assert_eq!((a, b), (1, 2));

    // try_join! short-circuits on the first Err.
    let ok: Result<(u8, u8), &str> = tokio::try_join!(async { Ok(1) }, async { Ok(2) });
    assert_eq!(ok.unwrap(), (1, 2));

    let failed: Result<(u8, u8), &str> =
        tokio::try_join!(async { Ok(1) }, async { Err("nope") });
    assert_eq!(failed.unwrap_err(), "nope");
}
```

**When to use it:** several independent awaits whose results you all need.
`try_join!` for the fallible version, where one failure should abandon the rest —
and note it does exactly that, dropping the others rather than waiting.

### Async I/O

#### `tokio::fs`

The async mirror of `std::fs`.

```
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let path = std::env::temp_dir().join("tokio-page-demo.txt");

    tokio::fs::write(&path, b"contents").await?;
    let read_back = tokio::fs::read_to_string(&path).await?;
    assert_eq!(read_back, "contents");

    tokio::fs::remove_file(&path).await?;
    Ok(())
}
```

**When to use it:** file work inside a task. Be aware of what it actually is —
most platforms have no async file I/O, so these call the blocking syscalls on
the `spawn_blocking` pool. The benefit is not speed; it is that the async
threads keep running.

#### `AsyncReadExt` and `AsyncWriteExt`

The async counterparts of `Read` and `Write`.

```
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // A Vec<u8> is an AsyncWrite, and a &[u8] an AsyncRead.
    let mut sink: Vec<u8> = Vec::new();
    sink.write_all(b"hello ").await?;
    sink.write_all(b"world").await?;
    assert_eq!(sink, b"hello world");

    let mut source = &b"abc"[..];
    let mut buf = String::new();
    source.read_to_string(&mut buf).await?;
    assert_eq!(buf, "abc");
    Ok(())
}
```

**When to use it:** reading and writing sockets, files and pipes. The extension
traits carry the convenience methods, so both imports are needed even when the
type already implements the base trait — a "no method named `write_all`" error
is almost always a missing `AsyncWriteExt`.

#### `tokio::net::TcpListener`

Accepting connections, one task per connection.

```
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 16];
        let n = socket.read(&mut buf).await.unwrap();
        socket.write_all(&buf[..n]).await.unwrap(); // <- echo it back
    });

    let mut client = TcpStream::connect(addr).await?;
    client.write_all(b"ping").await?;
    let mut reply = [0u8; 4];
    client.read_exact(&mut reply).await?;

    assert_eq!(&reply, b"ping");
    server.await.unwrap();
    Ok(())
}
```

**When to use it:** any TCP server. The shape is the whole point — `accept` in a
loop, `spawn` per connection, and the runtime multiplexes thousands of them onto
a few threads. Compare [`mio`](mio.md), which is what this is built on and shows
the bookkeeping tokio is doing for you.
