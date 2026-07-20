---
title: "Dependency injection via traits/generics"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Decoupling"]
related_syntax: []
see_also: ["Traits", "Trait bounds", "Trait objects & dynamic dispatch (dyn Trait)"]
---

## Explanation

Without a class hierarchy or a dependency-injection framework, Rust
achieves the same decoupling goal — code depending on an abstraction
rather than a concrete implementation — through trait bounds and trait
objects directly: a function can accept `&impl Logger` for a
statically-resolved dependency, or `&dyn Logger` if the concrete type
must vary at runtime.

`run` depends only on "something implementing `Logger`" — a test can pass
in a mock implementation, production code can pass in a real one, and
neither `run` nor `Logger` needs to know which concrete types will ever
implement it. This is the same inversion-of-control idea dependency
injection frameworks in other languages provide via containers and
runtime wiring, but here it's expressed directly in the function
signature and checked entirely at compile time (for `impl Trait`/generic
bounds) or resolved via a simple vtable (for `dyn Trait`) — no separate
framework, reflection, or runtime container needed.

## Basic usage example

```
trait Logger {
    fn log(&self, msg: &str);
}

struct ConsoleLogger;
impl Logger for ConsoleLogger {
    fn log(&self, msg: &str) { println!("{msg}"); }
}

fn run(logger: &impl Logger) { // <- depends on the abstraction, not a concrete logger type
    logger.log("started");
}

run(&ConsoleLogger);
```

## Best practices & deeper information

### Scenario: Serving a web endpoint

An axum handler shouldn't hardcode a concrete database or user store —
depending on a trait lets the handler's dependencies be swapped (real
service, mock, different backend) without touching the handler itself.

```
// [dependencies] axum = "0.8", tokio = { version = "1", features = ["full"] }
use axum::extract::State;
use std::sync::Arc;

trait UserStore {
    fn find_name(&self, id: u32) -> Option<String>;
}

struct AppState {
    users: Arc<dyn UserStore + Send + Sync>, // <- handler depends on the trait, not a concrete store
}

async fn get_user(State(state): State<Arc<AppState>>) -> String {
    state.users.find_name(1).unwrap_or_else(|| "unknown".into())
}
```

**Why this way:** wiring the concrete `UserStore` implementation once, at
startup, and injecting only the trait into handlers keeps request-handling
code decoupled from *which* store backs it — the same inversion-of-control
[axum's docs](https://docs.rs/axum/) build shared application state
around via `State<T>` extractors.

### Scenario: Testing

The same trait seam that decouples a handler from its real dependency in
production lets a test swap in an in-memory mock, with no server, network,
or database involved.

```
trait UserStore { // the same seam the handler above depends on
    fn find_name(&self, id: u32) -> Option<String>;
}

struct MockUserStore;
impl UserStore for MockUserStore { // <- test double implementing the same trait as the real store
    fn find_name(&self, id: u32) -> Option<String> {
        if id == 1 { Some("Ada".into()) } else { None }
    }
}

#[test]
fn returns_known_user_name() {
    let store: Box<dyn UserStore> = Box::new(MockUserStore);
    assert_eq!(store.find_name(1), Some("Ada".into()));
}
```

**Why this way:** because handler code only ever calls through
`UserStore`, this test never starts a server or talks to a real database
— the
[Rust Book's mock-object example](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html#a-use-case-for-interior-mutability-mock-objects)
and the wider DI pattern both rely on the trait boundary being the only
thing production and test code have in common.

## Embedded Rust Notes

**Full support.** No allocator dependency — this is precisely the pattern
`embedded-hal` is built around: application/driver code is generic over
a trait (e.g. a GPIO or SPI trait), decoupled from which vendor's
concrete peripheral implementation is plugged in at the top level.
