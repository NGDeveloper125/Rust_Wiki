---
title: "continue"
kind: keyword
embedded_support: full
groups: ["Control Flow", Basics]
related_concepts: []
related_syntax: [loop, while, for, break]
see_also: [break]
---

## Explanation

`continue` skips the rest of the current loop iteration's body and jumps
straight to the next iteration's condition check (for `while`/`for`) or
back to the top of the body (for `loop`) — for example, `continue`d inside
a `for` loop over `0..10` whenever a value is even leaves only the odd
ones printed.

`continue` never carries a value — unlike `break`, there is no
`continue value;` form, since "continuing" doesn't produce a result the
way exiting a `loop` can. Like `break`, it can target a labeled outer loop
explicitly with `continue 'label;` from inside a nested loop.

## Usage examples

### Skipping an iteration with `continue`

```
for x in 0..5 {
    if x == 2 {
        continue; // <- skips the rest of this iteration, jumps to `x = 3`
    }
    println!("{x}");
}
```

### Working with collections

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

Pairing `continue` with a `let ... else` guard keeps the
"happy path" of the loop body unindented — the
[Book's `let...else` section](https://doc.rust-lang.org/book/ch06-03-if-let.html)
frames this "stay on the happy path" shape as the reason `let...else`
exists, over wrapping the rest of the iteration in a nested `if`.

### Validating input

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

Rejecting one bad record with `continue` keeps
validation co-located with iteration for a one-off batch; when the same
validation logic is needed at multiple call sites, prefer a constructor
that returns `Result` instead, per the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/) idea
of making invalid states unrepresentable rather than filtered out ad hoc.

## Explanation (Embedded)

`continue` behaves identically under `#![no_std]` — no `std` dependency,
same core-language skip-to-next-iteration semantics. It shows up
naturally in two embedded shapes: filtering out a known-bad sample from a
burst read without nesting the rest of the loop body in an `if`, and
skipping over an address that didn't respond during a bus scan so the
scan keeps going instead of aborting.

## Usage examples (Embedded)

### Skipping a bad reading from a burst of ADC samples

```
let raw_samples: [u16; 8] = read_adc_burst();
let mut total: u32 = 0;
let mut count = 0;

for sample in raw_samples {
    if sample == ADC_INVALID_READING {
        continue; // <- skip a known-bad sample, move to the next one
    }
    total += sample as u32;
    count += 1;
}
```

### Skipping an address that didn't respond during a bus scan

```
for addr in 0x08..0x78 {
    if i2c.ping(addr).is_err() {
        continue; // <- this address didn't ack; move on to the next candidate
    }
    register_device(addr);
}
```
