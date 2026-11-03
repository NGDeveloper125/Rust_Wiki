---
title: "url"
version: "2.5.8"
publisher: "Simon Sapin (SimonSapin), Ana (Hoverbear), Sean McArthur (seanmonstar), cargo publish, rust-url"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-11-03"
summary: "The WHATWG URL Standard in Rust — the parser a browser would use. It normalises as it parses, resolves relative links against a base, and hands back the pieces as borrowed slices."
domain: "HTTP, web & RPC"
categories: ["encoding", "web-programming", "parser-implementations"]
repository: "https://github.com/servo/rust-url"
docs: "https://docs.rs/url/2.5.8/url/"
---

## Overview

URLs look like strings and behave like a grammar. `url` implements the
[WHATWG URL Standard](https://url.spec.whatwg.org/) — the same rules a browser
applies — which means **parsing is also normalising**. That is the single most
important thing to know about the crate, because it changes your input:

```
use url::Url;

// A missing path becomes "/".
assert_eq!(Url::parse("http://example.com").unwrap().as_str(), "http://example.com/");

// Scheme and host are lowercased; the path's case is left alone.
assert_eq!(
    Url::parse("HTTP://Example.COM/A/b").unwrap().as_str(),
    "http://example.com/A/b",
);

// A default port for the scheme is dropped.
assert_eq!(Url::parse("http://x.test:80/a").unwrap().as_str(), "http://x.test/a");
assert_eq!(Url::parse("http://x.test:8080/a").unwrap().as_str(), "http://x.test:8080/a");

// Dot segments are resolved, not preserved.
assert_eq!(Url::parse("http://x.test/a/./b/../c").unwrap().as_str(), "http://x.test/a/c");

// Unicode hosts become punycode; illegal characters get percent-encoded.
assert_eq!(Url::parse("http://münchen.de/weg").unwrap().as_str(), "http://xn--mnchen-3ya.de/weg");
assert_eq!(Url::parse("http://x.test/a b").unwrap().as_str(), "http://x.test/a%20b");
```

That is a feature: it is why two spellings of the same address compare equal,
and why you can hand the result to an HTTP client without wondering what it
will do with it. It is also why `url` is the wrong tool if your job is to
preserve a URL byte-for-byte — a signature over the original text, say, or a
proxy that must forward exactly what it received.

**It parses absolute URLs only.** The most common first surprise is not a
quirk, it is the spec:

```
use url::{ParseError, Url};

// A bare host is a *relative reference*, not a URL.
assert_eq!(Url::parse("example.com/path"), Err(ParseError::RelativeUrlWithoutBase));
assert_eq!(Url::parse("/just/a/path"), Err(ParseError::RelativeUrlWithoutBase));
assert_eq!(Url::parse("//example.com/x"), Err(ParseError::RelativeUrlWithoutBase));

// Relative input needs a base to resolve against.
let base = Url::parse("https://example.com/docs/").unwrap();
assert_eq!(base.join("/just/a/path").unwrap().as_str(), "https://example.com/just/a/path");
```

A `Url` is one `String` plus a handful of byte offsets into it — 88 bytes on a
64-bit target. Every accessor is a borrow into that string rather than a fresh
allocation, so reading the parts is cheap and `as_str()` is free. The cost
lands elsewhere: **24 transitive dependencies**, nearly all of them the ICU
Unicode tables that `idna` needs to turn `münchen.de` into punycode correctly.
If that weight is unacceptable and you do not need IDNA, you want a smaller
parser or `http::Uri`, not a feature flag — there isn't one.

**Against [`http`](http.md)'s `Uri`**, which you already have if you are using
`hyper` or `reqwest`. They disagree on nearly everything, and the differences
are the reason to pick one:

- `Uri` accepts a relative reference (`"/health".parse::<Uri>()` is fine);
  `Url` requires an absolute URL.
- `Uri` normalises almost nothing: it keeps `Example.COM` as written, keeps a
  redundant `:80`, and leaves `/a/./b/../c` alone. `Url` fixes all three.
- `Uri` has no notion of resolving against a base, no IDNA, and no
  percent-encoding of what you give it.
- `Uri` is cheap to construct and cheap to depend on. `Url` is neither, and in
  exchange it is correct about the hard cases.

So: `Uri` for the inside of an HTTP stack, where you are handling a request
target that has already been validated. `Url` when the input came from a user,
a config file, an HTML page or a redirect — anywhere the question "is this even
a URL, and what does it actually point at" is a real one.

**One rule explains most of the surprises below: nothing is decoded on the way
out.** Every accessor hands you the percent-encoded text as it sits in the
URL — `path()`, `fragment()`, `password()`, all of them — and `query_pairs()` is
the single exception. Decoding is deliberately somebody else's job:
`url` re-exports `form_urlencoded` for query strings, but a percent-encoded
*path* needs the `percent-encoding` crate as a direct dependency of your own.

`serde` support is behind the `serde` feature, and serialises as the normalised
string:

```
use url::Url;

let parsed: Url = serde_json::from_str("\"https://a.example/x?q=1\"").unwrap();
assert_eq!(parsed.host_str(), Some("a.example"));
assert_eq!(serde_json::to_string(&parsed).unwrap(), "\"https://a.example/x?q=1\"");

// Deserialisation fails the same way parsing does.
let bad = serde_json::from_str::<Url>("\"not a url\"");
assert!(bad.is_err());
```

**MSRV is 1.63.** `no_std` works with `default-features = false`, but needs
`alloc` — the whole design rests on an owned `String`.

## When to use it

### Use case: Resolving a link against the page it came from

Every crawler, feed reader and redirect follower needs this, and hand-rolling
it is how you get `https://example.com/a/b/../../../etc/passwd`.

```
use url::Url;

fn resolve_all(page: &str, links: &[&str]) -> Vec<String> {
    let base = Url::parse(page).expect("page URL is absolute");
    links
        .iter()
        .filter_map(|link| base.join(link).ok())
        .map(|url| url.to_string())
        .collect()
}

fn main() {
    let found = resolve_all(
        "https://example.com/blog/2026/post.html",
        &["intro.html", "/about", "../index.html", "?page=2", "#top",
          "//cdn.example.net/x.js", "https://other.test/y"],
    );

    assert_eq!(found, [
        // A sibling: the base's last segment is REPLACED, not appended to.
        "https://example.com/blog/2026/intro.html",
        // Root-relative.
        "https://example.com/about",
        // Up one directory.
        "https://example.com/blog/index.html",
        // Query only: same path.
        "https://example.com/blog/2026/post.html?page=2",
        // Fragment only.
        "https://example.com/blog/2026/post.html#top",
        // Protocol-relative: inherits https.
        "https://cdn.example.net/x.js",
        // Already absolute: the base is ignored.
        "https://other.test/y",
    ]);
}
```

**Why it fits:** `join` is the WHATWG relative-resolution algorithm, which
handles all seven of those shapes and a dozen more you have not thought of.
**The trap is the first one.** Because the base is `.../post.html` and not
`.../post.html/`, joining `intro.html` replaces `post.html` — you get
`/blog/2026/intro.html`, not `/blog/2026/post.html/intro.html`. That is correct
and it is what a browser does, but if you built the base yourself and forgot
the trailing slash, every link you resolve silently loses a directory. When the
base names a directory, end it in `/`.

### Use case: Reading and building a query string

The part people most often do with `format!` and come to regret.

```
use url::Url;

fn main() {
    // Reading: query_pairs() percent-decodes, and `+` means space.
    let url = Url::parse("https://search.test/s?q=rust+web&tag=a&tag=b&empty=").unwrap();
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    assert_eq!(pairs, [
        ("q".to_string(), "rust web".to_string()),
        ("tag".to_string(), "a".to_string()),
        ("tag".to_string(), "b".to_string()),
        ("empty".to_string(), String::new()),
    ]);

    // A repeated key is not deduplicated; collect the ones you want.
    let tags: Vec<_> = url.query_pairs().filter(|(k, _)| k == "tag").map(|(_, v)| v).collect();
    assert_eq!(tags, ["a", "b"]);

    // Writing: append_pair escapes for you.
    let mut built = Url::parse("https://search.test/s").unwrap();
    built
        .query_pairs_mut()
        .append_pair("q", "rust & web")
        .append_pair("note", "100%");
    assert_eq!(built.as_str(), "https://search.test/s?q=rust+%26+web&note=100%25");

    // And it round-trips.
    assert_eq!(built.query_pairs().next().unwrap().1, "rust & web");

    // clear() empties the query but leaves the `?` behind.
    built.query_pairs_mut().clear();
    assert_eq!(built.query(), Some(""));
    assert_eq!(built.as_str(), "https://search.test/s?");

    // set_query(None) is how you remove it.
    built.set_query(None);
    assert_eq!(built.query(), None);
    assert_eq!(built.as_str(), "https://search.test/s");
}
```

**Why it fits:** the escaping rules here are not guessable — `&` becomes `%26`
in a value but is the separator between pairs, space becomes `+` in form
encoding but `%20` elsewhere in a URL, and `%` has to escape itself.
`query_pairs_mut` knows all of it. Two things to carry away: a query is a
*multimap*, so treating `query_pairs()` as a dictionary quietly drops repeated
keys; and `clear()` is not the same as removing the query, which is the kind of
difference that only shows up when something downstream compares URLs as
strings.

### Use case: Vetting a URL that came from outside

Before you fetch it, decide whether you should.

```
use url::{Host, Url};

#[derive(Debug, PartialEq)]
enum Reject {
    NotAUrl,
    BadScheme,
    NoHost,
    Private,
}

fn vet(input: &str) -> Result<Url, Reject> {
    let url = Url::parse(input).map_err(|_| Reject::NotAUrl)?;

    // Allowlist, never a denylist: "javascript", "data" and "file" are all URLs.
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Reject::BadScheme);
    }

    match url.host().ok_or(Reject::NoHost)? {
        // Reject literal addresses that point back at us.
        Host::Ipv4(ip) if ip.is_loopback() || ip.is_private() => Err(Reject::Private),
        Host::Ipv6(ip) if ip.is_loopback() => Err(Reject::Private),
        Host::Domain(name) if name == "localhost" => Err(Reject::Private),
        _ => Ok(url),
    }
}

fn main() {
    assert!(vet("https://api.example.com/v1").is_ok());

    assert_eq!(vet("example.com"), Err(Reject::NotAUrl));
    assert_eq!(vet("javascript:alert(1)"), Err(Reject::BadScheme));
    assert_eq!(vet("file:///etc/passwd"), Err(Reject::BadScheme));
    assert_eq!(vet("http://127.0.0.1:8080/admin"), Err(Reject::Private));
    assert_eq!(vet("http://[::1]/admin"), Err(Reject::Private));
    assert_eq!(vet("http://localhost/admin"), Err(Reject::Private));

    // Normalisation closes the obvious ways round a naive string check.
    assert_eq!(vet("HTTP://LOCALHOST/admin"), Err(Reject::Private));
    assert_eq!(vet("http://127.0.0.1./admin"), Err(Reject::Private));
}
```

**Why it fits:** this is the check a string comparison cannot do. `host()`
returns a parsed `Host`, so `127.0.0.1` arrives as an `Ipv4Addr` you can ask
`is_private()` rather than a string you have to pattern-match; and the
normalisation that happens during parsing is what makes `HTTP://LOCALHOST` and
`http://127.0.0.1.` fail the same check as their canonical spellings. Two
caveats, because this example is not a complete SSRF defence: a hostname can
still *resolve* to a private address, which only DNS can tell you (see
`socket_addrs`), and the scheme test must be an allowlist — new URL schemes
keep being invented, and `cannot_be_a_base()` is true for more of them than you
will remember to list.

## API map

Parsing, seventeen accessors onto the result, setters that keep it valid, and
the file-path bridge.

### Parsing

#### `Url::parse` and `FromStr`

The entry point, and the only way to make a `Url` from text.

```
use url::Url;

fn main() {
    let a = Url::parse("https://example.test/a?b=c#d").unwrap();

    // FromStr is the same thing, which is handy with `?` and in config structs.
    let b: Url = "https://example.test/a?b=c#d".parse().unwrap();
    assert_eq!(a, b);

    // Display and as_str both give the normalised serialisation.
    assert_eq!(a.to_string(), a.as_str());
    assert_eq!(String::from(a.clone()), "https://example.test/a?b=c#d");

    // Non-special schemes parse, and are left much more alone.
    let mail = Url::parse("mailto:someone@example.test").unwrap();
    assert_eq!(mail.scheme(), "mailto");
    assert_eq!(mail.path(), "someone@example.test");
    assert!(mail.cannot_be_a_base());
    assert!(!mail.is_special());
}
```

**When to use it:** on any URL-shaped string entering your program, once, as
early as you can — the parsed value is the thing to pass around, not the
string. Note that parsing is not cheap enough to do in a loop over the same
input: it allocates and copies, so parse once and clone or borrow after. For a
`&'static str` you are certain of, `parse().unwrap()` in a `OnceLock` beats
re-parsing.

#### `ParseError` and `SyntaxViolation`

Why it failed, and what it quietly fixed.

```
use url::{ParseError, Url};

fn main() {
    // The variants you will actually match on.
    assert_eq!(Url::parse("example.test/x"), Err(ParseError::RelativeUrlWithoutBase));
    assert_eq!(Url::parse("http://"), Err(ParseError::EmptyHost));
    assert_eq!(Url::parse("http://a.test:99999/"), Err(ParseError::InvalidPort));
    assert_eq!(Url::parse("http://[zz]/"), Err(ParseError::InvalidIpv6Address));

    // A malformed scheme is reported as a relative URL, not a scheme error.
    assert_eq!(Url::parse("htt p://a.test/"), Err(ParseError::RelativeUrlWithoutBase));

    // Violations are things the spec says to accept *and* repair.
    let seen = std::cell::RefCell::new(Vec::new());
    let repaired = Url::options()
        .syntax_violation_callback(Some(&|v| seen.borrow_mut().push(v.description())))
        .parse("http://example.test/a\\b?x=1 2")
        .unwrap();

    // The backslash became a slash and the space was encoded.
    assert_eq!(repaired.as_str(), "http://example.test/a/b?x=1%202");
    assert_eq!(*seen.borrow(), ["backslash", "non-URL code point"]);
}
```

**When to use it:** `ParseError` implements `Error` and `Display`, so most code
just uses `?` or `map_err`; match on the variants only when you want to tell a
user *which* part of their input was wrong. `SyntaxViolation` is the more
interesting one and almost nobody knows it exists: it is how you find out the
parser silently repaired the input, which matters if you are linting
user-supplied URLs or wondering why what you stored is not what you typed. Note
the callback is `Fn`, not `FnMut`, so collecting from it needs a `Cell` or
`RefCell`.

#### `Url::options` and `Url::parse_with_params`

Parsing with a base, and building a query at parse time.

```
use url::Url;

fn main() {
    let base = Url::parse("https://api.example.test/v2/").unwrap();

    // The reusable form of join, for when the base is fixed and the input varies.
    let parser = Url::options().base_url(Some(&base));
    assert_eq!(parser.parse("users/7").unwrap().as_str(), "https://api.example.test/v2/users/7");
    assert_eq!(parser.parse("/root").unwrap().as_str(), "https://api.example.test/root");

    // An absolute input ignores the base, which is what makes this safe to
    // apply to untrusted strings only if you re-check the host afterwards.
    let escaped = parser.parse("https://elsewhere.test/x").unwrap();
    assert_eq!(escaped.host_str(), Some("elsewhere.test"));

    // parse_with_params appends an escaped query.
    let search = Url::parse_with_params(
        "https://search.test/s",
        &[("q", "rust lang"), ("page", "2")],
    ).unwrap();
    assert_eq!(search.as_str(), "https://search.test/s?q=rust+lang&page=2");
}
```

**When to use it:** `options().base_url(..)` when one base serves many inputs —
an API client resolving endpoint paths, a crawler working through one page's
links — since it reads better than `base.join(..)` repeated and makes the base
the configured thing. `parse_with_params` is the one-liner for a search URL;
past two or three parameters, build the `Url` and use `query_pairs_mut` so you
are not maintaining a tuple slice. The `escaped` assertion above is the warning
worth keeping: a base does not confine the result.

### Reading the parts

#### `scheme`, `is_special` and `cannot_be_a_base`

What kind of URL you are holding.

```
use url::Url;

fn main() {
    let web = Url::parse("https://example.test/a").unwrap();
    assert_eq!(web.scheme(), "https");     // always lowercase
    assert!(web.is_special());             // http, https, ws, wss, ftp, file
    assert!(!web.cannot_be_a_base());
    assert!(web.has_host());

    let mail = Url::parse("mailto:a@b.test").unwrap();
    assert_eq!(mail.scheme(), "mailto");
    assert!(!mail.is_special());
    assert!(mail.cannot_be_a_base());      // no authority, no path segments
    assert!(!mail.has_host());

    // "file" is special, can be a base, and yet has no host and no port.
    let file = Url::parse("file:///tmp/x").unwrap();
    assert!(file.is_special());
    assert!(!file.cannot_be_a_base());
    assert!(!file.has_host());
    assert_eq!(file.host_str(), None);
    assert_eq!(file.port_or_known_default(), None);
}
```

**When to use it:** `scheme()` for the allowlist every piece of code accepting
outside URLs needs. `cannot_be_a_base()` is the one that saves you from runtime
surprises: it is true for `mailto:`, `data:`, `blob:` and friends, and for those
`path_segments()` returns `None` and `path_segments_mut()` returns `Err` — so
check it before reaching for either. `is_special()` tells you whether WHATWG's
scheme-specific rules apply, which is mostly interesting because it predicts
what the setters will refuse. Note that the three are independent, and `file:`
is the awkward case that proves it: special, a perfectly good base, and still
no host — so test `has_host()` rather than inferring it from the scheme.

#### `host`, `host_str` and `domain`

Three accessors, and they are not interchangeable.

```
use url::{Host, Url};

fn main() {
    let named = Url::parse("https://www.example.test/a").unwrap();
    let v4 = Url::parse("https://93.184.216.34/a").unwrap();
    let v6 = Url::parse("https://[2606:2800:220:1::1]/a").unwrap();

    // host_str: the text as it appears in the URL — note the IPv6 brackets.
    assert_eq!(named.host_str(), Some("www.example.test"));
    assert_eq!(v4.host_str(), Some("93.184.216.34"));
    assert_eq!(v6.host_str(), Some("[2606:2800:220:1::1]")); // <- brackets KEPT

    // host: parsed, so you can tell a name from an address.
    assert!(matches!(named.host(), Some(Host::Domain("www.example.test"))));
    assert!(matches!(v4.host(), Some(Host::Ipv4(_))));
    assert!(matches!(v6.host(), Some(Host::Ipv6(_))));

    // domain: Some only for a registrable name, None for an address.
    assert_eq!(named.domain(), Some("www.example.test"));
    assert_eq!(v4.domain(), None);
    assert_eq!(v6.domain(), None);

    // Unicode arrives as punycode; there is no accessor that undoes it.
    let idn = Url::parse("https://münchen.de/x").unwrap();
    assert_eq!(idn.host_str(), Some("xn--mnchen-3ya.de"));
}
```

**When to use it:** `host()` whenever the decision depends on *what sort* of
host it is — any kind of private-address or same-origin check, where treating
`127.0.0.1` as a string is how bugs get in. `domain()` when you specifically
want a name and an IP literal should not qualify, such as matching a cookie
domain or a certificate. `host_str()` for display and logging. Two gotchas it
carries, both of which `host()` avoids: an IPv6 host comes back **with its
square brackets**, so `host_str()` is not something you can hand to a resolver
unmodified, and a Unicode name comes back as punycode — `xn--mnchen-3ya.de`,
which needs `idna::domain_to_unicode` to turn back into `münchen.de` for a
human to read.

#### `port` and `port_or_known_default`

The explicit port, or the one the scheme implies.

```
use url::Url;

fn main() {
    let explicit = Url::parse("https://example.test:8443/a").unwrap();
    assert_eq!(explicit.port(), Some(8443));
    assert_eq!(explicit.port_or_known_default(), Some(8443));

    // A default port is not stored, so `port()` is None.
    let implicit = Url::parse("https://example.test/a").unwrap();
    assert_eq!(implicit.port(), None);
    assert_eq!(implicit.port_or_known_default(), Some(443));

    // Writing the default explicitly is normalised away.
    assert_eq!(Url::parse("https://example.test:443/a").unwrap().port(), None);

    // Only special schemes have a known default.
    assert_eq!(Url::parse("redis://example.test/0").unwrap().port_or_known_default(), None);
}
```

**When to use it:** `port_or_known_default()` almost always — it is what you
need to open a connection or build a `host:port` pair, and `port()` returning
`None` for `https://example.test` is the classic cause of a client that
connects to port 0 or panics. Reach for plain `port()` only when you genuinely
care whether the author wrote a port down, such as reproducing their input or
deciding whether to show it.

#### `path` and `path_segments`

The path, percent-encoded, and split.

```
use url::Url;

fn main() {
    let url = Url::parse("https://example.test/docs/a%20b/c.txt").unwrap();

    // Still encoded: `path()` does NOT decode.
    assert_eq!(url.path(), "/docs/a%20b/c.txt");
    assert_eq!(
        url.path_segments().unwrap().collect::<Vec<_>>(),
        ["docs", "a%20b", "c.txt"],
    );

    // Decoding is a separate crate.
    let last = url.path_segments().unwrap().next_back().unwrap();
    assert_eq!(percent_encoding::percent_decode_str(last).decode_utf8_lossy(), "c.txt");

    // A trailing slash is an empty final segment, and "/" is one empty segment.
    assert_eq!(
        Url::parse("https://example.test/a/").unwrap().path_segments().unwrap().collect::<Vec<_>>(),
        ["a", ""],
    );
    assert_eq!(
        Url::parse("https://example.test/").unwrap().path_segments().unwrap().collect::<Vec<_>>(),
        [""],
    );

    // An encoded slash stays inside one segment — decode and it reappears.
    let sneaky = Url::parse("https://example.test/dir/a%2Fb").unwrap();
    assert_eq!(sneaky.path_segments().unwrap().collect::<Vec<_>>(), ["dir", "a%2Fb"]);
}
```

**When to use it:** `path()` for routing and logging, `path_segments()` whenever
you would otherwise call `split('/')` — it gets the leading slash and the
cannot-be-a-base case right, where your `split` will not. **The asymmetry to
burn in: `path()` is encoded, `query_pairs()` is decoded.** Mixing them up gives
you a filename literally called `a%20b`. And note the last assertion before you
decode a segment into a filesystem path: `%2F` decodes to `/`, so a segment that
looked like one component becomes two.

#### `query` and `query_pairs`

The raw query, and the decoded pairs.

```
use url::Url;

fn main() {
    let url = Url::parse("https://example.test/s?q=a+b&n=1%262&flag").unwrap();

    // Raw: exactly the bytes between `?` and `#`, undecoded.
    assert_eq!(url.query(), Some("q=a+b&n=1%262&flag"));

    // Decoded pairs, in order, with duplicates kept.
    assert_eq!(
        url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect::<Vec<_>>(),
        [
            ("q".to_string(), "a b".to_string()),
            ("n".to_string(), "1&2".to_string()),
            ("flag".to_string(), String::new()), // <- a bare key, not a missing one
        ],
    );

    // No query at all is distinct from an empty one.
    assert_eq!(Url::parse("https://example.test/s").unwrap().query(), None);
    assert_eq!(Url::parse("https://example.test/s?").unwrap().query(), Some(""));
}
```

**When to use it:** `query_pairs()` for anything key/value, which is almost
always — it handles `+`, `%`-escapes and repeated keys so you do not. Values
come back as `Cow<str>`, borrowed when no decoding was needed, so
`into_owned()` only allocates where it must. Use the raw `query()` when the
query is not form-encoded at all (some APIs put JSON or a signature there) or
when you must reproduce it verbatim. The `Some("")` versus `None` distinction
matters if you compare URLs as strings downstream.

#### `as_str`, `Position` and `origin`

The whole thing, a slice of it, and its security context.

```
use url::{Origin, Position, Url};

