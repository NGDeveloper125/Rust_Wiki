---
title: "num-bigint"
version: "0.5.1"
publisher: "Josh Stone (cuviper)"
publisher_url: "https://crates.io/users/cuviper"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-11-22"
summary: "Arbitrary-precision integers: `BigUint` and `BigInt` grow to whatever size the arithmetic needs. Pure Rust, two dependencies, and the default choice unless you are pushing six-figure digit counts."
domain: "Numerics & math"
categories: ["algorithms", "data-structures", "science"]
repository: "https://github.com/rust-num/num-bigint"
docs: "https://docs.rs/num-bigint/0.5.1/num_bigint/"
---

## Overview

`u128` runs out at 39 digits. `num-bigint` does not run out: `BigUint` and
`BigInt` are heap-allocated digit vectors that grow to fit whatever the
arithmetic produces, so a factorial, a modular exponentiation over a 2048-bit
modulus, or a financial total that must not round is exact rather than
wrapped.

It is the default answer in Rust for this, and the reasons are boring in the
good way. Pure Rust, so it cross-compiles anywhere. MIT or Apache-2.0. An MSRV
of **1.60**, which is old enough to be a non-issue. Exactly two dependencies,
`num-integer` and `num-traits`, both from the same family — so if you already
use [`num-traits`](num-traits.md) for generic numeric code, `BigUint` slots into
the same `Zero`/`One`/`Num`/`Integer` bounds you already have.

The interesting question is whether it is *fast enough*, and the honest answer
is "for most things, yes, and the ceiling arrives sooner than you would guess".
Measured on this crate at 0.5.1, two operands of 10,000 decimal digits each
(~33,220 bits):

- parse from decimal — 277 µs
- multiply — 254 µs
- divide, 20,000 digits by 10,000 — 547 µs
- `to_string` on the 20,000-digit product — 2,433 µs

Two things fall out of those numbers. First, **decimal conversion costs more
than the arithmetic** — an order of magnitude more than a multiply — so a
pipeline that parses and prints on every step is spending its time in base
conversion, not in mathematics. Base 16 is not affected, because a
power-of-two radix is a bit-shuffle rather than a long division: on a
10,000-digit value, `to_str_radix(16)` takes 21 µs against 931 µs for
`to_str_radix(10)`, a factor of 44.

Second, the crate is competitive but not the fastest. Against
[`malachite`](malachite.md) on the same operands it is about 1.7× slower on a
multiply — but at 100,000 digits that gap becomes 5.4×, because malachite
reaches for sub-quadratic algorithms that this crate does not have. num-bigint
does use Karatsuba and Toom-3 multiplication, Burnikel-Ziegler division and
divide-and-conquer radix conversion (all added over the 0.4 series), so it is
far from naive; it just stops improving sooner.

So: **num-bigint until profiling says otherwise.** If the profile says
otherwise, [`malachite`](malachite.md) is faster everywhere and the gap widens
with size, at the cost of an LGPL-3.0 licence; [`dashu`](dashu.md) is the
permissively-licensed `no_std`-first alternative; and `rug` wraps GMP for the
genuine ceiling, if you can accept LGPL and a C build — its build script
refuses the `x86_64-pc-windows-msvc` target outright.

One thing it is **not**: constant-time. Operations branch and allocate based on
operand values, so secret-dependent arithmetic leaks timing. For cryptography
where that matters, `crypto-bigint` is the fixed-width constant-time option.
`num-bigint` is for correctness and range, not for secrets.

`no_std` is supported with `default-features = false`, but `alloc` is always
required — the numbers live on the heap by construction.

Finally, on the version: **0.5 is a `rand` upgrade and nothing else.** The
single breaking change splits the old `rand` feature into `rand_0_9`,
`rand_0_10`, `rand_core_0_9` and `rand_core_0_10`, splits the `RandBigInt`
trait into `BigRng09` and `BigRng010`, and deprecates the `gen_*` methods in
favour of `random_*`. If you do not generate random big integers, moving from
0.4 to 0.5 changes nothing in your code. 0.5.1 is a fix for a regression in the
Burnikel-Ziegler division path, so prefer it over 0.5.0.

