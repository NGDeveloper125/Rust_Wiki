---
title: "malachite"
version: "0.14.0"
publisher: "Mikhail Hogrefe (mhogrefe)"
publisher_url: "https://crates.io/users/mhogrefe"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-11-22"
summary: "The fastest pure-Rust arbitrary-precision arithmetic library, with algorithms partly derived from GMP, FLINT and MPFR. Trait-heavy and explicit: you choose the rounding mode and every operation tells you which way it rounded. LGPL-3.0-only."
domain: "Numerics & math"
categories: ["mathematics"]
repository: "https://github.com/mhogrefe/malachite"
docs: "https://docs.rs/malachite/0.14.0/malachite/"
---

## Overview

`malachite` is an arbitrary-precision arithmetic library covering `Natural`,
`Integer`, `Rational` and `Float`, plus polynomials, vectors and Gaussian
integers. Its own description is "efficient algorithms partially derived from
GMP, FLINT, and MPFR", and the README states plainly: "Parts of Malachite are
derived from GMP, FLINT, and MPFR."

**Settle the licence before anything else.** The crate is **LGPL-3.0-only** —
not the `MIT OR Apache-2.0` that most of the Rust ecosystem uses, and not a
choice the author made arbitrarily, since the libraries the algorithms come from
carry the same family of licence. For an internal tool, a server binary you do
not distribute, or anything already GPL-compatible, this is a non-event. For a
statically-linked proprietary binary you ship to customers it is a question for
whoever handles licensing at your organisation, and it should be asked before
the dependency is in your tree rather than after. Nothing else on this page
matters if the answer is no.

If the answer is yes, what you get is speed. Measured against
[`num-bigint`](num-bigint.md) 0.5.1 and [`dashu`](dashu.md) 0.6.2 on the same
machine, two operands of 10,000 decimal digits each (~33,220 bits):

- multiply — **150 µs**, against num-bigint's 254 µs and dashu's 212 µs
- parse from decimal — **174 µs**, against 277 µs and 275 µs
- `to_string` of the 20,000-digit product — **1,101 µs**, against 2,433 µs and 2,140 µs
- divide 20,000 digits by 10,000 — 452 µs, against 547 µs and dashu's 407 µs

A clear win, but not a dramatic one — under 2× on most operations. The dramatic
part is what happens when the numbers grow. At 100,000 digits a multiply takes
**1.67 ms**, against num-bigint's 8.99 ms and dashu's 14.3 ms. Scaled from
10,000 to 100,000 digits — ten times the input — malachite's multiply time grew
about 11×, num-bigint's about 35× and dashu's about 68×. That is the difference
between sub-quadratic algorithms and Karatsuba, and it means **the case for this
crate gets stronger the bigger your numbers are**. Below a few thousand digits
the gap is small enough that licence and ergonomics should decide.

The API is trait-heavy and deliberately explicit, which is the other thing to
know before adopting it. Almost nothing is an inherent method: `ZERO`, `ONE` and
`TWO` are trait constants, and `div_round`, `floor_sqrt`, `mod_pow`, `gcd` and
the rest each come from their own trait under `malachite::base::num`. You will
spend your first hour adding imports. In exchange you get an API where ambiguity
is pushed into the signature rather than into the documentation:

- Rounding is a parameter, not a convention. `RoundingMode` has six variants and
  distinguishes `Down`/`Up` (toward and away from zero) from `Floor`/`Ceiling`
  (toward negative and positive infinity), which for negative operands are four
  different answers.
- **Every rounding operation returns the direction it rounded**, as a
  `std::cmp::Ordering` alongside the value. So does converting to `f64`.
- Where another library gives you one truncating `sqrt`, this gives you
  `floor_sqrt`, `ceiling_sqrt` and `checked_sqrt` — the last returning `None`
  unless the root is exact.
- `RoundingMode::Exact` turns an inexact operation into a panic, which makes
  "this division must divide evenly" an assertion rather than a comment.

