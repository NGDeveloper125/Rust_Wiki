---
title: "dashu"
version: "0.6.2"
publisher: "Jacob Zhong (cmpute)"
publisher_url: "https://crates.io/users/cmpute"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-11-22"
summary: "A family of arbitrary-precision number types under one roof: integers, decimal and binary floats with correctly-rounded transcendentals, rationals and complex numbers. Pure Rust, full `no_std`, permissively licensed."
domain: "Numerics & math"
categories: ["parsing", "mathematics", "no-std"]
repository: "https://github.com/cmpute/dashu"
docs: "https://docs.rs/dashu/0.6.2/dashu/"
---

## Overview

`dashu` is not a big-integer crate. It is a *family* of arbitrary-precision
numeric types that share conventions, and the author describes the goal plainly:
"a Rust native alternative to GNU GMP + MPFR + MPC". The crate you depend on is
an umbrella that re-exports five others, so what you actually get is:

- `dashu::integer` — `UBig` (unsigned) and `IBig` (signed)
- `dashu::float` — `FBig`, an arbitrary-precision float parameterised by
  rounding mode *and* radix, plus `DBig` for base-10
- `dashu::rational` — `RBig`, exact fractions, and `Relaxed`
- `dashu::complex` — `CBig`
- `dashu::base` — the traits all of the above implement

That breadth is the reason to choose it. If you need big integers and nothing
else, [`num-bigint`](num-bigint.md) is the more conventional choice and has a
far larger installed base. What `num-bigint` cannot do at all is arbitrary
precision *floating point* or exact rationals, and reaching for a second crate
means two sets of conventions, two conversion stories and two error types. Here
`UBig`, `DBig` and `RBig` interoperate because they were designed together.

Three concrete advantages fall out of that design, and all of them matter more
in practice than a benchmark:

- **Transcendentals are correctly rounded.** `exp`, `ln`, the trigonometric and
  hyperbolic functions, `sqrt`, `powf`, and the constants `pi` and `e` are
  available on `FBig` at whatever precision you ask for, each rounded correctly
  rather than approximately.
- **Lossy conversions say so.** `to_f64` returns
  `Approximation<f64, Sign>` — `Exact(v)` or `Inexact(v, direction)` — so you
  learn both that the value was rounded and which way. Compare `num-bigint`,
  where `to_f64` hands back an `Option` and silently approximates.
- **serde emits a decimal string.** `UBig::from(255u32)` serialises as `"255"`,
  not as an array of internal limbs, so the JSON is readable by other languages
  and stable against changes to the crate's internals.

It is pure Rust, MIT or Apache-2.0, has **full `no_std` support**, and declares
an MSRV of **1.68** for a default build — the README notes some optional
features need more, `rkyv_v08` wanting 1.81.

On speed: it is competitive at moderate sizes and falls behind at large ones.
Two operands of 10,000 decimal digits each, measured against `num-bigint`
0.5.1 and [`malachite`](malachite.md) 0.14.0 on the same machine:

- multiply — dashu 212 µs, num-bigint 254 µs, malachite 150 µs
- divide 20,000 by 10,000 digits — dashu 407 µs, num-bigint 547 µs, malachite 452 µs
- parse from decimal — dashu 275 µs, num-bigint 277 µs, malachite 174 µs

At 100,000 digits the ordering changes: a multiply takes dashu **14.3 ms**
against num-bigint's 9.0 ms and malachite's 1.7 ms. So dashu's integer
multiplication scales worse than either, and if you expect to grow past roughly
ten thousand digits you should measure rather than assume. Choose this crate
for the type family, the licence and `no_std`, not as a drop-in speed upgrade.

One ergonomic wrinkle to know before you start: **most of the interesting
methods are trait methods, and the traits live in `dashu::base`.** `bit_len`,
`bit`, `gcd`, `div_rem`, `abs`, `is_power_of_two` and more will all fail to
resolve until you import the trait. `use dashu::base::*;` at the top of a module
doing serious arithmetic saves a lot of compiler round-trips.

## When to use it

### Use case: Big integers that cross a language boundary as text

You need exact integer arithmetic beyond `u128`, and the values have to travel
through JSON to something that is not Rust. The arithmetic is the easy part; the
representation on the wire is where crates differ, and `dashu` writes a plain
decimal string.