fn main() {
    let url = Url::parse("https://u:p@example.test:8443/a/b?q=1#f").unwrap();

    assert_eq!(url.as_str(), "https://u:p@example.test:8443/a/b?q=1#f");

    // Slicing by component boundary, with no allocation.
    assert_eq!(&url[Position::BeforeHost..Position::AfterPort], "example.test:8443");
    assert_eq!(&url[Position::BeforePath..Position::AfterPath], "/a/b");
    assert_eq!(&url[..Position::AfterPath], "https://u:p@example.test:8443/a/b");

    // Watch the boundary names: `BeforeQuery` sits AFTER the `?`.
    assert_eq!(&url[..Position::BeforeQuery], "https://u:p@example.test:8443/a/b?");

    // Origin: scheme, host and port, which is what "same site" really means.
    assert_eq!(url.origin().ascii_serialization(), "https://example.test:8443");
    let other = Url::parse("https://example.test:8443/elsewhere").unwrap();
    assert_eq!(url.origin(), other.origin());

    // Anything without a tuple origin is opaque, and never equal to anything.
    let data = Url::parse("data:,hello").unwrap();
    assert!(matches!(data.origin(), Origin::Opaque(_)));
    assert_eq!(data.origin().ascii_serialization(), "null");
    assert_ne!(data.origin(), Url::parse("data:,hello").unwrap().origin());
}
```

**When to use it:** `as_str()` to store or send it. `Position` slicing to strip
a component without rebuilding — `&url[..Position::AfterPath]` is the clean way
to drop the query and fragment, and is far better than `set_query(None)` on a
clone. Use `origin()` for same-origin decisions rather than comparing hosts,
because it folds in the scheme and the default port for you. Note what is
opaque: `file:`, `data:` and `blob:` URLs get a fresh `Origin::Opaque` on every
parse, so two identical `data:` URLs compare unequal — correct per spec, and a
trap if you were using origin as a cache key.

### Changing a URL

#### `join` and `make_relative`

Resolve a reference against this URL, and the reverse.

```
use url::Url;

