---
title: "digest"
version: "0.11.3"
publisher: "Artyom Pavlov (newpavlov), traits"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-06"
summary: "The traits every RustCrypto hash implements. Write code once against `Digest` and it works with SHA-2, BLAKE2, SHA-3 or MD5 — and swapping the algorithm becomes a type parameter."
categories: ["cryptography", "traits", "no-std"]
repository: "https://github.com/RustCrypto/traits"
---

## Overview

`digest` contains no hash function. It defines the traits that hash functions
implement, so that `sha2`, `sha3`, `blake2`, `md-5` and the rest all present the
same interface:

```
use digest::Digest;
use sha2::Sha256;

// One-shot, when you have all the data.
let hash = Sha256::digest(b"hello world");
assert_eq!(hash.len(), 32);

// Streaming, when you don't.
let mut hasher = Sha256::new();
hasher.update(b"hello ");
hasher.update(b"world");
assert_eq!(hasher.finalize(), hash); // <- same result either way
```

You will almost always add it alongside an implementation rather than on its
own. Its download rank comes from being the shared dependency of every
RustCrypto hash, not from direct use — the exception being code that is
*generic* over the algorithm, which is where the crate earns its place.

**What it buys you is substitutability.** A function written as
`fn checksum<D: Digest>(...)` works with any hash, so upgrading SHA-1 to SHA-256
is a change at the call site rather than a rewrite. That matters more than it
sounds: hash algorithms get broken, and the code that survives that is the code
where the algorithm is a parameter.

**What it does not do is make the choice safe.** `digest` is an interface, and
an interface has no opinion about whether MD5 is a reasonable thing to use in
2026. It isn't — MD5 and SHA-1 are both broken for collision resistance and
belong only in code reading legacy formats. For general hashing use SHA-256 or
BLAKE3; for passwords use none of these, because a fast hash is exactly the
wrong tool — `argon2` or `bcrypt` exist for that and are deliberately slow.

Two shapes are worth separating. **Hashing** answers "what is the fingerprint of
this data" and anyone can compute it. **A MAC** answers "did someone with the
key produce this", and lives behind the `Mac` trait in this same crate, usually
via `hmac`. Reaching for a plain hash where you needed a MAC is a real
vulnerability, not a style issue.

The crate is `no_std` with `alloc` optional, requires Rust 1.85, and its output
is a fixed-size array type rather than a `Vec` — the length is part of the type,
so a 32-byte hash cannot be mistaken for a 20-byte one.

## When to use it

### Use case: Code that works with any hash

The reason to name `digest` directly. A generic function lets the caller choose,
and lets you change the default later without touching the implementation.

```
use digest::Digest;
use sha2::{Sha256, Sha512};

/// Fingerprint a record with whichever hash the caller wants.
fn fingerprint<D: Digest>(fields: &[&str]) -> Vec<u8> {
    let mut hasher = D::new();
    for field in fields {
        hasher.update(field.as_bytes());
        hasher.update(b"\x1f"); // <- separator, so ["ab","c"] != ["a","bc"]
    }
    hasher.finalize().to_vec()
}

let record = ["user", "42"];
assert_eq!(fingerprint::<Sha256>(&record).len(), 32);
assert_eq!(fingerprint::<Sha512>(&record).len(), 64);
```

**Why it fits:** the algorithm is a type parameter, so migrating is a one-line
change at each call. Note the separator — concatenating fields without one lets
different records hash identically, which is a bug the type system cannot catch
for you.

### Use case: Hashing a file without reading it into memory

Streaming is the reason `update` and `finalize` are separate calls. A large file
is hashed a chunk at a time, with memory that doesn't depend on its size.

```
use digest::Digest;
use sha2::Sha256;
use std::io::Read;

fn hash_reader(mut source: impl Read) -> std::io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = source.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

let digest = hash_reader(&b"hello world"[..]).unwrap();
assert_eq!(digest, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
```

