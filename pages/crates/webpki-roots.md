---
title: "webpki-roots"
version: "1.0.11"
publisher: "Joe Birr-Pixton (ctz), Dirkjan Ochtman (djc), publishers"
no_std: "yes"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-11-08"
summary: "Mozilla's set of trusted certificate authorities, compiled into your binary as a single `const`. It is what a `rustls` client validates against when you do not want to read the host's certificate store — and the version you pin is the trust policy you ship."
domain: "Crypto, hashing & TLS"
categories: ["cryptography", "network-programming", "no-std"]
repository: "https://github.com/rustls/webpki-roots"
docs: "https://docs.rs/webpki-roots/1.0.11/webpki_roots/"
---

## Overview

A TLS client cannot verify a server without knowing which certificate
authorities it is willing to trust. The operating system has a list;
`webpki-roots` is the other option — the list Mozilla's root programme
maintains, generated from the
[Common CA Database](https://www.ccadb.org/) and compiled straight into your
binary.

The entire public API is one item:

```
use webpki_roots::TLS_SERVER_ROOTS;

// A `const` slice of trust anchors. 118 of them in 1.0.11.
assert_eq!(TLS_SERVER_ROOTS.len(), 118);
assert!(TLS_SERVER_ROOTS.iter().all(|a| !a.subject_public_key_info.as_ref().is_empty()));
```

There is nothing else — no functions, no types, no features. It is a data crate,
and almost everything worth saying about it is about the data rather than the
code. It depends only on `rustls-pki-types`, is unconditionally `#![no_std]`,
and costs about **55 KB** of binary — 55,808 bytes between two release builds
that differ only in whether they read the anchors, of which 53,639 bytes are the
DER itself.

**The decision it represents.** A compiled-in root store behaves identically on
every machine you deploy to, which is exactly what you want in a `scratch`
container with no `/etc/ssl` and exactly what you want in CI. The cost is that
it is frozen at build time: it carries no revocation data, honours no
administrator policy, and cannot see a CA your organisation installed on the
host. The crate's own documentation is unusually direct about the boundary — it
is *"suitable for use in applications that can always be recompiled and
instantly deployed"*, and for anything shipped to end users it points at
[`rustls-platform-verifier`](https://docs.rs/rustls-platform-verifier) instead.
Take that seriously; the alternatives are compared in the API map below.

**It is actively maintained, and that is the point.** Eight releases came out in
the twelve months to October 2026, because the root programme keeps changing.
This is not a crate to pin and forget, which is covered in its own use case.

One unusual thing to note before you vendor it: the licence is
**CDLA-Permissive-2.0**, not the `MIT OR Apache-2.0` you see on everything else
in this ecosystem, because the data is a derived work of CCADB's and carries the
[CCADB Data Usage Terms](https://www.ccadb.org/rootstores/usage#ccadb-data-usage-terms).
It is permissive and imposes no conditions on your own code, but a tool that
scans dependency licences against an allow-list will stop on it.

MSRV is 1.71, matching [`rustls`](rustls.md).

## When to use it

### Use case: A client that must behave the same everywhere

The common case, and the reason most people reach for it: a service in a
minimal container, where there is no system certificate store to read.

```
use rustls::{ClientConfig, RootCertStore};

// Collecting the slice is the tidiest of the three ways in.
let roots: RootCertStore = webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect();
assert_eq!(roots.len(), 118);

let config = ClientConfig::builder()
    .with_root_certificates(roots)
    .with_no_client_auth();

// Policy only -- nothing has been negotiated and no I/O has happened.
assert!(config.alpn_protocols.is_empty());
```

**Why it fits:** the trust decision travels with the binary, so a failure in
staging reproduces in production and a `FROM scratch` image needs no extra
layer. The rest of the client — connections, ALPN, the crypto provider — belongs
to [`rustls`](rustls.md) and is documented there; this crate's whole
contribution is the one line that fills the store.

### Use case: Deciding whether to pin the version

Ordinarily you pin a dependency to keep a build reproducible. Here the
dependency *is* a policy document, and pinning it freezes that policy. The
numbers are worth seeing, because they are larger than people expect.

```
// Between 1.0.4 (November 2025) and 1.0.11 (October 2026):
//
//   1.0.4   142 anchors
//   1.0.9   121
//   1.0.10  120
//   1.0.11  118
//
// 29 anchors removed, 5 added. Among the removals: DigiCert Global Root CA,
// all three Entrust Root Certification Authority anchors, Google's GTS Root
// R2, all four AffirmTrust roots, and all three Trustwave roots.
use webpki_roots::TLS_SERVER_ROOTS;

// So this assertion is a statement about one release, not about the web.
assert_eq!(TLS_SERVER_ROOTS.len(), 118);

// What a stale pin means in practice: a build still on 1.0.4 keeps trusting
// 24 more anchors than the current root programme does, and no warning says so.
assert!(TLS_SERVER_ROOTS.len() < 142);
```

**Why it fits:** it reframes the update as a security task rather than
housekeeping. The direction of travel matters — removals outnumber additions
roughly six to one, and a root is removed when the programme has stopped
trusting the CA, so an old pin fails *open*: it accepts certificates a current
browser would reject. The practical answer is to let the patch version float and
treat the update like any other dependency bump, or, if your policy forbids
floating versions, to put the bump on a calendar. Note also that nothing about
the crate can detect a root revoked *between* releases; if that gap matters,
you want the platform verifier.

### Use case: Answering "which CAs do we actually trust?"

A question that arrives from an auditor, a security review, or a customer
questionnaire. The data is right there — but not in a form you can print.

```
use webpki_roots::TLS_SERVER_ROOTS;
use x509_parser::prelude::*;

// The stored `subject` is the *contents* of the X.501 Name SEQUENCE with its
// header stripped, so a DER parser rejects it as-is. Put the header back.
fn as_name_der(contents: &[u8]) -> Vec<u8> {
    let mut out = vec![0x30]; // SEQUENCE
    let n = contents.len();
    if n < 0x80 {
        out.push(n as u8);
    } else if n < 0x100 {
        out.extend_from_slice(&[0x81, n as u8]);
    } else {
        out.extend_from_slice(&[0x82, (n >> 8) as u8, (n & 0xff) as u8]);
    }
    out.extend_from_slice(contents);
    out
}

let names: Vec<String> = TLS_SERVER_ROOTS
    .iter()
    .map(|a| {
        let der = as_name_der(a.subject.as_ref());
        let (_, name) = X509Name::from_der(&der).unwrap();
        name.to_string()
    })
    .collect();

assert_eq!(names.len(), 118); // <- every one decodes
assert_eq!(names[0], "C=US, O=DigiCert, Inc., CN=DigiCert TLS ECC P384 Root G5");
assert!(names.iter().any(|n| n.contains("Telia RSA TLS Root CA v3")));
```

**Why it fits:** it is the only way to get a human-readable answer, and the
re-wrapping step is the part nobody guesses. Handing the raw `subject` to
`X509Name::from_der` fails with `UnexpectedTag { expected: Some(Tag(16)),
actual: Tag(17) }` — tag 16 is SEQUENCE, tag 17 is the SET that begins the first
relative distinguished name — which is a confusing error to meet if you assumed
the field held a complete name. Caching the result is sensible; parsing 118
names on every request is pointless work.

## API map

One `const` and the type of its elements. The entries below therefore cover the
shape of the data and the three decisions around it: how to hand it to
`rustls`, when to use something else, and what it costs on a target without
`std`.

### The root set

#### `TLS_SERVER_ROOTS`

The whole crate.

```
use webpki_roots::TLS_SERVER_ROOTS;

// It is a `const`, so it is usable where a constant is required.
const ROOT_COUNT: usize = TLS_SERVER_ROOTS.len();
assert_eq!(ROOT_COUNT, 118);

// And being a `const` rather than a `static`, each use is a fresh copy of the
// slice reference -- `to_vec()` allocates every time it is called.
let a = TLS_SERVER_ROOTS.to_vec();
let b = TLS_SERVER_ROOTS.to_vec();
assert_eq!(a.len(), b.len());
```

**When to use it:** whenever a client needs the public web's trust store and you
have decided a compiled-in copy is right. Build the `RootCertStore` once at
startup and share it behind an `Arc` — `to_vec()` copies 118 anchors, and doing
that per connection is a measurable waste in a hot client.

#### `TrustAnchor`, and which crate owns it

The element type is not this crate's.

```
// `webpki_roots::TrustAnchor` does not exist -- the import inside the crate is
// private, and naming it that way fails with E0603, "struct `TrustAnchor` is
// private". It comes from `rustls-pki-types`, which you either depend on
// directly or reach through rustls:
use rustls::pki_types::TrustAnchor;

let first: &TrustAnchor<'static> = &webpki_roots::TLS_SERVER_ROOTS[0];
assert!(!first.subject.as_ref().is_empty());
```

**When to use it:** whenever you write the type down — in a function signature,
a struct field, or a turbofish. Most code never needs to, because
`TLS_SERVER_ROOTS.iter().cloned().collect()` infers everything. Sharing the type
with `rustls` is deliberate and is why no conversion is needed at the boundary.

#### `subject`

The CA's distinguished name, DER-encoded.

```
use webpki_roots::TLS_SERVER_ROOTS;

let a = &TLS_SERVER_ROOTS[0];
let bytes = a.subject.as_ref();

// Raw DER, not text. The first byte is 0x31 -- a SET, the start of the first
// relative distinguished name -- because the enclosing SEQUENCE header is
// not stored.
assert_eq!(bytes[0], 0x31);
assert!(!bytes.is_empty());

// The CN is in there, which is why a lossy conversion looks almost readable
// and is never the right way to read it.
assert!(String::from_utf8_lossy(bytes).contains("DigiCert TLS ECC P384 Root G5"));
```

**When to use it:** for matching and display, never for trust decisions — the
chain builder matches on it for you. To render it, re-wrap it in a SEQUENCE
header and hand it to an X.509 parser, as the third use case shows. Reaching for
`from_utf8_lossy` because the name happens to be visible inside the DER will
work on most anchors and produce rubbish on the ones with non-ASCII names.

#### `subject_public_key_info`

The key that signatures on the next certificate down are checked against.

```
use webpki_roots::TLS_SERVER_ROOTS;

let a = &TLS_SERVER_ROOTS[0];
// A DER SubjectPublicKeyInfo: algorithm identifier plus the key bits.
assert!(a.subject_public_key_info.as_ref().len() > 40);
assert!(TLS_SERVER_ROOTS.iter().all(|a| !a.subject_public_key_info.as_ref().is_empty()));

// This -- not the certificate -- is what a trust anchor fundamentally is.
// There is no full root certificate in this crate, only the pieces path
// building needs.
```

**When to use it:** essentially never directly; it exists so the verifier can do
its job. It is worth knowing it is here because it explains why the crate is
only ~55 KB for 118 CAs: it stores names, keys and constraints rather than whole
certificates, which is all RFC 5280 path validation needs from an anchor.

#### `name_constraints`

An optional restriction on the names a CA is allowed to certify.

```
use webpki_roots::TLS_SERVER_ROOTS;

let constrained: Vec<_> = TLS_SERVER_ROOTS
    .iter()
    .filter(|a| a.name_constraints.is_some())
    .collect();

// Exactly one anchor in 1.0.11 carries them: TUBITAK Kamu SM, restricted by
// the root programme to the Turkish national domain. The nine bytes decode to
// a permittedSubtrees with a single dNSName of ".tr".
assert_eq!(constrained.len(), 1);
let nc = constrained[0].name_constraints.as_ref().unwrap();
assert_eq!(nc.as_ref().len(), 9);
assert!(nc.as_ref().ends_with(b".tr"));
```

**When to use it:** as context when a certificate from a CA you can see in the
store is rejected anyway. The verifier enforces this, so an otherwise valid
chain under a constrained root fails for a name outside its permitted subtrees.
It is also the clearest illustration of what a root programme does beyond
include and exclude: it can admit a CA for part of the namespace only.

### Wiring it into rustls

#### `RootCertStore` from the slice

Three spellings, all equivalent.

```
use rustls::RootCertStore;
use webpki_roots::TLS_SERVER_ROOTS;

// Struct literal -- the form you will see in most code and in rustls' docs.
let a = RootCertStore { roots: TLS_SERVER_ROOTS.to_vec() };

// Collect -- the tidiest.
let b: RootCertStore = TLS_SERVER_ROOTS.iter().cloned().collect();

// Extend an existing store, to add the public web to a private CA.
let mut c = RootCertStore::empty();
c.extend(TLS_SERVER_ROOTS.iter().cloned());

assert_eq!((a.len(), b.len(), c.len()), (118, 118, 118));

// `extend` does not deduplicate -- the store is a plain Vec.
c.extend(TLS_SERVER_ROOTS.iter().cloned());
assert_eq!(c.len(), 236);
```

**When to use it:** `collect` for a store that is only the public web, `extend`
when you are combining sources — a corporate CA loaded from disk plus the
Mozilla set, which is the usual shape for a client that talks to both internal
and external services. Build it once; see
[`RootCertStore`](rustls.md) on the rustls page for adding your own
certificates.

#### `WebPkiServerVerifier` from the slice

The verifier underneath `ClientConfig`, built directly.

```
use rustls::client::danger::ServerCertVerifier;
use rustls::client::WebPkiServerVerifier;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::RootCertStore;
use std::sync::Arc;

let roots = Arc::new(RootCertStore {
    roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
});
let verifier = WebPkiServerVerifier::builder(roots).build().unwrap();

// A certificate no public CA ever signed is rejected, which is the whole job.
let issued = rcgen::generate_simple_self_signed(vec!["example.com".to_string()]).unwrap();
let end_entity = CertificateDer::from(issued.cert.der().to_vec());

let outcome = verifier.verify_server_cert(
    &end_entity,
    &[],
    &ServerName::try_from("example.com").unwrap(),
    &[],
    UnixTime::now(),
);

assert!(matches!(
    outcome,
    Err(rustls::Error::InvalidCertificate(
        rustls::CertificateError::UnknownIssuer
    ))
));
```

**When to use it:** when you need to validate a chain outside a live connection
— checking a certificate you were handed out of band, or writing a test that
asserts your trust configuration rejects what it should. For an ordinary client,
`with_root_certificates` builds this for you and there is no reason to touch it.
Note the import is from `rustls::client::danger`: the trait is marked that way
because *implementing* it is dangerous, not because calling it is.

### Choosing it over the alternatives

#### `rustls-native-certs` and `rustls-platform-verifier`

The two other ways to answer the same question.

```
// None of these is a drop-in for the others; they differ in what they can see.
//
// webpki-roots            compiled in. Identical everywhere, needs no host
//                         support, no revocation data, cannot see a locally
//                         installed CA, and is as current as your last build.
//
// rustls-native-certs     reads the OS store into a RootCertStore, so an
//                         enterprise CA works. Still no revocation checking,
//                         and behaviour now varies by host.
//
// rustls-platform-verifier delegates the whole decision to the platform:
//                         revocation, administrator policy and OS constraints
//                         included. The one to use for software shipped to
//                         end users.
use webpki_roots::TLS_SERVER_ROOTS;

assert!(!TLS_SERVER_ROOTS.is_empty());
```

**When to use it:** pick `webpki-roots` for a server-side client you control and
redeploy — a backend calling an API, a crawler, a CI job, a container with no
certificate store. Pick `rustls-platform-verifier` for anything a user installs
and keeps, because that is where a frozen trust store does real harm and where
revocation matters. Pick `rustls-native-certs` in the middle case: you want the
host's view, typically because somebody runs a TLS-inspecting proxy with a
private CA. These are not exclusive — a common arrangement is the native store
with this crate `extend`ed on top, so an internal CA is honoured without losing
the public web when the host store is empty.

### Running without std

#### `no_std` support

Unconditional, with no feature to enable.

```
// This compiles unchanged for a bare-metal target such as thumbv7m-none-eabi:
// the crate is `#![no_std]` with `#![forbid(unsafe_code)]`, and its only
// dependency, rustls-pki-types, is no_std-capable too. No feature flag and no
// `default-features = false` are involved.
use webpki_roots::TLS_SERVER_ROOTS;

// Nothing here allocates -- it is a const slice of borrowed DER.
let total: usize = TLS_SERVER_ROOTS
    .iter()
    .map(|a| a.subject.as_ref().len() + a.subject_public_key_info.as_ref().len())
    .sum();

assert_eq!(total, 53_630);  // plus 9 more bytes of name constraints
```

**When to use it:** on an embedded target that speaks TLS, where there is no
operating system store to read and the platform verifier does not apply, so a
compiled-in set is the only option available. Budget for it: those 53,639 bytes
of DER plus the anchor table come to roughly 55 KB of flash, which on a small
microcontroller is a real fraction of the part. If that is too much, the slice is
ordinary data — filtering it down to the handful of CAs your device's endpoints
actually use is a legitimate and much smaller trust store, at the cost of
breaking when one of them rotates.
