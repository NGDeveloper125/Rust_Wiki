---
title: "hyper"
version: "1.12.0"
publisher: "Sean McArthur (seanmonstar), Core"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-26"
summary: "The HTTP implementation under most of the Rust ecosystem — correct, fast, and deliberately low-level. Since 1.0 it hands you one connection at a time and leaves the conveniences to `hyper-util` and to crates built on top."
domain: "HTTP, web & RPC"
categories: ["network-programming", "web-programming::http-server", "web-programming::http-client"]
repository: "https://github.com/hyperium/hyper"
docs: "https://docs.rs/hyper/1.12.0/hyper/"
---

## Overview

`hyper` is the HTTP engine almost everything in Rust is built on: `reqwest`,
`axum`, `tonic`, `warp` and Cargo's own downloader all sit on it. It implements
HTTP/1 and HTTP/2 — parsing, framing, connection management, the protocol rules
— and nothing above that.

**The most important thing to know is what 1.0 removed.** Earlier versions had
`hyper::Server::bind(..).serve(..)` and a `hyper::Client` that managed a
connection pool for you. Both are gone. 1.0 deliberately narrowed the crate to
the protocol layer, so what you get now is one connection at a time:

```
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;

async fn hello(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("hi"))))
}

// There is no Server type. You are handed one connection and serve it.
async fn serve_one(stream: tokio::net::TcpStream) {
    let _ = http1::Builder::new()
        .serve_connection(TokioIo::new(stream), service_fn(hello))
        .await;
}

let _ = serve_one;
```

The conveniences moved to `hyper-util`, which carries the connection pool
(`client::legacy::Client`), a listener-driven server, and the protocol
auto-detection builder (`server::conn::auto`). It is versioned 0.x on purpose:
those APIs are still settling, which is precisely why they are not in `hyper`
1.x.

**So the honest guidance is that most people should not use this crate
directly.** If you want an HTTP client, use `reqwest`. If you want a web server,
use `axum` — which is maintained by the same project and is a thin layer over
this. Reach for `hyper` when you are building one of those layers, implementing
a protocol that rides on HTTP, or need control the wrappers do not expose.

**Nothing is enabled by default.** `default = []`, so a bare dependency compiles
to almost nothing and your imports will not resolve. Pick what you need:
`client`, `server`, `http1`, `http2`, or `full` for all of them.

**Three pieces of 1.x plumbing you cannot avoid.** They account for most of the
confusion when following an older example:

- **Bodies are a trait, not a type.** `http_body_util` supplies the
  implementations — `Full` for a complete buffer, `Empty` for none, `BoxBody`
  when the type must be uniform, `StreamBody` for streaming. `BodyExt::collect`
  is how you read one.
- **`hyper` has its own `Read`/`Write` traits**, not `tokio`'s, so a
  `TcpStream` must be wrapped in `hyper_util::rt::TokioIo`. The "trait bound
  `TcpStream: hyper::rt::Read` is not satisfied" error means you forgot.
- **Request and Response come from the [`http`](http.md) crate**, so the types
  are shared with every other HTTP crate and the page for those is over there.

It requires `tokio` in practice (the runtime is pluggable in principle,
`TokioExecutor` in practice), MSRV is 1.63, and it is not `no_std`.

## When to use it

### Use case: Serving HTTP when you own the accept loop

A complete server, which in 1.x means writing the loop yourself.

```
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use tokio::net::TcpListener;

async fn route(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.uri().path() {
        "/health" => Ok(Response::new(Full::new(Bytes::from("ok")))),
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("not found")))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Port 0 asks the OS for a free port, so this runs anywhere.
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let server = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            // TokioIo adapts tokio's traits to hyper's.
            let io = TokioIo::new(stream);
            let _ = http1::Builder::new()
                .serve_connection(io, service_fn(route))
                .await;
        }
    });

    // Prove it answers, using hyper as the client too.
    let stream = tokio::net::TcpStream::connect(addr).await?;
    let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;
    let driver = tokio::spawn(async move {
        let _ = conn.await;
    });

    let request = Request::builder()
        .uri("/health")
        .header(hyper::header::HOST, addr.to_string())
        .body(Full::new(Bytes::new()))?;

    let response = sender.send_request(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Dropping the sender closes the connection. Without this, keep-alive
    // leaves serve_connection waiting for a second request and the await below
    // never returns.
    drop(sender);
    driver.await?;

    server.await?;
    Ok(())
}
```

**Why it fits:** the loop is the point. Owning `accept` means you decide the
concurrency limit, where TLS is layered in, how connections are counted and shed,
and what happens on shutdown — which is why a proxy, a load balancer or a
protocol gateway wants this crate rather than a framework. For an ordinary web
application that control is a cost rather than a benefit, and `axum` writes this
loop better than you will. In real code each accepted connection goes in its own
`tokio::spawn`; this example serves one so that it terminates — and note the
`drop(sender)`, which is what lets it: an HTTP/1.1 connection stays open for
reuse, so the server keeps waiting until the client's side is closed.