**Why it fits:** an 8 KiB buffer hashes an 8 GiB file. The one-shot `digest`
method would need the whole thing resident first, which is the difference
between working and running out of memory.

### Use case: Verifying a value in constant time

Comparing a computed hash against an expected one with `==` leaks timing
information. For anything an attacker can submit repeatedly, that matters.

```
use digest::Digest;
use sha2::Sha256;

fn matches(data: &[u8], expected: &[u8]) -> bool {
    let actual = Sha256::digest(data);
    // Not `actual[..] == expected`: that returns early on the first difference.
    actual.len() == expected.len()
        && actual.iter().zip(expected).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

let data = b"payload";
let good = Sha256::digest(data);
assert!(matches(data, &good));
assert!(!matches(data, &[0u8; 32]));
```

**Why it fits:** the fold examines every byte regardless, so the time taken says
nothing about how far the comparison got. In real code reach for `subtle`'s
`ConstantTimeEq` rather than hand-rolling this — and for MACs use
`Mac::verify`, which does it for you.

## API map

The crate is traits. Every entry below is a method a hash gets by implementing
`Digest`, so `use digest::Digest;` plus an implementation crate is the normal
setup. Examples use `sha2::Sha256` throughout; any other hash substitutes
directly.

### One-shot hashing

#### `Digest::digest`

Hashes a whole input in one call.

```
use digest::Digest;
use sha2::Sha256;

let hash = Sha256::digest(b"abc");
let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();

assert_eq!(hash.len(), 32);
assert_eq!(hex, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
```

**When to use it:** whenever the data is already in memory, which is most of the
time. It is a `Self`-less associated function, so it reads as
`Sha256::digest(..)` and needs no binding. Note that the output array does not
implement `LowerHex`, so hex means iterating the bytes as above — or the `hex`
crate, if you do it often.

#### `Digest::output_size`

The digest length in bytes, without hashing anything.

```
use digest::Digest;
use sha2::{Sha256, Sha512};

assert_eq!(Sha256::output_size(), 32);
assert_eq!(Sha512::output_size(), 64);
```

**When to use it:** sizing a buffer or validating a stored hash's length before
comparing. In generic code it is how you learn the size without a value in hand
— useful when parsing a format that stores digests back to back.

### Streaming

#### `Digest::new` and `update`

Creates a hasher and feeds it data incrementally.

```
use digest::Digest;
use sha2::Sha256;

let mut hasher = Sha256::new();
hasher.update(b"hello ");
hasher.update("world"); // <- &str works too, via AsRef<[u8]>

let streamed = hasher.finalize();
assert_eq!(streamed, Sha256::digest(b"hello world"));
```

**When to use it:** data arriving in pieces — a file, a socket, a structure you
walk. Repeated `update` calls are exactly equivalent to one call with the
concatenation, which is what makes chunk size a free choice.

#### `Digest::finalize`

Consumes the hasher and produces the digest.

```
use digest::Digest;
use sha2::Sha256;

let hasher = Sha256::new().chain_update(b"data");
let out = hasher.finalize();

assert_eq!(out.len(), 32);
// `hasher` is gone — finalize takes self, so it cannot be used twice.
```

**When to use it:** once, at the end. Taking `self` is deliberate: a hasher that
has produced its output is in no state to continue, and the type system enforces
that rather than leaving it to a runtime check.

#### `Digest::chain_update`

`update` in builder form, returning the hasher.

```
use digest::Digest;
use sha2::Sha256;

let out = Sha256::new()
    .chain_update(b"a")
    .chain_update(b"b")
    .chain_update(b"c")
    .finalize();

assert_eq!(out, Sha256::digest(b"abc"));
```

**When to use it:** hashing a fixed sequence of pieces in one expression, with
no `mut` binding. `new_with_prefix(data)` is the shorthand for a `new` followed
by a single `update`.

#### `Digest::finalize_reset`

Produces the digest and returns the hasher to its initial state.

