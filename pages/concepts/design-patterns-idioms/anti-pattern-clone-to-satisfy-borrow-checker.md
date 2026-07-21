---
title: "Anti-pattern: cloning to satisfy the borrow checker"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Anti-patterns", "Design Patterns & Idioms", "Move Semantics"]
related_syntax: []
see_also: ["Copy vs Clone", "mem::take / mem::replace", "Borrowing (shared references)"]
---

## Explanation

[Copy vs Clone](../ownership-borrowing/copy-vs-clone.md) explains what
`.clone()` does and when duplicating a value is the right call — that's
about `Clone` as a genuine design choice. This page is about its misuse:
reaching for `.clone()` not because the code actually needs a second,
independent copy, but purely to make a borrow-checker error disappear.
The tell is that nothing in the resulting program ever reads the clone as
a *distinct* value — it exists solely because the original was still
borrowed somewhere the compiler could see, and duplicating it was the
fastest way to make that inconvenience go away.

It's tempting for an obvious reason: it always works. Given almost any
"cannot borrow `x` as mutable because it is also borrowed as immutable"
error, inserting `.clone()` somewhere in the chain makes the error vanish
and the code compiles — often within seconds, without the author having
to understand *why* the borrow checker objected in the first place. That
immediacy is exactly the trap: the code now runs, but it's paying a real,
ongoing cost (an allocation, sometimes inside a hot loop, sometimes
turning what should be O(n) into something markedly worse) to avoid a
five-minute investigation into restructuring the borrows.

The correct move is almost always to shrink or reshape the borrow
instead of duplicating the data: end the immutable borrow before the
mutable one begins (often just by moving a `let` binding later, or
wrapping the read in its own small block so it drops before the write),
index by position instead of holding a live reference across the
mutation, or split the borrow across disjoint fields/slices so the two
operations are provably non-overlapping to the compiler. Where an owned
value genuinely does need to move out of a `&mut` place — not be
duplicated, *moved* — [`mem::take`/`mem::replace`](mem-take-and-mem-replace.md)
solves that without allocating at all. A `.clone()` should be the
deliberate last resort once those options are ruled out, not the first
thing reached for.

Not every clone born from a borrow-checker error is this anti-pattern —
sometimes the data genuinely does need to live independently past the
point where the original is still in use, and cloning is the honest
answer. The anti-pattern is specifically clones added reflexively,
without asking whether a second copy was ever actually needed.

## Basic usage example

```
struct Order {
    id: u64,
    total_cents: u64,
}

fn apply_discount(orders: &mut [Order], discount_cents: u64) {
    for order in orders.iter_mut() { // <- one mutable borrow of the whole slice, no separate read borrow needed
        order.total_cents = order.total_cents.saturating_sub(discount_cents);
    }
}

let mut orders = vec![Order { id: 1, total_cents: 500 }, Order { id: 2, total_cents: 1200 }];
apply_discount(&mut orders, 100);
println!("{}", orders[0].total_cents); // 400
```

## Best practices & deeper information

### Scenario: Modifying an existing object

Finding the largest order and then logging that it was processed looks
like it needs an immutable borrow of `self.orders` (to search) and a
mutable borrow of `self` (to log) alive at the same time — cloning the
whole order list "fixes" the resulting compile error, but ending the
search borrow before logging removes the conflict for free.

```
struct Order {
    id: u64,
    total_cents: u64,
}

struct OrderBook {
    orders: Vec<Order>,
    processing_log: Vec<u64>,
}

impl OrderBook {
    fn record_processing(&mut self, id: u64) {
        self.processing_log.push(id);
    }

    fn process_largest(&mut self) {
        // AVOID: cloning the whole Vec just to end the search borrow early
        // let snapshot = self.orders.clone();
        // if let Some(order) = snapshot.iter().max_by_key(|o| o.total_cents) {
        //     self.record_processing(order.id);
        // }

        // PREFER: pull out just the id, which ends the borrow of `self.orders` right away
        let largest_id = self.orders.iter().max_by_key(|o| o.total_cents).map(|o| o.id);
        if let Some(id) = largest_id {
            self.record_processing(id); // <- self.orders is no longer borrowed here
        }
    }
}

let mut book = OrderBook {
    orders: vec![Order { id: 1, total_cents: 500 }, Order { id: 2, total_cents: 1200 }],
    processing_log: Vec::new(),
};
book.process_largest();
println!("{:?}", book.processing_log); // [2]
```