MSRV is **1.90.0**, the newest of the three crates here and worth checking
against your toolchain policy. Default features are `floats`,
`naturals_and_integers`, `rationals` and `std`; serde support is the
`enable_serde` feature.

**Choosing between the three.** Take [`num-bigint`](num-bigint.md) for the
default case — permissive licence, old MSRV, huge installed base, and fast
enough below a few thousand digits. Take [`dashu`](dashu.md) when you want
decimal floats or exact rationals with a permissive licence and full `no_std`.
Take `malachite` when the arithmetic is genuinely your bottleneck, the numbers
are large, and LGPL-3.0 is acceptable where you are shipping.

## When to use it

### Use case: Integer arithmetic where the operands keep getting bigger

A cryptographic experiment, a number-theory search, a computer-algebra step — the
sort of workload where the operand size is a parameter you keep raising. This is
where the algorithmic difference stops being a benchmark curiosity.

```
use malachite::base::num::arithmetic::traits::Factorial;
use malachite::Natural;
use std::str::FromStr;

fn main() {
    // 100! -- the same 158-digit value num-bigint produces, computed by a
    // dedicated factorial implementation rather than a product loop.
    let f = Natural::factorial(100);
    assert_eq!(f.to_string().len(), 158);
    assert!(f.to_string().starts_with("933262154439441526816992388562667"));

    // Ten thousand digits, squared. The point is that this stays quick as
    // the digit count climbs.
    let big = Natural::from_str(&"9".repeat(10_000)).unwrap();
    let squared = &big * &big;
    assert_eq!(squared.to_string().len(), 20_000);

    // Internals are addressable when you need them: 64-bit limbs,
    // least-significant first.
    let n = Natural::from_str("123456789012345678901234567890").unwrap();
    assert_eq!(n.limb_count(), 2);
    assert_eq!(n.to_limbs_asc(), vec![14083847773837265618, 6692605942]);
}
```

**Why it fits:** the operations you are repeating — multiplication, division,
base conversion — are each the fastest available in pure Rust, and the advantage
widens rather than narrows as operands grow. `to_string` being twice as fast
matters more than it sounds, because base-10 conversion is often the most
expensive step in a pipeline that reports its results.

### Use case: Rounding you can control and audit

Integer division has to discard information, and most libraries pick a
convention and document it. Here you pass the convention in and get told what
was discarded, which turns a class of silent off-by-one bugs into something the
type system makes you look at.

```
use malachite::base::num::arithmetic::traits::DivRound;
use malachite::base::rounding_modes::RoundingMode;
use malachite::{Integer, Natural};
use std::cmp::Ordering;

fn main() {
    let n = Natural::from(7u32);
    let d = Natural::from(2u32);

    // The quotient, plus how it was rounded.
    assert_eq!((&n).div_round(&d, RoundingMode::Floor), (Natural::from(3u32), Ordering::Less));
    assert_eq!((&n).div_round(&d, RoundingMode::Ceiling), (Natural::from(4u32), Ordering::Greater));
    assert_eq!((&n).div_round(&d, RoundingMode::Nearest), (Natural::from(4u32), Ordering::Greater));

    // For a negative operand, Down and Floor are NOT the same thing, and
    // neither are Up and Ceiling. Four distinct answers:
    let a = Integer::from(-7);
    let b = Integer::from(2);
    assert_eq!((&a).div_round(&b, RoundingMode::Down).0, Integer::from(-3));    // toward zero
    assert_eq!((&a).div_round(&b, RoundingMode::Up).0, Integer::from(-4));      // away from zero
    assert_eq!((&a).div_round(&b, RoundingMode::Floor).0, Integer::from(-4));   // toward -inf
    assert_eq!((&a).div_round(&b, RoundingMode::Ceiling).0, Integer::from(-3)); // toward +inf
}
```

**Why it fits:** the `Ordering` is the part that earns its keep. Code that
divides and then compares, or divides and then accumulates an error term, needs
to know whether the quotient it holds is above or below the true value — and
here it is handed that for free rather than recomputing `q * d == n`.

