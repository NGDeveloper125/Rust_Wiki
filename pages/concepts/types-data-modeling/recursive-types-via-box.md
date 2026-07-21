---
title: "Recursive types (via Box<T>)"
area: "Types & Data Modeling"
embedded_support: partial
groups: ["Types & Data Modeling", "Boxing", "Recursive Data Structures"]
related_syntax: []
see_also: ["Smart pointers (Box<T>)", "Enums (algebraic data types)"]
---

## Explanation

A type that contains itself directly — a linked-list node holding another
node, a tree node holding child nodes of the same type — can't be
represented as-is, because the compiler needs to know a type's exact size
at compile time, and a type containing itself would have to be infinitely
large to compute that: an enum like `Cons(i32, List)` would need `List`
to already know its own size before it could compute it.

Wrapping the recursive occurrence in [`Box<T>`](../ownership-borrowing/smart-pointers-box.md)
breaks the infinite-size problem: for any sized pointee, a `Box` is one
pointer-sized value regardless of what it points to (a trait object or
slice makes it a two-word fat pointer, but `Box<List>` here points to a
sized `List`), so `List` above has a fixed, computable size (an enum
discriminant plus the larger of its variants, one of which is just a
pointer) even though it logically contains an unbounded chain of itself. This is the standard, idiomatic
way to write linked lists, trees, and other self-referential data
structures in Rust — the indirection through the heap is exactly what
makes the recursion possible to represent at all.

## Basic usage example

```
enum List {
    Cons(i32, Box<List>),
    Nil,
}
use List::{Cons, Nil};

let list = Cons(1, Box::new(Cons(2, Box::new(Nil)))); // <- Box gives each Cons a fixed, known size
```

**Restriction:** each level of nesting is a real heap allocation, and
dropping a very long chain recurses one stack frame per element — an
extremely deep list can overflow the stack on drop, which is why
production code often reworks deeply recursive structures into an
iterative form instead.

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

Matching directly on a `Box`-recursive enum reads no differently than
matching any other enum — each arm binds `rest: &Box<List>`, and the
recursive `sum(rest)` call deref-coerces the `&Box<List>` to `&List` at
each step.

```
enum List {
    Cons(i32, Box<List>),
    Nil,
}
use List::{Cons, Nil};

fn sum(list: &List) -> i32 {
    match list { // <- recurses through the Box at each step, one match arm per shape
        Cons(value, rest) => value + sum(rest), // <- rest: &Box<List>, matched and recursed into directly
        Nil => 0,
    }
}

let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
println!("{}", sum(&list)); // 6
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch15-01-box.html) walks
through this exact `Cons`-list shape — matching straight through the
`Box` keeps the recursive walk as simple to read as any non-boxed enum,
despite the heap indirection each `Cons` involves underneath.

## Explanation (Embedded)

As [`box`'s embedded section](../../syntax/keywords/box.md) covers,
`Box<T>` lives in `alloc`, not `core`, so `Cons(i32, Box<List>)` above
only compiles once a crate pulls in `alloc` and configures a
`#[global_allocator]` — and there's no `heapless::Box` waiting to fill
the gap the way `heapless::Vec` fills in for `Vec`, because a
fixed-capacity "box" is a contradiction: the whole reason to reach for
`Box` here is that the recursive type's size isn't statically bounded,
while every `heapless` type exists specifically to fix a size at compile
time.

The honest no-heap alternative is therefore a different *design*, not a
substitute type. The standard one is an index-based arena: instead of
each node owning a `Box` pointing to the next/child node, every node
lives in one fixed-capacity array (or `heapless::Vec<Node, N>`), and a
node refers to another node by storing its plain integer index into that
array rather than a pointer. The whole structure — however deep it
logically nests — occupies exactly `N * size_of::<Node>()` bytes, fixed
at compile time, and "removing" or "linking" a node becomes ordinary
index bookkeeping instead of anything the allocator needs to be involved
in. The tradeoff is real: the type system no longer tracks the
ownership/lifetime relationship between nodes the way it does for
`Box` — a stale or out-of-range index is a logic bug the compiler can't
catch the way a dangling `Box` can't exist in the first place — so an
arena's indices need to be validated at the boundary (bounds-checked
indexing, or a generational index scheme) the way any other embedded
input does.

The second, often simpler alternative is to sidestep genuine recursion
altogether: many structures that look naturally recursive (a fixed number
of filter stages, a bounded menu depth, a small protocol nesting limit)
actually have a known maximum depth in practice, in which case a flat
`[T; N]` array replaces the recursive type entirely, with no indices and
no arena bookkeeping at all — the same "is the size really unbounded, or
just currently modeled that way" question the classic Explanation's array-
vs-recursive-structure choice already raises, just pushed one level
further.

## Basic usage example (Embedded)

```
struct Node {
    value: i32,
    next: Option<usize>, // <- index into the arena instead of Box<Node>; None plays Nil's role
}

struct Arena {
    nodes: [Option<Node>; 8], // <- fixed capacity, no heap
    len: usize,
}

impl Arena {
    fn push_front(&mut self, value: i32, next: Option<usize>) -> usize {
        let idx = self.len;
        self.nodes[idx] = Some(Node { value, next });
        self.len += 1;
        idx // <- caller's "pointer" is now just this integer
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Boxing and heap allocation

A linked list of sensor-calibration steps, applied in sequence, needs the
same "each step points to the next" shape a `Box`-based `Cons` list has —
but on a target with no allocator, an index-based arena gives that same
shape without ever calling into `alloc`.

```
struct Step {
    gain: f32,
    next: Option<usize>, // <- plays the role Box<List> plays classically, as a plain index instead of a pointer
}

struct Pipeline {
    steps: [Option<Step>; 4],
    head: Option<usize>,
}

fn apply(pipeline: &Pipeline, mut value: f32) -> f32 {
    let mut current = pipeline.head;
    while let Some(idx) = current {
        if let Some(step) = &pipeline.steps[idx] {
            value *= step.gain;
            current = step.next; // <- walks the chain via indices, no Box, no heap indirection
        } else {
            break;
        }
    }
    value
}
```

**Why this way:** on a target with no `#[global_allocator]` configured —
common for the smallest microcontrollers — `Box<List>` simply doesn't
compile, and there's no `heapless::Box` to reach for instead, since a
fixed-capacity box is a contradiction in terms; an index-based arena is
the standard way the embedded ecosystem gets the same "each element
points to the next" shape without ever touching a heap.

### Scenario: Designing a public API

An arena's public API should surface "no more room" as an explicit
`Result`, the same way `heapless::Vec::push` does, rather than silently
overwriting a slot or panicking on an out-of-bounds arena index.

```
struct Arena {
    nodes: [Option<Node>; 8],
    len: usize,
}

struct Node { value: i32, next: Option<usize> }

impl Arena {
    fn try_push(&mut self, value: i32, next: Option<usize>) -> Result<usize, i32> {
        if self.len == self.nodes.len() {
            return Err(value); // <- arena full: explicit failure instead of a silent overwrite or panic
        }
        let idx = self.len;
        self.nodes[idx] = Some(Node { value, next });
        self.len += 1;
        Ok(idx)
    }
}
```

**Why this way:** an arena's fixed capacity is exactly the same kind of
compile-time bound a `heapless::Vec<T, N>` enforces, so it deserves the
same API discipline — reporting exhaustion as a `Result` the caller must
handle, rather than a bug that only surfaces once the arena happens to
fill up in the field.
