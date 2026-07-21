---
title: "RAII & the Drop trait"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Systems / Low-Level Programming", "Unique to Rust", "Coming from C / C++"]
related_syntax: []
see_also: ["Ownership", "Memory safety without a garbage collector"]
---

## Explanation

RAII — Resource Acquisition Is Initialization — ties a resource's
lifetime to a value's scope: acquiring the resource happens in a
constructor, releasing it happens automatically when the value is
dropped. Rust inherits this idea directly from C++, and builds it into
the language as the default way *any* resource — heap memory, a file
handle, a mutex lock, a network socket — gets cleaned up.

The mechanism is the `Drop` trait: implementing `fn drop(&mut self)` lets
a type run arbitrary cleanup code the instant its owner goes out of
scope, with no need for the programmer to remember to call it — the
compiler inserts the call automatically at every point a value's owner's
scope ends, including on early returns and (by default) during a panic's
unwinding.

Combined with ownership's single-owner guarantee, this is what lets Rust
promise deterministic, automatic cleanup without a garbage collector:
there's never ambiguity about *when* a value's resources should be
released, because there's never ambiguity about who owns it or when that
owner's scope ends. This is a stricter, more automatic version of the
same discipline C++ programmers already practice by hand with RAII guard
types — Rust just makes the compiler enforce that every type follows it,
rather than relying on the programmer to write correct destructors and
never forget to use them.

## Basic usage example

```
struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        println!("cleaning up");
    }
}

{
    let _g = Guard;
    println!("using resource");
} // <- _g goes out of scope here: drop() runs automatically
```

## Best practices & deeper information

### Scenario: Managing resources (RAII)

A file-writing session wraps an open file and guarantees its buffered
contents are flushed no matter how the enclosing function exits —
including an early `return`.

```
struct LogSession {
    file: std::io::BufWriter<std::fs::File>, // <- BufWriter buffers in userspace, so flushing is meaningful
}

impl Drop for LogSession {
    fn drop(&mut self) {
        use std::io::Write;
        let _ = self.file.flush(); // <- runs on every exit path: normal return, early return, or panic
    }
}

fn write_entries(session: &mut LogSession, entries: &[&str]) -> std::io::Result<()> {
    use std::io::Write;
    for e in entries {
        if e.is_empty() {
            return Ok(()); // <- only the borrow of session ends here; Drop (and the flush) fires at the end of the owning scope
        }
        writeln!(session.file, "{e}")?;
    }
    Ok(())
}
```

**Why this way:** tying cleanup to a value's scope via `Drop` means every
exit path — including ones added later by a maintainer who forgets about
cleanup — runs it automatically, instead of relying on a `finally`-style
block or manual discipline at every return site; the
[Rust Book](https://doc.rust-lang.org/book/ch15-03-drop.html) presents
this as RAII's central guarantee.

### Scenario: Querying a database

A database transaction is represented as a guard value — if the code
returns early or panics before calling `.commit()`, `Drop` rolls the
transaction back automatically instead of leaving it half-applied.

```
// [dependencies] sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }, tokio = { version = "1", features = ["full"] }
use sqlx::PgPool;

async fn transfer_funds(pool: &PgPool, from: i64, to: i64, cents: i64) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?; // <- transaction guard: rolls back on Drop unless committed

    sqlx::query("UPDATE accounts SET balance_cents = balance_cents - $1 WHERE id = $2")
        .bind(cents).bind(from).execute(&mut *tx).await?;
    sqlx::query("UPDATE accounts SET balance_cents = balance_cents + $1 WHERE id = $2")
        .bind(cents).bind(to).execute(&mut *tx).await?;

    tx.commit().await?; // <- only this makes the changes permanent; any earlier `?` drops `tx` first
    Ok(())
}
```

**Why this way:** representing an open transaction as a guard whose
`Drop` rolls back by default means a failed query midway through (any of
the `?`s above) can't accidentally leave a half-applied transfer
committed — sqlx models `Transaction` exactly this way, per the
[sqlx docs](https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html), so
RAII does the safety work a hand-written try/rollback block would
otherwise need to get right manually.

## Explanation (Embedded)

`Drop` is core-language and needs no allocator or `std`, so it runs on a
microcontroller exactly as described above: deterministically, the moment
a value's owner's scope ends, with the compiler inserting the call at
every exit path including early returns. What's genuinely different in
embedded code is *why* that guarantee matters more, not less, than on a
hosted target. A hosted program has an OS underneath it that will reclaim
file descriptors, memory, and locks even if a process is killed outright
— `Drop` is a convenience there, not the last line of defense. Bare-metal
firmware has no such backstop: if a peripheral is left enabled, a GPIO pin
is left driving high, or a critical section is left entered, nothing else
in the system will ever notice or clean it up. `Drop` running
deterministically, with no OS involved, is exactly the guarantee that
makes RAII a safety mechanism on bare metal rather than just tidiness.

This is why HAL crates model peripheral access as guard types so
routinely: a type returned by "enter a critical section" or "start using
this pin as an output" carries the responsibility for undoing that state,
and its `Drop` impl is what runs that undo — on a normal return, an early
return, or unwinding from a panic — without the caller having to remember
a matching "release" call at every possible exit.

## Basic usage example (Embedded)

```
struct CriticalSection;

impl CriticalSection {
    fn enter() -> Self { // <- disables interrupts on construction
        // ... disable interrupts
        CriticalSection
    }
}

impl Drop for CriticalSection {
    fn drop(&mut self) {
        // ... re-enable interrupts, unconditionally
    }
}

fn read_shared_counter() -> u32 {
    let _guard = CriticalSection::enter();
    42 // placeholder for a real read of interrupt-shared state
} // <- _guard drops here: interrupts are re-enabled even on an early return above it
```

## Best practices & deeper information (Embedded)

### Scenario: Managing resources (RAII)

A GPIO pin borrowed as an output for one operation should return to a
known-safe default (input, high-impedance) the instant it's done being
used, even if the function driving it returns early or panics partway
through.

```
struct OutputPin { number: u8 } // stand-in for a HAL output-pin type

struct DriveHigh<'a> { pin: &'a mut OutputPin }

impl<'a> DriveHigh<'a> {
    fn new(pin: &'a mut OutputPin) -> Self {
        // ... set the pin high
        DriveHigh { pin }
    }
}

impl<'a> Drop for DriveHigh<'a> {
    fn drop(&mut self) {
        // ... set the pin back to input/high-impedance, regardless of how the guard's scope ended
        let _ = self.pin.number;
    }
}

fn pulse(pin: &mut OutputPin, should_abort: bool) {
    let _guard = DriveHigh::new(pin);
    if should_abort {
        return; // <- _guard still drops here: the pin is returned to a safe state either way
    }
    // ... hold the pin high for the pulse duration
}
```

**Why this way:** tying "pin driven high" to a guard's scope means every
exit path — success, an early `return`, or a panic — leaves the pin in a
known-safe state without the function needing a matching cleanup call at
each one; this is the same "cleanup runs no matter how the scope ends"
guarantee the
[Rust Book](https://doc.rust-lang.org/book/ch15-03-drop.html) describes
for `Drop` generally, applied here with no OS process teardown available
to fall back on if the guard were forgotten.