```
use dashu::integer::{IBig, UBig};
use std::str::FromStr;

fn main() {
    let n = UBig::from_str("123456789012345678901234567890").unwrap();
    let doubled = &n * 2u8;

    assert_eq!(doubled.to_string(), "246913578024691357802469135780");

    // serde gives you the decimal string, so any language can read it.
    assert_eq!(serde_json::to_string(&n).unwrap(),
               "\"123456789012345678901234567890\"");
    assert_eq!(serde_json::to_string(&UBig::from(255u32)).unwrap(), "\"255\"");
    assert_eq!(serde_json::to_string(&IBig::from(-255)).unwrap(), "\"-255\"");

    // ...and reads it back.
    let back: UBig = serde_json::from_str("\"123456789012345678901234567890\"").unwrap();
    assert_eq!(back, n);

    // IBig packs its sign into the same footprint as UBig.
    assert_eq!(std::mem::size_of::<UBig>(), 24);
    assert_eq!(std::mem::size_of::<IBig>(), 24);
}
```

**Why it fits:** the serialised form is the one a human would write and a
JavaScript or Python consumer can parse, which removes an entire category of
integration bug. It also will not drift: a limb array is a window onto a crate's
internals, and a decimal string is a number.

### Use case: Decimal arithmetic that must not drift

Money, invoices, tax, anything where a tenth is a real quantity rather than an
approximation. `f64` cannot represent 0.1, so sums of money accumulate error
that eventually shows up in a reconciliation report. `DBig` is base-10, so it
can.

```
use dashu::float::DBig;
use std::str::FromStr;

fn main() {
    let a = DBig::from_str("0.1").unwrap();
    let b = DBig::from_str("0.2").unwrap();

    assert_eq!((&a + &b).to_string(), "0.3");
    assert_eq!(&a + &b, DBig::from_str("0.3").unwrap());

    // The same sum in f64, for contrast.
    assert_eq!(0.1f64 + 0.2, 0.30000000000000004);
    assert!(0.1f64 + 0.2 != 0.3);

    // A hundred tenths is exactly ten, which is the property a ledger needs.
    let total = (0..100).fold(DBig::ZERO, |acc, _| acc + DBig::from_str("0.1").unwrap());
    assert_eq!(total, DBig::from_str("10.0").unwrap());

    // The value is a significand and a base-10 exponent, not a binary fraction.
    assert_eq!(format!("{:?}", a.repr()), "1 * 10 ^ -1");
    assert_eq!(a.digits(), 1);
}
```

Watch the precision rule here: parsing takes the precision cap **from how the
literal was written**, counting the digits present. `"0.1"` carries a precision
of 2 and `"0.100"` carries 4, even though they are the same number — so a
calculation seeded from a short literal can hit its cap sooner than you expect.
Set it explicitly with `with_precision` when it matters.

**Why it fits:** base 10 in, base 10 out, with no representable-value surprises
in between, and the arithmetic operators are the ordinary ones. The alternative
— scaled integers in a `i64` of "cents" — works until something needs three
decimal places or a division, and then you are implementing this by hand.

### Use case: A mathematical constant to as many digits as you want

You need π or e or √2 to forty digits, or a function evaluated to a precision no
primitive float reaches. `f64` gives you about 15 significant decimal digits and
then stops.

```
use dashu::float::DBig;

fn main() {
    // Precision is carried on the value; with_precision returns a Rounded,
    // so .value() unwraps it.
    let one = DBig::ONE.with_precision(40).value();

    // 4 * atan(1) = pi, correctly rounded at the requested precision.
    assert_eq!(
        (one.clone().atan() * DBig::from(4u8)).to_string(),
        "3.141592653589793238462643383279502884197"
    );
    assert_eq!(
        one.clone().exp().to_string(),
        "2.718281828459045235360287471352662497757"
    );

    let two = DBig::from(2u8).with_precision(40).value();
    assert_eq!(two.clone().sqrt().to_string(), "1.41421356237309504880168872420969807857");
    assert_eq!(two.ln().to_string(), "0.6931471805599453094172321214581765680755");

    // Precision really is a cap you choose, not a property of the type.
    let third_30 = DBig::ONE.with_precision(30).value() / DBig::from(3u8);
    assert_eq!(third_30.to_string(), "0.333333333333333333333333333333");
    assert_eq!(third_30.precision(), 30);

    let third_10 = DBig::ONE.with_precision(10).value() / DBig::from(3u8);
    assert_eq!(third_10.to_string(), "0.3333333333");
}
```

