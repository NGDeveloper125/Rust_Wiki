---
title: "http"
version: "1.5.0"
publisher: "Carl Lerche (carllerche), Sean McArthur (seanmonstar)"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-29"
summary: "The shared vocabulary of HTTP in Rust — `Request`, `Response`, `HeaderMap`, `StatusCode`, `Uri`. Types only: it sends nothing and serves nothing, which is exactly why every client and server agrees on it."
domain: "HTTP, web & RPC"
categories: ["http", "web", "types"]
repository: "https://github.com/hyperium/http"
---

## Overview

`http` is a vocabulary, not a library. It defines what a request and a response
*are* — method, URI, version, headers, body — and implements no networking at
all. There is no client here, no server, no `send`:

```
use http::{Request, Response, StatusCode};

let request = Request::get("https://example.com/users")
    .header("accept", "application/json")
    .body(())
    .unwrap();

assert_eq!(request.method(), "GET");
assert_eq!(request.uri().path(), "/users");

let response = Response::builder()
    .status(StatusCode::NOT_FOUND)
    .body("no such user")
    .unwrap();

assert_eq!(response.status(), 404);
assert!(response.status().is_client_error());
```

**That emptiness is the whole point.** Because `hyper`, `reqwest`, `axum`,
`tower-http` and `tonic` all speak these types, a piece of middleware written
against `http::Request` works with any of them. Authentication, logging, rate
limiting and tracing become reusable across the ecosystem instead of being
written once per framework — which is why the crate sits high on the download
charts while almost nobody depends on it deliberately.

**The body is a type parameter, and `http` says nothing about it.**
`Request<T>` is generic: `T` might be `()`, a `String`, `Bytes`, or a streaming
type from `http-body`. This crate never reads or writes it. So a function taking
`Request<B>` is agnostic about how the body arrives, and the framework decides
what `B` actually is.

Two things are worth knowing before you use it.

**Headers are not a `HashMap<String, String>`.** `HeaderMap` is keyed by
`HeaderName`, which is case-insensitive and pre-validated, and a name can appear
more than once — `Set-Cookie` routinely does. `get` returns the *first* value;
reaching for `get_all` is what you want when repetition is meaningful, and
forgetting that is a real bug rather than a style point.

**Header values are bytes, not text.** `HeaderValue` is not a `String`, because
HTTP permits bytes that are not UTF-8. Reading one back is `to_str()`, which
returns a `Result` — and treating that as infallible is how a client falls over
on a server it didn't expect.

It has no meaningful dependencies, requires Rust 1.57, and is not `no_std`.
You will rarely add it directly: a framework re-exports it, and depending on it
yourself is mainly for writing middleware or a client-agnostic library.

## When to use it

### Use case: Middleware that works with any framework

The reason the crate exists. A function over `http::Request` can sit in front of
hyper, axum or tower without knowing which.

```
use http::{HeaderValue, Request, StatusCode};

/// Reject a request without a bearer token, whatever produced it.
fn authorize<B>(request: &Request<B>) -> Result<&str, StatusCode> {
    let header = request
        .headers()
        .get(http::header::AUTHORIZATION)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let value = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    value.strip_prefix("Bearer ").ok_or(StatusCode::UNAUTHORIZED)
}

let ok = Request::builder()
    .header("authorization", HeaderValue::from_static("Bearer abc123"))
    .body(())
    .unwrap();
assert_eq!(authorize(&ok), Ok("abc123"));

let missing = Request::builder().body(()).unwrap();
assert_eq!(authorize(&missing), Err(StatusCode::UNAUTHORIZED));

// A non-UTF-8 header is a client error, not a panic.
let bad = Request::builder()
    .header("authorization", HeaderValue::from_bytes(&[0xff, 0xfe]).unwrap())
    .body(())
    .unwrap();
assert_eq!(authorize(&bad), Err(StatusCode::BAD_REQUEST));
```

**Why it fits:** `B` is unconstrained, so this works whether the body is a
`String`, a stream or nothing at all. The third case is the one hand-written
middleware usually gets wrong — `to_str` really can fail, and the type makes you
decide what that means.

### Use case: Pulling a request apart

Routing and validation need the pieces, and the `Uri` type already has them
parsed.

```
use http::Uri;

let uri: Uri = "https://api.example.com:8443/v1/users?active=true#top"
    .parse()
    .unwrap();

assert_eq!(uri.scheme_str(), Some("https"));
assert_eq!(uri.host(), Some("api.example.com"));
assert_eq!(uri.port_u16(), Some(8443));
assert_eq!(uri.path(), "/v1/users");
assert_eq!(uri.query(), Some("active=true"));

// A relative URI is legal and has no authority — which is what a server sees.
let relative: Uri = "/health".parse().unwrap();
assert_eq!(relative.path(), "/health");
assert_eq!(relative.host(), None);
```

**Why it fits:** splitting a URL with `split('/')` breaks on ports, escapes and
relative forms. `Uri` has done the parsing, so routing works on `path()` and
never sees the rest. Note the fragment is absent — `#top` is client-side and is
never sent, so the type does not expose it.

