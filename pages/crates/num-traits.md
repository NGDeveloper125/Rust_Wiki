---
title: "num-traits"
version: "0.2.19"
publisher: "Josh Stone (cuviper)"
publisher_url: "https://crates.io/users/cuviper"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-19"
summary: "The numeric traits the standard library leaves out — `Zero`, `One`, `Num`, `Float`, `PrimInt`, and the checked, wrapping and saturating operations — so a function can be generic over number types."
domain: "Numerics & math"
categories: ["algorithms", "science", "no-std"]
repository: "https://github.com/rust-num/num-traits"
docs: "https://docs.rs/num-traits/0.2.19/num_traits/"
---

## Overview

The standard library gives you `Add`, `Mul` and the other operator traits, and
then stops. There is no `Zero`, no `One`, no trait for "is a number", and no way
to be generic over `f32` and `f64` together. So this does not compile:

```
use std::ops::Add;

fn total<T: Add<Output = T> + Copy>(values: &[T]) -> T {
    // There is no way to produce the starting value: std has no Zero.
    // values.iter().fold(T::???, |acc, &v| acc + v)
    values.iter().copied().reduce(|a, b| a + b).unwrap()
}

// Which means an empty slice has to panic, or the signature has to change.
assert_eq!(total(&[1, 2, 3]), 6);
```

`num-traits` supplies the missing pieces, and the same function becomes
straightforward:

```
use num_traits::Num;

fn total<T: Num + Copy>(values: &[T]) -> T {
    values.iter().fold(T::zero(), |acc, &v| acc + v)
}

// One implementation, every numeric type.
assert_eq!(total(&[1i32, 2, 3]), 6);
assert_eq!(total(&[1.5f64, 2.5]), 4.0);
assert_eq!(total::<i32>(&[]), 0); // and an empty slice is no longer a problem
```

That is the whole purpose: traits abstract enough that one implementation serves
`i32`, `u64`, `f32`, `f64` and the arbitrary-precision types too. It is the
foundation of the `num` family — `num-bigint`, `num-rational`, `num-complex` all
implement these — and the reason [`nalgebra`](nalgebra.md), `ndarray` and
[`rand`](rand.md) can be generic over scalars at all.

**One thing to decide honestly before reaching for it: generic numeric code in
Rust is laborious, and often not worth it.** Bounds accumulate — a function
needing division, comparison and a float constant can end up with four or five —
and the error messages when a bound is missing are poor. If your function will
only ever be called with `f64`, write it for `f64`. If it needs two or three
concrete types, a macro over those types is frequently clearer than a generic
signature. The crate earns its place when you are *publishing* a numeric API and
cannot know what your callers will use.

**A detail that confuses people on first contact.** Many of these traits
duplicate methods the primitives already have inherently, and the inherent ones
win name resolution — plus the trait versions take `&self` and `&Self`:

```
use num_traits::CheckedAdd;

// On a concrete primitive, `250u8.checked_add(10)` is std's inherent method.
// The trait's version needs explicit call syntax, and references.
assert_eq!(CheckedAdd::checked_add(&250u8, &10), None);

// In generic code there is no inherent method to shadow it, so it just works.
fn safe_add<T: CheckedAdd>(a: T, b: T) -> Option<T> {
    a.checked_add(&b)
}
assert_eq!(safe_add(250u8, 10), None);
assert_eq!(safe_add(1u8, 2), Some(3));
```

So the traits are for generic code, not for calling on concrete types — where
the inherent method is shorter and does the same thing.

The dependency is as light as it gets: nothing required, MSRV 1.60, and `no_std`
by turning off the default `std` feature. **In `no_std` you will also want the
`libm` feature**, because `Float`'s transcendental methods — `sqrt`, `sin`, `ln`
— are implemented by the platform's libm, and without it those methods simply do
not exist.