**Why it fits:** these are the digits of π, not an approximation that happens to
agree for a while — the crate documents its transcendentals as correctly
rounded, so the last digit it gives you is the right one. Getting this from
`num-bigint` is not a matter of extra work; the functions do not exist there at
all.

## API map

The crate is a family, so the first group below is orientation rather than API.
Remember that many methods come from traits in `dashu::base`.

### Finding your way around

#### The modules, and the type aliases over them

Four numeric modules plus a trait module, and a set of friendlier aliases at the
crate root.

```
use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;

fn main() {
    // dashu::integer  -> UBig, IBig
    // dashu::float    -> FBig, DBig, CachedFBig
    // dashu::rational -> RBig, Relaxed
    // dashu::complex  -> CBig, CachedCBig
    // dashu::base     -> the traits (BitTest, Gcd, DivRem, Abs, Sign, ...)
    //
    // The crate root also aliases them in plainer words:
    //   Natural  = UBig      Integer = IBig
    //   Real     = FBig      Decimal = DBig
    //   Rational = RBig      Complex = CBig
    let n: dashu::Natural = UBig::from(7u8);
    let i: dashu::Integer = IBig::from(-7);
    let r: dashu::Rational = RBig::from_parts(1.into(), UBig::from(3u8));

    assert_eq!(n.to_string(), "7");
    assert_eq!(i.to_string(), "-7");
    assert_eq!(r.to_string(), "1/3");
}
```

**When to use it:** reach for the aliases in application code, where `Decimal`
reads better than `DBig`, and the real names in library signatures, where a
reader may not know the aliases exist. Import `dashu::base::*` in any module
doing more than addition.

### Creating integers

#### From primitives, the constants, and `ones`

```
use dashu::integer::{IBig, UBig};

fn main() {
    assert_eq!(UBig::from(42u64).to_string(), "42");
    assert_eq!(IBig::from(-42i64).to_string(), "-42");

    assert_eq!(UBig::ZERO.to_string(), "0");
    assert_eq!(UBig::ONE.to_string(), "1");

    // n one-bits, which is 2^n - 1 and cheaper than computing it.
    assert_eq!(UBig::ones(8), UBig::from(255u32));
    assert_eq!(UBig::ones(128).to_string(), u128::MAX.to_string());

    // No From<i32> for UBig; a negative has nowhere to go.
    assert!(UBig::try_from(IBig::from(-1)).is_err());
    assert_eq!(UBig::try_from(IBig::from(5)).unwrap(), UBig::from(5u8));
}
```

**When to use it:** `ones` whenever you want a bit mask of a given width, which
is the common way to build a modulus or a field mask. The associated constants
need no trait in scope, unlike `num_traits::Zero`.

#### `ubig!` and `ibig!`

Literal macros, including radix forms. `static_ubig!` and friends give a
`'static` reference instead of a value.

```
use dashu::integer::UBig;
use dashu::{ibig, ubig};

fn main() {
    assert_eq!(ubig!(1234).to_string(), "1234");
    assert_eq!(ibig!(-99).to_string(), "-99");

    // Two radix spellings, both accepted.
    assert_eq!(ubig!(0xff), UBig::from(255u32));
    assert_eq!(ubig!(ff base 16), UBig::from(255u32));
    assert_eq!(ubig!(1010 base 2), UBig::from(10u32));

    // Arbitrarily large, with no parsing at runtime.
    assert_eq!(
        ubig!(123456789012345678901234567890).to_string(),
        "123456789012345678901234567890"
    );
}
```

A bad literal is a build failure, not a runtime one:

```
// no-compile
use dashu::ubig;

fn main() {
    // error: proc macro panicked
    //   message: Invalid digit in the literal!
    let _ = ubig!(12xy);
}
```

**When to use it:** for every constant in your source. It moves the parse to
compile time and the mistake to the compiler. There is also `fbig!`, `dbig!`,
`rbig!` and `cbig!` for the other types, each with a `static_` variant.

#### `from_str_radix`, `from_str_with_radix_prefix` and the byte constructors