### Use case: Carrying typed data alongside a request

`Extensions` is a per-request map keyed by type, for values that are not headers.

```
use http::Request;

#[derive(Clone, Debug, PartialEq)]
struct UserId(u64);

#[derive(Clone, Debug, PartialEq)]
struct RequestId(String);

let mut request = Request::builder().body(()).unwrap();

// Middleware inserts what it learned.
request.extensions_mut().insert(UserId(42));
request.extensions_mut().insert(RequestId("abc".to_string()));

// A later layer reads it back, by type.
assert_eq!(request.extensions().get::<UserId>(), Some(&UserId(42)));
assert_eq!(
    request.extensions().get::<RequestId>(),
    Some(&RequestId("abc".to_string())),
);

// A type nobody inserted is simply absent.
assert_eq!(request.extensions().get::<u32>(), None);
```

**Why it fits:** an authentication layer resolves a user and later layers need
it, but it is not a header and should not be serialised. Keying by type means
two layers cannot collide over a string key — though it also means only one
value per type, so wrap in a newtype rather than storing a bare `String`.

## API map

Six types carry the crate: `Request` and `Response` as the messages, `HeaderMap`
for headers, and `Method`, `StatusCode` and `Uri` for the fields with structure.

### Messages

#### `Request::builder`

Builds a request, with a body of whatever type you choose.

```
use http::Request;

let request = Request::builder()
    .method("POST")
    .uri("https://example.com/items")
    .header("content-type", "application/json")
    .body(r#"{"name":"widget"}"#)
    .unwrap();

assert_eq!(request.method(), "POST");
assert_eq!(request.body(), &r#"{"name":"widget"}"#);
```

**When to use it:** constructing a request to send, or one to feed a handler in
a test. `body` finishes the builder and returns `Result`, because an invalid
header or URI supplied along the way is only reported at the end — so `unwrap`
here is checking everything you passed, not just the body.

#### `Request::get` and friends

Shorthands for the common methods.

```
use http::Request;

let request = Request::get("/health").body(()).unwrap();
assert_eq!(request.method(), "GET");
assert_eq!(request.uri().path(), "/health");

let post = Request::post("/items").body(()).unwrap();
assert_eq!(post.method(), "POST");
```

**When to use it:** whenever the method is fixed and known, which is most of the
time. `Request::get(uri)` is `Request::builder().method("GET").uri(uri)`,
returning the same `Builder` — so headers still chain on afterwards.

#### `Response::builder`

The same shape for the other direction.

```
use http::{Response, StatusCode};

let response = Response::builder()
    .status(StatusCode::CREATED)
    .header("location", "/items/1")
    .body(())
    .unwrap();

assert_eq!(response.status(), StatusCode::CREATED);
assert_eq!(response.headers()["location"], "/items/1");
```

**When to use it:** producing a response in a handler, and building fixtures in
tests. `Response::new(body)` is the shorthand when the status is 200 and there
are no headers.

#### `into_parts` and `from_parts`

Splitting a message from its body, and reassembling it.

```
use http::Request;

let request = Request::post("/items").body("payload").unwrap();

let (parts, body) = request.into_parts();
assert_eq!(parts.method, "POST");
assert_eq!(body, "payload");

// Swap the body for a different type, keeping everything else.
let rebuilt = Request::from_parts(parts, body.len());
assert_eq!(rebuilt.body(), &7);
assert_eq!(rebuilt.method(), "POST");
```

**When to use it:** transforming a body while keeping the metadata — decoding,
buffering, converting a stream to bytes. This is the core move of body
middleware, and it is why the body being a type parameter matters.

### Headers

#### `HeaderMap::get` and `insert`

The map, keyed case-insensitively.

```
use http::HeaderMap;
use http::header::{CONTENT_TYPE, HeaderValue};

let mut headers = HeaderMap::new();
headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

// Lookup ignores case, because header names are case-insensitive.
assert_eq!(headers.get("content-type").unwrap(), "application/json");
assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
assert_eq!(headers.get(CONTENT_TYPE).unwrap(), "application/json");

// insert replaces; a missing header is None rather than an error.
assert!(headers.get("accept").is_none());
```

**When to use it:** reading and setting single-valued headers. Use the constants
from `http::header` where they exist — they are pre-validated and cannot be
misspelled, which a string literal can.

#### `append` and `get_all`

Headers that legitimately appear more than once.

```
use http::HeaderMap;
use http::header::{HeaderValue, SET_COOKIE};

let mut headers = HeaderMap::new();
headers.append(SET_COOKIE, HeaderValue::from_static("a=1"));
headers.append(SET_COOKIE, HeaderValue::from_static("b=2"));

// get returns only the first — a real source of dropped cookies.
assert_eq!(headers.get(SET_COOKIE).unwrap(), "a=1");

// get_all returns every value.
let all: Vec<&str> = headers
    .get_all(SET_COOKIE)
    .iter()
    .map(|v| v.to_str().unwrap())
    .collect();
assert_eq!(all, ["a=1", "b=2"]);
```

