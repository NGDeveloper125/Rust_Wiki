---
title: "ref"
kind: keyword
embedded_support: full
groups: ["Ownership & Borrowing"]
related_concepts: ["Borrowing (shared references)", "Mutable borrowing"]
related_syntax: ["&", "let", "match"]
see_also: ["&", "match"]
---

## Explanation

`ref` appears inside a pattern, directly before the identifier being bound:
`ref name` (and `ref mut name` for a mutable borrow). It changes the
**binding mode** of that one identifier from the pattern default — bind by
moving or copying the matched place — to binding by reference instead. It
can appear anywhere an identifier pattern can: a `let` pattern
(`let ref x = value;`), a function parameter, or, most commonly, inside a
`match` arm's destructuring pattern (`Variant { ref field } => ...`).

Before Rust 2018's "default binding modes" (a.k.a. match ergonomics), `ref`
was the *only* way to bind part of a matched value by reference rather than
moving it out. Match ergonomics changed the common case: when the scrutinee
being matched is itself a reference (`&value` or `&mut value`), the compiler
now automatically shifts every subsequent binding in the pattern to bind by
reference, so `Some(x)` matched against a `&Option<T>` binds `x: &T`
directly, with no `ref` written anywhere. This is why `ref` shows up far
less in modern Rust than it did before 2018 — most code that used to need it
now gets the same effect for free by matching on a reference.

`ref` is still genuinely needed in two situations match ergonomics doesn't
cover. First: matching directly on an **owned** value (not a reference) when
you want to borrow only part of it, so the whole value remains usable
afterward — match ergonomics has nothing to activate it if the scrutinee
was never a reference to begin with. Second: fine-grained control inside a
single pattern where different bindings need different modes (for instance,
binding one field mutably while another is left to move or copy normally) —
ergonomics infers one mode top-down from the scrutinee, so an explicit `ref`
or `ref mut` is the way to override it for just one binding.

`ref` is not a modifier on the value being matched (it has no effect on the
type of the scrutinee) and it is not the same token as `&`: `&pattern`
matches against a reference (it expects the scrutinee to already be a
reference and "un-borrows" one layer), while `ref pattern-binding` produces
a reference during the match, regardless of what the scrutinee's type is.

## Usage examples

### Borrowing a field with `ref` instead of moving it

```
struct Ticket { code: String }

let ticket = Ticket { code: String::from("A-102") };
let Ticket { ref code } = ticket; // <- `ref` borrows `code` instead of moving it out of `ticket`

println!("{code}");
println!("{}", ticket.code); // still valid: `code` was only borrowed, `ticket` wasn't consumed
```

### Branching on data (pattern matching)

Matching directly on an owned enum (not a reference to one) while still
needing to return that same value afterward requires borrowing the field
you inspect, rather than letting the match move it out.

```
enum Shipment {
    Pending { tracking_code: String },
    Delivered,
}

fn log_tracking(shipment: Shipment) -> Shipment {
    match shipment {
        Shipment::Pending { ref tracking_code } => {
            // <- `ref` borrows `tracking_code` instead of moving it out of `shipment`
            println!("still pending: {tracking_code}");
        }
        Shipment::Delivered => println!("delivered"),
    }
    shipment // still whole: only `tracking_code` was borrowed during the match, never moved
}
```

Matching `shipment` by value would otherwise move
`tracking_code` out of it the moment that arm's pattern binds, making
`shipment` only partially usable afterward — the
[Rust Reference's binding modes section](https://doc.rust-lang.org/reference/patterns.html#binding-modes)
documents `ref` as exactly the tool for keeping a matched-on value intact
when its scrutinee isn't already a reference.

### Sharing data with multiple references

Destructuring a struct to read one field while continuing to use the whole
struct afterward needs that one field bound by reference, not moved out.

```
struct Invoice {
    number: String,
    total_cents: u64,
}

let invoice = Invoice { number: String::from("INV-2048"), total_cents: 4599 };
let Invoice { ref number, .. } = invoice; // <- `ref` borrows `number`, leaving `invoice` itself intact

println!("{number} totals {}", invoice.total_cents); // both `number` and `invoice` are usable here
```

Without `ref`, binding `number` in this pattern would move
the `String` out of `invoice`, making `invoice.total_cents` (and the rest of
`invoice`) inaccessible afterward — the
[Rust Reference](https://doc.rust-lang.org/reference/patterns.html#binding-modes)
covers `ref` as the way to keep the source place valid when only one field
needs to be shared out of it.

### Mutating through a reference

Mutating one field of an owned enum in place, through a `match`, needs
`ref mut` — without it, a `Copy` field like a counter would bind as a
detached copy, and mutating the copy would silently do nothing to the
original.

```
enum Task {
    Active { retries: u32 },
    Done,
}

fn record_retry(mut task: Task) -> Task {
    match task {
        Task::Active { ref mut retries } => {
            // <- `ref mut` mutably borrows the real field; without it, `retries: u32` would just be a copy
            *retries += 1;
        }
        Task::Done => {}
    }
    task
}
```

`u32` is `Copy`, so without `ref mut` the pattern would
bind `retries` as an independent copy and `*retries += 1` would mutate
nothing that survives the match — the
[Rust Reference's binding modes section](https://doc.rust-lang.org/reference/patterns.html#binding-modes)
is explicit that `ref mut` is required to get a genuine mutable borrow into
the matched place itself.

## Explanation (Embedded)

`ref`/`ref mut` behave identically under `#![no_std]` — pure
pattern-binding syntax with no runtime representation, so there's
nothing allocator- or target-specific about them. The situation that
calls for `ref` in embedded code is the same one that calls for it
anywhere: matching directly on an owned value (not a reference to one)
while still needing the rest of that value intact afterward — for
instance, inspecting one field of a hardware-status struct read back
from a peripheral without consuming the whole struct in the process.

## Usage examples (Embedded)

### Borrowing a field while matching an owned hardware-status struct

```
struct LinkStatus {
    speed_mbps: u32,
    duplex: Duplex,
}

enum Duplex { Half, Full }

fn log_link(status: LinkStatus) -> LinkStatus {
    match status {
        LinkStatus { ref duplex, speed_mbps } => {
            // <- `ref` borrows `duplex`; `speed_mbps` (a Copy u32) is still bound normally alongside it
            match duplex {
                Duplex::Full => defmt::info!("full duplex @ {}", speed_mbps),
                Duplex::Half => defmt::info!("half duplex @ {}", speed_mbps),
            }
        }
    }
    status // still whole: only `duplex` was borrowed during the match, never moved out
}
```

### Mutating a retry counter with `ref mut` in an owned driver-status enum

```
enum SensorState {
    Faulted { retries: u8 },
    Ready,
}

fn record_retry(mut state: SensorState) -> SensorState {
    match state {
        SensorState::Faulted { ref mut retries } => {
            // <- `ref mut`: mutably borrows the real field so the increment survives past the match
            *retries += 1;
        }
        SensorState::Ready => {}
    }
    state
}
```