```
use dashu::integer::UBig;
use std::str::FromStr;

fn main() {
    assert_eq!(UBig::from_str("255").unwrap(), UBig::from(255u32));
    assert_eq!(UBig::from_str_radix("ff", 16).unwrap(), UBig::from(255u32));

    // Reads an 0x / 0o / 0b prefix and tells you which base it found.
    let (v, radix) = UBig::from_str_with_radix_prefix("0xff").unwrap();
    assert_eq!((v, radix), (UBig::from(255u32), 16));

    // Big-endian and little-endian byte input.
    assert_eq!(UBig::from_be_bytes(&[1, 0]), UBig::from(256u32));
    assert_eq!(UBig::from_le_bytes(&[1, 0]), UBig::from(1u32));
    assert_eq!(UBig::from(256u32).to_be_bytes().as_ref(), &[1u8, 0]);
}
```

**When to use it:** `from_str_with_radix_prefix` for anything a human typed —
config, a CLI flag, a REPL — because it accepts the prefixes people actually
write and reports the base it used. `from_str_radix` when the base is fixed by
the format.

### Integer arithmetic

#### Operators, and the `UBig` subtraction panic

As with every unsigned big integer, subtracting a larger value panics.

```
use dashu::integer::{IBig, UBig};

fn main() {
    let a = UBig::from(1u32);
    let b = UBig::from(2u32);

    // thread panicked: UBig result must not be negative
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| &a - &b));
    assert!(r.is_err());

    // thread panicked: divisor must not be 0
    let d = std::panic::catch_unwind(|| UBig::from(1u32) / UBig::ZERO);
    assert!(d.is_err());

    assert_eq!(IBig::from(1) - IBig::from(2), IBig::from(-1));
    assert_eq!(&UBig::from(10u32) * &UBig::from(10u32), UBig::from(100u32));
}
```

**When to use it:** default to `IBig` unless the domain is provably
non-negative. `UBig` earns its place as a type-level guarantee for things like
hashes, key material and bit masks, where negative is meaningless.

#### `pow`, `sqr`, `cubic`, `nth_root` and `ilog`

```
use dashu::integer::UBig;

fn main() {
    assert_eq!(UBig::from(2u32).pow(100).to_string(),
               "1267650600228229401496703205376");

    // Named shorthands rather than pow(2) and pow(3).
    assert_eq!(UBig::from(12u32).sqr(), UBig::from(144u32));
    assert_eq!(UBig::from(3u32).cubic(), UBig::from(27u32));

    // Truncated roots and integer logarithm.
    assert_eq!(UBig::from(81u32).nth_root(4), UBig::from(3u32));
    assert_eq!(UBig::from(10u32).nth_root(2), UBig::from(3u32)); // floor, not round
    assert_eq!(UBig::from(1000u32).ilog(&UBig::from(10u32)), 3);
}
```

**When to use it:** `sqr` in a tight loop instead of `&x * &x`, since it can use
a squaring algorithm rather than general multiplication. `ilog` to size a buffer
for a radix conversion. The roots truncate, so test `r.pow(n) == x` if you need
to know the root was exact.

#### Division: `div_rem` and the Euclid variants

`/` and `%` truncate toward zero, matching Rust's primitives. The `DivEuclid`
and `RemEuclid` traits give the non-negative-remainder convention.

```
use dashu::base::{DivEuclid, DivRem, RemEuclid};
use dashu::integer::{IBig, UBig};

fn main() {
    let a = IBig::from(-7);
    let b = IBig::from(2);

    assert_eq!(&a / &b, IBig::from(-3));
    assert_eq!(&a % &b, IBig::from(-1));

    // Both halves in one division.
    assert_eq!(a.clone().div_rem(b.clone()), (IBig::from(-3), IBig::from(-1)));

    // Euclidean division. The quotient is signed...
    let q: IBig = a.clone().div_euclid(b.clone());
    assert_eq!(q, IBig::from(-4));

    // ...and the remainder is a UBig, because a Euclidean remainder is never
    // negative and the type says so.
    let r: UBig = a.rem_euclid(b);
    assert_eq!(r, UBig::from(1u32));
}
```

**When to use it:** `div_rem` whenever you want both results — it is one
division instead of two. `rem_euclid` when the remainder indexes into
something, and note the return type does real work there: it is `UBig` even
though the input was `IBig`, so the "this cannot be negative" reasoning happens
once in the type system rather than at every use site. These are trait methods
that consume their operands, so `clone` where you need the value afterwards.

#### `gcd`, `gcd_ext` and `remove`

