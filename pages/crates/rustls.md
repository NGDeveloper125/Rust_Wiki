---
title: "rustls"
version: "0.23.46"
publisher: "Joe Birr-Pixton (ctz), Dirkjan Ochtman (djc), publishers"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-25"
summary: "A TLS implementation in Rust, opinionated by omission: TLS 1.3 and 1.2 only, no legacy ciphers, no renegotiation, and post-quantum key exchange on by default. The state machine is separate from the I/O."
domain: "Crypto, hashing & TLS"
categories: ["cryptography", "network-programming"]
repository: "https://github.com/rustls/rustls"
docs: "https://docs.rs/rustls/0.23.46/rustls/"
---

## Overview

`rustls` is a TLS library written in Rust, and the one the ecosystem has
standardised on — `reqwest`, `hyper` and Cargo itself use it. Being memory-safe
is the headline, but the more consequential design decision is what it refuses
to do.

**It is opinionated by omission.** There is no TLS 1.0 or 1.1, no RC4, 3DES or
DES, no renegotiation, no TLS-level compression, no static RSA key exchange.
These are not off-by-default options you can switch back on — they are absent
from the implementation. If you need to talk to a peer that only speaks TLS 1.0,
`rustls` cannot and will not, and that is the intended answer. TLS 1.3 is the
baseline; TLS 1.2 is present behind the `tls12` feature, which is on by default.

**Post-quantum key exchange is on by default**, which is worth knowing because
it is recent and it changes what goes on the wire:

```
use rustls::{ClientConfig, RootCertStore};

let config = ClientConfig::builder()
    .with_root_certificates(RootCertStore::empty())
    .with_no_client_auth();

// The default provider offers a hybrid post-quantum group first.
let provider = rustls::crypto::CryptoProvider::get_default().unwrap();
let groups: Vec<_> = provider.kx_groups.iter().map(|g| g.name()).collect();

assert_eq!(format!("{:?}", groups[0]), "X25519MLKEM768");
let _ = config;
```

`X25519MLKEM768` is a hybrid of X25519 and ML-KEM-768, so a recorded handshake
is not decryptable later by a quantum computer. It comes from the
`prefer-post-quantum` default feature.

**The first error nearly everyone hits is the crypto provider.** `rustls` does
not implement the cryptography itself — it delegates to a provider, either
`aws-lc-rs` (the default) or `ring`. Building a config with *neither* or *both*
compiled in panics:

the panic reads *"Could not automatically determine the process-level
CryptoProvider from Rustls crate features"*, and goes on to tell you to call
`CryptoProvider::install_default()` or to make sure exactly one of the
`aws-lc-rs` and `ring` features is enabled.

That happens most often when a dependency enables `ring` while you have the
defaults, making the choice ambiguous. The fix is in the message: enable exactly
one, or call `CryptoProvider::install_default()` once at startup before any
config is built.

**Two more things it deliberately does not give you.** Root certificates are not
bundled — add `webpki-roots` for the Mozilla set compiled into your binary, or
`rustls-native-certs` to read the operating system's store. And it performs no
I/O: the core is a state machine that consumes and produces bytes, so you supply
the transport. `Stream`/`StreamOwned` wrap it over anything `Read + Write` for
blocking code, and `tokio-rustls` does the same for async.

In practice you will often not use this crate directly. A `reqwest` or `hyper`
client configures it for you, and reaching for `rustls` yourself means you are
writing a server, implementing a protocol, or need control over verification.
MSRV is 1.71, `no_std` is supported by turning off the `std` feature, and a
`fips` feature exists for the validated `aws-lc-rs` build.

## When to use it

### Use case: A client that validates certificates properly

The baseline configuration, and the one to copy.

```
use rustls::{ClientConfig, RootCertStore};

// The Mozilla root programme's CA set, compiled in.
let roots = RootCertStore {
    roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
};

let config = ClientConfig::builder()
    .with_root_certificates(roots)
    .with_no_client_auth();

// Nothing is negotiated yet; this is just policy.
assert!(config.alpn_protocols.is_empty());
assert!(!config.enable_sni == false); // SNI is on by default
```

