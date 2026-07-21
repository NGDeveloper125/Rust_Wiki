---
title: "Match guards"
area: "Pattern Matching"
embedded_support: full
groups: ["Pattern Matching", "Designing Robust Data Models"]
related_syntax: [match, if, "|"]
see_also: ["match expressions", "Destructuring", "Exhaustiveness checking"]
---

## Explanation

A match guard is an extra `if condition` attached to a
[`match`](match-expressions.md) arm, checked only after the arm's pattern
has already matched — the arm only actually runs if both the shape
matches *and* the guard is true. Where a plain pattern can only ask "is
this value shaped like X," a guard lets the same arm also ask an
arbitrary question about the value it just bound, without inventing a
new variant or a separate pattern just to express that condition.

Guards exist because patterns alone can't express everything worth
branching on. A pattern can distinguish `Some(n)` from `None`, but it
can't by itself distinguish "`Some(n)` where `n` is negative" from
"`Some(n)` where `n` is zero or more" — that's a runtime comparison, not
a structural shape, and a guard is where that comparison belongs. This
keeps [destructuring](destructuring.md) and conditional logic cleanly
separated: the pattern says what shape and which bindings, the guard
says what has to additionally be true about them.

The trade-off to know is that guards sit outside
[exhaustiveness checking](exhaustiveness-checking.md): the compiler
verifies every *pattern* is covered, but it can't reason about whether a
set of guards on otherwise-identical patterns covers every possible
value — so a `match` with guards still needs a final catch-all arm (or
guards that are visibly exhaustive together, like covering every range)
to compile, and it's on the author to make sure that catch-all is
actually correct rather than a silent gap.

Guards are most valuable exactly where a set of arms would otherwise be
identical patterns differing only by a condition — several numeric
ranges over the same variant, or a tuple pattern that needs one field to
satisfy a check the pattern itself can't express.

## Basic usage example

```
let n: i32 = -3;

let sign = match n {
    n if n < 0 => "negative", // <- guard: same pattern shape (any i32), extra condition on its value
    0 => "zero",
    _ => "positive",
};

println!("{sign}");
```

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

An HTTP client needs to categorize a response by status code range;
every arm matches the same shape (a `u16`), so guards are what actually
distinguish them.

```
fn category(status: u16) -> &'static str {
    match status {
        code if (200..300).contains(&code) => "success", // <- guard tests the range, pattern alone can't
        code if (300..400).contains(&code) => "redirect",
        code if (400..500).contains(&code) => "client error",
        code if (500..600).contains(&code) => "server error",
        _ => "unknown",
    }
}

println!("{}", category(404));
```

**Why this way:** a bare `u16` pattern can't express "in this range,"
so a guard is the correct tool rather than forcing the ranges into
separate enum variants that don't otherwise exist — the
[Rust Reference on match guards](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guards)
documents guards as arbitrary boolean expressions evaluated after the
pattern matches.

### Scenario: Validating input

A signup form only allows a minor to register with a parent's consent
recorded; the two fields together decide whether the arm applies, which
a pattern on either field alone can't express.

```
struct Signup {
    age: u8,
    has_parental_consent: bool,
}

fn is_eligible(signup: &Signup) -> bool {
    match signup {
        Signup { age, .. } if *age >= 18 => true,
        Signup { has_parental_consent, .. } if *has_parental_consent => true, // <- guard checks the other field
        _ => false,
    }
}

let signup = Signup { age: 15, has_parental_consent: true };
println!("{}", is_eligible(&signup));
```

**Why this way:** the eligibility rule genuinely depends on a
relationship between two fields, not just their shapes, so a guard
keeps that rule readable as a condition rather than encoding "adult OR
consenting minor" as extra enum variants that would only exist to dodge
the guard — matching the
[Rust Book's](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#extra-conditionals-with-match-guards)
description of guards as the place for logic a pattern can't express.

## Explanation (Embedded)

Match guards work identically under `#![no_std]` — same core-language
mechanism, same generated comparison code, no allocator or runtime
involvement. They're the natural way to classify a raw sensor or
register reading into a range-based category, because the categories
are numeric bands over a single scalar rather than distinct shapes a
plain pattern could tell apart on its own: a guard like `n if n >
THRESHOLD` states "in this band" directly, where the pattern alone only
sees "some integer." The same caveat carries over unchanged too: guards
sit outside exhaustiveness checking, so a set of threshold guards over a
sensor's full reading range still needs a final catch-all arm, and it's
on the firmware author to confirm that catch-all is actually correct
rather than a silently-missed band of readings.

## Basic usage example (Embedded)

```
fn classify(reading_mv: i32) -> &'static str {
    match reading_mv {
        n if n > 3000 => "over-voltage", // <- guard: same pattern shape (any i32), extra condition on its value
        n if n < 500 => "under-voltage",
        _ => "nominal",
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Branching on data (pattern matching)

A temperature sensor's raw ADC reading needs to be classified into
idle/warning/critical bands to decide whether to throttle a heater;
every arm matches the same shape (an `i32`), so guards are what actually
distinguish the bands.

```
fn thermal_state(adc_counts: i32) -> &'static str {
    match adc_counts {
        n if n > 3800 => "critical", // <- guard tests the band, pattern alone can't
        n if n > 3000 => "warning",
        _ => "nominal",
    }
}

let state = thermal_state(3550);
```

**Why this way:** a bare `i32` pattern can't express "above this
threshold," so a guard is the correct tool rather than inventing enum
variants for bands that don't otherwise exist in the hardware's own
model — the
[Rust Reference on match guards](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guards)
documents guards as arbitrary boolean expressions evaluated after the
pattern matches, which is exactly what a threshold check is.

### Scenario: Validating input

A motor driver only allows a high-current mode request when the
requested current exceeds a threshold and a hardware safety interlock is
confirmed closed; the two fields together decide whether the arm
applies, which neither field's pattern alone can express.

```
struct DriveRequest {
    requested_ma: u16,
    interlock_closed: bool,
}

fn mode_for(request: &DriveRequest) -> &'static str {
    match request {
        DriveRequest { requested_ma, interlock_closed }
            if *requested_ma > 2000 && *interlock_closed => "high-current", // <- guard checks both fields together
        DriveRequest { interlock_closed, .. } if !*interlock_closed => "locked-out",
        _ => "standard",
    }
}
```

**Why this way:** the high-current mode is genuinely gated on both
fields together, not on either field's shape alone, so a guard keeps
that safety condition readable as a single condition rather than
encoding "current above X AND interlock closed" as extra enum variants
that would only exist to dodge the guard — matching the
[Rust Book's](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#extra-conditionals-with-match-guards)
description of guards as the place for logic a pattern can't express,
doubly relevant when the condition is a safety interlock rather than a
UI nicety.