## When to use it

### Use case: Exact arithmetic that overflows every primitive

Combinatorics overflows fast. `100!` has 158 digits and needs 525 bits, so
there is no primitive that holds it and no floating-point type that holds it
*exactly*. The point of a big integer here is not range for its own sake — it
is that every digit is right.

```
use num_bigint::BigUint;
use num_traits::One;

fn factorial(n: u32) -> BigUint {
    (1..=n).map(BigUint::from).product()
}

fn main() {
    let f = factorial(100);

    assert_eq!(f.to_string().len(), 158);
    assert_eq!(f.bits(), 525);
    assert!(f.to_string().ends_with("000000000000000000000000"));

    // A binomial coefficient: the intermediate factorials are enormous, the
    // answer is not, and the division is exact with no rounding anywhere.
    let c = factorial(100) / (factorial(50) * factorial(50));
    assert_eq!(c.to_string(), "100891344545564193334812497256");

    // f64 would have given you an approximation of this and no warning.
    assert_eq!(BigUint::from(u128::MAX) + BigUint::one(),
               BigUint::parse_bytes(b"340282366920938463463374607431768211456", 10).unwrap());
}
```

**Why it fits:** the arithmetic reads exactly like primitive arithmetic — `*`,
`/`, `.product()` — and the only thing that changed is that it cannot be wrong.
`Sum` and `Product` are implemented, so iterator chains work, and the result
carries no error type because there is no overflow to report.

### Use case: Modular exponentiation over a large modulus

`modpow` is the operation behind RSA, Diffie-Hellman and most primality tests.
Computing `base^exp` first and then reducing is not an option — the intermediate
value would be astronomically large — so the modular reduction has to happen
inside the exponentiation, which is what `modpow` does.

```
use num_bigint::BigUint;
use num_traits::One;

fn main() {
    // 2^127 - 1, a Mersenne prime.
    let p = BigUint::parse_bytes(b"170141183460469231731687303715884105727", 10).unwrap();
    let two = BigUint::from(2u32);

    // Fermat's little theorem: 2^(p-1) = 1 (mod p) for prime p.
    assert_eq!(two.modpow(&(&p - BigUint::one()), &p), BigUint::one());

    // modinv gives the multiplicative inverse, or None when gcd != 1.
    let inv = BigUint::from(3u32).modinv(&p).unwrap();
    assert_eq!(inv.to_string(), "113427455640312821154458202477256070485");
    assert_eq!((BigUint::from(3u32) * &inv) % &p, BigUint::one());

    assert_eq!(BigUint::from(2u32).modinv(&BigUint::from(4u32)), None);
}
```

Note that this is the right shape for a *test* or a tool, not for handling live
key material: the implementation is not constant-time, so an attacker who can
measure how long your exponentiation took learns something about the exponent.

**Why it fits:** `modpow` is the one operation where a general-purpose big
integer type genuinely has to be told your intent, and it is here, correct, and
takes `&self` so nothing is consumed. `modinv` closes the loop for the
extended-Euclid step you would otherwise write yourself.

### Use case: A number too large for any primitive, arriving over the wire

An API hands you an integer as a decimal string because JSON numbers cannot
carry it, or a binary protocol hands you a fixed-width big-endian field. Either
way you need to read it, do exact arithmetic, and put it back in the format the
other side expects.

