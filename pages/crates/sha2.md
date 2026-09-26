---
title: "sha2"
version: "0.11.0"
publisher: "Tony Arcieri (tarcieri), Artyom Pavlov (newpavlov), hashes"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-19"
summary: "The SHA-2 family — SHA-256, SHA-512 and the truncated variants — as pure-Rust implementations of the `Digest` traits, with hardware acceleration where the CPU offers it."
domain: "Crypto, hashing & TLS"
categories: ["cryptography", "hashing", "no-std"]
repository: "https://github.com/RustCrypto/hashes"
---

## Overview

`sha2` implements the SHA-2 family: the hash functions behind TLS certificates,
Git's newer object format, package checksums, JWT signatures and most
content-addressed storage. It is the concrete counterpart to
[`digest`](digest.md), which defines the traits — so the calling code looks the
same as for any other hash, and the choice you are making here is the algorithm:

```
use sha2::{Digest, Sha256};

let hash = Sha256::digest(b"hello world");
let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();

assert_eq!(hex, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
```

**SHA-2 is not broken.** That distinguishes it from MD5 and SHA-1, which are
broken for collision resistance and belong only in code reading legacy formats.
SHA-256 is the sensible default for a general-purpose hash in 2026, and nothing
on the horizon changes that.

**Which variant, though, is a real question with three answers.**

- **SHA-256** — the default. Widest support, 32-byte output, what everyone means
  by "SHA-2" unless they say otherwise.
- **SHA-512** — *faster than SHA-256 on 64-bit CPUs without SHA extensions*,
  because it works on 64-bit words. Counter-intuitive, and worth measuring if
  you hash a lot.
- **SHA-512/256** — SHA-512 truncated to 32 bytes. Same output size as SHA-256,
  the speed of SHA-512, and immune to length extension. Underused.

**Length extension is the sharp edge.** Given `SHA-256(secret || message)` and
the length of the secret, an attacker can compute
`SHA-256(secret || message || padding || anything)` without knowing the secret.
So a hash is not a MAC: if you are authenticating a message, use HMAC (from
`hmac`, over this crate) rather than hashing a secret and a payload together.
SHA-384 and SHA-512/256 are not vulnerable, because truncation discards the
state an attacker would need — which is the main reason to prefer them when a
bare hash really must carry a secret.

**And not for passwords.** SHA-2 is designed to be fast, which is exactly wrong
for a password hash; `argon2` and `bcrypt` are deliberately slow and memory-hard.
Fast hashing of a password is how a leaked database becomes a list of plaintext
passwords.

The crate is pure Rust, `no_std` with `alloc` optional, requires Rust 1.85, and
uses SHA-NI or ARMv8 crypto instructions automatically when the target has them —
which is often several times faster than the software path, and is why
hand-rolling this is never worthwhile.

## When to use it

### Use case: Content addressing

Naming a blob by its hash, so identical content shares an identity and
corruption is detectable.

```
use sha2::{Digest, Sha256};

fn content_id(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

let a = content_id(b"the same bytes");
let b = content_id(b"the same bytes");
let c = content_id(b"different bytes");

assert_eq!(a, b);      // <- identical input, identical id
assert_ne!(a, c);
assert_eq!(a.len(), 64); // 32 bytes as hex
```

**Why it fits:** collision resistance is what makes the id trustworthy — you can
treat two equal hashes as equal content. This is what Git, container registries
and build caches all do, and SHA-256 is the usual choice because everything
already speaks it.

### Use case: Verifying a downloaded file

Checking that bytes arrived intact and unmodified, against a published checksum.

```
use sha2::{Digest, Sha256};
use std::io::Read;

fn verify(mut source: impl Read, expected_hex: &str) -> std::io::Result<bool> {
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = source.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let actual: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    Ok(actual.eq_ignore_ascii_case(expected_hex))
}

let data = &b"hello world"[..];
let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";

assert!(verify(data, expected).unwrap());
assert!(!verify(&b"tampered"[..], expected).unwrap());
```

**Why it fits:** streaming means an 8 KiB buffer verifies a file of any size.
Note what this does and does not prove — it detects corruption and casual
tampering, but an attacker who can replace the download can usually replace the
published checksum too. A signature is what binds the hash to an identity.

### Use case: Avoiding length extension

When a bare hash has to cover a secret, the variant choice stops being cosmetic.

```
use sha2::{Digest, Sha256, Sha512_256};

let secret = b"server-side key";
let message = b"amount=10";

// Both produce 32 bytes, so they are interchangeable in storage.
let naive = Sha256::digest([secret.as_slice(), message.as_slice()].concat());
let safer = Sha512_256::digest([secret.as_slice(), message.as_slice()].concat());

assert_eq!(naive.len(), 32);
assert_eq!(safer.len(), 32);
assert_ne!(naive.as_slice(), safer.as_slice());

// The real fix for authentication is a MAC, not a cleverer hash.
```

**Why it fits:** the two hashes are the same size and cost, and only one leaks
the internal state an extension attack needs. That said, the comment matters more
than the code — if the goal is authenticity, HMAC is the answer, and choosing
`Sha512_256` only limits the damage of doing it the wrong way.

## API map

Everything here comes from the [`digest`](digest.md) traits, which `sha2`
re-exports — so `use sha2::{Digest, Sha256};` is the whole import. The entries
below are the variants and the handful of methods worth seeing in context.

### The variants

#### `Sha256`

The default: 256-bit output, and what most protocols specify.

```
use sha2::{Digest, Sha256};

let hash = Sha256::digest(b"abc");

assert_eq!(hash.len(), 32);
assert_eq!(
    hash.iter().map(|b| format!("{b:02x}")).collect::<String>(),
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
);
```