The last release was 0.2.19 in May 2024, which the backlog check flags as a gap.
It is not abandonment: the repository is unarchived and still receiving commits,
and the crate is a stable set of trait definitions that has deliberately not
changed. There is little to release when the job is done.

## When to use it

### Use case: Publishing a numeric function your callers parameterise

The case that justifies the trait bounds.

```
use num_traits::Float;

/// Mean of a slice, for any floating-point type.
fn mean<T: Float>(values: &[T]) -> Option<T> {
    if values.is_empty() {
        return None;
    }
    // T::from converts, and returns Option because the conversion may not fit.
    let count = T::from(values.len())?;
    let sum = values.iter().fold(T::zero(), |acc, &v| acc + v);
    Some(sum / count)
}

assert_eq!(mean(&[1.0f64, 2.0, 3.0]), Some(2.0));
assert_eq!(mean(&[1.0f32, 2.0, 3.0]), Some(2.0f32));
assert_eq!(mean::<f64>(&[]), None);
```

**Why it fits:** `f32` and `f64` share no trait in `std` beyond the operators, so
without `Float` this is two functions or a macro. `Float` also brings the
constants and predicates that numeric code needs — `nan()`, `infinity()`,
`epsilon()`, `is_finite()` — which is usually the real reason to want it rather
than the arithmetic. Note `T::from(...)?`: converting a `usize` to a float is
fallible in the general case, and the trait is honest about it.

### Use case: Arithmetic that must not silently overflow

The checked, wrapping and saturating families, used generically.

```
use num_traits::{CheckedMul, Saturating, WrappingAdd};

// Fail rather than panic or wrap.
fn product<T: CheckedMul + num_traits::One + Copy>(values: &[T]) -> Option<T> {
    values.iter().try_fold(T::one(), |acc, &v| acc.checked_mul(&v))
}

assert_eq!(product(&[2u8, 3, 4]), Some(24));
assert_eq!(product(&[200u8, 2]), None); // would overflow u8

// The other two policies, on concrete types via explicit syntax.
assert_eq!(WrappingAdd::wrapping_add(&250u8, &10), 4);
assert_eq!(Saturating::saturating_add(250u8, 10), 255);
```

**Why it fits:** a release build wraps on overflow where a debug build panics, so
a numeric library that cares has to pick a policy explicitly — and these traits
let it do so generically. `try_fold` with `checked_mul` is the idiomatic shape:
the first overflow short-circuits to `None` rather than producing a wrong answer
that propagates. For concrete types the inherent methods are better; this is for
when `T` is a parameter.

### Use case: Converting between numeric types without hiding the failure

`as` is a cast that truncates silently; these are not.

```
use num_traits::{AsPrimitive, FromPrimitive, NumCast, ToPrimitive};

let big: i64 = 300;

// Checked: None when the value does not fit.
assert_eq!(big.to_u8(), None);
assert_eq!(big.to_i32(), Some(300));
assert_eq!(<u8 as NumCast>::from(300i64), None);

// Lossy conversions are reported too.
assert_eq!(i32::from_f64(3.9), Some(3)); // truncates toward zero
assert_eq!(i32::from_f64(f64::NAN), None);

// AsPrimitive is the deliberate `as` equivalent: 300u8 wraps to 44.
let truncated: u8 = big.as_();
assert_eq!(truncated, 44);
```

**Why it fits:** `300i64 as u8` compiles, produces 44, and says nothing. In a
generic function you often cannot even write `as`, because it is not defined for
a type parameter — so `ToPrimitive` and `NumCast` are both the safer option and
the only option. Keep `AsPrimitive` for when truncation is genuinely what you
want; its `as_()` is unchecked by design, and naming it makes that visible in a
way `as` does not.

## API map

Dozens of traits, organised by what they abstract. The entries below cover the
ones worth knowing by name; the rest follow the same patterns.

### The core abstractions

#### `Zero` and `One`

The additive and multiplicative identities.