fn main() {
    let base = Url::parse("https://example.test/a/b/c").unwrap();

    // The last segment of the base is replaced.
    assert_eq!(base.join("d").unwrap().as_str(), "https://example.test/a/b/d");
    // Unless the base ends in a slash.
    let dir = Url::parse("https://example.test/a/b/").unwrap();
    assert_eq!(dir.join("d").unwrap().as_str(), "https://example.test/a/b/d");
    assert_eq!(dir.join("").unwrap().as_str(), "https://example.test/a/b/");

    // make_relative goes the other way, and is None across origins.
    let target = Url::parse("https://example.test/a/x").unwrap();
    assert_eq!(base.make_relative(&target), Some("../x".to_string()));
    assert_eq!(base.make_relative(&base.join("?q=1").unwrap()), Some("?q=1".to_string()));
    assert_eq!(base.make_relative(&Url::parse("https://other.test/a").unwrap()), None);
}
```

**When to use it:** `join` for every relative reference you did not write
yourself — `href` values, `Location` headers, `$ref` pointers. Remember it is
*resolution*, not concatenation: to append a path segment to a URL you own, use
`path_segments_mut().push(..)` instead, which never discards anything.
`make_relative` is for emitting links — a static site generator writing
`../x` rather than an absolute URL — and returns `None` whenever a relative
form could not express the difference.

#### `set_path` and `path_segments_mut`

Replace the path, or edit it a segment at a time.

```
use url::Url;