```
use dashu::base::{ExtendedGcd, Gcd};
use dashu::integer::UBig;

fn main() {
    assert_eq!(UBig::from(48u32).gcd(&UBig::from(18u32)), UBig::from(6u32));

    // Bezout coefficients come back as a plain tuple: (gcd, x, y).
    let (g, x, y) = UBig::from(48u32).gcd_ext(UBig::from(18u32));
    assert_eq!(g, UBig::from(6u32));
    assert_eq!((x.to_string(), y.to_string()), ("-1".to_string(), "3".to_string()));

    // remove divides out every copy of a factor and reports how many there
    // were: 48 = 2^4 * 3.
    let mut n = UBig::from(48u32);
    assert_eq!(n.remove(&UBig::from(2u32)), Some(4));
    assert_eq!(n, UBig::from(3u32));

    // A factor that does not divide gives Some(0) and leaves the value
    // alone -- it is a count, so zero is the honest answer.
    let mut m = UBig::from(49u32);
    assert_eq!(m.remove(&UBig::from(2u32)), Some(0));
    assert_eq!(m, UBig::from(49u32));

    // None is reserved for a factor of 1, where removal would never finish.
    let mut one = UBig::from(49u32);
    assert_eq!(one.remove(&UBig::ONE), None);
}
```

**When to use it:** `remove` is the one worth remembering — stripping powers of
two before a Miller-Rabin test, or factoring out a known small prime, is a
hand-written loop in most crates and a single call here. Read its return as a
count rather than a success flag: `Some(0)` means the factor was not present,
and `None` means you passed 1. `gcd_ext` returns a tuple where
[`num-bigint`](num-bigint.md) returns a struct; same information, less
ceremony.

#### `bit_len`, `bit`, `count_ones` and `split_bits`

Bit queries, from `dashu::base::BitTest`.

```
use dashu::base::{BitTest, PowerOfTwo};
use dashu::integer::UBig;

fn main() {
    let x = UBig::from(0b1011_0000u32);

    assert_eq!(x.bit_len(), 8);
    assert!(x.bit(4));
    assert!(!x.bit(0));
    assert_eq!(x.trailing_zeros(), Some(4));
    assert_eq!(x.count_ones(), 3);
    assert_eq!(&x << 8usize, UBig::from(45056u32));
    assert!(UBig::from(64u32).is_power_of_two());

    // split_bits cuts the value in two at a bit position: (low, high).
    assert_eq!(x.clone().split_bits(4), (UBig::ZERO, UBig::from(0b1011u32)));

    // Zero has no bits and no trailing zeros to report.
    assert_eq!(UBig::ZERO.bit_len(), 0);
    assert_eq!(UBig::ZERO.trailing_zeros(), None);

    let mut y = UBig::ZERO;
    y.set_bit(100);
    assert_eq!(y.bit_len(), 101);
}
```

**When to use it:** `split_bits` for limb-wise algorithms and for splitting a
wide value into two halves without masking by hand. `bit_len` to size buffers
and report key lengths. `set_bit` and `clear_bit` take just the index — there is
no value argument, unlike `num-bigint`'s `set_bit`.

### Modular arithmetic

#### `ConstDivisor`, `reduce` and `Reduced`

Modular arithmetic works by building a divisor once and reducing values into it.
The modulus is then part of the value's type-level story, and the operators do
the reduction for you.

```
use dashu::integer::fast_div::ConstDivisor;
use dashu::integer::UBig;

fn main() {
    let ring = ConstDivisor::new(UBig::from(1_000_000_007u32));

    let x = ring.reduce(2u32);
    assert_eq!(x.to_string(), "2 (mod 1000000007)"); // <- Display shows both

    // Arithmetic stays reduced.
    assert_eq!((ring.reduce(5u32) + ring.reduce(7u32)).residue(), UBig::from(12u32));
    assert_eq!(x.clone().pow(&UBig::from(100u32)).residue(), UBig::from(976371285u32));

    // Modular inverse, None when the gcd is not 1.
    assert_eq!(ring.reduce(3u32).inv().unwrap().residue(), UBig::from(333333336u32));
    let even = ConstDivisor::new(UBig::from(4u32));
    assert!(even.reduce(2u32).inv().is_none());
}
```

The trap is that the ring instance is the identity, not the modulus:

```
use dashu::integer::fast_div::ConstDivisor;
use dashu::integer::UBig;

fn main() {
    let a = ConstDivisor::new(UBig::from(1_000_000_007u32));
    let b = ConstDivisor::new(UBig::from(1_000_000_007u32)); // same modulus!

    // thread panicked: Modulo values from different rings
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        a.reduce(1u32) + b.reduce(1u32)
    }));
    assert!(r.is_err());
}
```