```
use num_bigint::{BigInt, BigUint, Sign};

fn main() {
    // Text in, text out.
    let n = BigUint::parse_bytes(b"10000000000000000000000000000000000000000", 10).unwrap();
    assert_eq!((&n / 7u32).to_string(), "1428571428571428571428571428571428571428");
    assert_eq!((&n % 7u32).to_string(), "4");

    // Bytes in, bytes out -- and there are two byte formats, which is the
    // part that bites. Magnitude-plus-sign:
    let v = BigInt::from(-2);
    assert_eq!(v.to_bytes_be(), (Sign::Minus, vec![2u8]));

    // ...versus two's complement:
    assert_eq!(v.to_signed_bytes_be(), vec![0xfeu8]);
    assert_eq!(BigInt::from_signed_bytes_be(&v.to_signed_bytes_be()), v);

    // The same single byte 0xff means two different numbers depending on
    // which constructor reads it.
    assert_eq!(BigInt::from_signed_bytes_be(&[0xff]), BigInt::from(-1));
    assert_eq!(BigInt::from_bytes_be(Sign::Plus, &[0xff]), BigInt::from(255));
}
```

**Why it fits:** both directions are covered without a dependency on a
serialisation framework, and the byte constructors are explicit about sign
handling rather than guessing. If the field is two's complement — which it
usually is in a binary protocol — `from_signed_bytes_be` is the one that is
right, and `from_bytes_be` will silently give you a positive number instead.

## API map

Everything is an inherent method on `BigUint` or `BigInt` unless noted.
`num-integer` and `num-traits` supply several of the operations through traits,
and those are called out where they appear.

### Creating one

#### From primitives, and the constants

`From` is implemented for every unsigned primitive into `BigUint`, and for every
signed and unsigned primitive into `BigInt`. The zero and one values are
associated constants, and `new_const` builds a small value in a `const` item.

```
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};

const SEVEN: BigUint = BigUint::new_const(7);
const NEG_SEVEN: BigInt = BigInt::new_const(-7);

fn main() {
    assert_eq!(BigUint::from(42u64).to_string(), "42");
    assert_eq!(BigInt::from(-42i64).to_string(), "-42");

    assert_eq!(BigUint::ZERO, BigUint::zero());
    assert_eq!(BigUint::ONE, BigUint::one());
    assert_eq!(BigInt::NEG_ONE.to_string(), "-1");

    assert_eq!(SEVEN.to_string(), "7");
    assert_eq!(NEG_SEVEN.to_string(), "-7");

    // There is no From<i32> for BigUint -- a negative has nowhere to go.
    assert_eq!(BigUint::try_from(BigInt::from(-1)).is_err(), true);
    assert_eq!(BigUint::try_from(BigInt::from(5)).unwrap(), BigUint::from(5u32));
}
```

**When to use it:** the constants in any `const` or `static`, and `From` for
everything else. Prefer `BigUint::ZERO` over `BigUint::zero()` where you have a
choice: it needs no trait in scope.

#### `from_str`, `from_str_radix` and `parse_bytes`

Three ways in from text. `FromStr` handles decimal with an optional sign,
`from_str_radix` (the `Num` trait) takes a base and returns a `Result`, and
`parse_bytes` takes `&[u8]` and returns an `Option`.

```
use num_bigint::{BigInt, BigUint};
use num_traits::Num;
use std::str::FromStr;

fn main() {
    assert_eq!(BigUint::from_str("255").unwrap(), BigUint::from(255u32));
    assert_eq!(BigUint::from_str_radix("ff", 16).unwrap(), BigUint::from(255u32));
    assert_eq!(BigUint::parse_bytes(b"ff", 16), Some(BigUint::from(255u32)));
    assert_eq!(BigInt::from_str("-255").unwrap(), BigInt::from(-255));

    // Errors are plain and a little terse.
    assert_eq!(
        BigUint::from_str("abc").unwrap_err().to_string(),
        "invalid digit found in string"
    );
    assert_eq!(
        BigUint::from_str("").unwrap_err().to_string(),
        "cannot parse integer from empty string"
    );
    // A sign is a digit error for BigUint, not a range error.
    assert!(BigUint::from_str("-1").is_err());
}
```

