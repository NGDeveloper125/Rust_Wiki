---
title: "Weak references (Weak<T>)"
area: "Ownership & Borrowing"
embedded_support: partial
groups: ["Ownership & Borrowing", "Reference Counting", "Sharing & Mutating Data Safely"]
related_syntax: []
see_also: ["Shared ownership (Rc & Arc)"]
---

## Explanation

`Weak<T>` is a non-owning companion to `Rc<T>`/`Arc<T>`: holding a `Weak`
reference to a value doesn't keep it alive and doesn't count toward its
strong-reference count, only its separate weak count. To actually use the
value, you `.upgrade()` a `Weak<T>` into an `Option<Rc<T>>` (or
`Option<Arc<T>>`) — `Some` if the value is still alive, `None` if every
strong owner has already dropped it.

This exists specifically to break reference cycles. Two `Rc`-owned
structures that hold strong references to each other (a parent pointing
at its child, and that child pointing back at its parent) would otherwise
never reach a reference count of zero — each keeps the other alive
forever, a memory leak reference counting can't detect or prevent on its
own. Making one direction of such a cycle a `Weak` reference instead (very
commonly: parents hold strong references to their children — a parent
owns its children — while each child holds only a `Weak` reference back
up to its parent) breaks the cycle
while still letting either side reach the other when needed.

## Basic usage example

```
use std::rc::{Rc, Weak};

let strong = Rc::new(5);
let weak: Weak<i32> = Rc::downgrade(&strong); // <- doesn't count toward strong_count

match weak.upgrade() { // <- must upgrade to access the value; can fail
    Some(v) => println!("{v}"),
    None => println!("value already dropped"),
}
```

**Restriction:** a `Weak<T>` cannot be dereferenced directly — it must be
upgraded via `.upgrade()` first, which returns `None` if every strong
owner has already dropped the value.

## Best practices & deeper information

### Scenario: Shared ownership

A tree where each child needs to reference its parent would create a
reference cycle if that back-reference were a strong `Rc` — the parent
keeps the child alive, the child keeps the parent alive, and neither is
ever freed.

```
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Node {
    name: String,
    parent: RefCell<Weak<Node>>,       // <- back-reference: Weak, doesn't keep the parent alive
    children: RefCell<Vec<Rc<Node>>>,  // forward reference: Rc, parent legitimately owns its children
}

let parent = Rc::new(Node {
    name: "root".into(),
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(Vec::new()),
});

let child = Rc::new(Node {
    name: "leaf".into(),
    parent: RefCell::new(Rc::downgrade(&parent)), // <- doesn't increment parent's strong count
    children: RefCell::new(Vec::new()),
});

parent.children.borrow_mut().push(Rc::clone(&child));

let parent_ref = child.parent.borrow().upgrade(); // <- must upgrade; None if parent were already dropped
if let Some(p) = parent_ref {
    println!("{}'s parent is {}", child.name, p.name);
}
```

**Why this way:** making only one direction of the cycle `Weak`
(conventionally: parents hold strong references down to their children,
the pointer back up to the parent is `Weak`) is what lets `parent`'s strong count reach
zero and free the tree once it's no longer reachable from outside — the
[Rust Book](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)
walks through exactly this parent/child shape as the standard fix for
reference cycles; see
[Shared ownership (Rc & Arc)](shared-ownership-rc-arc.md) for when plain
`Rc` is enough.

## Explanation (Embedded)

`Weak<T>` is built directly on top of `Rc<T>`/`Arc<T>` — it lives in
`alloc`, and needs the same `extern crate alloc;` plus a configured
`#[global_allocator]` that its strong-reference counterparts need. See
[Shared ownership (Rc & Arc)](shared-ownership-rc-arc.md) for that
caveat in full, including the honest point that `Rc`/`Arc` are already
uncommon in embedded code, since embedded rarely has the OS-thread-based
multi-owner use case they were designed for, and interrupt-boundary
sharing is usually better served by a `'static`
`critical_section::Mutex<RefCell<T>>` instead. `Weak` inherits that
unpopularity and then some: it only exists to solve a problem —
reference cycles between `Rc`/`Arc`-owned values — that only arises once
you're already using `Rc`/`Arc` for shared ownership in the first place.
If a project isn't reaching for `Rc` much, it has correspondingly little
need for `Weak` either.

