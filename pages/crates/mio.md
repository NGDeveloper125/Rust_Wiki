---
title: "mio"
version: "1.2.3"
publisher: "Carl Lerche (carllerche), mio-core"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-21"
summary: "A thin, portable wrapper over the operating system's readiness API — epoll, kqueue, IOCP. The event loop `tokio` is built on, and almost never what application code should use directly."
domain: "Async runtimes & concurrency"
categories: ["networking", "async", "io"]
repository: "https://github.com/tokio-rs/mio"
---

## Overview

Every operating system has a way to ask "tell me when any of these sockets is
ready": `epoll` on Linux, `kqueue` on the BSDs and macOS, IOCP on Windows. They
are the foundation of every async runtime, and they have nothing in common with
each other.

`mio` is one portable interface over all of them, and nothing more. You register
sources with a `Poll`, block until something is ready, and then do the I/O
yourself:

```
use mio::{Events, Interest, Poll, Token};
use mio::net::TcpListener;
use std::time::Duration;

let mut poll = Poll::new().unwrap();
let mut events = Events::with_capacity(128);

let mut listener = TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
poll.registry()
    .register(&mut listener, Token(0), Interest::READABLE)
    .unwrap();

// Nothing is connecting, so this returns empty after the timeout.
poll.poll(&mut events, Some(Duration::from_millis(10))).unwrap();
assert_eq!(events.iter().count(), 0);
```

**You probably want `tokio` instead.** That is the honest headline. `mio` has no
`async`, no `await`, no tasks, no timers and no buffering — it tells you a socket
is ready and leaves the rest to you. `tokio` is built directly on it and provides
all of that. Its download rank here is `tokio`'s, not its own.

`mio` is the right choice when you are **building** a runtime or an event loop,
embedding one in an existing application with its own main loop, or need the
readiness primitives without pulling in a runtime. If you are writing a server,
use `tokio`.

**The model is readiness, not completion.** `poll` says "this socket will
probably not block now"; it does not do the read. So every I/O call must still
handle `WouldBlock`, and *"probably"* is doing real work in that sentence — a
readiness notification can be spurious, and the socket can become unready
between the notification and your call.

**And notifications are edge-triggered.** You are told when readiness *changes*,
not while it persists. If a socket has 8 KiB waiting and you read 1 KiB, you will
not be told again — the remaining 7 KiB sits there until new data arrives. The
rule that follows is absolute: **read until `WouldBlock`**, every time. This is
the single most common way to write a mio program that mysteriously stalls.

It requires Rust 1.71, and its features are opt-in: `os-poll` for the polling
itself and `net` for the TCP and UDP types, so `mio = { version = "1", features
= ["os-poll", "net"] }` is the usual line. `polling` and `calloop` are the
alternatives worth knowing, both smaller and less tied to `tokio`'s needs.

## When to use it

### Use case: One thread serving many connections

The reason readiness APIs exist: thousands of mostly-idle sockets on one thread,
instead of a thread apiece.

```
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::io::{Read, Write};
use std::time::Duration;

const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);
const ACCEPTED: Token = Token(2);

fn main() -> std::io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(16);

    let mut listener = TcpListener::bind("127.0.0.1:0".parse().unwrap())?;
    let addr = listener.local_addr()?;
    poll.registry().register(&mut listener, SERVER, Interest::READABLE)?;

    let mut client = TcpStream::connect(addr)?;
    poll.registry().register(&mut client, CLIENT, Interest::WRITABLE)?;

    let mut accepted: Option<TcpStream> = None;
    let mut received = Vec::new();

    // A real loop runs forever; this one stops once the message arrives.
    for _ in 0..20 {
        poll.poll(&mut events, Some(Duration::from_millis(200)))?;

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (mut sock, _peer) = listener.accept()?;
                    poll.registry().register(&mut sock, ACCEPTED, Interest::READABLE)?;
                    accepted = Some(sock);
                }
                CLIENT if event.is_writable() => {
                    client.write_all(b"ping")?;
                }
                ACCEPTED => {
                    if let Some(sock) = accepted.as_mut() {
                        let mut buf = [0u8; 64];
                        match sock.read(&mut buf) {
                            Ok(n) => received.extend_from_slice(&buf[..n]),
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                            Err(e) => return Err(e),
                        }
                    }
                }
                _ => {}
            }
        }

        if !received.is_empty() {
            break;
        }
    }

    assert_eq!(received, b"ping");
    Ok(())
}
```

**Why it fits:** one `Poll`, one thread, any number of connections, and the
token is how you know which one woke you. Note how much is left to you —
tracking the sockets, matching tokens back to state, handling `WouldBlock`. That
bookkeeping is most of what a runtime does for you, and seeing it is the best
argument for using one.