fn main() {
    let mut url = Url::parse("https://example.test/old/path?q=1").unwrap();

    // set_path replaces wholesale, and does not treat ? or # as delimiters.
    url.set_path("/new/a b");
    assert_eq!(url.as_str(), "https://example.test/new/a%20b?q=1");
    url.set_path("no-leading-slash");
    assert_eq!(url.path(), "/no-leading-slash"); // <- added for you

    // path_segments_mut appends and escapes, which is what you want for user data.
    let mut api = Url::parse("https://example.test/v1").unwrap();
    api.path_segments_mut().unwrap().push("users").push("my name");
    assert_eq!(api.as_str(), "https://example.test/v1/users/my%20name");

    api.path_segments_mut().unwrap().pop();
    assert_eq!(api.path(), "/v1/users");
    api.path_segments_mut().unwrap().clear().push("v2");
    assert_eq!(api.path(), "/v2");

    // Err for a cannot-be-a-base URL; check before unwrapping.
    let mut mail = Url::parse("mailto:a@b.test").unwrap();
    assert!(mail.path_segments_mut().is_err());
}
```

**When to use it:** `path_segments_mut().push(..)` for building a path out of
values — an id, a name, a filename — because it percent-encodes each segment,
so a `/` in the data cannot break out into a new path component. That is a real
injection defence, and `format!("/users/{name}")` is not. `set_path` when you
have a whole path already and trust it; note it escapes but does not re-parse,
so a `?` in the string becomes `%3F` in the path rather than starting a query.
The `unwrap()` on `path_segments_mut` is only safe once you know the URL can be
a base.

#### `set_query` and `query_pairs_mut`

Replace the query, or build it from pairs.

```
use url::Url;