```
use num_traits::{One, Zero};

assert_eq!(i32::zero(), 0);
assert_eq!(f64::one(), 1.0);

// is_zero is a method, not a comparison, so it works without PartialEq.
assert!(0u8.is_zero());
assert!(!1u8.is_zero());

// ConstZero and ConstOne give the values as associated constants.
const START: i32 = <i32 as num_traits::ConstZero>::ZERO;
assert_eq!(START, 0);
```

**When to use it:** whenever a generic function needs a starting value — a fold,
an accumulator, an empty sum or product. They are the two most-used traits in
the crate and the reason it exists. Prefer `is_zero()` over `== T::zero()`: it
avoids a `PartialEq` bound and is cheaper for big-integer types.
`ConstZero`/`ConstOne` exist for `const` contexts where a method call will not
do.

#### `Num`, `NumAssign` and the operator bundles

Convenience bounds that bundle the operators together.

```
use num_traits::{Num, NumAssign};

// Num = PartialEq + Zero + One + the four arithmetic operators.
fn sum_of_squares<T: Num + Copy>(values: &[T]) -> T {
    values.iter().fold(T::zero(), |acc, &v| acc + v * v)
}
assert_eq!(sum_of_squares(&[1i32, 2, 3]), 14);

// NumAssign adds +=, -=, *=, /=, %=.
fn accumulate<T: NumAssign + Copy>(values: &[T]) -> T {
    let mut total = T::zero();
    for &v in values {
        total += v;
    }
    total
}
assert_eq!(accumulate(&[1.0f64, 2.0]), 3.0);

// Num also brings radix parsing.
assert_eq!(i32::from_str_radix("ff", 16), Ok(255));
```

**When to use it:** `Num` as the default bound for "any number" — it is far
shorter than spelling out `Add + Sub + Mul + Div + Rem + Zero + One + PartialEq`,
and it is what the rest of the ecosystem bounds on. `NumAssign` when you mutate
an accumulator in place. `NumRef` and `RefNum` exist for types too large to copy,
such as big integers, where operating on references matters.

#### `Float` and `Real`

Floating-point behaviour, generically.

```
use num_traits::Float;

fn describe<T: Float>(value: T) -> &'static str {
    if value.is_nan() {
        "nan"
    } else if value.is_infinite() {
        "infinite"
    } else {
        "finite"
    }
}

assert_eq!(describe(f64::nan()), "nan");
assert_eq!(describe(f64::infinity()), "infinite");
assert_eq!(describe(1.0f32), "finite");

// The methods you expect, plus the constants.
assert_eq!(2.0f64.sqrt(), std::f64::consts::SQRT_2);
assert_eq!(2.0f64.mul_add(3.0, 1.0), 7.0);
assert!(f64::epsilon() > 0.0);
```

**When to use it:** for code genuinely parameterised over precision — a solver
used at `f32` on a GPU and `f64` on a CPU. **It requires `std` or the `libm`
feature**, since `sqrt` and friends come from the platform's math library; a
`no_std` build without `libm` will report the methods as missing rather than
explaining why. `Real` is the same surface minus NaN and infinity, for types that
are real numbers without floating-point quirks. `FloatConst` carries π, e and
the rest.

#### `PrimInt`

The bit-level integer operations.

```
use num_traits::PrimInt;

fn popcount<T: PrimInt>(value: T) -> u32 {
    value.count_ones()
}

assert_eq!(popcount(0b1011u8), 3);
assert_eq!(popcount(0xFFFFu16), 16);

// Shifts, rotations and endianness are all here.
assert_eq!(1u8.rotate_left(1), 2);
assert_eq!(0x1234u16.swap_bytes(), 0x3412);
assert_eq!(8u32.leading_zeros(), 28);
```

**When to use it:** bit manipulation that should work for any integer width — a
bitset, a compression routine, a hash. It bounds on the primitive integers
specifically, so unlike `Num` it will not accept a big integer, which is usually
what you want when you are counting bits in a fixed-width word.

### Conversion and bounds

