---
title: "as"
kind: keyword
embedded_support: full
groups: ["Types & Data Structures", "Modules, Crates & Visibility"]
related_concepts: ["Numeric types & overflow behavior"]
related_syntax: []
see_also: ["use"]
---

## Explanation

`as` performs an explicit type cast: `expr as Type`. Between numeric
types, `as` covers every direction a plain conversion method doesn't
already handle implicitly-for-free: widening (`u8 as u32`), narrowing
(`u32 as u8`), signed/unsigned reinterpretation (`i32 as u32`),
integer-to-float and float-to-integer (`5 as f64`, `5.9 as i32`), and
casts between `bool`/`char` and their integer representations (`true as
u8`, `'a' as u32`). `as` is also how a value is cast between raw pointer
types, and between a raw pointer and an integer address (`ptr as usize`).

The exact numeric behavior is worth knowing precisely, because `as` never
panics — it always produces some value, unlike `TryFrom`. Integer
narrowing **truncates**: it keeps only the low-order bits that fit the
target width, discarding the rest (`300i32 as u8` is `44`, not a clamp or
an error). Integer widening sign-extends a signed source or zero-extends
an unsigned one. Float-to-integer casts **saturate**: a float outside the
target integer's range clamps to that type's `MIN` or `MAX` instead of
producing an undefined or wrapped result, and `NaN` casts to `0` — this
saturating behavior has been guaranteed since Rust 1.45 and is safe to
rely on. Truncation toward zero applies to the fractional part of a
float-to-integer cast (`3.9 as i32` is `3`, not `4`); see [Numeric types &
overflow behavior](../../concepts/types-data-modeling/numeric-types-overflow-behavior.md)
for the broader rules governing arithmetic overflow, which `as` casts are
exempt from precisely because they're explicit conversions, not
operations that can overflow.

`as` reads left-to-right and binds tighter than most binary operators but
looser than method calls, so `x as i64 + 1` casts `x` first, while `-x as
i64` casts the already-negated value; parenthesizing a cast's operand
when it's anything but a simple name or call avoids ambiguity for the
reader even where the precedence rules already resolve it correctly.

`as` also has a completely unrelated second meaning inside a `use`
declaration, where it renames an imported item (`use std::io::Result as
IoResult;`) rather than casting a value — the keyword is shared, but the
two uses have nothing else in common; the full renaming grammar belongs
on the [`use`](use.md) page.

## Usage examples

### Casting between numeric types

```
let count: u32 = 10;
let total: i64 = count as i64; // <- `as` widens u32 to i64 explicitly
```

**Restriction:** `as` performs a lossy conversion silently when narrowing
— `1000i32 as u8` compiles and truncates to `232` with no warning at the
call site; reach for `u8::try_from(1000i32)` instead whenever the value
might not fit and the caller needs to detect that.

### Numeric computation

Averaging integer readings requires converting at least one operand to a
floating-point type before dividing, since Rust never converts between
numeric types implicitly.

```
let readings: [u32; 4] = [68, 72, 65, 70];

let total: u32 = readings.iter().sum();
let average = total as f64 / readings.len() as f64; // <- both operands cast to f64 before dividing
println!("average heart rate: {average:.1}");
```

`total / readings.len()` wouldn't compile at all —
`u32` divided by `usize` is a type mismatch — and even if the types
matched, integer division would silently truncate the result; casting
both sides to `f64` first is the direct way to get a fractional average,
rather than routing through a fallible `TryFrom` for a conversion that
can't actually fail here.

### Bit manipulation and flags

Casting a fieldless enum variant to its integer discriminant with `as` is
the standard way to turn a typed value back into the raw byte a register
or wire protocol expects.

```
#[repr(u8)]
enum LinkStatus {
    Down = 0,
    Negotiating = 1,
    Up = 2,
}

fn to_register_byte(status: LinkStatus) -> u8 {
    status as u8 // <- `as` reads the enum's discriminant directly, given `#[repr(u8)]` fixes its width
}
```

`as` is the only way to go from a fieldless enum back
to its discriminant value — there's no method for it — and pairing it
with an explicit `#[repr(u8)]` (see [`#[repr(...)]`](../attributes/repr.md))
guarantees the byte `as` produces matches the width the register or
protocol actually expects, rather than whatever discriminant size the
compiler would otherwise pick.

## Explanation (Embedded)

`as` casting behaves identically under `#![no_std]` — the same
truncating-narrow / sign-or-zero-extending-widen / saturating-float-to-int
rules apply on any target, with no allocator or OS involvement in the
mechanism itself. Where this shows up constantly in embedded code is
reading a peripheral register or an ADC channel: a hardware read is
almost always typed at some fixed width (a 12-bit ADC result held in a
`u16`, a 32-bit status register), and narrowing it to whatever width a
protocol or a downstream API actually needs (`as u8` to pack it into a
byte for a UART frame, say) is routine. The behavior worth being
deliberate about is exactly the classic Explanation's warning: `as`
truncates silently rather than reporting that a value didn't fit, which
matters more in embedded code than in most hosted code, since a
truncated sensor reading or register value is a silent correctness bug
that can be much harder to notice on a device with no console to print a
warning to. `u8::try_from(reading)` (checked with `.ok()` or a `match` on
the `Result`) is the safe alternative whenever a value genuinely might
not fit and the caller needs to detect that rather than silently keep
the low bits.

## Usage examples (Embedded)

### Narrowing an ADC reading with `as`, and the `try_from` alternative

```
fn pack_for_uart(adc_reading: u16) -> u8 {
    adc_reading as u8 // <- `as`: truncates silently; fine only if the caller already knows the top bits are unused
}

fn pack_for_uart_checked(adc_reading: u16) -> Result<u8, &'static str> {
    u8::try_from(adc_reading).map_err(|_| "reading exceeds one byte") // <- reports rather than truncates
}
```

### Casting an integer address to a raw pointer for MMIO

```
const GPIOA_ODR: *mut u32 = 0x4001_0814 as *mut u32; // <- `as`: integer literal cast to a raw pointer for MMIO access

fn set_pa5() {
    unsafe { GPIOA_ODR.write_volatile(GPIOA_ODR.read_volatile() | (1 << 5)) }
}
```