fn main() {
    let mut url = Url::parse("https://example.test/s?old=1").unwrap();

    // Raw: escapes illegal characters, keeps the structure you wrote.
    url.set_query(Some("a=1&b=two words"));
    assert_eq!(url.as_str(), "https://example.test/s?a=1&b=two%20words");

    // Removing it entirely.
    url.set_query(None);
    assert_eq!(url.as_str(), "https://example.test/s");

    // Pairs: appends to whatever is there, form-encoding each value.
    url.query_pairs_mut().append_pair("a", "1");
    url.query_pairs_mut().append_pair("b", "two words");
    assert_eq!(url.as_str(), "https://example.test/s?a=1&b=two+words");

    // Rebuilding from scratch: clear within the same borrow, then append.
    url.query_pairs_mut().clear().append_pair("only", "this");
    assert_eq!(url.as_str(), "https://example.test/s?only=this");

    // Both encodings decode identically, which is why round-tripping is safe.
    let a = Url::parse("https://example.test/s?x=a%20b").unwrap();
    let b = Url::parse("https://example.test/s?x=a+b").unwrap();
    assert_ne!(a, b);
    assert_eq!(a.query_pairs().next(), b.query_pairs().next());
}
```

**When to use it:** `query_pairs_mut` for key/value queries, always — it is the
only one of the two that escapes `&` and `=` inside a value, so it is the only
one safe to hand user data to. Note it *appends*: call `.clear()` first if you
mean to replace, and do it in the same expression so you are not juggling two
borrows. Use `set_query` for a query you are copying verbatim or one that is not
form-encoded. The last two assertions are worth remembering: `%20` and `+` are
different URLs that mean the same thing, so compare the decoded pairs, never
the strings.

#### `set_scheme`, `set_host` and `set_port`

The authority setters, and what they refuse.

```
use url::Url;