**Why this way:** cloning `self.orders` only to search the clone and
throw it away duplicates every order for no reason — the actual fix is
to extract just the `id`, an owned `u64`, which naturally ends the
borrow on `self.orders` before `record_processing` needs `&mut self`;
the
[Rust Design Patterns' anti-patterns section](https://rust-unofficial.github.io/patterns/anti_patterns/borrow_clone.html)
calls out exactly this reflex and recommends restructuring the borrow
over reaching for `.clone()`.

## Explanation (Embedded)

The anti-pattern is identical, and arguably worse: everything the classic
Explanation says about a reflexive `.clone()` masking a borrow-checker
error applies unchanged in `#![no_std]` firmware, but the cost of getting
it wrong is no longer just "wasted CPU cycles nobody notices." A desktop
process cloning an unnecessary `Vec<u8>` inside a hot loop burns memory
bandwidth and allocator time that all but disappears in a profiler; the
same `.clone()` on an embedded target duplicates a buffer out of a flash-
and-RAM budget measured in kilobytes, on a device with no swap and no
graceful recovery from an out-of-memory condition beyond a hard fault or
a reset. Even where the clone lives entirely on the stack (a
`Copy`-derived struct copied instead of borrowed), it still burns stack
budget that on a constrained target is a small, fixed allocation set at
link time — a reflex that's merely inefficient on a desktop can be the
difference between a firmware image that fits and one that doesn't, or a
task whose stack overflows into a neighboring one.

The fix is the same fix: reshape the borrow — end the read before the
write starts, index instead of holding a live reference, split the
borrow across fields — rather than duplicating the data. Nothing about
embedded changes *how* to fix it; it just raises the stakes of leaving
it unfixed.

## Basic usage example (Embedded)

```
struct SensorFrame {
    samples: [i16; 64], // <- fixed-size buffer: no heap, but still 128 bytes of RAM per copy
}

fn largest_sample(frame: &SensorFrame) -> i16 {
    // PREFER: read through a borrow; no reason to duplicate 128 bytes of samples
    *frame.samples.iter().max().unwrap()
}

let frame = SensorFrame { samples: [0; 64] };
let peak = largest_sample(&frame); // <- one borrow, zero copies
```

## Best practices & deeper information (Embedded)

### Scenario: Modifying an existing object

A driver holds the last full calibration table read from a sensor and
wants to log the largest offset whenever it recalibrates — cloning the
whole table just to end an immutable borrow early would duplicate a
buffer the device's RAM budget can't spare twice.

```
struct CalibrationTable {
    offsets: [i16; 32], // 64 bytes — small on a desktop, real weight on a microcontroller
}

struct SensorDriver {
    calibration: CalibrationTable,
    recalibration_count: u32,
}

impl SensorDriver {
    fn record_recalibration(&mut self) {
        self.recalibration_count += 1;
    }

    fn recalibrate(&mut self) {
        // AVOID: cloning 64 bytes of offsets just to end the search borrow early
        // let snapshot = self.calibration.offsets.clone();
        // let _max = snapshot.iter().max();
        // self.record_recalibration();

        // PREFER: pull out just the owned i16, ending the borrow on `self.calibration` immediately
        let max_offset = self.calibration.offsets.iter().copied().max();
        let _ = max_offset;
        self.record_recalibration(); // <- self.calibration is no longer borrowed here
    }
}

let mut driver = SensorDriver {
    calibration: CalibrationTable { offsets: [0; 32] },
    recalibration_count: 0,
};
driver.recalibrate();
```

**Why this way:** cloning the 64-byte `offsets` array to search it and
throw the copy away doubles a buffer that, on a target with a few
kilobytes of RAM total, competes directly with stack space for
interrupt handlers and other tasks — extracting an owned `i16` instead
ends the borrow just as effectively with zero extra bytes, the same
restructuring the
[Rust Design Patterns' anti-patterns section](https://rust-unofficial.github.io/patterns/anti_patterns/borrow_clone.html)
recommends, just with a resource-constrained reason to take it more
seriously.
