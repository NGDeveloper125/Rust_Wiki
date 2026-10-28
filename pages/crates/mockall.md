---
title: "mockall"
version: "0.15.0"
publisher: "Alan Somers (asomers)"
publisher_url: "https://crates.io/users/asomers"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-28"
summary: "Generates mock implementations of your traits, so a test can stand in for a database or an HTTP client and assert how it was called — including how many times and in what order."
domain: "Testing & benchmarking"
categories: ["development-tools::testing"]
repository: "https://github.com/asomers/mockall"
docs: "https://docs.rs/mockall/0.15.0/mockall/"
---

## Overview

Code that talks to a database, a payment provider or the filesystem is awkward
to test: the real dependency is slow, needs credentials, or cannot be made to
fail on demand. The usual answer is to depend on a trait rather than the
concrete type, and `mockall` generates the test implementation of that trait for
you.

```
use mockall::automock;

#[automock]
pub trait Store {
    fn get(&self, key: &str) -> Option<String>;
}

pub fn lookup_or_default(store: &dyn Store, key: &str) -> String {
    store.get(key).unwrap_or_else(|| "default".to_string())
}

#[test]
fn falls_back_when_the_key_is_missing() {
    let mut store = MockStore::new();

    // Say what the mock should do when called.
    store.expect_get().returning(|_| None);

    assert_eq!(lookup_or_default(&store, "absent"), "default");
}
```

`#[automock]` on a trait generates a `MockStore` struct with an `expect_get`
method for each trait method. You program the mock with expectations, then let
the code under test use it.

**The part that distinguishes a mock from a hand-written stub is the
assertions.** A stub returns canned values; a mock also verifies *how* it was
used, and fails the test in its own `Drop` if the expectation was not met:

```
use mockall::{automock, predicate::eq};

#[automock]
pub trait Cache {
    fn fetch(&self, id: u32) -> Option<String>;
}

#[test]
fn consults_the_cache_exactly_once() {
    let mut cache = MockCache::new();

    cache
        .expect_fetch()
        .with(eq(7u32))   // only for this argument
        .times(1)         // exactly once, or the test fails
        .returning(|_| Some("hit".to_string()));

    assert_eq!(cache.fetch(7), Some("hit".to_string()));
}
```

That `times(1)` is checked when the mock is dropped, so a test that *never*
calls the method fails too — which is how you assert that a cache was consulted
rather than bypassed.

**Put `#[automock]` behind `cfg_attr`.** Written plainly it generates the mock
in release builds too, which is dead weight in your shipped binary. The
idiomatic form is `#[cfg_attr(test, automock)]`, and the mock is then available
only under `cfg(test)`.

**The honest caveat: mocking couples your tests to your implementation.** A test
asserting that `fetch` was called once with `7` will break when you legitimately
restructure the code to batch lookups, even though behaviour is unchanged. That
is a real maintenance cost, and it is why mocks are best reserved for
dependencies that are genuinely hard to use for real — network, clock,
randomness, payment. For a pure function, an in-memory `HashMap`, or anything
cheap and deterministic, a real implementation makes a better test double and
does not break when you refactor.

**Reach for `mock!` rather than `#[automock]`** when the trait is not yours to
annotate — a trait from another crate, or a struct's inherent methods. It
declares the same thing externally at the cost of restating the signatures.

Generic traits, associated types, `async fn` and static methods are all
supported, with caveats the documentation spells out; static methods in
particular need a synchronisation mutex, because their expectations are global
rather than per-instance. MSRV is 1.77.

## When to use it

### Use case: Testing error handling you cannot otherwise trigger

Making a dependency fail on demand.

```
use mockall::automock;

#[derive(Debug, PartialEq)]
pub enum SaveError {
    Rejected(String),
}

#[cfg_attr(test, automock)]
pub trait Database {
    fn insert(&mut self, row: &str) -> Result<u64, String>;
}

pub fn save(db: &mut dyn Database, row: &str) -> Result<u64, SaveError> {
    db.insert(row).map_err(SaveError::Rejected)
}

#[test]
fn wraps_a_database_failure() {
    let mut db = MockDatabase::new();

    // The real database would need a disk full or a constraint violation.
    db.expect_insert()
        .returning(|_| Err("disk full".to_string()));

    assert_eq!(
        save(&mut db, "a row"),
        Err(SaveError::Rejected("disk full".to_string()))
    );
}
```

**Why it fits:** error paths are the least-tested part of most codebases
precisely because provoking them for real is hard — you would have to fill a
disk, revoke a credential or unplug a network. A mock makes the failure a single
line, which turns "we think this error is handled" into a test. This is the
strongest argument for mocking, and it applies even if you avoid mocks
elsewhere.

