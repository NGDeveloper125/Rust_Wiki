---
title: "base64"
version: "0.23.1"
publisher: "Alice Maz (alicemaz), Marshall Pierce (marshallpierce)"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-29"
summary: "Base64 encoding and decoding, with the alphabet and padding rules made explicit. Pick an engine — standard, URL-safe, padded or not — and call `encode` and `decode` on it."
categories: ["encoding", "no-std"]
repository: "https://github.com/marshallpierce/rust-base64"
---

## Overview

Base64 turns arbitrary bytes into 64 printable characters, so binary survives a
channel that only carries text: a JSON field, a `data:` URL, an HTTP header, an
email body.

The awkward part is that "base64" is not one format. Two choices vary, and both
have to match at the other end:

- **The alphabet.** Standard base64 finishes with `+` and `/`. Neither is safe
  in a URL or filename, so the URL-safe variant uses `-` and `_` instead.
- **Padding.** Standard base64 pads with `=` to a multiple of four characters.
  JWTs, and much else, omit it.

That gives four combinations in common use, and getting the wrong one produces
either a decode error or — worse — output the other side quietly rejects. This
crate's design is a response to that: you choose an **engine**, and the engine
carries the alphabet and padding rules.

```
use base64::prelude::*;

// Four bytes chosen so the two alphabets visibly disagree.
let data = [0xfb, 0xff, 0xbf, 0x01];

assert_eq!(BASE64_STANDARD.encode(data), "+/+/AQ==");
assert_eq!(BASE64_URL_SAFE.encode(data), "-_-_AQ==");
assert_eq!(BASE64_URL_SAFE_NO_PAD.encode(data), "-_-_AQ"); // <- no padding
```

**If you are coming from an older version, the API changed in 0.21.** The free
functions `base64::encode` and `base64::decode` still exist but are deprecated:
they hard-coded the standard alphabet, which was exactly the ambiguity above.
Everything now goes through the `Engine` trait, and `base64::prelude::*` brings
in the trait plus the four common engines under names like `BASE64_STANDARD`.
Most examples you find online predate this and won't compile.

Two things worth stating plainly:

**Base64 is not encryption, compression or a checksum.** It is a reversible
encoding anyone can undo, and it makes data about a third larger. Encoding a
secret protects nothing.

**Decoding untrusted input can fail, and should be handled.** `decode` returns
`Result`, and the `DecodeError` variants distinguish an invalid character from a
bad length from a wrongly-padded final symbol — which is usually enough to tell
you the sender used a different variant rather than that the data is corrupt.

The crate has no dependencies, requires Rust 1.71, and builds without `std`:
`alloc` covers the `String`/`Vec` returning methods, and with neither feature
you can still encode and decode into slices you provide. It is the de facto
standard for this in Rust, and the alternatives are worth considering only for
`data-encoding`, which covers base32 and hex through one configurable API.

## When to use it

### Use case: A token in a URL

URLs are where the alphabet choice bites. `+` and `/` need percent-encoding, and
`=` padding is often stripped by the systems that carry tokens, so URL-safe and
unpadded is the combination that survives.

```
use base64::prelude::*;

let token_bytes = [0xfb, 0xff, 0xbf, 0x01];

let url_safe = BASE64_URL_SAFE_NO_PAD.encode(token_bytes);
assert_eq!(url_safe, "-_-_AQ");

// The standard alphabet would have produced characters a URL cannot carry.
let standard = BASE64_STANDARD.encode(token_bytes);
assert_eq!(standard, "+/+/AQ==");
assert!(standard.contains('+') && standard.contains('/'));

// Round-trips with the same engine.
let back = BASE64_URL_SAFE_NO_PAD.decode(&url_safe).unwrap();
assert_eq!(back, token_bytes);
```

**Why it fits:** the difference is visible in the output — the same four bytes
become `-_-_AQ` or `+/+/AQ==`. Choosing the engine at the point of use is what
stops that being a runtime surprise in someone else's system.

### Use case: Embedding a small file in text

A logo in a stylesheet, an attachment in JSON, a fixture in a test: base64 is
how bytes travel inside a text format.

```
use base64::prelude::*;

let png_bytes = [0x89, b'P', b'N', b'G', 0x0d, 0x0a];

let data_url = format!("data:image/png;base64,{}", BASE64_STANDARD.encode(png_bytes));
assert!(data_url.starts_with("data:image/png;base64,iVBORw0K"));

// And back out again.
let encoded = data_url.split_once(',').unwrap().1;
let decoded = BASE64_STANDARD.decode(encoded).unwrap();
assert_eq!(decoded, png_bytes);
```

**Why it fits:** `data:` URLs specify standard base64 with padding, so
`BASE64_STANDARD` is the correct engine and the code says which one it is. Bear
the size in mind: the encoded form is about 33% larger, which is why this suits
icons rather than photographs.

### Use case: Decoding input you did not create

Anything arriving from a client may be truncated, mistyped, or encoded with a
different variant. The error tells you which.