The radix argument is checked with a panic, not with the `Option`:

```
use num_bigint::BigUint;

fn main() {
    // parse_bytes returns Option, so a radix mistake looks recoverable. It
    // is not: the radix is asserted separately.
    //
    //   thread panicked: The radix must be within 2...36
    let r = std::panic::catch_unwind(|| BigUint::parse_bytes(b"12", 1));
    assert!(r.is_err());

    // The None case is reserved for bad digits.
    assert_eq!(BigUint::parse_bytes(b"12", 2), None);
}
```

**When to use it:** `from_str` for user input and config, `from_str_radix` when
the base comes from the data, `parse_bytes` when you have bytes and do not want
to validate UTF-8 first. Validate any caller-supplied radix yourself — the
`Option` will not catch it.

#### `from_bytes_be`, `from_signed_bytes_be` and friends

Two different binary conventions, and picking the wrong one produces a valid
wrong answer rather than an error. The `_be`/`_le` pairs cover both byte orders.

```
use num_bigint::{BigInt, BigUint, Sign};

fn main() {
    // BigUint: magnitude only.
    assert_eq!(BigUint::from_bytes_be(&[0x01, 0x00]), BigUint::from(256u32));
    assert_eq!(BigUint::from_bytes_le(&[0x01, 0x00]), BigUint::from(1u32));

    // BigInt, magnitude plus an explicit sign.
    assert_eq!(BigInt::from_bytes_be(Sign::Minus, &[0x02]), BigInt::from(-2));

    // BigInt, two's complement -- the sign is in the bytes.
    assert_eq!(BigInt::from_signed_bytes_be(&[0xfe]), BigInt::from(-2));
    assert_eq!(BigInt::from_signed_bytes_be(&[0x7f]), BigInt::from(127));
    assert_eq!(BigInt::from_signed_bytes_be(&[0x80]), BigInt::from(-128));

    // from_radix_be takes digits in an arbitrary base up to 256.
    assert_eq!(BigUint::from_radix_be(&[1, 0], 256), Some(BigUint::from(256u32)));
}
```

**When to use it:** `from_signed_bytes_be` for a signed field in a binary
protocol, which is nearly always two's complement. `from_bytes_be` for an
unsigned field, or when the sign travels separately. Empty input is zero in
both, so a zero-length field will not error.

#### `BigInt::from_biguint`, `Sign`, `magnitude` and `into_parts`

A `BigInt` is a `BigUint` plus a `Sign`, and the crate lets you work with the
halves directly. `Sign` has three variants, not two.

```
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Zero;

fn main() {
    let n = BigInt::from_biguint(Sign::Minus, BigUint::from(1234u32));
    assert_eq!(n.to_string(), "-1234");

    assert_eq!(n.sign(), Sign::Minus);
    assert_eq!(n.magnitude(), &BigUint::from(1234u32)); // <- unsigned, borrowed
    assert_eq!(n.clone().into_parts(), (Sign::Minus, BigUint::from(1234u32)));

    // Zero has its own sign, so there are three variants and no -0.
    assert_eq!(BigInt::zero().sign(), Sign::NoSign);
    assert_eq!(BigInt::from_biguint(Sign::Minus, BigUint::zero()).sign(), Sign::NoSign);

    // Only a non-negative BigInt converts back.
    assert_eq!(n.to_biguint(), None);
    assert_eq!(BigInt::from(5).to_biguint(), Some(BigUint::from(5u32)));
}
```

**When to use it:** `magnitude` when you want the absolute value without
cloning, `into_parts` when you are re-encoding into a format with a separate
sign field. Match on `Sign` exhaustively — code that assumes `Plus` or `Minus`
will be wrong for zero.

### Arithmetic

#### Operators, and the `BigUint` subtraction panic

All the normal operators are implemented, by value and by reference. The one
sharp edge is that `BigUint` is unsigned and has no wrapping behaviour, so a
subtraction that would go negative **panics**.

