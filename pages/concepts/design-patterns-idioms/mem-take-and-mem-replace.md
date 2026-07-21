---
title: "mem::take / mem::replace"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Interior Mutability", "Sharing & Mutating Data Safely"]
related_syntax: []
see_also: ["Copy vs Clone", "Anti-pattern: cloning to satisfy the borrow checker", "Interior mutability (Cell & RefCell)"]
---

## Explanation

`std::mem::replace(dest, new_value)` swaps the value behind a `&mut T`
for a new one and hands back the old value by ownership, all in one
step. `std::mem::take(dest)` is the common special case where the
replacement is `T::default()` — it's exactly `mem::replace(dest,
T::default())`, and it only needs `T: Default`. Both exist to solve the
same problem: the borrow checker never lets you move a value out of a
place reachable only through `&mut self` while leaving that place empty,
because a struct field (or any other memory location) can't be left in a
"nothing here" state — every place must always hold a valid value of its
type. `mem::take`/`mem::replace` satisfy that requirement by immediately
plugging the hole with a default or a caller-supplied replacement, in the
same operation that hands the original value out.

This matters most during a state transition on `&mut self`: turning one
variant of an enum field into another, or moving a `Vec` out of a struct
to process it and hand back an empty one, requires *taking* the old
value by ownership rather than merely reading or mutating it in place.
Without `mem::take`, the tempting workaround is `self.field.clone()` —
which works, but pays for a full duplicate of whatever the field holds
just to satisfy a compiler rule, when the code never actually wanted two
copies to exist (see [the clone-to-satisfy-the-borrow-checker
anti-pattern](anti-pattern-clone-to-satisfy-borrow-checker.md)).
`mem::take` gets the same owned value out for free — no allocation, no
copy — because it's a genuine move, not a duplication.

`Option::take` is `mem::take` specialized to `Option<T>` (its default is
always `None`), which is why it needs no `T: Default` bound of its own —
reaching for `mem::take` on a bare `Option` field is equivalent to
calling `.take()` directly, so the general-purpose function is really
reserved for fields that aren't already `Option` (a `Vec`, a `String`, an
enum with a cheap default variant).

## Basic usage example

```
use std::mem;

struct Batch {
    items: Vec<u32>,
}

let mut batch = Batch { items: vec![1, 2, 3] };
let taken = mem::take(&mut batch.items); // <- moves the Vec out, leaves an empty Vec::default() behind
println!("{:?} {:?}", taken, batch.items); // [1, 2, 3] []
```

## Best practices & deeper information

### Scenario: Modifying an existing object

A batch processor needs to drain its pending items and hand them to a
worker while leaving `self` in a valid, empty state — `mem::take` moves
the `Vec` out without cloning it and without ever leaving the field
uninitialized.

```
use std::mem;

struct BatchQueue {
    pending: Vec<String>,
}

impl BatchQueue {
    fn push(&mut self, item: String) {
        self.pending.push(item);
    }

    fn drain(&mut self) -> Vec<String> {
        mem::take(&mut self.pending) // <- moves `pending` out, replaces it with Vec::default() (empty)
    }
}

let mut queue = BatchQueue { pending: Vec::new() };
queue.push("order-1".to_string());
queue.push("order-2".to_string());

let batch = queue.drain(); // owns the items; queue.pending is now empty, not uninitialized
println!("{batch:?}");
```

**Why this way:** `mem::take` moves the `Vec` out in one step instead of
cloning it just to satisfy the "every field must hold a value" rule,
which the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/mem-replace.html)
book documents as the idiomatic way to change or extract a value behind
`&mut self` without an unnecessary allocation.

### Scenario: Interior mutability

Swapping the string held inside a `RefCell` for a new one, while getting
the old string back to log it, needs to move the old value out of a
borrowed `&mut String` — exactly what `mem::replace` is for.

```
use std::cell::RefCell;
use std::mem;

struct Session {
    last_message: RefCell<String>,
}

impl Session {
    fn set_message(&self, new_message: String) -> String {
        let mut slot = self.last_message.borrow_mut(); // <- RefMut<String>, derefs to &mut String
        mem::replace(&mut *slot, new_message) // <- swaps in `new_message`, returns the old one by value
    }
}