```
use base64::prelude::*;
use base64::DecodeError;

fn decode_token(input: &str) -> Result<Vec<u8>, String> {
    BASE64_URL_SAFE_NO_PAD.decode(input).map_err(|e| match e {
        DecodeError::InvalidByte(pos, byte) => {
            format!("invalid character {:?} at {pos}", byte as char)
        }
        DecodeError::InvalidLength(len) => format!("truncated: length {len}"),
        other => format!("malformed: {other}"),
    })
}

assert!(decode_token("-_-_AQ").is_ok());

// A standard-alphabet token fed to a URL-safe engine fails at the offending byte.
let err = decode_token("+/+/AQ").unwrap_err();
assert_eq!(err, "invalid character '+' at 0");
```

**Why it fits:** `InvalidByte` naming `+` at position 0 says the sender used the
standard alphabet, which is a configuration mismatch rather than corruption. A
bare "decode failed" would send you looking in the wrong place.

## API map

Everything runs through the `Engine` trait, so `use base64::prelude::*;` — which
imports the trait and the four common engines — is the first line of most code
using this crate.

### Choosing an engine

#### `BASE64_STANDARD`

The standard alphabet (`+`, `/`) with `=` padding: RFC 4648 §4, and what most
things mean by "base64".

```
use base64::prelude::*;

let encoded = BASE64_STANDARD.encode(b"any bytes");
assert_eq!(encoded, "YW55IGJ5dGVz");

// Padding appears when the input length is not a multiple of three.
assert_eq!(BASE64_STANDARD.encode(b"a"), "YQ==");
assert_eq!(BASE64_STANDARD.encode(b"abc"), "YWJj");
```

**When to use it:** MIME, `data:` URLs, HTTP Basic credentials, and anywhere a
specification just says "base64". It is the right default when you control both
ends and nothing forbids `+` or `/`.

#### `BASE64_URL_SAFE_NO_PAD`

The URL-safe alphabet (`-`, `_`) with no padding: RFC 4648 §5, and what JWTs
use.

```
use base64::prelude::*;

let encoded = BASE64_URL_SAFE_NO_PAD.encode(b"a");
assert_eq!(encoded, "YQ"); // <- no trailing ==

// Every character is safe in a URL path or query.
let encoded = BASE64_URL_SAFE_NO_PAD.encode([0xff, 0xff, 0xff]);
assert!(!encoded.contains('+') && !encoded.contains('/'));
```

**When to use it:** tokens, identifiers and anything appearing in a URL,
filename or header. `BASE64_URL_SAFE` is the padded form, for the rarer specs
that want `=` retained; the unpadded one is much more common in practice.

#### `GeneralPurpose::new`

Builds an engine from an alphabet and a config, for the combinations the presets
don't cover.

```
use base64::alphabet;
use base64::engine::general_purpose::{GeneralPurpose, NO_PAD};
use base64::Engine;

// The bcrypt alphabet, which is neither of the RFC 4648 ones.
let engine = GeneralPurpose::new(&alphabet::BCRYPT, NO_PAD);

let encoded = engine.encode(b"abc");
assert_eq!(engine.decode(&encoded).unwrap(), b"abc");
```

**When to use it:** interoperating with a system that chose its own alphabet —
bcrypt, crypt, IMAP's modified UTF-7, all of which ship as constants in
`alphabet`. Also the way to build a decoder that tolerates missing padding, via
`PAD_INDIFFERENT`.

#### `Alphabet::new`

Defines a 64-character alphabet from scratch, validating it.

```
use base64::alphabet::Alphabet;

let ok = Alphabet::new("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
assert!(ok.is_ok());

// Duplicates and wrong lengths are rejected rather than silently accepted.
assert!(Alphabet::new("AA").is_err());
```

**When to use it:** a bespoke or legacy encoding you have to match exactly. It
checks length, duplicates and that no character is `=`, so a typo is a build- or
startup-time error instead of data nobody can decode later.

### Encoding

#### `Engine::encode`

Bytes to a `String`.

```
use base64::prelude::*;

assert_eq!(BASE64_STANDARD.encode(b"hello"), "aGVsbG8=");
assert_eq!(BASE64_STANDARD.encode("hello"), "aGVsbG8="); // <- &str works too
assert_eq!(BASE64_STANDARD.encode([0u8; 3]), "AAAA");
```

**When to use it:** the ordinary case. It takes anything `AsRef<[u8]>`, so
strings, arrays, `Vec`s and slices all work without conversion, and it allocates
one `String` of exactly the right size.

#### `Engine::encode_string`

Appends to an existing `String` instead of allocating a new one.

```
use base64::prelude::*;

let mut out = String::from("data:image/png;base64,");
BASE64_STANDARD.encode_string(b"\x89PNG", &mut out);

assert_eq!(out, "data:image/png;base64,iVBORw==");
```

**When to use it:** building a larger string around the encoded value, or
encoding repeatedly into a reused buffer. It appends rather than replacing, which
is what makes the prefix case a single allocation.

#### `Engine::encode_slice`

Writes into a caller-provided `&mut [u8]`, allocating nothing.

