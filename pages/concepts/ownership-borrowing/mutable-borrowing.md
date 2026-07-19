---
title: "Mutable borrowing"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing"]
related_syntax: ["&", mut]
see_also: ["Borrowing (shared references)", "The borrow checker", "Interior mutability (Cell & RefCell)"]
---

## Explanation

A mutable reference (`&mut T`) grants temporary, exclusive write access to
a value without transferring ownership. Unlike a shared reference, only
one `&mut T` to a given value can exist at a time, and while it exists, no
other reference — shared or mutable — to that same value may exist
alongside it.

This exclusivity is the mechanism that rules out data races at compile
time: two threads (or even two pieces of code on the same thread) can
never simultaneously read-while-writing or write-while-writing the same
data, because the compiler statically guarantees a `&mut T` is always
alone. This rule is often summarized as "aliasing XOR mutability" — a
value can have multiple readers *or* one writer, never both at once.

The restriction can feel strict when you genuinely need shared, mutable
access (a cache multiple parts of a program update, a graph with back
edges) — that's precisely the gap
[interior mutability](interior-mutability.md) (`Cell`/`RefCell`, and their
thread-safe counterparts `Mutex`/`RwLock`) exists to fill, by moving the
exclusivity check from compile time to run time in a controlled way.

## Basic usage example

```
let mut v = vec![1, 2, 3];
let r = &mut v; // <- exclusive: no other reference to v may exist while r is alive
r.push(4);
println!("{r:?}");
```

**Restriction:** while `r` is alive, not even the owner (`v`) can be read
or written through another path — this is enforced at compile time.

## Best practices & deeper information

### Scenario: Mutating through a reference

A function that normalizes sensor readings in place only needs write
access, not custody of the collection — `&mut [f32]` says so directly,
instead of taking and returning ownership of the whole `Vec`.

```
fn clamp_readings(readings: &mut [f32], max: f32) {
    for r in readings.iter_mut() { // <- `&mut` lets each element be updated in place
        if *r > max {
            *r = max;
        }
    }
}

let mut readings = vec![12.4, 99.9, 3.1];
clamp_readings(&mut readings, 50.0);
println!("{readings:?}");
```

**Why this way:** taking `&mut [f32]` instead of `Vec<f32>` by value lets
the caller keep ownership of the collection and avoids an unnecessary
move — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/flexibility.html)
favor the signature that asks for the least access a function actually
needs.

### Scenario: Modifying an existing object

Updating a user's profile through a method that takes `&mut self` keeps
related fields consistent with each other, instead of relying on every
call site to update several public fields correctly on its own.

```
struct UserProfile {
    display_name: String,
    login_count: u32,
}

impl UserProfile {
    fn record_login(&mut self, name: Option<&str>) { // <- exclusive access needed to update both fields together
        if let Some(name) = name {
            self.display_name = name.to_string();
        }
        self.login_count += 1;
    }
}

let mut user = UserProfile { display_name: "guest".into(), login_count: 0 };
user.record_login(Some("nimrod"));
```

**Why this way:** grouping related mutations behind one `&mut self`
method keeps the struct's invariants (here, `login_count` always
reflecting a login) enforced in one place — the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/index.html)
book generally favors encapsulated mutation over exposing raw mutable
fields that every caller has to update correctly by hand.

## Embedded Rust Notes

**Full support.** No allocator or `std` dependency. The exclusivity
guarantee is especially valuable for embedded code touching shared
peripheral state from both a main loop and an interrupt handler — it's
part of what a sound embedded Rust design (e.g. RTIC's resource model)
leans on to rule out data races between them at compile time.