### Use case: An exact answer, or none at all

Sometimes an approximation is not an acceptable result and you would rather find
out immediately. Malachite has a consistent vocabulary for this: `checked_`
prefixes return `Option`, `div_exact` assumes exactness, and
`RoundingMode::Exact` asserts it.

```
use malachite::base::num::arithmetic::traits::{
    CeilingSqrt, CheckedSqrt, DivExact, DivRound, FloorSqrt,
};
use malachite::base::num::factorization::traits::IsSquare;
use malachite::base::rounding_modes::RoundingMode;
use malachite::Natural;
use std::cmp::Ordering;

fn main() {
    let ten = Natural::from(10u32);

    // Three questions, three methods -- no single ambiguous `sqrt`.
    assert_eq!((&ten).floor_sqrt(), Natural::from(3u32));
    assert_eq!((&ten).ceiling_sqrt(), Natural::from(4u32));
    assert_eq!((&ten).checked_sqrt(), None); // <- 10 is not a perfect square
    assert_eq!(Natural::from(16u32).checked_sqrt(), Some(Natural::from(4u32)));
    assert!(Natural::from(16u32).is_square());

    // div_exact when you know it divides.
    assert_eq!(Natural::from(6u32).div_exact(Natural::from(2u32)), Natural::from(3u32));

    // RoundingMode::Exact makes "must be exact" an assertion.
    assert_eq!(
        Natural::from(6u32).div_round(Natural::from(2u32), RoundingMode::Exact),
        (Natural::from(3u32), Ordering::Equal)
    );

    // thread panicked: Division is not exact
    let r = std::panic::catch_unwind(|| {
        Natural::from(7u32).div_round(Natural::from(2u32), RoundingMode::Exact)
    });
    assert!(r.is_err());
}
```

**Why it fits:** "is this a perfect square" and "take the integer square root"
are different questions, and most libraries make you ask the second and then
verify. Having `checked_sqrt` return `Option` collapses that into one call that
cannot be got wrong, and the same `floor_`/`ceiling_`/`checked_` triple appears
for roots and logarithms, so the vocabulary transfers.

## API map

Almost everything below is a trait method. The trait paths are given with each
group, and the first entry explains the layout.

### Finding the traits

#### The `base` module and the trait paths

`malachite::base` re-exports `malachite-base`, where the traits live. These are
the paths you will actually need.

```
// The four trait modules that matter, with an example from each:
use malachite::base::num::arithmetic::traits::{DivRound, Gcd, ModPow, Pow};
use malachite::base::num::basic::traits::{Infinity, NaN, One, Two, Zero};
use malachite::base::num::conversion::traits::{FromStringBase, RoundingFrom, ToStringBase};
use malachite::base::num::factorization::traits::IsSquare;
use malachite::base::rounding_modes::RoundingMode;
use malachite::Natural;

fn main() {
    // num::basic::traits       -> the constants (ZERO, ONE, TWO, NAN, INFINITY)
    // num::arithmetic::traits  -> arithmetic (DivRound, ModPow, Gcd, Pow, ...)
    // num::conversion::traits  -> conversions (RoundingFrom, ToStringBase, ...)
    // num::factorization::traits -> IsSquare, IsPower, primality
    // base::rounding_modes     -> RoundingMode
    assert_eq!(Natural::ONE + Natural::TWO, Natural::from(3u32));
    assert!(Natural::from(16u32).is_square());
    assert_eq!(Natural::from(255u32).to_string_base(16), "ff");
    let _ = (RoundingMode::Nearest, f64::NAN, f64::INFINITY);
}
```

**When to use it:** import the traits you need per module rather than hunting
one at a time — a compiler error will name the trait but you still have to find
its module, and `is_square` living under `factorization` rather than
`arithmetic` is the sort of thing that costs a minute each time. For a module
doing heavy arithmetic, glob-import `arithmetic::traits`.