```
use num_bigint::{BigInt, BigUint};
use num_traits::CheckedSub;

fn main() {
    let a = BigUint::from(1u32);
    let b = BigUint::from(2u32);

    // thread panicked: Cannot subtract b from a because b is larger than a.
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| &a - &b));
    assert!(r.is_err());

    // Two ways out: ask first, or use the signed type.
    assert_eq!(a.checked_sub(&b), None);
    assert_eq!(BigInt::from(1) - BigInt::from(2), BigInt::from(-1));

    // Reference operands avoid cloning; at least one side must own or borrow
    // consistently, and &a op &b always works.
    let big = BigUint::from(10u32);
    assert_eq!(&big * &big, BigUint::from(100u32));
    assert_eq!(&big - &a, BigUint::from(9u32));
}
```

**When to use it:** reach for `BigInt` by default unless you are sure the domain
is non-negative, and use `checked_sub` on `BigUint` wherever the operands come
from outside your control. This is the single most common way a `num-bigint`
program panics in production.

#### `pow`, `modpow` and `modinv`

Exponentiation by squaring, the modular version, and the modular inverse.

```
use num_bigint::{BigInt, BigUint};
use num_traits::One;

fn main() {
    assert_eq!(BigUint::from(2u32).pow(100u32).to_string(),
               "1267650600228229401496703205376");

    let m = BigUint::from(1_000_000_007u32);
    assert_eq!(BigUint::from(2u32).modpow(&BigUint::from(100u32), &m).to_string(),
               "976371285");

    // BigInt::modpow reduces into [0, modulus), so a negative base still
    // gives a non-negative result.
    assert_eq!(BigInt::from(-3).modpow(&BigInt::from(3), &BigInt::from(7)),
               BigInt::one());

    // Both panic on a zero modulus; BigInt::modpow also panics on a negative
    // exponent.
    //   "attempt to calculate with zero modulus!"
    //   "negative exponentiation is not supported!"
    assert!(std::panic::catch_unwind(|| {
        BigUint::one().modpow(&BigUint::one(), &BigUint::ZERO)
    }).is_err());
    assert!(std::panic::catch_unwind(|| {
        BigInt::from(2).modpow(&BigInt::from(-1), &BigInt::from(7))
    }).is_err());
}
```

**When to use it:** `pow` for a plain power with a `u32` exponent, `modpow`
whenever a modulus is involved — never `pow` followed by `%`, which computes an
enormous intermediate for no reason. Guard the modulus against zero yourself;
neither method returns a `Result`.

#### Division: three answers, and how to pick one

For negative operands there are three defensible results, and the crate offers
all three. `/` and `%` truncate toward zero like Rust's primitives;
`div_floor`/`mod_floor` round toward negative infinity; `div_euclid`/`rem_euclid`
guarantee a non-negative remainder.

```
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Euclid;

fn main() {
    let a = BigInt::from(-7);
    let b = BigInt::from(2);

    assert_eq!((&a / &b, &a % &b), (BigInt::from(-3), BigInt::from(-1)));
    assert_eq!((a.div_floor(&b), a.mod_floor(&b)), (BigInt::from(-4), BigInt::from(1)));
    assert_eq!((a.div_euclid(&b), a.rem_euclid(&b)), (BigInt::from(-4), BigInt::from(1)));

    // floor and Euclid agree for a positive divisor and diverge for a
    // negative one: Euclid's remainder is never negative, floor's takes the
    // sign of the divisor.
    let c = BigInt::from(-2);
    assert_eq!(a.mod_floor(&c), BigInt::from(-1));
    assert_eq!(a.rem_euclid(&c), BigInt::from(1));

    // One pass for both halves.
    assert_eq!(a.div_rem(&b), (BigInt::from(-3), BigInt::from(-1)));
    assert_eq!(a.div_mod_floor(&b), (BigInt::from(-4), BigInt::from(1)));

    // Division by zero panics, as it does for primitives.
    assert!(std::panic::catch_unwind(|| BigInt::from(1) / BigInt::ZERO).is_err());
}
```