### Use case: Asserting a sequence of calls

When the order of operations is the thing you care about.

```
use mockall::{automock, Sequence};

#[cfg_attr(test, automock)]
pub trait Transaction {
    fn begin(&mut self);
    fn write(&mut self, data: &str);
    fn commit(&mut self) -> Result<(), String>;
}

pub fn store(tx: &mut dyn Transaction, data: &str) -> Result<(), String> {
    tx.begin();
    tx.write(data);
    tx.commit()
}

#[test]
fn commits_after_writing() {
    let mut tx = MockTransaction::new();
    let mut seq = Sequence::new();

    // Each expectation must be satisfied in this order.
    tx.expect_begin().times(1).in_sequence(&mut seq).return_const(());
    tx.expect_write().times(1).in_sequence(&mut seq).return_const(());
    tx.expect_commit()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|| Ok(()));

    assert!(store(&mut tx, "payload").is_ok());
}
```

**Why it fits:** committing before writing is a bug that returns `Ok` and loses
data, so the ordering *is* the behaviour under test and no return value reveals
it. `Sequence` is the tool for a protocol — open before send before close,
lock before modify before unlock. Use it sparingly for the reason in the
Overview: an ordering assertion is the most implementation-coupled kind there
is, and it will break on a refactor that is actually fine.

### Use case: Standing in for the clock or randomness

Making non-determinism deterministic.

```
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Clock {
    fn now_secs(&self) -> u64;
}

pub fn is_expired(clock: &dyn Clock, issued_at: u64, ttl: u64) -> bool {
    clock.now_secs() >= issued_at + ttl
}

#[test]
fn expiry_is_testable_without_waiting() {
    let mut clock = MockClock::new();
    clock.expect_now_secs().return_const(1_000u64);

    assert!(!is_expired(&clock, 950, 100)); // expires at 1050
    assert!(is_expired(&clock, 800, 100));  // expired at 900
}
```

**Why it fits:** the alternative is `std::thread::sleep` in a test, which makes
the suite slow and flaky at once. Abstracting the clock behind a one-method
trait costs almost nothing in production — one indirection — and makes every
time-dependent branch testable at any instant you like, including ones you could
not otherwise reach, such as a timestamp far in the future. The same reasoning
covers a random number source and a UUID generator.

## API map

One attribute, one macro for the cases it cannot cover, and an expectation
builder.

### Generating mocks

#### `#[automock]`

Generates a mock for a trait you own.

```
use mockall::automock;

// cfg_attr keeps the generated code out of release builds.
#[cfg_attr(test, automock)]
pub trait Notifier {
    fn send(&self, message: &str) -> Result<(), String>;
    fn pending(&self) -> usize;
}

#[test]
fn mock_is_named_after_the_trait() {
    let mut notifier = MockNotifier::new();
    notifier.expect_pending().return_const(0usize);

    assert_eq!(notifier.pending(), 0);
}
```

**When to use it:** on any trait of yours that a test needs to replace. The
generated type is `Mock` plus the trait name, with one `expect_<method>` per
method. It must be applied to the trait *definition*, so for a foreign trait
use `mock!` instead. It also works on an `impl` block, which generates a mock of
a struct's inherent methods — useful but awkward, since the code under test then
has to be generic over something.

#### `mock!`

Declaring a mock for a type you do not control.

```
use mockall::mock;

// A trait from another crate, which you cannot annotate.
pub trait ForeignTrait {
    fn describe(&self) -> String;
}

mock! {
    pub Thing {}            // the mock struct, plus any inherent methods

    impl ForeignTrait for Thing {
        fn describe(&self) -> String;
    }
}

#[test]
fn uses_the_declared_mock() {
    let mut thing = MockThing::new();
    thing.expect_describe().returning(|| "mocked".to_string());

    assert_eq!(thing.describe(), "mocked");
}
```

**When to use it:** for a foreign trait, a struct from another crate, or
anything `#[automock]` cannot be attached to. The cost is restating every
signature you want mocked, which then has to be kept in step with the real one
— a compile error if the real trait changes, which is at least loud. Note the
naming: `mock! { pub Thing {} }` produces `MockThing`.

### Expectations

#### `returning` and `return_const`

Deciding what a call produces.