**Why it fits:** the builder's type states make it hard to skip a step —
`with_root_certificates` has to be answered before `with_no_client_auth`
becomes available, so you cannot accidentally construct a config that trusts
nothing or everything. This is the part that the many "how do I disable
certificate verification" answers get wrong: there is no flag for it, because
turning off verification turns off TLS's only guarantee. If you genuinely need
it for a test against a self-signed certificate, add that certificate to a
`RootCertStore` instead, as the handshake example below does.

### Use case: A server presenting a certificate

The other half, with a certificate generated on the spot.

```
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;

// A self-signed certificate, as a test or development server would use.
let issued = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
let certificate = CertificateDer::from(issued.cert.der().to_vec());
let key = PrivateKeyDer::try_from(issued.signing_key.serialize_der()).unwrap();

let config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(vec![certificate], key)
    .unwrap();

assert!(config.alpn_protocols.is_empty());
```

**Why it fits:** `with_single_cert` is the common case — one certificate chain
and one key, for one name. It returns `Result` because the key and certificate
must actually match, which is the error you want at startup rather than on the
first connection. For real deployment the chain comes from a file via
`rustls-pemfile`, and for multiple names `with_cert_resolver` lets you choose
per connection from the SNI value. Note the argument order mirrors the client's:
client authentication policy first, then your own identity.

### Use case: Driving the handshake yourself, with no network

The state machine, which is what makes `rustls` embeddable anywhere.

```
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection};
use std::io::{Read, Write};
use std::sync::Arc;

let issued = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
let certificate = CertificateDer::from(issued.cert.der().to_vec());
let key = PrivateKeyDer::try_from(issued.signing_key.serialize_der()).unwrap();

let server_config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(vec![certificate.clone()], key)
    .unwrap();

// Trust exactly that certificate, and nothing else.
let mut roots = RootCertStore::empty();
roots.add(certificate).unwrap();
let client_config = ClientConfig::builder()
    .with_root_certificates(roots)
    .with_no_client_auth();

let name = ServerName::try_from("localhost").unwrap();
let mut client = ClientConnection::new(Arc::new(client_config), name).unwrap();
let mut server = ServerConnection::new(Arc::new(server_config)).unwrap();

// Carry bytes between the two by hand. No sockets involved.
for _ in 0..12 {
    let mut to_server = Vec::new();
    while client.wants_write() {
        client.write_tls(&mut to_server).unwrap();
    }
    if !to_server.is_empty() {
        server.read_tls(&mut to_server.as_slice()).unwrap();
        server.process_new_packets().unwrap();
    }

    let mut to_client = Vec::new();
    while server.wants_write() {
        server.write_tls(&mut to_client).unwrap();
    }
    if !to_client.is_empty() {
        client.read_tls(&mut to_client.as_slice()).unwrap();
        client.process_new_packets().unwrap();
    }

    if !client.is_handshaking() && !server.is_handshaking() {
        break;
    }
}

assert!(!client.is_handshaking());
assert_eq!(client.protocol_version(), Some(rustls::ProtocolVersion::TLSv1_3));

// And application data travels the same way.
client.writer().write_all(b"hello").unwrap();
let mut wire = Vec::new();
while client.wants_write() {
    client.write_tls(&mut wire).unwrap();
}
server.read_tls(&mut wire.as_slice()).unwrap();
server.process_new_packets().unwrap();

let mut received = Vec::new();
server.reader().read_to_end(&mut received).ok();
assert_eq!(&received, b"hello");
```

**Why it fits:** because the connection never touches a socket, `rustls` works
over anything — a Unix socket, a QUIC datagram, a serial link, a test harness
with no network at all. That is also why this example is runnable offline, and
why it is the clearest way to understand the four methods the whole API rests
on: `wants_write`/`write_tls` to get bytes out, `read_tls`/`process_new_packets`
to put bytes in. For ordinary blocking code you would not write this loop —
`StreamOwned` does it for you.

## API map

Two configs, two connections, and the plumbing between them. Everything else is
policy you can usually leave alone.

### Configuration

#### `ClientConfig::builder`

Client policy, built through type states.

```
use rustls::{ClientConfig, RootCertStore};

let roots = RootCertStore {
    roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
};

let mut config = ClientConfig::builder()
    .with_root_certificates(roots)
    .with_no_client_auth();

// Post-construction settings live on the struct.
config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
assert_eq!(config.alpn_protocols.len(), 2);
```