**When to use it:** `div_rem` whenever you want both results, since it costs one
division instead of two. For the mode: `rem_euclid` if the remainder indexes
into something and must not be negative, `%` if you are matching the behaviour
of Rust's primitive integers, `mod_floor` if you are matching Python or a
mathematical convention.

#### `sqrt`, `cbrt` and `nth_root`

Truncated integer roots, from the `Roots` trait.

```
use num_bigint::BigUint;

fn main() {
    let n = BigUint::parse_bytes(b"10000000000000000000000000000000000000000", 10).unwrap();
    assert_eq!(n.sqrt().to_string(), "100000000000000000000");

    assert_eq!(BigUint::from(27u32).cbrt(), BigUint::from(3u32));
    assert_eq!(BigUint::from(81u32).nth_root(4), BigUint::from(3u32));

    // Truncating, not rounding: floor of the real root.
    assert_eq!(BigUint::from(80u32).nth_root(4), BigUint::from(2u32));
    assert_eq!(BigUint::from(10u32).sqrt(), BigUint::from(3u32));
}
```

**When to use it:** primality trial division, perfect-square tests, anything
where `(x as f64).sqrt()` would lose precision — which is any value above 2^53.
Check `r * r == n` yourself if you need to know whether the root was exact.

#### `gcd`, `lcm` and `extended_gcd`

From `num_integer::Integer`, implemented for both types.

```
use num_bigint::{BigInt, BigUint};
use num_integer::Integer;

fn main() {
    let a = BigUint::from(48u32);
    let b = BigUint::from(18u32);

    assert_eq!(a.gcd(&b), BigUint::from(6u32));
    assert_eq!(a.lcm(&b), BigUint::from(144u32));
    assert!(a.is_even());
    assert!(!a.is_multiple_of(&BigUint::from(5u32)));

    // Bezout coefficients: gcd = a*x + b*y.
    let eg = BigInt::from(48).extended_gcd(&BigInt::from(18));
    assert_eq!(eg.gcd, BigInt::from(6));
    assert_eq!(&eg.x * 48 + &eg.y * 18, eg.gcd);
}
```

**When to use it:** reducing fractions, CRT reconstruction, and the inverse
calculation that `modinv` otherwise hides. `extended_gcd` is the one worth
knowing about, because writing it correctly by hand is fiddly.

### Inspecting one

#### `bits`, `bit`, `count_ones` and `trailing_zeros`

Bit-level queries. `bits()` is the position of the highest set bit plus one —
the number of bits needed to represent the value — and it is zero for zero.

```
use num_bigint::BigUint;
use num_traits::Zero;

fn main() {
    let x = BigUint::from(0b1011_0000u32);

    assert_eq!(x.bits(), 8);
    assert!(x.bit(4));
    assert!(!x.bit(0));
    assert_eq!(x.trailing_zeros(), Some(4));
    assert_eq!(x.count_ones(), 3);
    assert_eq!((&x << 8u32).bits(), 16);

    // Zero is the edge case in both: no bits, and no trailing zeros to report.
    assert_eq!(BigUint::zero().bits(), 0);
    assert_eq!(BigUint::zero().trailing_zeros(), None);

    let mut y = BigUint::zero();
    y.set_bit(100, true);
    assert_eq!(y.bits(), 101);
    assert_eq!(y, BigUint::ONE << 100u32);
}
```

**When to use it:** `bits()` to size a buffer or to report a key length, `bit`
and `set_bit` for bitmap-like use, `trailing_zeros` to strip powers of two
before a primality test. Note `count_ones` and `trailing_ones` exist on
`BigUint` only — `BigInt` has neither, since the answer would depend on a
two's-complement width it does not have.