```
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Repo {
    fn count(&self) -> usize;
    fn name_of(&self, id: u32) -> Option<String>;
}

#[test]
fn return_values() {
    let mut repo = MockRepo::new();

    // A fixed value, cloned on each call.
    repo.expect_count().return_const(3usize);

    // A closure, which receives the arguments.
    repo.expect_name_of()
        .returning(|id| if id == 1 { Some("one".into()) } else { None });

    assert_eq!(repo.count(), 3);
    assert_eq!(repo.name_of(1), Some("one".to_string()));
    assert_eq!(repo.name_of(2), None);
}
```

**When to use it:** `return_const` for a constant, which is shorter and does not
need a closure; `returning` when the result depends on the arguments, which is
how you make a mock behave like a small fake. `return_once` is for a value that
cannot be cloned — it may only be called once, and mockall enforces that.
A method with no expectation set **panics** when called, which is deliberate:
it tells you the code did something the test did not describe.

#### `with` and predicates

Constraining which calls an expectation matches.

```
use mockall::automock;
use mockall::predicate::{always, eq, gt};

#[cfg_attr(test, automock)]
pub trait Limiter {
    fn allow(&self, cost: u32) -> bool;
}

#[test]
fn different_answers_per_argument() {
    let mut limiter = MockLimiter::new();

    // Expectations are matched in the order they were declared.
    limiter.expect_allow().with(eq(0u32)).return_const(true);
    limiter.expect_allow().with(gt(100u32)).return_const(false);
    limiter.expect_allow().with(always()).return_const(true);

    assert!(limiter.allow(0));
    assert!(!limiter.allow(500));
    assert!(limiter.allow(5));
}
```

**When to use it:** whenever one method should answer differently depending on
its input, which is most realistic mocks. The predicates live in
`mockall::predicate` — `eq`, `ne`, `lt`, `gt`, `in_iter`, `str::contains`,
`function` for an arbitrary closure, and `always` as the catch-all. Order
matters: the first expectation whose predicate matches wins, so put the
catch-all last or it will shadow everything after it.

#### `times` and `never`

Asserting how often a call happens.

```
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Mailer {
    fn send(&self, to: &str);
    fn flush(&self);
}

#[test]
fn call_counts_are_checked_on_drop() {
    let mut mailer = MockMailer::new();

    mailer.expect_send().times(2).return_const(());
    mailer.expect_flush().never();

    mailer.send("a@example.com");
    mailer.send("b@example.com");
    // Calling flush() here would panic; not calling send() twice would fail
    // the test when the mock is dropped.
}
```

**When to use it:** `times` to pin an exact count, a range (`times(1..3)`), or
`times(0)`; `never()` as the readable form of zero. The checking happens in
`Drop`, which has one consequence worth knowing: if the test panics for another
reason first, you may see the unmet-expectation panic instead of the real
failure. `checkpoint()` verifies and clears expectations mid-test when you want
the failure reported earlier.

#### `Sequence`

Ordering across expectations.

```
use mockall::{automock, Sequence};

#[cfg_attr(test, automock)]
pub trait Lock {
    fn acquire(&mut self);
    fn release(&mut self);
}

#[test]
fn release_follows_acquire() {
    let mut lock = MockLock::new();
    let mut seq = Sequence::new();

    lock.expect_acquire().times(1).in_sequence(&mut seq).return_const(());
    lock.expect_release().times(1).in_sequence(&mut seq).return_const(());

    lock.acquire();
    lock.release();
}
```

**When to use it:** for a protocol whose order is part of the contract. One
`Sequence` can span several mocks, which is how you assert that the database was
written before the message was published. An expectation joined to a sequence
needs an exact `times`, since "in order" is meaningless without knowing how many
calls to expect.

### Methods needing care

#### Static methods and `async fn`

The two cases with extra rules.

```
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Fetcher {
    // Mocked like any other method.
    async fn fetch(&self, url: &str) -> Result<String, String>;

    // No &self: the expectation is global, not per-instance.
    fn version() -> u32;
}

#[tokio::test]
async fn async_methods_work_normally() {
    let mut fetcher = MockFetcher::new();
    // The closure returns the output type directly, not a future.
    fetcher
        .expect_fetch()
        .returning(|_| Ok("body".to_string()));

    assert_eq!(fetcher.fetch("/").await, Ok("body".to_string()));
}
```

**When to use it:** `async fn` in traits is supported, and the pleasant surprise
is that `returning` takes a closure producing the output value rather than a
future — mockall builds the future around it, so the expectation reads exactly
like a synchronous one. Static methods are the awkward case: with no `self`, their expectations are stored
globally, so two tests setting them will interfere when run in parallel —
mockall generates a `MockFetcher::version_context()` guard and the documented
pattern is a `static Mutex` held for the duration of each such test. If you can
avoid static methods in a mocked trait, do.