**When to use it:** `Set-Cookie`, `Via`, `Warning`, and any header a proxy may
add to. The distinction matters: `insert` replaces every existing value and
`append` adds one, so using `insert` in a loop silently keeps only the last.

#### `HeaderValue`

A header's value: bytes, with construction that validates.

```
use http::header::HeaderValue;

// Free for a literal known at compile time.
let json = HeaderValue::from_static("application/json");
assert_eq!(json.to_str().unwrap(), "application/json");

// Fallible from runtime data, because control characters are not allowed.
assert!(HeaderValue::from_str("fine").is_ok());
assert!(HeaderValue::from_str("bad\nvalue").is_err()); // <- header injection

// Values may be non-UTF-8, so reading back is a Result.
let raw = HeaderValue::from_bytes(&[0xff]).unwrap();
assert!(raw.to_str().is_err());
```

**When to use it:** whenever you set a header from data you did not write. The
rejection of `\n` is the important one — it is what stops a user-supplied value
injecting a second header, and it is why building headers by string
concatenation is a security bug rather than a style choice.

#### `HeaderName`

The key type, lowercase and validated.

```
use http::header::{HeaderName, CONTENT_LENGTH};

let name = HeaderName::from_static("x-request-id");
assert_eq!(name.as_str(), "x-request-id");

// Constants exist for the standard ones.
assert_eq!(CONTENT_LENGTH.as_str(), "content-length");

// Invalid names are rejected rather than sent.
assert!(HeaderName::from_bytes(b"bad header").is_err());
```

**When to use it:** for custom headers, once, in a constant — `from_static`
panics on an invalid name, which is what you want at startup rather than per
request. Note names normalise to lowercase, so `as_str` never gives back the
casing you typed.

### Fields with structure

#### `StatusCode`

The response status, with constants and classification.

```
use http::StatusCode;

assert_eq!(StatusCode::OK.as_u16(), 200);
assert_eq!(StatusCode::NOT_FOUND.canonical_reason(), Some("Not Found"));

// Classification, rather than comparing ranges by hand.
assert!(StatusCode::CREATED.is_success());
assert!(StatusCode::MOVED_PERMANENTLY.is_redirection());
assert!(StatusCode::BAD_REQUEST.is_client_error());
assert!(StatusCode::BAD_GATEWAY.is_server_error());

// Any number in range is valid, including ones nobody standardised.
assert!(StatusCode::from_u16(299).is_ok());
assert!(StatusCode::from_u16(99).is_err());
```

**When to use it:** everywhere a status appears. The `is_*` predicates are worth
preferring over `>= 200 && < 300` — they read as intent, and they cannot get the
boundary wrong. `from_u16` accepting unassigned codes is deliberate: servers do
use them, and rejecting one would break a client for no reason.

#### `Method`

The request method, with constants and cheap comparison.

```
use http::Method;

assert_eq!(Method::GET, "GET");
assert_eq!(Method::POST.as_str(), "POST");

// Parsing, including extension methods the crate doesn't name.
let custom: Method = "PROPFIND".parse().unwrap();
assert_eq!(custom.as_str(), "PROPFIND");

// Safety and idempotence, which routing and retries care about.
assert!(Method::GET.is_safe());
assert!(Method::PUT.is_idempotent());
assert!(!Method::POST.is_idempotent());
```

**When to use it:** routing and retry logic. `is_idempotent` is the one that
earns its keep — a client deciding whether to retry a failed request needs
exactly that answer, and hard-coding a list of methods gets extension methods
wrong.

#### `Uri`

The target, parsed into its parts.

```
use http::Uri;

let uri = Uri::from_static("https://example.com/a/b?x=1");

assert_eq!(uri.scheme_str(), Some("https"));
assert_eq!(uri.authority().unwrap().as_str(), "example.com");
assert_eq!(uri.path(), "/a/b");
assert_eq!(uri.query(), Some("x=1"));

// Rebuilding from parts, for rewriting a request's target.
let mut parts = uri.into_parts();
parts.path_and_query = Some("/c?y=2".parse().unwrap());
let rewritten = Uri::from_parts(parts).unwrap();
assert_eq!(rewritten.to_string(), "https://example.com/c?y=2");
```

**When to use it:** routing, proxying and building requests. `into_parts` and
`from_parts` are how a proxy retargets a request without reassembling a string —
which is both faster and immune to the escaping mistakes string surgery invites.
Query *parsing* is not here; `form_urlencoded` or `serde_urlencoded` does that.

#### `Version`

Which HTTP version the message belongs to.

```
use http::{Request, Version};

let request = Request::builder()
    .version(Version::HTTP_2)
    .uri("/")
    .body(())
    .unwrap();

assert_eq!(request.version(), Version::HTTP_2);

// The default is HTTP/1.1.
assert_eq!(Request::builder().body(()).unwrap().version(), Version::HTTP_11);
```

**When to use it:** rarely in application code — the client or server negotiates
it. It matters when behaviour differs by version: HTTP/2 forbids some headers
HTTP/1.1 requires, so a middleware that sets `Connection` needs to know which it
is talking to.