### Use case: Making a request over a connection you control

The client side, one connection at a time.

```
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::rt::TokioIo;

pub async fn fetch(
    host: &str,
    port: u16,
    path: &str,
) -> Result<(hyper::StatusCode, Bytes), Box<dyn std::error::Error + Send + Sync>> {
    let stream = tokio::net::TcpStream::connect((host, port)).await?;

    // handshake returns a sender and the connection task, which must be driven.
    let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let request = Request::builder()
        .uri(path)
        .header(hyper::header::HOST, host)
        .body(Empty::<Bytes>::new())?;

    let response = sender.send_request(request).await?;
    let status = response.status();

    // collect() drains the body; to_bytes() flattens it.
    let body = response.into_body().collect().await?.to_bytes();
    Ok((status, body))
}

// Not called here, since it would need a server.
let _ = fetch;
```

**Why it fits:** there is no pool, no redirect following, no retry and no TLS —
you get exactly one HTTP conversation over a socket you opened. That is right
when you are speaking to a known peer over a known transport: a sidecar on a
Unix socket, a health check, an internal service behind a mesh. For anything
facing the open internet, `reqwest` handles DNS, TLS, pooling, redirects and
compression, and reimplementing those is not a good use of an afternoon. Note
the spawned `connection` future: forget it and the request hangs forever,
because nothing is driving the protocol.

### Use case: Bodies that are not a single buffer

Streaming out, and reading in with a limit.

```
use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full, Limited};
use std::convert::Infallible;

// BoxBody gives one return type for several body kinds.
fn body_for(found: bool) -> BoxBody<Bytes, Infallible> {
    if found {
        Full::new(Bytes::from("payload")).boxed()
    } else {
        Empty::<Bytes>::new().boxed()
    }
}

#[tokio::main]
async fn main() {
    assert_eq!(
        body_for(true).collect().await.unwrap().to_bytes(),
        Bytes::from("payload")
    );
    assert!(body_for(false).collect().await.unwrap().to_bytes().is_empty());

    // Limited refuses a body larger than the cap, instead of buffering it.
    let oversized = Limited::new(Full::new(Bytes::from(vec![b'x'; 100])), 10);
    assert!(oversized.collect().await.is_err());

    let acceptable = Limited::new(Full::new(Bytes::from("small")), 100);
    assert!(acceptable.collect().await.is_ok());
}
```

**Why it fits:** `Limited` is the one to internalise, because the default is
dangerous: an incoming body is a stream of unknown length, and
`collect().await` on an untrusted request will happily buffer gigabytes into
memory. Wrapping it caps the damage and returns an error you can turn into a 413.
`BoxBody` solves the other everyday problem — two branches of a handler
producing different body types, which `impl Body` cannot reconcile any more than
it can two iterators.

## API map

Four areas: the server connection, the client connection, the service
abstraction, and bodies. The request and response types themselves belong to
[`http`](http.md).

### Serving

#### `server::conn::http1::Builder`

Serves one HTTP/1 connection.

```
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::time::Duration;

async fn hello(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("hi"))))
}

// The builder carries the per-connection protocol settings.
let builder = {
    let mut b = http1::Builder::new();
    b.keep_alive(true)
        .header_read_timeout(Duration::from_secs(30))
        .max_buf_size(64 * 1024);
    b
};

let _serve = |stream: tokio::net::TcpStream| async move {
    builder.serve_connection(TokioIo::new(stream), service_fn(hello)).await
};
```

**When to use it:** once per accepted connection, inside its own task. The
settings are where the protective defaults live — `header_read_timeout` is what
stops a slowloris client holding a connection open with a trickle of headers, and
it is worth setting explicitly on anything public. `with_upgrades()` on the
returned future is how you reach WebSockets, since the upgrade has to escape the
HTTP state machine. `server::conn::http2::Builder` is the same shape for HTTP/2,
and `hyper_util::server::conn::auto` picks between them by sniffing the preface.

#### `service_fn` and the `Service` trait

Turning a function into a request handler.

```
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use std::convert::Infallible;

async fn handler(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    // The request owns its body, so reading it consumes the request.
    let path = req.uri().path().to_string();
    Ok(Response::new(Full::new(Bytes::from(path))))
}

// service_fn wraps any async fn(Request) -> Result<Response, E>.
let service = service_fn(handler);
let _ = service;
```

**When to use it:** for anything short of a framework. The signature is the whole
contract: in a `Request<Incoming>`, out a `Result<Response<B>, E>`. `Infallible`
as the error type is the usual choice for a handler that turns its own failures
into status codes, which is what you want — an `Err` from a service aborts the
connection rather than producing a response. Implementing `Service` by hand is
for when the handler needs state or readiness, which is the door `tower`
middleware comes through.

### Requesting

#### `client::conn::http1::handshake`

Opens one client connection.