### Creating one

#### From primitives, the trait constants, and `const_from`

```
use malachite::base::num::basic::traits::{One, Two, Zero};
use malachite::{Integer, Natural, Rational};
use std::str::FromStr;

fn main() {
    assert_eq!(Natural::from(42u64).to_string(), "42");
    assert_eq!(Integer::from(-42i64).to_string(), "-42");
    assert_eq!(Rational::from(3).to_string(), "3");

    // Constants come from traits, so the trait must be in scope.
    assert_eq!(Natural::ZERO.to_string(), "0");
    assert_eq!(Natural::ONE.to_string(), "1");
    assert_eq!(Natural::TWO.to_string(), "2");

    // A const constructor for small values.
    const SEVEN: Natural = Natural::const_from(7);
    assert_eq!(SEVEN.to_string(), "7");

    assert_eq!(Natural::from_str("255").unwrap(), Natural::from(255u32));

    // Footprints, for comparison with the other crates.
    assert_eq!(std::mem::size_of::<Natural>(), 24);
    assert_eq!(std::mem::size_of::<Integer>(), 32);
    assert_eq!(std::mem::size_of::<Rational>(), 56);
}
```

**When to use it:** `const_from` for a compile-time constant that fits a limb.
Remember that `Natural::ZERO` will not compile without
`malachite::base::num::basic::traits::Zero` imported — the single most common
first-contact error with this crate.

#### `from_string_base` and `to_string_base`

Radix conversion both ways, plus the usual format traits.

```
use malachite::base::num::conversion::traits::{FromStringBase, ToStringBase};
use malachite::Natural;

fn main() {
    let n = Natural::from(255u32);

    assert_eq!(n.to_string_base(16), "ff");
    assert_eq!(n.to_string_base(2), "11111111");
    assert_eq!(n.to_string_base(36), "73");

    assert_eq!(Natural::from_string_base(16, "ff"), Some(Natural::from(255u32)));
    assert_eq!(Natural::from_string_base(16, "zz"), None); // <- Option, not panic

    assert_eq!(format!("{n:x}"), "ff");
    assert_eq!(format!("{n:b}"), "11111111");
    assert_eq!(format!("{n:o}"), "377");
}
```

**When to use it:** `from_string_base` returns `None` for a bad digit rather
than panicking, which makes it safe on untrusted input — unlike
[`num-bigint`](num-bigint.md)'s `parse_bytes`, which asserts its radix
separately. Use the format traits for the common bases and `to_string_base` when
the base is data.

### Arithmetic

#### Operators, the `Natural` subtraction panic, and `checked_sub`

`Natural` is unsigned, so a subtraction that would go negative panics — with a
message that includes both operands, which is more than most crates give you.

```
use malachite::base::num::arithmetic::traits::CheckedSub;
use malachite::base::num::basic::traits::Zero;
use malachite::{Integer, Natural};

fn main() {
    let a = Natural::from(1u32);
    let b = Natural::from(2u32);

    // thread panicked: Cannot subtract a number from a smaller number.
    //                  self: 1, other: 2
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| &a - &b));
    assert!(r.is_err());

    // The checked form, which works on references.
    assert_eq!((&a).checked_sub(&b), None);
    assert_eq!((&b).checked_sub(&a), Some(Natural::from(1u32)));

    assert_eq!(Integer::from(1) - Integer::from(2), Integer::from(-1));

    // thread panicked: division by zero
    let d = std::panic::catch_unwind(|| Natural::from(1u32) / Natural::ZERO);
    assert!(d.is_err());
}
```

**When to use it:** `checked_sub` on any `Natural` whose operands you did not
produce yourself. The panic message naming both values is a genuine debugging
advantage over the alternatives, which tell you only that it happened.

#### `pow`, `factorial` and `div_mod`

