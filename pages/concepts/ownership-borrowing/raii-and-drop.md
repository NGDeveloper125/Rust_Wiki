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
    file: std::fs::File,
}

impl Drop for LogSession {
    fn drop(&mut self) {
        use std::io::Write;
        let _ = self.file.flush(); // <- runs on every exit path: normal return, early return, or panic
    }
}

fn write_entries(session: &LogSession, entries: &[&str]) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = &session.file;
    for e in entries {
        if e.is_empty() {
            return Ok(()); // <- session still gets flushed: Drop runs when it goes out of scope
        }
        writeln!(file, "{e}")?;
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
// [dependencies] sqlx = "0.8", tokio = { version = "1", features = ["full"] }
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

## Embedded Rust Notes

**Full support** — and arguably more central to embedded Rust than to
hosted code. RAII is the idiomatic way embedded HAL crates model
peripheral ownership: a driver struct's `Drop` impl can disable a
peripheral, release a pin back to a default state, or turn off a clock
the instant it goes out of scope, with no OS process teardown to lean on
as a safety net the way a hosted program implicitly has.
