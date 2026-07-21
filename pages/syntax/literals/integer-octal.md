---
title: "Octal integer literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["Numeric types & overflow behavior"]
related_syntax: [integer-suffixes, digit-separator]
see_also: [integer-decimal, integer-hexadecimal, integer-binary]
---

## Explanation

A base-8 integer literal, prefixed with `0o`, as in `0o755`.

Note the letter `o`, not a digit `0` — unlike C's ambiguous leading-zero
octal notation (`0755`), Rust requires the explicit `0o` prefix, so a
literal like `0755` is just decimal 755, never accidentally
misinterpreted as octal.

## Usage examples

### Assigning a file permission mode

```
let mode = 0o644; // <- `0o` prefix marks a base-8 (octal) integer literal
```

**Restriction:** only *digits* `0`–`7` may appear after the `0o` prefix —
`0o8` is a compile error (underscores and a type suffix like `0o644_u16`
are still allowed).

### Bit manipulation and flags

Unix-style file permission bits are conventionally written and read in
octal, since each digit maps exactly onto one `rwx` triplet.

```
let mode: u32 = 0o755; // <- octal literal: owner rwx, group rx, other rx — matches `ls -l` directly

let owner_bits = (mode >> 6) & 0o7;
let group_bits = (mode >> 3) & 0o7;
let other_bits = mode & 0o7;

assert_eq!((owner_bits, group_bits, other_bits), (0o7, 0o5, 0o5));
```

Unix permission tooling (`chmod 755`, `ls -l`) is itself
built around octal because each digit is one 3-bit `rwx` group — a
mapping that's lost if the same mode is written in hex or decimal; for
masks that aren't naturally 3-bit-grouped, hex is the more common
convention — see [integer-hexadecimal](integer-hexadecimal.md).

## Explanation (Embedded)

Octal literals mean exactly the same thing under `#![no_std]`, but
honestly there's little embedded-specific to say beyond that: register
addresses and bitmasks are conventionally written in hex or binary (see
[integer-hexadecimal](integer-hexadecimal.md) and
[integer-binary](integer-binary.md)), and bare-metal firmware has no
Unix-style file-permission model for octal's one real everyday use case
to attach to. Where `0o` does show up in genuine embedded Rust is on the
"embedded Linux" side of the ecosystem — a program running on an
SBC-class device (a Raspberry Pi controlling a production line, say)
under embedded Linux still has a real filesystem, so file and sysfs
device permissions look exactly like they would in any other Rust
program built for Linux. On a genuinely bare-metal, no-filesystem
target, octal is legal but essentially unused.

## Usage examples (Embedded)

### Setting sysfs GPIO export permissions on an embedded Linux board

```
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn set_gpio_permissions(path: &str) -> std::io::Result<()> {
    let perms = fs::Permissions::from_mode(0o660); // <- octal literal: same Unix permission convention as any hosted Linux program
    fs::set_permissions(path, perms)
}
```