#### `ToPrimitive`, `FromPrimitive` and `NumCast`

The checked conversion traits.

```
use num_traits::{FromPrimitive, NumCast, ToPrimitive};

// ToPrimitive: this value, as that type, if it fits.
assert_eq!(300i64.to_u8(), None);
assert_eq!(300i64.to_u16(), Some(300));
assert_eq!(3.9f64.to_i32(), Some(3));

// FromPrimitive: build this type from a given primitive.
assert_eq!(u8::from_i64(300), None);
assert_eq!(u8::from_i64(30), Some(30));

// NumCast: generic over both ends.
fn convert<T: NumCast, U: NumCast>(value: T) -> Option<U> {
    U::from(value)
}
assert_eq!(convert::<i64, u8>(30), Some(30u8));
assert_eq!(convert::<i64, u8>(300), None);
```

**When to use it:** at the edges of generic numeric code, where a `usize` length
or a parsed `i64` has to become the caller's `T`. Every method returns `Option`
because the conversion may not fit or may lose precision, and that `None` is the
information `as` throws away. `NumCast` is the one to bound on when both source
and destination are parameters.

#### `Bounded`

The minimum and maximum of a type.

```
use num_traits::Bounded;

// Generic over width, so one function covers every integer type.
fn range<T: Bounded + std::fmt::Debug>() -> (T, T) {
    (T::min_value(), T::max_value())
}

assert_eq!(range::<i16>(), (-32768, 32767));
assert_eq!(range::<u8>(), (0, 255));
```

**When to use it:** for a running minimum or maximum, where the accumulator must
start at the opposite extreme, and for range checks in generic code. On concrete
types `i16::MIN` is better — it is a constant rather than a method call, and
reads more plainly.

### Operation policies

#### `Signed` and `Unsigned`

Distinguishing types that have a sign.

```
use num_traits::Signed;

fn magnitude<T: Signed>(value: T) -> T {
    value.abs()
}

assert_eq!(magnitude(-5i32), 5);
assert_eq!(magnitude(-1.5f64), 1.5);
assert_eq!((-5i32).signum(), -1);
assert!((-5i32).is_negative());
```

**When to use it:** when a function needs `abs` or `signum`, which unsigned types
cannot meaningfully provide — bounding on `Signed` is how you say so in the
signature. `Unsigned` is a marker with no methods, used to require the opposite.
Note `abs` on the most negative integer still overflows, exactly as the inherent
method does.

#### `Euclid`

Division and remainder that stay non-negative.

```
use num_traits::Euclid;

// Rust's % takes the sign of the dividend...
assert_eq!(-7i32 % 3, -1);

// ...while Euclidean remainder is always non-negative.
assert_eq!(Euclid::rem_euclid(&-7i32, &3), 2);
assert_eq!(Euclid::div_euclid(&-7i32, &3), -3);
```

**When to use it:** wrapping an index into a range, computing a modular hash,
anything where a negative remainder is wrong. The distinction catches people
out: `-7 % 3` being `-1` is correct for Rust and useless as an array index.
`CheckedEuclid` is the non-panicking variant for division by zero.

#### `Pow`

Exponentiation, generically.

```
use num_traits::Pow;

// Integer exponent on an integer base.
assert_eq!(Pow::pow(2u32, 10u32), 1024);

// Float base and float exponent, which is a different operation.
assert_eq!(Pow::pow(2.0f64, 0.5f64), std::f64::consts::SQRT_2);

// The free function, with a checked counterpart.
assert_eq!(num_traits::pow(3i32, 4), 81);
assert_eq!(num_traits::checked_pow(2u8, 10), None); // overflows u8
```

**When to use it:** generic code raising a value to a power, where the inherent
`pow` differs between integers and floats in both name and signature. `Pow` is
implemented for many base/exponent combinations, so the trait is doing real
unifying work here rather than just renaming. `checked_pow` is the one to reach
for with untrusted exponents, since the result grows fast enough to overflow
almost immediately.