### Use case: Waking the loop from another thread

An event loop blocked in `poll` cannot notice a channel message. `Waker` is the
escape hatch.

```
use mio::{Events, Poll, Token, Waker};
use std::sync::Arc;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(8);

    const WAKE: Token = Token(10);
    let waker = Arc::new(Waker::new(poll.registry(), WAKE)?);

    // Another thread has work for the loop.
    let handle = {
        let waker = Arc::clone(&waker);
        std::thread::spawn(move || waker.wake())
    };

    poll.poll(&mut events, Some(Duration::from_secs(5)))?;

    let woke: Vec<Token> = events.iter().map(|e| e.token()).collect();
    assert_eq!(woke, vec![WAKE]);

    handle.join().unwrap()?;
    Ok(())
}
```

**Why it fits:** without it, a shutdown signal or a queued job would wait for the
poll timeout, and a loop with no timeout would never see it at all. The `Waker`
is the standard way to bridge "something happened elsewhere" into a readiness
loop, and it is safe to call from any thread.

### Use case: Changing what you care about

A connection that has nothing to send should not be woken for writability.
Re-registering narrows the interest.

```
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(8);

    let mut listener = TcpListener::bind("127.0.0.1:0".parse().unwrap())?;
    let registry = poll.registry();

    registry.register(&mut listener, Token(0), Interest::READABLE)?;

    // Later: also care about writability.
    registry.reregister(&mut listener, Token(0), Interest::READABLE | Interest::WRITABLE)?;

    // Or stop hearing about it entirely.
    registry.deregister(&mut listener)?;

    poll.poll(&mut events, Some(Duration::from_millis(10)))?;
    assert_eq!(events.iter().count(), 0); // <- deregistered, so silent

    Ok(())
}
```

**Why it fits:** a socket registered for writability is ready almost always, so
leaving it registered spins the loop. The usual pattern is to register for
readability, add writability only while a write buffer is non-empty, and drop it
again once drained.

## API map

Five types carry the whole crate: `Poll` to wait, `Registry` to register,
`Token` to identify, `Interest` to say what you care about, and `Events` to
receive the results.

### The event loop

#### `Poll::new` and `poll`

Creates the OS polling handle, and blocks until something is ready.

```
use mio::{Events, Poll};
use std::time::Duration;

let mut poll = Poll::new().unwrap();
let mut events = Events::with_capacity(64);

// No sources registered, so this just waits out the timeout.
poll.poll(&mut events, Some(Duration::from_millis(5))).unwrap();
assert!(events.is_empty());

// None means block indefinitely — only safe once something can wake it.
```

**When to use it:** once per event loop, at the centre of it. `poll` reuses the
`Events` buffer rather than allocating, which is why it takes `&mut`. Passing
`None` blocks forever, so a loop that does that needs a `Waker` or a registered
source that will eventually fire.

#### `Events`

The buffer `poll` fills, and what you iterate afterwards.

```
use mio::{Events, Poll};
use std::time::Duration;

let mut poll = Poll::new().unwrap();
let mut events = Events::with_capacity(128);

poll.poll(&mut events, Some(Duration::from_millis(5))).unwrap();

assert!(events.capacity() >= 128);
for event in events.iter() {
    let _ = event.token();
}
```

**When to use it:** allocate once, outside the loop. The capacity caps how many
readiness notifications one `poll` returns — too small and you make extra
syscalls, too large and you waste memory. A few hundred is typical.

#### `Registry`

The handle used to register sources, cloneable and usable from other threads.

```
use mio::net::TcpListener;
use mio::{Interest, Poll, Token};

let poll = Poll::new().unwrap();
let registry = poll.registry();

let mut listener = TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
registry.register(&mut listener, Token(1), Interest::READABLE).unwrap();

// try_clone gives a registry another thread can use to add sources.
let other = registry.try_clone().unwrap();
assert!(other.try_clone().is_ok());
```

**When to use it:** whenever registering. The split from `Poll` is deliberate —
`poll` needs `&mut Poll`, but registration only needs `&Registry`, so a worker
thread can add sources while the loop is blocked.

### Identifying sources

#### `Token`

An opaque `usize` you choose, handed back with every event.

```
use mio::Token;

const LISTENER: Token = Token(0);
const TIMER: Token = Token(1);

// Tokens are just numbers, so an index into your own storage works.
let connections: Vec<&str> = vec!["a", "b", "c"];
let token_for_second = Token(100 + 1);

assert_eq!(LISTENER.0, 0);
assert_ne!(LISTENER, TIMER);
assert_eq!(connections[token_for_second.0 - 100], "b");
```

**When to use it:** to map an event back to your own state. Named constants for
the fixed sources and an offset index for the dynamic ones is the usual scheme;
a `Slab` keyed by token is the next step up, and is exactly what `tokio` does
internally.

