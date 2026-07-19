---
title: "continue"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [loop, while, for, break]
see_also: [break]
---

## Explanation

`continue` skips the rest of the current loop iteration's body and jumps
straight to the next iteration's condition check (for `while`/`for`) or
back to the top of the body (for `loop`):

```
for x in 0..10 {
    if x % 2 == 0 {
        continue;
    }
    println!("{x}"); // only odd numbers
}
```

`continue` never carries a value — unlike `break`, there is no
`continue value;` form, since "continuing" doesn't produce a result the
way exiting a `loop` can. Like `break`, it can target a labeled outer loop
explicitly with `continue 'label;` from inside a nested loop.

## Basic usage example

```
for x in 0..5 {
    if x == 2 {
        continue; // <- skips the rest of this iteration, jumps to `x = 3`
    }
    println!("{x}");
}
```

## Best practices & deeper information

### Scenario: Working with collections

Averaging a batch of raw sensor readings means some entries won't parse —
`continue` skips a malformed one and moves straight to the next, without
nesting the rest of the loop body inside an `if`.

```
let readings = ["21.5", "invalid", "19.8", "", "23.1"];
let mut total = 0.0;
let mut count = 0;

for raw in readings {
    let Ok(value) = raw.parse::<f64>() else {
        continue; // <- skip this malformed reading, move to the next one
    };
    total += value;
    count += 1;
}

let average = total / count as f64;
```

**Why this way:** pairing `continue` with a `let ... else` guard keeps the
"happy path" of the loop body unindented — the
[Book's control-flow chapter](https://doc.rust-lang.org/book/ch03-05-control-flow.html)
favors this early-exit shape over wrapping the rest of the iteration in a
nested `if`.

### Scenario: Validating input

Building a list of valid users from raw records means rejecting
out-of-range ages as they're encountered, without aborting the whole
batch.

```
struct User { name: String, age: i32 }

let raw_ages = [("alice", 30), ("bob", -5), ("carol", 45)];
let mut valid_users = Vec::new();

for (name, age) in raw_ages {
    if age < 0 || age > 150 {
        continue; // <- reject the out-of-range record, skip straight to the next one
    }
    valid_users.push(User { name: name.to_string(), age });
}
```

**Why this way:** rejecting one bad record with `continue` keeps
validation co-located with iteration for a one-off batch; when the same
validation logic is needed at multiple call sites, prefer a constructor
that returns `Result` instead, per the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/) idea
of making invalid states unrepresentable rather than filtered out ad hoc.

## Embedded Rust Notes

**Full support.** No `std` dependency; works identically in `#![no_std]`.