```
use bytes::Bytes;
use http_body_util::Empty;
use hyper::Request;
use hyper_util::rt::TokioIo;

pub async fn one_request(
    addr: std::net::SocketAddr,
) -> Result<hyper::StatusCode, Box<dyn std::error::Error + Send + Sync>> {
    let stream = tokio::net::TcpStream::connect(addr).await?;
    let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;

    // The connection future drives the protocol and must be polled.
    tokio::spawn(async move {
        let _ = connection.await;
    });

    // HTTP/1.1 requires a Host header; hyper does not invent one.
    let request = Request::builder()
        .uri("/")
        .header(hyper::header::HOST, addr.to_string())
        .body(Empty::<Bytes>::new())?;

    Ok(sender.send_request(request).await?.status())
}

let _ = one_request;
```

**When to use it:** when you have the socket and want exactly one connection's
worth of HTTP. The two-value return is the part to get right — `sender` issues
requests and `connection` is a future that must be driven for anything to
happen, so it goes in a `spawn`. `sender.send_request` can be called repeatedly
for keep-alive, and `ready()` tells you when the connection can take another.
For pooling, DNS and TLS, use `hyper_util::client::legacy::Client` or `reqwest`.

### Bodies

#### `Incoming` and `BodyExt::collect`

Reading a body that arrives over the network.

```
use http_body_util::{BodyExt, Limited};
use hyper::body::Incoming;
use hyper::Request;

// Always bound an untrusted body before buffering it.
pub async fn read_capped(
    req: Request<Incoming>,
    limit: usize,
) -> Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>> {
    let body = Limited::new(req.into_body(), limit);
    Ok(body.collect().await?.to_bytes())
}

let _ = read_capped;
```

**When to use it:** on every incoming body. `Incoming` is a stream of frames
with no length guarantee, and `collect()` buffers all of it — so the `Limited`
wrapper is not optional hardening but the difference between a service and a
memory exhaustion bug. `collect()` yields a `Collected`, from which `to_bytes()`
gives one contiguous buffer and `aggregate()` gives a `Buf` without copying.
`BodyExt` has to be in scope for any of it.

#### `Full`, `Empty` and `BoxBody`

The outgoing body types.

```
use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Body;
use std::convert::Infallible;

// A complete buffer, with a known length.
let full = Full::new(Bytes::from("12345"));
assert_eq!(full.size_hint().exact(), Some(5));

// No body at all, for a 204 or a HEAD response.
let empty = Empty::<Bytes>::new();
assert_eq!(empty.size_hint().exact(), Some(0));

// Erased, so two branches can share a return type.
let erased: BoxBody<Bytes, Infallible> = Full::new(Bytes::from("x")).boxed();
let _ = erased;
```

**When to use it:** `Full` for a response you have already built — a rendered
page, a JSON document — since the known length lets `hyper` set `Content-Length`
and avoid chunked encoding. `Empty` where a body would be wrong. `BoxBody` at
any boundary where the body type must be uniform, which in practice means most
handler signatures and every trait object. The boxing costs an allocation and a
virtual call per frame, which is irrelevant next to the network.

#### `StreamBody` and `Frame`

Sending a body you do not have all of yet.

```
use bytes::Bytes;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use std::convert::Infallible;

#[tokio::main]
async fn main() {
    // Each item is a Frame, which may be data or trailers.
    let chunks: Vec<Result<Frame<Bytes>, Infallible>> = vec![
        Ok(Frame::data(Bytes::from("first "))),
        Ok(Frame::data(Bytes::from("second"))),
    ];

    let body = StreamBody::new(futures_util::stream::iter(chunks));
    let collected = body.collect().await.unwrap().to_bytes();

    assert_eq!(collected, Bytes::from("first second"));
}
```

**When to use it:** for a response generated as it goes — a large file, a
database cursor, server-sent events. The body is sent with chunked encoding
since the length is unknown, and the client can start reading before you have
finished producing. `Frame` is an enum over data and trailers, which is why the
stream's item type looks heavier than a plain `Bytes`: HTTP/2 trailers are
expressible here, and `Frame::data` is the common case.

#### `hyper_util::rt::TokioIo`

Bridging `tokio`'s I/O traits to `hyper`'s.

```
use hyper_util::rt::TokioIo;

// hyper::rt::Read and Write are hyper's own traits, so tokio types need wrapping.
let _wrap = |stream: tokio::net::TcpStream| TokioIo::new(stream);

// It is a transparent adapter: inner() and into_inner() get the stream back.
let _unwrap = |io: TokioIo<tokio::net::TcpStream>| io.into_inner();
```

**When to use it:** on every stream you hand to `hyper`, which in a `tokio`
program is all of them. The reason it exists is that `hyper` 1.x defines its own
`Read`/`Write` traits so it is not tied to one runtime — a principled choice
whose visible cost is this wrapper. It composes, so a TLS stream from
`tokio-rustls` wraps the same way, and `TokioExecutor` is its counterpart for
HTTP/2, which needs to spawn tasks of its own.