```
use malachite::base::num::arithmetic::traits::{DivMod, Factorial, Pow};
use malachite::Natural;

fn main() {
    assert_eq!(
        Natural::from(2u32).pow(100).to_string(),
        "1267650600228229401496703205376"
    );

    // A dedicated factorial rather than a fold over a range.
    assert_eq!(Natural::factorial(20).to_string(), "2432902008176640000");
    assert_eq!(Natural::factorial(100).to_string().len(), 158);

    // Quotient and remainder in one division.
    assert_eq!(
        Natural::from(7u32).div_mod(Natural::from(2u32)),
        (Natural::from(3u32), Natural::from(1u32))
    );
}
```

**When to use it:** `div_mod` instead of separate `/` and `%`, which does the
work twice. `factorial` and its neighbours (`double_factorial`,
`multifactorial`, `subfactorial`) exist because these show up constantly in
combinatorics and a naive product is slower than the algorithm the crate uses.

#### `div_round` and `RoundingMode`

The six rounding modes, and the `Ordering` that comes back with every result.

```
use malachite::base::num::arithmetic::traits::DivRound;
use malachite::base::rounding_modes::RoundingMode;
use malachite::Natural;
use std::cmp::Ordering;

fn main() {
    let n = Natural::from(7u32);
    let d = Natural::from(2u32);

    // Down, Up, Floor, Ceiling, Nearest, Exact.
    assert_eq!((&n).div_round(&d, RoundingMode::Down), (Natural::from(3u32), Ordering::Less));
    assert_eq!((&n).div_round(&d, RoundingMode::Up), (Natural::from(4u32), Ordering::Greater));
    assert_eq!((&n).div_round(&d, RoundingMode::Nearest), (Natural::from(4u32), Ordering::Greater));

    // Equal means the result was exact and nothing was discarded.
    assert_eq!(
        Natural::from(6u32).div_round(Natural::from(2u32), RoundingMode::Nearest),
        (Natural::from(3u32), Ordering::Equal)
    );
}
```

**When to use it:** whenever the rounding direction is part of the problem —
pagination, bucket sizing, fixed-point scaling, anything where "round up" and
"round away from zero" are not the same requirement. `Exact` is the one to reach
for in a place where you believe the division is exact: it converts a wrong
belief into a panic at the point of the mistake instead of a wrong number
further downstream.

### Roots and logarithms

#### `floor_sqrt`, `ceiling_sqrt`, `checked_sqrt` and the root family

The same triple appears for square roots, nth roots and base-2 logarithms.

```
use malachite::base::num::arithmetic::traits::{
    CeilingSqrt, CheckedSqrt, FloorRoot, FloorSqrt,
};
use malachite::Natural;

fn main() {
    assert_eq!(Natural::from(10u32).floor_sqrt(), Natural::from(3u32));
    assert_eq!(Natural::from(10u32).ceiling_sqrt(), Natural::from(4u32));
    assert_eq!(Natural::from(10u32).checked_sqrt(), None);
    assert_eq!(Natural::from(16u32).checked_sqrt(), Some(Natural::from(4u32)));

    assert_eq!(Natural::from(81u32).floor_root(4), Natural::from(3u32));
    assert_eq!(Natural::from(80u32).floor_root(4), Natural::from(2u32));
}
```

**When to use it:** pick the one that matches your question instead of
truncating and adjusting. `CeilingRoot`, `CheckedRoot`, `FloorLogBase2`,
`CeilingLogBase2` and `CheckedLogBase2` follow the identical pattern, so once
you know the convention you can guess the name you want.

### Modular arithmetic and number theory

#### `mod_pow`, `mod_inverse`, `gcd` and `extended_gcd`