**When to use it:** once per process, wrapped in an `Arc` and shared — building
a config is expensive and a connection only borrows it. The builder sequence is
deliberate: each method returns a different type, so the compiler enforces that
you have chosen roots and a client-auth policy. ALPN is set afterwards on the
struct and matters for HTTP/2, where the protocol is negotiated during the
handshake rather than after it.

#### `ServerConfig::builder` and `with_single_cert`

Server policy and identity.

```
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;

let issued = rcgen::generate_simple_self_signed(vec!["example.com".to_string()]).unwrap();
let chain = vec![CertificateDer::from(issued.cert.der().to_vec())];
let key = PrivateKeyDer::try_from(issued.signing_key.serialize_der()).unwrap();

let mut config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(chain, key)
    .unwrap();

config.alpn_protocols = vec![b"h2".to_vec()];
assert_eq!(config.alpn_protocols.len(), 1);
```

**When to use it:** for a server with one identity. `with_client_cert_verifier`
replaces `with_no_client_auth` for mutual TLS, and `with_cert_resolver` replaces
`with_single_cert` when the certificate depends on the requested name. The chain
is leaf-first, with intermediates after it and the root omitted — getting that
order wrong produces a verification failure on the client that is tedious to
diagnose from the server side.

#### `RootCertStore`

The set of certificate authorities you trust.

```
use rustls::RootCertStore;

// The bundled Mozilla set.
let bundled = RootCertStore {
    roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
};
assert!(!bundled.is_empty());

// Or start empty and add only what you trust.
let mut pinned = RootCertStore::empty();
let issued = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
let certificate = rustls::pki_types::CertificateDer::from(issued.cert.der().to_vec());
pinned.add(certificate).unwrap();
assert_eq!(pinned.len(), 1);
```

**When to use it:** `webpki-roots` for talking to the public internet, since a
compiled-in set behaves identically everywhere and cannot be affected by the
host's configuration — though it also means a revoked root stays trusted until
you update the crate. `rustls-native-certs` reads the OS store instead, which is
what you want when an enterprise installs its own CA. An empty store plus
`add` is certificate pinning, and the right approach for a self-signed
development server.

### Connections

#### `ClientConnection` and `ServerConnection`

One per connection, holding the session state.

```
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore};
use std::sync::Arc;

let config = Arc::new(
    ClientConfig::builder()
        .with_root_certificates(RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth(),
);

// The name is checked against the certificate, so it is required up front.
let name = ServerName::try_from("example.com").unwrap();
let connection = ClientConnection::new(config, name).unwrap();

assert!(connection.is_handshaking());
assert_eq!(connection.protocol_version(), None); // nothing negotiated yet
```

**When to use it:** per connection, from a shared `Arc<Config>`. The
`ServerName` is not decoration — it is the name verified against the
certificate's subject alternative names, and it is also sent as SNI. Passing an
IP address works and matches only a certificate with that IP in it, which most
do not have. `ServerConnection::new` needs no name, since the client supplies it.

#### `StreamOwned` and `Stream`

Blocking I/O over an existing transport.

```
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::sync::Arc;

fn connect(host: &str) -> std::io::Result<()> {
    let config = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
            })
            .with_no_client_auth(),
    );

    let name = ServerName::try_from(host.to_string()).unwrap();
    let connection = ClientConnection::new(config, name).unwrap();
    let socket = std::net::TcpStream::connect((host, 443))?;

    // The result implements Read + Write, so it behaves like a socket.
    let mut tls = StreamOwned::new(connection, socket);
    use std::io::Write;
    tls.write_all(b"GET / HTTP/1.0\r\n\r\n")?;
    Ok(())
}

// Not called here: it would need the network.
let _ = connect;
```

**When to use it:** any synchronous client or server, which is most non-async
code. `StreamOwned` takes both by value and is the usual choice; `Stream`
borrows them, for when the connection must outlive the wrapper. The handshake
happens lazily on the first read or write, so a certificate error surfaces there
rather than at construction. For async, use `tokio-rustls` rather than wrapping
this in `spawn_blocking`.

#### `read_tls`, `write_tls` and `process_new_packets`