fn main() {
    let mut url = Url::parse("http://example.test/a").unwrap();

    // Special to special is fine, and the scheme is lowercased.
    assert!(url.set_scheme("HTTPS").is_ok());
    assert_eq!(url.as_str(), "https://example.test/a");

    // Crossing the special/non-special line is refused, both ways.
    assert!(url.set_scheme("mailto").is_err());
    assert!(url.set_scheme("my-app").is_err());
    assert!(Url::parse("mailto:a@b.test").unwrap().set_scheme("https").is_err());
    assert_eq!(url.scheme(), "https"); // unchanged by the failures

    // Setting the scheme's default port removes it instead.
    assert!(url.set_port(Some(8443)).is_ok());
    assert_eq!(url.as_str(), "https://example.test:8443/a");
    assert!(url.set_port(Some(443)).is_ok());
    assert_eq!(url.as_str(), "https://example.test/a");

    assert!(url.set_host(Some("other.test")).is_ok());
    assert_eq!(url.host_str(), Some("other.test"));

    // A special scheme must have a host, so removing it fails.
    assert!(url.set_host(None).is_err());

    // And neither setter applies to a cannot-be-a-base URL.
    assert!(Url::parse("mailto:a@b.test").unwrap().set_port(Some(25)).is_err());
}
```

**When to use it:** rewriting a URL you were given — upgrading `http` to
`https`, pointing a request at a test server, swapping a port. Every one of
these returns a `Result` and **the error type is `()`**, which tells you
nothing, so test the precondition yourself rather than reporting the failure
onward. The two refusals that catch people: `set_scheme` will not take you
across the special/non-special boundary, so `http` to `my-app` is an error even
though `my-app:` is a perfectly good scheme to parse; and a `None` host is only
legal for a non-special scheme. When the rewrite is this constrained, building a
fresh `Url` is often simpler than mutating one.

#### `set_fragment`, `set_username` and `set_password`

The remaining pieces.

```
use url::Url;