#### `Interest`

What you want to hear about: readable, writable, or both.

```
use mio::Interest;

let read = Interest::READABLE;
let both = Interest::READABLE | Interest::WRITABLE;

assert!(both.is_readable() && both.is_writable());
assert!(read.is_readable() && !read.is_writable());
```

**When to use it:** at every `register` and `reregister`. Registering for
writability when you have nothing to write is the classic mistake — a socket
with room in its send buffer is ready constantly, so the loop spins.

#### `Event`

One readiness notification: which token, and what kind.

```
use mio::{Events, Poll};
use std::time::Duration;

let mut poll = Poll::new().unwrap();
let mut events = Events::with_capacity(8);
poll.poll(&mut events, Some(Duration::from_millis(5))).unwrap();

for event in events.iter() {
    let _token = event.token();
    let _readable = event.is_readable();
    let _writable = event.is_writable();
    // Connection closed or errored — check these, or a dead socket spins.
    let _closed = event.is_read_closed() || event.is_write_closed();
    let _error = event.is_error();
}
```

**When to use it:** in the body of the loop. `is_read_closed` and `is_error` are
the ones people forget: a peer that hangs up leaves a socket permanently
readable, so a loop that only checks `is_readable` will spin on end-of-file
forever.

### Registration

#### `register`

Adds a source, with its token and interest.

```
use mio::net::TcpListener;
use mio::{Interest, Poll, Token};

let poll = Poll::new().unwrap();
let mut listener = TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();

poll.registry()
    .register(&mut listener, Token(0), Interest::READABLE)
    .unwrap();

// Registering the same source twice is an error, not a replacement.
assert!(poll
    .registry()
    .register(&mut listener, Token(0), Interest::READABLE)
    .is_err());
```

**When to use it:** once per source. The `&mut` is because registration can
modify the source — setting non-blocking mode, recording the handle — and the
error on double registration is the crate refusing to let two tokens silently
refer to one socket.

#### `reregister` and `deregister`

Change the interest, or stop hearing about a source.

```
use mio::net::TcpListener;
use mio::{Interest, Poll, Token};

let poll = Poll::new().unwrap();
let registry = poll.registry();
let mut listener = TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();

registry.register(&mut listener, Token(0), Interest::READABLE).unwrap();
registry.reregister(&mut listener, Token(0), Interest::WRITABLE).unwrap();
registry.deregister(&mut listener).unwrap();

// After deregistering, it can be registered again.
assert!(registry.register(&mut listener, Token(1), Interest::READABLE).is_ok());
```

**When to use it:** `reregister` when a connection's needs change — usually
adding writability while output is pending. `deregister` before dropping a
source you want to keep, though dropping it is enough when you are finished with
it entirely.

#### `Waker`

Wakes a blocked `poll` from another thread.

```
use mio::{Poll, Token, Waker};
use std::sync::Arc;

let poll = Poll::new().unwrap();
let waker = Arc::new(Waker::new(poll.registry(), Token(99)).unwrap());

// Cheap to clone and send anywhere.
let other = Arc::clone(&waker);
std::thread::spawn(move || other.wake()).join().unwrap().unwrap();
```

**When to use it:** any time work arrives from outside the loop — a job queue, a
shutdown flag, a timer thread. One `Waker` per `Poll` is the intended shape;
wrap it in an `Arc` and clone that rather than creating several.

### Network types

#### `net::TcpListener` and `net::TcpStream`

Non-blocking replacements for the `std` types.

```
use mio::net::{TcpListener, TcpStream};

let listener = TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
let addr = listener.local_addr().unwrap();

// connect returns immediately; the connection completes later.
let stream = TcpStream::connect(addr).unwrap();
assert!(stream.peer_addr().is_ok() || stream.take_error().is_ok());
```

**When to use it:** instead of `std::net`, which blocks. The key difference is
`connect` — it returns before the connection is established, so you register for
writability and treat the resulting event as "connected or failed", checking
`take_error` to tell which.

#### Reading until `WouldBlock`

The discipline edge-triggered notification demands.

```
use std::io::{ErrorKind, Read};

fn drain(source: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match source.read(&mut buf) {
            Ok(0) => break,                                   // <- peer closed
            Ok(n) => out.extend_from_slice(&buf[..n]),
            Err(e) if e.kind() == ErrorKind::WouldBlock => break, // <- drained
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}

let mut data = &b"hello"[..];
assert_eq!(drain(&mut data).unwrap(), b"hello");
```

**When to use it:** every single read, without exception. Stopping early leaves
data buffered with no further notification coming, which is the stall described
in the Overview. The `Interrupted` arm matters too — a signal can interrupt a
read that had more to give.