**When to use it:** any loop doing repeated arithmetic under one modulus — the
divisor is preconditioned once, so this is faster than a `%` per operation, and
`Display` showing `(mod n)` makes debugging easier. Build exactly one
`ConstDivisor` per modulus and pass it around; two with the same modulus will
panic when their values meet, which is the opposite of what the value-equality
instinct suggests.

### Floats

#### `FBig`, `DBig`, and where precision lives

`FBig<R, const BASE: Word>` puts the rounding mode in a type parameter and the
radix in a const generic. Precision is neither — it lives in the value's
context, and `with_precision` returns a `Rounded` because setting it may round.

```
use dashu::float::round::mode::Zero;
use dashu::float::{DBig, FBig};
use std::str::FromStr;

fn main() {
    // DBig is the base-10, round-to-nearest alias.
    let d = DBig::from_str("123.456").unwrap();
    assert_eq!(d.digits(), 6);
    assert_eq!(format!("{:?}", d.repr()), "123456 * 10 ^ -3");

    // Precision is a cap you set, and with_precision returns Rounded<Self>.
    let wide = DBig::ONE.with_precision(50).value();
    assert_eq!(wide.precision(), 50);

    // Parsing infers the cap from the digits written, so these differ even
    // though the values are equal.
    assert_eq!(DBig::from_str("0.1").unwrap().precision(), 2);
    assert_eq!(DBig::from_str("0.100").unwrap().precision(), 4);
    assert_eq!(DBig::from_str("0.1").unwrap(), DBig::from_str("0.100").unwrap());

    // A base-2 FBig parses C-style hex floats -- and prints in base 2.
    let f = FBig::<Zero, 2>::from_str("0x1.8p1").unwrap();
    assert_eq!(f.to_string(), "11"); // <- binary, not eleven
    assert_eq!(f.to_f64().value(), 3.0);
}
```

**When to use it:** `DBig` for anything decimal that humans read — money,
measurements, report figures. `FBig<_, 2>` when you are modelling binary
floating point or round-tripping C hex-float literals. Set precision explicitly
at the start of a calculation rather than inheriting whatever your input
literals implied, and remember a base-2 `FBig` displays in binary — call
`to_decimal()` before showing it to anyone.

#### Transcendental functions

`exp`, `ln`, the trigonometric and hyperbolic families, `sqrt`, `powf`, `hypot`,
`fma`, and the constants, all documented as correctly rounded.

```
use dashu::float::DBig;

fn main() {
    let two = DBig::from(2u8).with_precision(30).value();

    assert_eq!(two.clone().sqrt().to_string(), "1.41421356237309504880168872421");
    assert_eq!(two.clone().ln().to_string(), "0.693147180559945309417232121458");
    assert_eq!(two.clone().log10().to_string(), "0.301029995663981195213738894724");

    // An exact result prints without padding, whatever the precision cap.
    assert_eq!(two.clone().log2().to_string(), "1");

    let one = DBig::ONE.with_precision(30).value();
    assert_eq!(one.clone().exp().to_string(), "2.71828182845904523536028747135");
    assert_eq!((one.atan() * DBig::from(4u8)).to_string(), "3.14159265358979323846264338328");

    // Rounding is correct at the requested precision, so truncating a
    // higher-precision result agrees with computing it directly.
    let sqrt2_40 = DBig::from(2u8).with_precision(40).value().sqrt();
    assert!(sqrt2_40.to_string().starts_with("1.4142135623730950488016887242"));
}
```

**When to use it:** whenever `f64`'s fifteen digits are not enough and you care
that the last digit is right — reference values for a test suite, a numerical
algorithm that needs headroom, generating constants to bake into other code.
This is the capability that has no equivalent in [`num-bigint`](num-bigint.md).

### Rationals

#### `RBig` and `Relaxed`

Exact fractions, automatically reduced.

```
use dashu::integer::UBig;
use dashu::rational::RBig;
use std::str::FromStr;

fn main() {
    let third = RBig::from_str("1/3").unwrap();
    assert_eq!(third.to_string(), "1/3");

    // Reduction happens on construction and after arithmetic.
    assert_eq!(&third + RBig::from_str("1/6").unwrap(), RBig::from_str("1/2").unwrap());
    assert_eq!(RBig::from_str("2/4").unwrap().to_string(), "1/2");

    assert_eq!(third.numerator().to_string(), "1");
    assert_eq!(third.denominator().to_string(), "3");

    // The tenth harmonic number, exactly.
    let h10 = (1u32..=10)
        .map(|n| RBig::from_parts(1.into(), UBig::from(n)))
        .fold(RBig::ZERO, |a, b| a + b);
    assert_eq!(h10.to_string(), "7381/2520");
}
```

