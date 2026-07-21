---
title: "matches!"
kind: macro
embedded_support: full
groups: ["Errors & Assertions", "Macros & Metaprogramming"]
related_concepts: ["match expressions", "if let / while let"]
related_syntax: ["match", "if let"]
see_also: ["match expressions", "if let / while let"]
---

## Explanation

`matches!(value, pattern)` expands to a full `match` that returns `true`
for `pattern` and `false` for everything else — concretely, `match value
{ pattern => true, _ => false }` — collapsing that boilerplate into a
single boolean expression. An optional guard extends the pattern exactly
like a `match` arm's guard does: `matches!(value, pattern if condition)`
only returns `true` when both the pattern matches and `condition` holds.

Because it only ever produces a `bool`, `matches!` is a check, not an
extraction — any bindings introduced by `pattern` are discarded once the
macro returns; they exist only within the (invisible) generated `match`
arm, not in the surrounding scope. Reaching for it only makes sense when
the answer to "does this match?" is all that's needed; the moment code
needs the *data* bound by the pattern, a real `match` or
[`if let` / `while let`](../../concepts/pattern-matching/if-let-and-while-let.md)
is the right tool instead, since `matches!` has nowhere to hand that data
back to the caller.

## Usage examples

### Checking a variant without extracting its data

```
enum ConnectionState {
    Connected,
    Disconnected,
    Reconnecting { attempt: u32 },
}

let state = ConnectionState::Reconnecting { attempt: 2 };
let is_reconnecting = matches!(state, ConnectionState::Reconnecting { .. }); // <- true, without binding `attempt`
```

### Branching on data (pattern matching)

Filtering a list of jobs down to the ones currently active is a yes/no
question per job — exactly what `matches!` is for instead of a full
`match` per element.

```
enum JobStatus {
    Queued,
    Running { worker_id: u32 },
    Finished,
    Failed { reason: String },
}

fn is_active(status: &JobStatus) -> bool {
    matches!(status, JobStatus::Queued | JobStatus::Running { .. }) // <- one boolean check, two accepted variants
}

let jobs = vec![JobStatus::Queued, JobStatus::Finished, JobStatus::Running { worker_id: 3 }];
let active_count = jobs.iter().filter(|job| is_active(job)).count();
```

The
[std docs](https://doc.rust-lang.org/std/macro.matches.html) note that a
`match` with two arms both returning `true`/`false` (or an `if let ...
else`) says the same thing with more ceremony; `matches!` states the
yes/no intent directly, including the `|` pattern-alternation shorthand
for "any of these variants."

### Validating input

Rejecting a request whose priority isn't one of a small allowed set uses
a pattern guard to narrow one variant further by a field value.

```
enum Priority {
    Low,
    Normal,
    High(u8), // escalation level
}

fn is_acceptable(priority: &Priority) -> bool {
    matches!(priority, Priority::Low | Priority::Normal)
        || matches!(priority, Priority::High(level) if *level <= 3) // <- pattern guard narrows the High case further
}

assert!(is_acceptable(&Priority::High(2)));
assert!(!is_acceptable(&Priority::High(9)));
```

A pattern guard lets a single boolean check express
"this variant, but only within these bounds," matching the
[Reference's match-guard semantics](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guards)
exactly as it would inside a full `match` — without the extra arm and
inner `if` a full `match` would otherwise need.

## Explanation (Embedded)

`matches!` expands to an ordinary `match` at compile time and introduces
no runtime dependency beyond the pattern match itself, so it behaves
identically under `#![no_std]` — there's no heap, no allocation, and
nothing hosted-only involved in a boolean pattern check. It's genuinely
handy in embedded code specifically for checking a hardware-status or
interrupt-flag enum inline — a peripheral register read decoded into an
enum, then immediately reduced to "is it one of the states I care about"
— without writing out a full `match` block just to get a `bool`.

## Usage examples (Embedded)

### Checking an interrupt flag inline

```
enum InterruptFlag {
    None,
    DataReady,
    Overrun,
    FrameError,
}

fn read_status_flag() -> InterruptFlag {
    // pretend this decodes a peripheral's status register into the enum
    InterruptFlag::DataReady
}

if matches!(read_status_flag(), InterruptFlag::DataReady | InterruptFlag::Overrun) {
    // <- one boolean check across two flag variants, no full match needed
    // handle the pending data
}
```

### Guarding a register write on device power state

A peripheral driver only allows a reconfiguration write while the device
is idle or lightly loaded, checked with a pattern guard against a field
decoded straight from a status register.

```
enum PowerState {
    Off,
    Standby,
    Active { load_percent: u8 },
}

fn can_reconfigure(state: &PowerState) -> bool {
    matches!(state, PowerState::Off | PowerState::Standby)
        || matches!(state, PowerState::Active { load_percent } if *load_percent < 5)
        // <- pattern guard narrows the Active case further, same mechanism as classic Rust
}
```