#### `to_u64`, `to_f64` and `TryFrom`

Narrowing back to a primitive. `ToPrimitive` returns `None` rather than
truncating; `TryFrom` gives an error type instead.

```
use num_bigint::BigUint;
use num_traits::ToPrimitive;

fn main() {
    let small = BigUint::from(42u32);
    let huge = BigUint::parse_bytes(b"123456789012345678901234567890", 10).unwrap();

    assert_eq!(small.to_u64(), Some(42));
    assert_eq!(huge.to_u64(), None); // <- does not wrap, does not truncate
    assert_eq!(u64::try_from(&small).unwrap(), 42);
    assert!(u64::try_from(&huge).is_err());

    // to_f64 is lossy but succeeds, which is the asymmetry to watch.
    assert_eq!(huge.to_f64(), Some(1.2345678901234568e29));
    assert_eq!(small.to_f64(), Some(42.0));
}
```

**When to use it:** `to_u64` or `TryFrom` at the boundary where a big integer
has to become an index, a length, or a database column. Do not use `to_f64` to
test magnitude — it will happily approximate a 10,000-digit number into
infinity-adjacent nonsense rather than refuse.

### Converting out

#### `to_string` and `to_str_radix`

Text out. The cost difference between decimal and a power-of-two radix is large
enough to change how you structure a program.

```
use num_bigint::BigUint;
use std::time::Instant;

fn main() {
    let n = BigUint::parse_bytes(b"255", 10).unwrap();
    assert_eq!(n.to_string(), "255");
    assert_eq!(n.to_str_radix(16), "ff");
    assert_eq!(n.to_str_radix(2), "11111111");
    assert_eq!(format!("{n:x}"), "ff");
    assert_eq!(format!("{n:b}"), "11111111");

    // Radix 10 needs repeated division; radix 16 is a bit-shuffle. On a
    // 10,000-digit value this measured 931 us against 21 us.
    let big = BigUint::parse_bytes(&vec![b'9'; 10_000], 10).unwrap();

    let t = Instant::now();
    let dec = big.to_str_radix(10);
    let dec_us = t.elapsed().as_secs_f64() * 1e6;

    let t = Instant::now();
    let hex = big.to_str_radix(16);
    let hex_us = t.elapsed().as_secs_f64() * 1e6;

    assert_eq!(dec.len(), 10_000);
    assert_eq!(hex.len(), 8_305);
    assert!(dec_us > hex_us, "decimal {dec_us:.0}us vs hex {hex_us:.0}us");
}
```

**When to use it:** `to_string` at the edges of the program only. Inside a
calculation keep values as `BigUint`, and if you need an intermediate textual
form for a cache key or a hash, use radix 16 or `to_bytes_be` rather than
decimal.

#### `to_bytes_be`, `to_signed_bytes_be` and the digit accessors

Binary out, in the same two conventions as the constructors, plus direct access
to the internal limbs.

```
use num_bigint::{BigInt, BigUint, Sign};

fn main() {
    let v = BigInt::from(-2);
    assert_eq!(v.to_bytes_be(), (Sign::Minus, vec![2u8]));
    assert_eq!(v.to_signed_bytes_be(), vec![0xfeu8]);
    assert_eq!(v.to_signed_bytes_le(), vec![0xfeu8]);

    // 2^128: limbs come out least-significant first.
    let n = BigUint::ONE << 128u32;
    assert_eq!(n.to_u64_digits(), vec![0, 0, 1]);
    assert_eq!(n.iter_u64_digits().collect::<Vec<u64>>(), vec![0, 0, 1]);
    assert_eq!(n.bits(), 129);

    // to_bytes_be is the minimal big-endian encoding, with no leading zero.
    assert_eq!(BigUint::from(255u32).to_bytes_be(), vec![0xff]);
    assert_eq!(BigUint::from(256u32).to_bytes_be(), vec![0x01, 0x00]);
    assert_eq!(BigUint::ZERO.to_bytes_be(), vec![0u8]);
}
```