fn main() {
    let mut url = Url::parse("https://example.test/a").unwrap();

    url.set_fragment(Some("section 2"));
    assert_eq!(url.as_str(), "https://example.test/a#section%202");
    assert_eq!(url.fragment(), Some("section%202")); // <- NOT what you set

    url.set_fragment(None);
    assert_eq!(url.as_str(), "https://example.test/a");

    assert!(url.set_username("user").is_ok());
    assert!(url.set_password(Some("p@ss")).is_ok());
    assert_eq!(url.as_str(), "https://user:p%40ss@example.test/a");
    assert_eq!(url.username(), "user");
    assert_eq!(url.password(), Some("p%40ss")); // <- also not decoded

    // Credentials need an authority.
    assert!(Url::parse("mailto:a@b.test").unwrap().set_username("x").is_err());
}
```

**When to use it:** `set_fragment` for client-side anchors, and it is the one
setter that is genuinely convenient — nothing downstream sees the fragment, so
there is nothing to break. The credential setters exist for completeness and
for `ftp://`-era protocols; prefer an `Authorization` header, because a URL with
a password in it ends up in logs, referrers and shell history. The assertions
above pin down the rule that holds across the whole crate: **a setter encodes
and the matching getter does not decode**, so what you put in is not what you
get back. `query_pairs()` is the only accessor anywhere in `url` that decodes.