```
use base64::prelude::*;
use base64::encoded_len;

let input = b"hello";
let needed = encoded_len(input.len(), true).unwrap();

let mut buf = vec![0u8; needed];
let written = BASE64_STANDARD.encode_slice(input, &mut buf).unwrap();

assert_eq!(&buf[..written], b"aGVsbG8=");
```

**When to use it:** hot paths and `no_std` without `alloc`. Size the buffer with
`encoded_len` — passing the `true` for padding that matches your engine — since
a buffer that is too small is an error rather than a truncation.

### Decoding

#### `Engine::decode`

Base64 text back to bytes, or a `DecodeError`.

```
use base64::prelude::*;

assert_eq!(BASE64_STANDARD.decode("aGVsbG8=").unwrap(), b"hello");

// Wrong alphabet, missing padding and stray characters are all errors.
assert!(BASE64_STANDARD.decode("aGVsbG8").is_err());
assert!(BASE64_STANDARD.decode("aGVsb!8=").is_err());
```

**When to use it:** whenever input comes from outside your process. Note the
second assertion: `BASE64_STANDARD` requires the padding, so text produced by a
`NO_PAD` engine will not decode with it. Use the matching engine, or a
`PAD_INDIFFERENT` config when you must accept both.

#### `DecodeError`

Which way the input was wrong.

```
use base64::prelude::*;
use base64::DecodeError;

// A character outside the engine's alphabet, with its position.
match BASE64_STANDARD.decode("ab*d") {
    Err(DecodeError::InvalidByte(pos, byte)) => {
        assert_eq!((pos, byte), (2, b'*'));
    }
    other => panic!("expected InvalidByte, got {other:?}"),
}

// A length that cannot be valid base64.
assert!(matches!(
    BASE64_STANDARD.decode("aGVsbG8"),
    Err(DecodeError::InvalidPadding | DecodeError::InvalidLength(_)),
));
```

**When to use it:** turning a failure into a message that helps. The position in
`InvalidByte` is the actionable part — pointing at `+` or `/` means the sender
used the standard alphabet, and pointing at `-` or `_` means they used the
URL-safe one.

#### `Engine::decode_vec`

Appends decoded bytes to an existing `Vec`.

```
use base64::prelude::*;

let mut buf = vec![0xff];
BASE64_STANDARD.decode_vec("aGk=", &mut buf).unwrap();

assert_eq!(buf, [0xff, b'h', b'i']);
```

**When to use it:** accumulating several decoded chunks, or reusing one buffer
across a loop. Like `encode_string` it appends, so clear the buffer first if you
mean to replace its contents.

#### `Engine::decode_slice`

Decodes into a caller-provided slice.

```
use base64::prelude::*;
use base64::decoded_len_estimate;

let input = "aGVsbG8=";
let mut buf = vec![0u8; decoded_len_estimate(input.len())];
let written = BASE64_STANDARD.decode_slice(input, &mut buf).unwrap();

assert_eq!(&buf[..written], b"hello");
```

**When to use it:** allocation-free decoding. `decoded_len_estimate` gives an
upper bound rather than the exact length — padding means the true size can be up
to two bytes smaller — so always slice by the returned count rather than
assuming the buffer is full.

### Streaming

#### `write::EncoderWriter`

A `Write` that base64-encodes everything written through it.

```
use base64::prelude::*;
use base64::write::EncoderWriter;
use std::io::Write;

let mut encoded = Vec::new();
{
    let mut writer = EncoderWriter::new(&mut encoded, &BASE64_STANDARD);
    writer.write_all(b"hello").unwrap();
    writer.finish().unwrap(); // <- flushes the final partial group
}

assert_eq!(String::from_utf8(encoded).unwrap(), "aGVsbG8=");
```

**When to use it:** encoding something too large to hold in memory, or piping
one stream into another. `finish` matters — the last group and its padding are
only written then, and dropping without it silently truncates the output.

#### `read::DecoderReader`

A `Read` that decodes base64 as it is read.

```
use base64::prelude::*;
use base64::read::DecoderReader;
use std::io::Read;

let mut input = "aGVsbG8=".as_bytes();
let mut decoder = DecoderReader::new(&mut input, &BASE64_STANDARD);

let mut out = Vec::new();
decoder.read_to_end(&mut out).unwrap();

assert_eq!(out, b"hello");
```

**When to use it:** decoding a large encoded body — an attachment, an upload —
without materialising the base64 text and the bytes at once. It composes with
anything taking a `Read`, so a decompressor or parser can sit directly on top.

#### `display::Base64Display`

Formats bytes as base64 without allocating a `String`.

```
use base64::prelude::*;
use base64::display::Base64Display;

let bytes = b"hello";
let rendered = format!("value={}", Base64Display::new(bytes, &BASE64_STANDARD));

assert_eq!(rendered, "value=aGVsbG8=");
```

**When to use it:** interpolating encoded bytes into a larger string or writing
them to a formatter — logs, `Display` impls, templates. It writes straight into
the formatter, so it avoids the intermediate `String` that `encode` would build
and immediately drop.