```
use malachite::base::num::arithmetic::traits::{ExtendedGcd, Gcd, Lcm, ModInverse, ModPow};
use malachite::{Integer, Natural};

fn main() {
    let p = Natural::from(1_000_000_007u32);

    assert_eq!(
        Natural::from(2u32).mod_pow(Natural::from(100u32), &p),
        Natural::from(976371285u32)
    );
    assert_eq!(
        Natural::from(3u32).mod_inverse(&p),
        Some(Natural::from(333333336u32))
    );

    assert_eq!(Natural::from(48u32).gcd(Natural::from(18u32)), Natural::from(6u32));
    assert_eq!(Natural::from(48u32).lcm(Natural::from(18u32)), Natural::from(144u32));

    // Bezout coefficients as a tuple: (gcd, x, y).
    let (g, x, y) = Natural::from(48u32).extended_gcd(Natural::from(18u32));
    assert_eq!(g, Natural::from(6u32));
    assert_eq!((x, y), (Integer::from(-1), Integer::from(3)));
}
```

**When to use it:** `mod_pow` for exponentiation under a modulus, always, rather
than `pow` then `%`. The crate also carries `JacobiSymbol`, `LegendreSymbol`,
`KroneckerSymbol`, `IsPower` and a `ModSqrt` family, which is a deeper
number-theory toolkit than either alternative offers — if you are writing
number theory rather than just using big integers, that breadth is a reason to
be here.

### Converting out

#### `RoundingFrom` for floats

Converting a big value to `f64` loses information, so this is a rounding
operation like any other: you pass the mode and get the direction back.

```
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;
use malachite::Natural;
use std::cmp::Ordering;
use std::str::FromStr;

fn main() {
    let big = Natural::from_str("123456789012345678901234567890").unwrap();

    // Nearest rounded down here, and says so.
    assert_eq!(
        f64::rounding_from(&big, RoundingMode::Nearest),
        (1.2345678901234568e29, Ordering::Less)
    );

    // Ceiling gives a different f64 -- the next one up.
    let (up, ord) = f64::rounding_from(&big, RoundingMode::Ceiling);
    assert_eq!(ord, Ordering::Greater);
    assert!(up > 1.2345678901234568e29);

    // Exact values report Equal.
    assert_eq!(
        f64::rounding_from(&Natural::from(4u32), RoundingMode::Nearest),
        (4.0, Ordering::Equal)
    );
}
```

**When to use it:** any time a big value becomes a float for display,
statistics or a graph. Being able to demand `Ceiling` matters for interval
arithmetic and for anything computing a bound that must not be understated —
neither [`num-bigint`](num-bigint.md) nor [`dashu`](dashu.md) lets you choose
the direction, though dashu at least reports it.

### Rationals

#### `Rational`

Exact fractions, reduced on construction.

```
use malachite::base::num::arithmetic::traits::Pow;
use malachite::base::num::conversion::traits::RoundingFrom;
use malachite::base::rounding_modes::RoundingMode;
use malachite::{Natural, Rational};
use std::cmp::Ordering;
use std::str::FromStr;

fn main() {
    let q = Rational::from_str("6/4").unwrap();
    assert_eq!(q.to_string(), "3/2"); // <- reduced immediately

    assert_eq!(q.numerator_ref().to_string(), "3");
    assert_eq!(q.denominator_ref().to_string(), "2");

    assert_eq!(
        Rational::from_naturals(Natural::from(2u32), Natural::from(4u32)).to_string(),
        "1/2"
    );

    assert_eq!(
        &Rational::from_str("1/3").unwrap() + Rational::from_str("1/6").unwrap(),
        Rational::from_str("1/2").unwrap()
    );
    assert_eq!(Rational::from_str("2/3").unwrap().pow(3i64).to_string(), "8/27");

    assert_eq!(f64::rounding_from(&q, RoundingMode::Nearest), (1.5, Ordering::Equal));
}
```

**When to use it:** exact arithmetic where no intermediate may be approximated —
continued fractions, exact linear algebra, probability weights. `numerator_ref`
and `denominator_ref` borrow rather than clone, which matters inside a loop.

### Floats

#### `Float`

An arbitrary-precision binary float in the MPFR tradition, with precision given
per value and the usual special values.