**When to use it:** `RBig` when every intermediate must stay exact — probability
weights, continued fractions, exact linear algebra on small systems, a sum of
reciprocals like the one above where decimal would accumulate error. `Relaxed`
is the variant that skips reduction, which is faster when you will reduce once at
the end; the cost is that equality and comparison are more expensive.

### Conversions

#### `Approximation`, `to_f64` and `TryFrom`

Lossy conversions return `Approximation`, which is `Exact(v)` or
`Inexact(v, Sign)` — the value plus the direction it was rounded.

```
use dashu::base::Approximation;
use dashu::integer::UBig;
use dashu::rational::RBig;
use std::str::FromStr;

fn main() {
    // Exact when the value fits a f64 exactly.
    assert!(matches!(UBig::from(4u32).to_f64(), Approximation::Exact(v) if v == 4.0));

    // Inexact, and it tells you which way.
    let big = UBig::from_str("123456789012345678901234567890").unwrap();
    match big.to_f64() {
        Approximation::Inexact(v, dir) => {
            assert_eq!(v, 1.2345678901234568e29);
            assert_eq!(format!("{dir:?}"), "Negative"); // rounded downward
        }
        Approximation::Exact(_) => panic!("should not be exact"),
    }

    // .value() when you do not care, which is at least explicit about it.
    assert_eq!(RBig::from_str("1/3").unwrap().to_f64().value(), 0.3333333333333333);

    // Narrowing to a primitive is TryFrom, and refuses rather than truncating.
    assert_eq!(u64::try_from(UBig::from(42u32)).unwrap(), 42);
    assert!(u64::try_from(big).is_err());
}
```

**When to use it:** match on `Approximation` anywhere the rounding matters, and
call `.value()` where it genuinely does not — the point is that the choice is
visible in the code. This is the clearest ergonomic win over `num-bigint`'s
`Option`-returning `to_f64`, which cannot tell you that it approximated.

### Serialising

#### `serde` support

With the `serde` feature every type round-trips as its textual form.

```
use dashu::float::DBig;
use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;
use std::str::FromStr;

fn main() {
    assert_eq!(serde_json::to_string(&UBig::from(255u32)).unwrap(), "\"255\"");
    assert_eq!(serde_json::to_string(&IBig::from(-255)).unwrap(), "\"-255\"");
    assert_eq!(serde_json::to_string(&DBig::from_str("0.1").unwrap()).unwrap(), "\"0.1\"");

    let r = RBig::from_str("1/3").unwrap();
    let j = serde_json::to_string(&r).unwrap();
    assert_eq!(serde_json::from_str::<RBig>(&j).unwrap(), r);

    assert_eq!(
        serde_json::from_str::<IBig>("\"-255\"").unwrap(),
        IBig::from(-255)
    );
}
```

**When to use it:** by default, and especially for a public API. A decimal
string is self-describing, stable across versions of this crate, and readable by
every other language — which is precisely what
[`num-bigint`](num-bigint.md)'s limb-array representation is not. See
[`serde`](serde.md) for the derive side.

### Running without std

#### `default-features = false`

The crate advertises full `no_std` support; `std` is simply a default feature.

```
use dashu::integer::UBig;
use dashu::ubig;

fn main() {
    // Cargo.toml for a no_std target:
    //
    //   dashu = { version = "0.6.2", default-features = false }
    //
    // The default features are `std` and `num-order`, so turning defaults off
    // drops both. An allocator is still required: the digits live on the heap.
    const MASK: UBig = UBig::ZERO;
    assert_eq!(MASK, UBig::ZERO);

    let v = ubig!(0xdeadbeef);
    assert_eq!(v.to_string(), "3735928559");
    assert_eq!(UBig::ones(32) - &v, ubig!(0x21524110));
}
```

**When to use it:** embedded work with an allocator, where this crate's `no_std`
story is more complete than most — the float and rational types work too, not
just the integers. If you have no heap at all, no crate in this family will
help; you want a fixed-width type instead.