The transport-agnostic core.

```
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore};
use std::sync::Arc;

let config = Arc::new(
    ClientConfig::builder()
        .with_root_certificates(RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth(),
);
let name = ServerName::try_from("example.com").unwrap();
let mut connection = ClientConnection::new(config, name).unwrap();

// The client speaks first, so there are bytes to send immediately.
assert!(connection.wants_write());

let mut outbound = Vec::new();
connection.write_tls(&mut outbound).unwrap();
assert!(!outbound.is_empty()); // the ClientHello

// And nothing to read until the peer replies.
assert!(connection.wants_read());
```

**When to use it:** when you own the event loop, or the transport is not a
socket. The contract is four calls: `wants_write` then `write_tls` to drain
outbound bytes, `read_tls` then `process_new_packets` to feed inbound ones.
`process_new_packets` is the one that can fail with a TLS error — a bad
certificate, a protocol violation — and it returns an `IoState` telling you how
much plaintext is now readable. Forgetting to call it after `read_tls` is the
usual reason a hand-written loop stalls.

#### `reader` and `writer`

Application data, once the handshake is done.

```
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore};
use std::io::Write;
use std::sync::Arc;

let config = Arc::new(
    ClientConfig::builder()
        .with_root_certificates(RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth(),
);
let name = ServerName::try_from("example.com").unwrap();
let mut connection = ClientConnection::new(config, name).unwrap();

// Writing is buffered and encrypted, not sent: write_tls does the sending.
connection.writer().write_all(b"request").unwrap();
assert!(connection.wants_write());
```

**When to use it:** for the plaintext either side of the encryption. `writer()`
buffers and encrypts but does not transmit — the bytes leave on the next
`write_tls`, which is the detail that surprises people whose data appears not to
send. `reader()` yields plaintext that `process_new_packets` has already
decrypted, and returns `WouldBlock` rather than zero when there is none yet, so
it is distinguishable from a closed connection.

### Cryptography and introspection

#### `CryptoProvider`

Choosing who does the maths.

```
use rustls::crypto::{aws_lc_rs, CryptoProvider};

// Nothing is installed until something asks for it.
assert!(CryptoProvider::get_default().is_none());

// Installing explicitly is what removes the ambiguity the panic complains about.
CryptoProvider::install_default(aws_lc_rs::default_provider()).unwrap();

let provider = CryptoProvider::get_default().unwrap();
assert!(!provider.cipher_suites.is_empty());

// The post-quantum hybrid group is offered first by default.
assert_eq!(
    format!("{:?}", provider.kx_groups[0].name()),
    "X25519MLKEM768"
);

// A second install fails rather than silently replacing the first.
assert!(CryptoProvider::install_default(aws_lc_rs::default_provider()).is_err());
```

**When to use it:** `install_default()` once at startup, before any config, when
the automatic choice is ambiguous — which is exactly the panic described in the
Overview. Note what the assertions above show: `get_default()` is `None` until
something installs a provider, and building a config installs one implicitly, so
the order of these calls matters. Installing twice is an error rather than a
silent replacement. This is also where you narrow the offered cipher suites or
key exchange groups — clone the default provider, edit the lists, and install
that. The `fips` feature selects a validated `aws-lc-rs` build, for when an
auditor requires it.

#### Negotiation results

What the handshake actually agreed.

```
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore};
use std::sync::Arc;

let config = Arc::new(
    ClientConfig::builder()
        .with_root_certificates(RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth(),
);
let name = ServerName::try_from("example.com").unwrap();
let connection = ClientConnection::new(config, name).unwrap();

// All None before the handshake completes, which is the honest answer.
assert_eq!(connection.protocol_version(), None);
assert!(connection.negotiated_cipher_suite().is_none());
assert!(connection.alpn_protocol().is_none());
```

**When to use it:** for logging and diagnostics, and for ALPN, where
`alpn_protocol()` tells you whether to speak HTTP/2 or HTTP/1.1 and is
load-bearing rather than informational. Every one of these returns `Option`
because nothing is settled until the handshake finishes — a `None` from
`protocol_version()` means you are still handshaking, not that something failed.
`negotiated_key_exchange_group()` is how you confirm the post-quantum group was
actually used, since the peer may not support it.