### Files and sockets

#### `from_file_path` and `to_file_path`

The bridge to the local filesystem.

```
use url::Url;

fn main() {
    // Platform-specific by design, so the test has to be too.
    #[cfg(windows)]
    {
        let url = Url::from_file_path(r"C:\data\my file.txt").unwrap();
        assert_eq!(url.as_str(), "file:///C:/data/my%20file.txt");
        assert_eq!(url.to_file_path().unwrap().to_str(), Some(r"C:\data\my file.txt"));

        // UNC paths become a host.
        let unc = Url::from_file_path(r"\\host\share\f.txt").unwrap();
        assert_eq!(unc.as_str(), "file://host/share/f.txt");
    }
    #[cfg(unix)]
    {
        let url = Url::from_file_path("/data/my file.txt").unwrap();
        assert_eq!(url.as_str(), "file:///data/my%20file.txt");
        assert_eq!(url.to_file_path().unwrap().to_str(), Some("/data/my file.txt"));
    }

    // A relative path has no URL, and the error carries no detail.
    assert!(Url::from_file_path("relative.txt").is_err());

    // A directory gets a trailing slash, which makes join() behave.
    let dir = Url::from_directory_path(std::env::temp_dir()).unwrap();
    assert!(dir.as_str().ends_with('/'));

    // And to_file_path refuses a non-file URL.
    assert!(Url::parse("https://example.test/a").unwrap().to_file_path().is_err());
}
```

**When to use it:** whenever a tool has to refer to local files and remote ones
through one type — a language server, a bundler, a documentation generator. Two
things to plan for. The path must be **absolute**, and `from_file_path` returns
`Err(())` with no explanation if it is not, so canonicalise first and report the
failure yourself. And these are genuinely platform-dependent: the same
`file://` URL means different paths on Windows and Unix, so a `file:` URL is
not portable between machines even though it looks like it should be. Use
`from_directory_path` when the path is a directory you will `join` onto —
without the trailing slash, `join` eats the last component.

#### `socket_addrs`

Resolving the host, with the scheme's port filled in.

```
use url::Url;

fn main() {
    // An IP literal needs no resolver.
    let url = Url::parse("http://127.0.0.1:8080/x").unwrap();
    assert_eq!(url.socket_addrs(|| None).unwrap(), ["127.0.0.1:8080".parse().unwrap()]);

    // The closure is a *fallback*, used only when the scheme has no known port.
    let implicit = Url::parse("http://127.0.0.1/x").unwrap();
    assert_eq!(implicit.socket_addrs(|| Some(9000)).unwrap(), ["127.0.0.1:80".parse().unwrap()]);

    // Here it is used, because "redis" has no default.
    let odd = Url::parse("redis://127.0.0.1/0").unwrap();
    assert_eq!(odd.socket_addrs(|| Some(6379)).unwrap(), ["127.0.0.1:6379".parse().unwrap()]);

    // No host, no addresses.
    assert!(Url::parse("mailto:a@b.test").unwrap().socket_addrs(|| None).is_err());
}
```

**When to use it:** when you are opening the connection yourself rather than
handing the URL to a client that resolves for you. It is the one method here
that does blocking I/O — it calls `ToSocketAddrs`, so a domain name means a DNS
lookup on the calling thread, which is wrong inside an async runtime. The
fallback-port semantics are the thing to get right: for `http` and `https` your
closure is never consulted, so returning `Some(8080)` from it will not redirect
anything. It is also the honest answer to the SSRF check two sections up — a
name that passes a host test can still resolve to `127.0.0.1`, and this is
where you would find that out.