**When to use it:** unless something tells you otherwise. It is the
interoperable choice — certificates, checksums, container digests and Git's
SHA-256 mode all use it, so a different variant needs a reason.

#### `Sha512`

512-bit output, and faster than SHA-256 on 64-bit CPUs lacking SHA extensions.

```
use sha2::{Digest, Sha512};

let hash = Sha512::digest(b"abc");

assert_eq!(hash.len(), 64);
assert!(hash.iter().map(|b| format!("{b:02x}")).collect::<String>().starts_with("ddaf35a1"));
```

**When to use it:** when you control both ends and are hashing enough data for
throughput to matter, or when a protocol asks for it. The 64-byte output is
twice the storage per record, which is the trade against the speed.

#### `Sha384` and `Sha512_256`

Truncated variants of SHA-512, which is what makes them length-extension
resistant.

```
use sha2::{Digest, Sha384, Sha512_256};

assert_eq!(Sha384::digest(b"abc").len(), 48);
assert_eq!(Sha512_256::digest(b"abc").len(), 32);

// Same length as SHA-256, but a different function entirely.
use sha2::Sha256;
assert_ne!(
    Sha512_256::digest(b"abc").as_slice(),
    Sha256::digest(b"abc").as_slice(),
);
```

**When to use it:** `Sha384` where a protocol specifies it — TLS cipher suites
commonly do. `Sha512_256` when you want SHA-256's output size without its length
extension weakness and interoperability is not a constraint. The last assertion
is the trap to avoid: same size, different values, so they are not
interchangeable across systems.

#### `Sha224`

224-bit output, for legacy interoperability.

```
use sha2::{Digest, Sha224};

assert_eq!(Sha224::digest(b"abc").len(), 28);
```

**When to use it:** essentially only when something else already uses it. It
offers no advantage over SHA-256 — same internal function, same speed, less
output — so it exists to match specifications rather than to be chosen.

### Hashing

#### One-shot and streaming

The two shapes, identical in result.

```
use sha2::{Digest, Sha256};

// Everything at once.
let all = Sha256::digest(b"hello world");

// Or a piece at a time.
let mut hasher = Sha256::new();
hasher.update(b"hello ");
hasher.update(b"world");
let streamed = hasher.finalize();

assert_eq!(all, streamed);
```

**When to use it:** `digest` when the data is in memory, `new`/`update`/
`finalize` when it arrives in pieces or is too large to hold. Repeated `update`
calls are exactly equivalent to one call on the concatenation, so chunk size is
a free choice.

#### Hashing structured data

Feeding several fields in needs a separator, or different inputs collide.

```
use sha2::{Digest, Sha256};

fn fingerprint(fields: &[&str]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes()); // <- length prefix
        hasher.update(field.as_bytes());
    }
    hasher.finalize().to_vec()
}

// Without the prefix, ["ab","c"] and ["a","bc"] would hash identically.
assert_ne!(fingerprint(&["ab", "c"]), fingerprint(&["a", "bc"]));
assert_eq!(fingerprint(&["ab", "c"]), fingerprint(&["ab", "c"]));
```

**When to use it:** any time the input is more than one value. Length-prefixing
is the robust habit — a separator byte works until a field contains that byte,
whereas a length cannot be forged by the content. The type system will not catch
this, so it has to be a habit.

#### `finalize_reset`

Reuses one hasher across many independent inputs.

```
use sha2::{Digest, Sha256};

let mut hasher = Sha256::new();
let mut digests = Vec::new();

for item in [b"one".as_slice(), b"two".as_slice()] {
    hasher.update(item);
    digests.push(hasher.finalize_reset());
}

assert_eq!(digests[0], Sha256::digest(b"one"));
assert_eq!(digests[1], Sha256::digest(b"two")); // <- not "onetwo"
```

**When to use it:** hashing many values in a loop without constructing a hasher
each time. The second assertion is the point — the reset is real, so the
previous input does not bleed into the next.

### Generic over the algorithm

#### Taking `D: Digest`

Since the variants share the traits, a function can accept any of them.

```
use sha2::{Digest, Sha256, Sha512, Sha512_256};

fn hex_of<D: Digest>(data: &[u8]) -> String {
    D::digest(data).iter().map(|b| format!("{b:02x}")).collect()
}

assert_eq!(hex_of::<Sha256>(b"abc").len(), 64);
assert_eq!(hex_of::<Sha512>(b"abc").len(), 128);
assert_eq!(hex_of::<Sha512_256>(b"abc").len(), 64);
```

**When to use it:** when the algorithm should be a decision at the call site or
in configuration rather than baked into the helper. This is the payoff for
`digest` existing: migrating from SHA-256 to something else later is a change at
the edges, not a rewrite.

#### With `hmac`

The keyed construction, which is what a hash alone cannot do.

```
use hmac::Hmac;
use sha2::{Digest, Sha256};
use sha2::digest::{KeyInit, Mac};

type HmacSha256 = Hmac<Sha256>;

let mut mac = HmacSha256::new_from_slice(b"shared key").unwrap();
mac.update(b"amount=10");
let tag = mac.finalize().into_bytes();

let mut check = HmacSha256::new_from_slice(b"shared key").unwrap();
check.update(b"amount=10");
assert!(check.verify_slice(&tag).is_ok());

// The wrong key fails, in constant time.
let mut wrong = HmacSha256::new_from_slice(b"other key").unwrap();
wrong.update(b"amount=10");
assert!(wrong.verify_slice(&tag).is_err());

let _ = Sha256::digest(b"unrelated");
```

**When to use it:** webhooks, signed cookies, API request signing — anywhere the
question is "did the key holder produce this". `Hmac<Sha256>` is the standard
construction and it sidesteps length extension entirely, which is why it is the
right answer rather than a cleverer way of concatenating a secret.