**When to use it:** `to_signed_bytes_be` to write a two's-complement protocol
field, `to_bytes_be` for an unsigned one. `iter_u64_digits` rather than
`to_u64_digits` when you only want to read the limbs, since it allocates
nothing. Remember the encoding is minimal-width: if the wire format wants a
fixed 32 bytes, you must left-pad it yourself.

### Random values

#### `BigRng010`, `random_biguint` and `random_bigint_range`

With the `rand_0_10` feature, an extension trait adds big-integer methods to any
`rand` generator. This is the part of the API that 0.5 changed.

```
use num_bigint::{BigRng010, BigUint};

fn main() {
    let mut rng = rand::rng();

    // A value with at most this many bits.
    let k = rng.random_biguint(256);
    assert!(k.bits() <= 256);

    // Half-open range, as everywhere else in Rust.
    let lo = BigUint::from(10u32);
    let hi = BigUint::from(20u32);
    for _ in 0..100 {
        let r = rng.random_biguint_range(&lo, &hi);
        assert!(r >= lo && r < hi);
    }

    // The signed version spans negative values too.
    let s = rng.random_bigint(64);
    assert!(s.bits() <= 64);
}
```

**When to use it:** test data, randomised property tests, and nonces where
cryptographic strength is not the requirement. The feature name must match your
`rand` version — `rand_0_10` for `rand` 0.10, `rand_0_9` for 0.9 — and on 0.5
the methods are `random_*`; the old `gen_*` names are deprecated, and the trait
is now `BigRng010` rather than `RandBigInt`. For key generation, use a crate
built for it rather than assembling one here.

### Serialising

#### `serde` support, and what it actually emits

With the `serde` feature both types round-trip. The representation is **not** a
decimal string — it is the internal digit array, which is portable between
platforms but not between languages.

```
use num_bigint::{BigInt, BigUint};

fn main() {
    let u = BigUint::from(255u32);
    assert_eq!(serde_json::to_string(&u).unwrap(), "[255]");

    // BigInt is [sign, [digits]], with sign as -1, 0 or 1 and the digits
    // little-endian u32s.
    let i = BigInt::from(-12345678901234567890i128);
    assert_eq!(
        serde_json::to_string(&i).unwrap(),
        "[-1,[3944680146,2874452364]]"
    );

    assert_eq!(serde_json::from_str::<BigInt>("[-1,[3944680146,2874452364]]").unwrap(), i);
    assert_eq!(serde_json::to_string(&BigUint::ZERO).unwrap(), "[]");
}
```

**When to use it:** only when both ends are Rust using the same crate. If the
JSON is a public API, or another language reads it, serialise the decimal
string yourself with `#[serde(with = ...)]` or a wrapper type — otherwise you
are publishing a limb array that nothing else can interpret, and that will
change shape if the crate's internal digit width ever does. See
[`serde`](serde.md) for the attribute machinery.

### Running without std

#### `default-features = false`

The crate works without `std`, but not without an allocator.

```
use num_bigint::BigUint;

fn main() {
    // Cargo.toml for a no_std target:
    //
    //   num-bigint = { version = "0.5.1", default-features = false }
    //
    // `alloc` is always required -- the digits live in a Vec, so there is no
    // fixed-size mode and no way to use this without a heap.
    //
    // Dropping `std` costs you the std::error::Error impls on the error types
    // and some internal floating-point approximations.
    let n = BigUint::from(2u32).pow(64u32);
    assert_eq!(n.to_string(), "18446744073709551616");
    assert_eq!(n.bits(), 65);
}
```

**When to use it:** embedded targets with an allocator configured. If you have
no heap at all, this crate cannot help — a fixed-width crate such as
`crypto-bigint` is the shape you need instead.
