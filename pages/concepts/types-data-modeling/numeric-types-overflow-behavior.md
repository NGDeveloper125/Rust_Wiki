---
title: "Numeric types & overflow behavior"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Numeric Safety"]
related_syntax: [integer-suffixes, float-suffixes]
see_also: []
---

## Explanation

Rust's integer types are explicit about width and signedness —
`u8`/`i8` through `u128`/`i128`, plus pointer-sized `usize`/`isize` — with
no single generic "number" type and no implicit widening between
different integer types (adding a `u8` and a `u32` directly is a compile
error; an explicit `as` cast or `.into()` conversion is required).

Overflow behavior is deliberately governed by the `overflow-checks`
compile setting (on by default in debug builds, off in release): when an
operation on *runtime* values overflows its type's range — say `a + b`
where `a` is already `255u8` — a checked build panics immediately,
surfacing the bug during development, while a release build instead
**wraps** silently (`255u8` + `1` becomes `0`) to avoid the runtime cost
of checking every operation. (A *constant* overflow the compiler can see
at compile time, like the literal `255u8 + 1`, is rejected outright as a
compile error regardless of profile.) Where either
behavior isn't good enough — you need guaranteed, defined behavior
regardless of build profile — explicit methods make the choice visible in
the code itself: `checked_add` (returns `None` on overflow),
`wrapping_add` (always wraps), `saturating_add` (clamps to the type's
max/min), and `overflowing_add` (returns the wrapped value plus a bool
flag).

This design trades a small amount of implicit safety (debug-only
overflow panics) for explicitness everywhere it actually matters, rather
than picking one runtime behavior and making every caller pay for it
unconditionally.

## Basic usage example

```
let a: u8 = 250;
let b: u8 = 10;

let sum = a.checked_add(b); // <- None: 260 doesn't fit in a u8, caught explicitly
println!("{sum:?}");
```

**Restriction:** writing plain `a + b` here instead hides the same
problem — it panics in a debug build but silently wraps to `4` in a
release build, so the two profiles behave differently unless you use an
explicit `checked_`/`wrapping_`/`saturating_` method.

## Best practices & deeper information

### Scenario: Numeric computation

Widening an accumulator before summing narrow values sidesteps overflow
entirely, which is cheaper than reaching for checked arithmetic for what
is otherwise ordinary, bounded addition.

```
let sensor_readings: [u16; 4] = [1200, 980, 1500, 1100];

let total: u32 = sensor_readings.iter().map(|&r| r as u32).sum(); // <- widen before summing: u16 could overflow
let average = total / sensor_readings.len() as u32;

println!("average reading: {average}");
```