```
use malachite::base::num::basic::traits::{Infinity, NaN, One, Two, Zero};
use malachite::{Float, Natural};
use std::cmp::Ordering;

fn main() {
    // Precision in bits is explicit, and construction reports its rounding.
    let (f, ord) = Float::from_natural_prec(Natural::from(2u32), 100);
    assert_eq!(ord, Ordering::Equal);
    assert!(f.to_string().starts_with("2.000"));

    assert_eq!(Float::ONE.to_string(), "1.0");
    assert_eq!(Float::TWO.to_string(), "2.0");
    assert_eq!(Float::ZERO.to_string(), "0.0");
    assert_eq!((Float::TWO + Float::TWO).to_string(), "4.0");

    // Special values are trait constants too.
    assert_eq!(Float::NAN.to_string(), "NaN");
    assert_eq!(Float::INFINITY.to_string(), "Infinity");

    // Converting an f64 in shows you what the f64 really held.
    assert_eq!(Float::from(0.1f64).to_string(), "0.10000000000000001");

    assert_eq!(std::mem::size_of::<Float>(), 40);
}
```

**When to use it:** high-precision binary floating point — numerical analysis,
reference values, anything where you would otherwise reach for MPFR. Note it is
*binary*: for decimal arithmetic where a tenth must be exact, this is the wrong
tool and [`dashu`](dashu.md)'s `DBig` is the right one. The `Float::from(0.1f64)`
line above is the clearest illustration of why.

### Serialising

#### `enable_serde`, and the hexadecimal representation

With the `enable_serde` feature the numeric types round-trip. The textual form
is **hexadecimal**, which is a third distinct convention among the three crates
on this topic.

```
use malachite::{Integer, Natural, Rational};
use std::str::FromStr;

fn main() {
    assert_eq!(serde_json::to_string(&Natural::from(255u32)).unwrap(), "\"0xff\"");
    assert_eq!(serde_json::to_string(&Integer::from(-255)).unwrap(), "\"-0xff\"");

    let big = Natural::from_str("123456789012345678901234567890").unwrap();
    assert_eq!(
        serde_json::to_string(&big).unwrap(),
        "\"0x18ee90ff6c373e0ee4e3f0ad2\""
    );

    // Rational is a struct of sign, numerator and denominator.
    assert_eq!(
        serde_json::to_string(&Rational::from_str("1/3").unwrap()).unwrap(),
        "{\"s\":true,\"n\":\"0x1\",\"d\":\"0x3\"}"
    );

    assert_eq!(serde_json::from_str::<Natural>("\"0xff\"").unwrap(), Natural::from(255u32));
}
```

**When to use it:** fine when both ends are Rust. If another language or a
public API consumes the JSON, note that none of the three crates here agrees on
a representation — this one writes `"0xff"`, [`dashu`](dashu.md) writes `"255"`,
and [`num-bigint`](num-bigint.md) writes the limb array `[255]`. For an
interoperable format, serialise a decimal string explicitly with
`#[serde(with = ...)]`; see [`serde`](serde.md).

### Running without std

#### `default-features = false`

```
use malachite::Natural;

fn main() {
    // Cargo.toml dropping std, and the sub-crates you do not need:
    //
    //   malachite = { version = "0.14.0", default-features = false,
    //                 features = ["naturals_and_integers"] }
    //
    // The default features are `std`, `naturals_and_integers`, `rationals`
    // and `floats`, so turning defaults off and naming only the integers
    // also drops Rational and Float from the build.
    //
    // There is also `32_bit_limbs`, which switches the limb width -- use it
    // on a 32-bit target where the default 64-bit limb is emulated.
    let n = Natural::from(2u32);
    assert_eq!((&n * &n).to_string(), "4");
}
```

**When to use it:** the feature split is worth knowing even on hosted targets,
because `floats` and `rationals` pull in whole sub-crates you may not want
compiling. An allocator is still required. Be aware the crate's own
documentation says little about `no_std` beyond the feature existing, so verify
your target builds before committing to it.