```
use digest::Digest;
use sha2::Sha256;

let mut hasher = Sha256::new();

hasher.update(b"first");
let a = hasher.finalize_reset();

hasher.update(b"second");
let b = hasher.finalize_reset();

assert_eq!(a, Sha256::digest(b"first"));
assert_eq!(b, Sha256::digest(b"second")); // <- not "firstsecond"
```

**When to use it:** hashing many independent inputs in a loop without
reallocating a hasher each time. The assertion is the point — the reset is real,
so the second digest is unaffected by the first input.

#### `Digest::finalize_into`

Writes the digest into a buffer you own.

```
use digest::{Digest, Output};
use sha2::Sha256;

let mut out = Output::<Sha256>::default();
Sha256::new().chain_update(b"data").finalize_into(&mut out);

assert_eq!(out, Sha256::digest(b"data"));
```

**When to use it:** `no_std` and hot paths, where you want no allocation and a
reused buffer. `Output<D>` is the fixed-size array type for that hash, so the
buffer cannot be the wrong length — the compiler checks it rather than a runtime
assert.

### Generic code

#### The `Digest` bound

Writing a function over any hash.

```
use digest::Digest;
use sha2::{Sha256, Sha512};

fn hex_of<D: Digest>(data: &[u8]) -> String {
    D::digest(data).iter().map(|b| format!("{b:02x}")).collect()
}

assert_eq!(hex_of::<Sha256>(b"abc").len(), 64); // 32 bytes, 2 chars each
assert_eq!(hex_of::<Sha512>(b"abc").len(), 128);
```

**When to use it:** any helper that shouldn't hard-code an algorithm — a content
store, a cache key, a signature scheme. This is the single best reason to depend
on `digest` explicitly rather than only on `sha2`.

#### `Output<D>`

The digest type for a given hash: a fixed-size byte array.

```
use digest::{Digest, Output};
use sha2::Sha256;

fn store(hash: Output<Sha256>) -> usize {
    hash.len()
}

assert_eq!(store(Sha256::digest(b"x")), 32);
```

**When to use it:** naming a digest in a struct field or a signature. Because
the length is in the type, a `Sha256` digest and a `Sha512` digest are different
types — you cannot accidentally store one where the other is expected, which a
`Vec<u8>` would allow.

#### `DynDigest`

The object-safe form, for choosing the algorithm at runtime.

```
use digest::{Digest, DynDigest};
use sha2::{Sha256, Sha512};

fn hasher_for(name: &str) -> Option<Box<dyn DynDigest>> {
    match name {
        "sha256" => Some(Box::new(Sha256::new())),
        "sha512" => Some(Box::new(Sha512::new())),
        _ => None,
    }
}

let mut hasher = hasher_for("sha256").unwrap();
hasher.update(b"data");
assert_eq!(hasher.finalize().len(), 32);
assert!(hasher_for("md4").is_none());
```

**When to use it:** when the algorithm comes from configuration or a file
header, so it cannot be a type parameter. It costs dynamic dispatch and an
allocation, and gives up the compile-time length guarantee — so prefer the
generic form whenever the choice is known at compile time.

### Message authentication

#### `Mac`

The keyed counterpart to `Digest`, for proving a message came from someone with
the key.

```
use digest::{KeyInit, Mac}; // <- KeyInit is what provides new_from_slice
use hmac::Hmac;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

let mut mac = HmacSha256::new_from_slice(b"shared secret").unwrap();
mac.update(b"message");
let tag = mac.finalize().into_bytes();

// Verification is a separate, constant-time operation.
let mut check = HmacSha256::new_from_slice(b"shared secret").unwrap();
check.update(b"message");
assert!(check.verify_slice(&tag).is_ok());

let mut wrong = HmacSha256::new_from_slice(b"other secret").unwrap();
wrong.update(b"message");
assert!(wrong.verify_slice(&tag).is_err());
```

**When to use it:** webhooks, session tokens, API request signing — anywhere the
question is authenticity rather than identity. Use `verify_slice` rather than
comparing tags yourself; it is constant-time, and a plain `==` on a tag is a
timing oracle.