**Why this way:** summing narrow integers directly risks overflowing the
narrow type the moment enough values accumulate; widening to a type with
real headroom avoids the risk up front instead of needing
`checked_add` at every step — the kind of unchecked arithmetic Clippy's
[`arithmetic_side_effects`](https://rust-lang.github.io/rust-clippy/master/#arithmetic_side_effects)
lint flags when it can't prove overflow is impossible.

### Scenario: Validating input

Once a value originates outside the program — a request body, a config
file, anything the caller could get wrong — plain arithmetic on it trusts
an assumption untrusted input is specifically there to violate.
`checked_*` turns that into a `Result` instead of a silent wraparound.

```
fn apply_discount(price_cents: u32, discount_cents: u32) -> Result<u32, &'static str> {
    price_cents
        .checked_sub(discount_cents) // <- untrusted input: a discount larger than the price must not silently wrap
        .ok_or("discount exceeds price")
}

apply_discount(500, 100); // Ok(400)
apply_discount(500, 900); // Err("discount exceeds price") instead of wrapping to a huge u32
```

**Why this way:** a release build silently wraps on overflow rather than
panicking, so `price_cents - discount_cents` on untrusted input would
compile and often "work" in testing, then wrap to a near-`u32::MAX`
value in production the first time a caller sends bad data —
`checked_sub` makes that failure explicit and impossible to ignore.

## Explanation (Embedded)

As [`+`](../../syntax/operators/plus.md)'s embedded section covers, a
release build — the profile that actually gets flashed to a device — has
overflow checks off by default, so an ordinary `a + b` wraps silently
instead of panicking; a device already in the field can't be
"recompiled in debug mode" to catch the bug later the way a desktop
program can. That single fact makes `checked_*`/`wrapping_*`/`saturating_*`
more load-bearing in firmware than in typical hosted code, for a simple
reason: the debug build's panic was never a *plan* for handling overflow,
just a development-time trip-wire, and firmware has no equivalent
trip-wire once it ships.

What's genuinely a design decision, though, is *which* of the three to
reach for, and that follows from what the quantity itself means:

- **`wrapping_*`** is correct, not just tolerated, when the quantity is
  *designed* to roll over — a free-running millis/tick counter
  incremented every interrupt is the canonical case. Wraparound there
  isn't a bug being suppressed; two's-complement wraparound arithmetic
  means `now.wrapping_sub(earlier)` still produces the correct elapsed
  time even after `now` has wrapped past its type's max, as long as the
  actual elapsed duration never exceeds the counter's range. Reaching for
  `checked_add` on a tick counter would be actively wrong — it would
  report a spurious failure on a rollover that isn't an error at all.
- **`saturating_*`** fits a quantity that has a real physical or logical
  ceiling/floor where clamping is itself the sane behavior — a PWM duty
  cycle that must never exceed 100%, an accumulated ADC reading being
  scaled toward a display range. Silently wrapping a duty cycle past its
  max could momentarily command a wildly wrong value to actuator
  hardware before the next control cycle corrects it; saturating clamps
  it to the safe boundary instead, which is the outcome actually wanted.
- **`checked_*`** is right when going out of range means something is
  actually wrong and the caller needs to know before acting on the
  result — a value parsed from a config blob in EEPROM/flash, an
  index computed from external input, anything where continuing past the
  overflow with *any* number (wrapped or clamped) would be worse than
  surfacing an explicit failure.

Picking among the three is a per-quantity design decision, not a
one-size-fits-all default — the same program legitimately uses
`wrapping_*` for its tick counter and `checked_*` for a value it just
parsed from storage.

## Basic usage example (Embedded)

```
struct Millis(u32);

impl Millis {
    fn on_tick(&mut self) {
        self.0 = self.0.wrapping_add(1); // <- correct by design: rollover is expected, not an error
    }

    fn elapsed_since(&self, earlier: u32) -> u32 {
        self.0.wrapping_sub(earlier) // <- still correct even if self.0 has wrapped past u32::MAX since `earlier`
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Numeric computation

A free-running millis counter should wrap on purpose; a PWM duty-cycle
value derived from it should never be allowed to wrap at all, because a
wrapped duty cycle is a plausible-looking but wildly wrong value that
could reach the hardware.

```
struct Millis(u32);

fn duty_cycle_percent(on_time_us: u32, period_us: u32) -> u8 {
    let ratio = (on_time_us * 100) / period_us; // assumes period_us > 0, checked elsewhere
    ratio.min(100) as u8 // <- PREFER: clamp explicitly, never let a computed duty cycle exceed the valid range
}

impl Millis {
    fn on_tick(&mut self) {
        self.0 = self.0.wrapping_add(1); // <- PREFER: rollover is the intended behavior for a tick counter
    }
}
```

**Why this way:** the tick counter's job is to keep counting forever
within a fixed width, so `wrapping_add` is the *correct* operation, not a
concession; the duty cycle's job is to stay within a hardware-meaningful
0–100 range, so clamping with `.min(100)` (or `saturating_*` on the raw
arithmetic before the cast) prevents an out-of-range value from ever
reaching a PWM register, where Clippy's
[`arithmetic_side_effects`](https://rust-lang.github.io/rust-clippy/master/#arithmetic_side_effects)
lint would flag the unchecked multiplication as worth a second look.

### Scenario: Validating input

A calibration constant read back from EEPROM or a config blob should be
checked before it's used in arithmetic that ultimately drives hardware —
unlike a tick counter, there's no "intended" out-of-range value here, so
overflow must surface as an explicit error instead of continuing with a
wrapped or clamped number that looks plausible but isn't.

```
fn scaled_reading(raw_adc: u16, calibration_gain: u16) -> Result<u32, &'static str> {
    (raw_adc as u32)
        .checked_mul(calibration_gain as u32) // <- corrupt calibration data must not silently produce a bogus reading
        .ok_or("calibration overflow: check stored gain value")
}
```

**Why this way:** a gain constant corrupted by a bad flash write or a
misconfigured EEPROM cell isn't a case where wrapping or clamping
produces anything meaningful — the only sound response is to refuse to
report a reading at all, which is exactly what surfacing the failure as
an explicit `Result` accomplishes, versus silently handing downstream
control logic a number that only looks like a valid sensor reading.