What embedded code needs instead is the same underlying idea `Weak`
provides — a non-owning reference that might not resolve — without the
heap and the refcounting machinery. The closest no-heap analogue is often
just a plain `Option<&'a T>`: instead of a runtime check ("has every
strong owner already dropped this?"), validity is tracked by the
borrow checker at compile time through the lifetime `'a`, so there's
nothing to `.upgrade()` at runtime — the reference either compiles, or it
doesn't live long enough and the compiler rejects it up front. For
longer-lived, more dynamic non-owning references (a task scheduler
handing out handles to entries that can later be removed), the usual
embedded pattern is an index into a fixed-size slab/arena — often paired
with a generation counter so a stale index into a reused slot is detected
as invalid — rather than a pointer at all: "might not resolve" becomes "the
generation stored in the handle doesn't match the slot's current
generation," checked with a plain comparison instead of reference-count
bookkeeping.

## Basic usage example (Embedded)

```
struct Sensor { reading: f32 }

fn last_active<'a>(sensors: &'a [Sensor], last_index: Option<usize>) -> Option<&'a Sensor> {
    // <- Option<&'a T>: "might not resolve" tracked by the borrow checker, not a runtime upgrade()
    last_index.and_then(|i| sensors.get(i))
}
```

## Best practices & deeper information (Embedded)

### Scenario: Shared ownership

A tree of sensor nodes where each child needs to reference its parent
would use `Weak<Node>` on a hosted target to avoid a reference cycle —
without an allocator, the no-heap analogue is a fixed-size arena plus
plain indices, with a generation counter standing in for `Weak`'s
"already dropped" check.

```
const MAX_NODES: usize = 16;

#[derive(Clone, Copy)]
struct NodeHandle { index: u8, generation: u8 } // <- the no-heap analogue of a Weak<Node>

struct NodeSlot {
    name: &'static str,
    generation: u8,
    occupied: bool,
}

struct Arena {
    slots: [NodeSlot; MAX_NODES],
}

impl Arena {
    fn resolve(&self, handle: NodeHandle) -> Option<&NodeSlot> {
        let slot = &self.slots[handle.index as usize];
        // <- the "might not resolve" check: stale handle into a reused/freed slot returns None, like upgrade()
        if slot.occupied && slot.generation == handle.generation {
            Some(slot)
        } else {
            None
        }
    }
}
```

**Why this way:** a fixed-size arena gives every node a compile-time-known
home with no heap allocation at all, and a generation counter recreates
`Weak::upgrade`'s "tell me if this has already gone away" check without
reference counting — the same shape the
[Rust Book's `Weak` chapter](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)
solves with `Rc`/`Weak`, reached here with plain arithmetic instead,
which is the standard no-heap substitute wherever a project can't or
won't configure a global allocator.

### Scenario: Designing a public API

A small task registry that hands out handles to registered tasks, some of
which can later be cancelled and removed, needs an API for "a reference
that might no longer be valid" — an index-based handle communicates that
contract without requiring `alloc` at all.

```
pub struct TaskHandle { index: u8, generation: u8 } // <- public API surface: looks like a lightweight Weak<Task>

pub struct TaskRegistry {
    generations: [u8; 8],
    active: [bool; 8],
}

impl TaskRegistry {
    pub fn is_valid(&self, handle: &TaskHandle) -> bool {
        // <- callers check validity explicitly, same contract as Weak::upgrade().is_some()
        self.active[handle.index as usize] && self.generations[handle.index as usize] == handle.generation
    }
}
```

**Why this way:** documenting the handle as "might not resolve, check
before use" up front — mirroring `Weak<T>`'s contract — sets the right
caller expectations without needing `alloc`, `Rc`, or a refcount at all;
the [API Guidelines](https://rust-lang.github.io/api-guidelines/documentation.html)
favor an API whose type signals its own fallibility over one that panics
on a stale handle and documents "don't do that" as the only safeguard.