let session = Session { last_message: RefCell::new("connected".to_string()) };
let previous = session.set_message("processing".to_string());
println!("{previous}"); // "connected"
```

**Why this way:** `RefCell::borrow_mut` only ever gives out a `&mut T`,
never ownership of the `T` itself, so `mem::replace` is the standard way
to pull an owned value out of interior-mutable storage while leaving a
valid replacement behind, per the
[std docs for `mem::replace`](https://doc.rust-lang.org/std/mem/fn.replace.html).

## Explanation (Embedded)

The mechanism is identical, and the "no allocation" property that's a
nice-to-have on a hosted target is close to a non-negotiable one on many
embedded ones: `mem::take` and `mem::replace` live in `core::mem`, need
no allocator, and work exactly the same on a stack-resident struct field
as on a heap-backed one — there's no `#![no_std]` caveat to state here at
all, unlike patterns that need `alloc`. What's genuinely worth
highlighting for embedded code is *where* this shows up most: driving a
state machine forward on `&mut self` without needing every state wrapped
in `Option` first, and without a clone.

A sensor driver modeled as an enum (`Idle`, `Sampling { started_at_ms:
u32 }`, `Done { reading: i16 }`) needs to move the *old* state out of
`&mut self.state` in order to construct the *new* one from data the old
state was holding — exactly the "can't leave a place empty" problem
`mem::take`/`mem::replace` solve. Wrapping the field in `Option<State>`
just to satisfy the borrow checker (`self.state.take()`, …, `self.state
= Some(new_state)`) works, but it means every read of `self.state`
elsewhere in the driver has to handle a `None` case that can never
actually happen in practice — `mem::replace(&mut self.state,
new_state)` sidesteps that entirely by moving the old state out and the
new one in in a single step, keeping the field's type as the enum
itself: no `Option` wrapper, no clone, no allocation.

## Basic usage example (Embedded)

```
use core::mem;

enum SensorState {
    Idle,
    Sampling { started_at_ms: u32 },
}

struct Driver {
    state: SensorState,
}

impl Driver {
    fn start_sampling(&mut self, now_ms: u32) {
        // <- moves the old state out, writes the new one in, no `Option` field needed
        let _old = mem::replace(&mut self.state, SensorState::Sampling { started_at_ms: now_ms });
    }
}

let mut driver = Driver { state: SensorState::Idle };
driver.start_sampling(1_000);
```

## Best practices & deeper information (Embedded)

### Scenario: Modifying an existing object

A sensor driver transitions from `Sampling` to `Done` and needs to carry
the `started_at_ms` timestamp out of the old state to compute an elapsed
time — `mem::replace` moves the whole old variant out by value so its
data can be used, with no clone and no `Option`-wrapped field anywhere in
the struct.

```
use core::mem;

enum SensorState {
    Idle,
    Sampling { started_at_ms: u32 },
    Done { elapsed_ms: u32, reading: i16 },
}

struct Driver {
    state: SensorState,
}

impl Driver {
    fn finish_sampling(&mut self, now_ms: u32, reading: i16) {
        let old = mem::replace(&mut self.state, SensorState::Idle); // <- moves the old variant out; field never sits empty
        self.state = match old {
            SensorState::Sampling { started_at_ms } => SensorState::Done {
                elapsed_ms: now_ms - started_at_ms,
                reading,
            },
            other => other, // no-op transition if called from the wrong state
        };
    }
}

let mut driver = Driver { state: SensorState::Sampling { started_at_ms: 1_000 } };
driver.finish_sampling(1_250, 421);
```

**Why this way:** `mem::replace` gets ownership of `started_at_ms` out of
the old `Sampling` variant without cloning the enum or wrapping `state`
in `Option<SensorState>` just to satisfy the borrow checker — on a
target with no allocator, this is not an optimization over cloning, it's
the only option, since `SensorState` here holds no `Clone` impl and
wrapping every read site in `Option` handling for a state that's always
present would be pure boilerplate; the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/mem-replace.html)
book documents this exact shape as the idiomatic way to change a value
behind `&mut self`.
