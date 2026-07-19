# Rust Concept Inventory

> Master checklist of every concept page to build (§1.2 / §3 of
> [PAGES_DESIGN.md](PAGES_DESIGN.md)). Companion to
> [SYNTAX_INVENTORY.md](SYNTAX_INVENTORY.md) — that file lists every syntax
> *token*; this one lists every language/ecosystem *idea*.
>
> **Sources:** [The Rust Book](https://doc.rust-lang.org/book/),
> [The Rust Reference](https://doc.rust-lang.org/reference/),
> [Rust Design Patterns & Idioms](https://rust-unofficial.github.io/patterns/)
> (this one is where the "Functional Optics," "Typestate," and most of the
> idioms/anti-patterns below come from — it's a great source that doesn't get
> cited as often as the Book).
>
> **How this file works — read this first:** §1 below ("By Language Area")
> is the **master list**. Every concept atom is defined there exactly once,
> with a one-line gloss, grouped the traditional way (Ownership, Types,
> Traits, …). Every other section (§2 onward) is a **different facet** —
> the same atoms, resliced along a different axis (paradigm, task, what's
> unique to Rust, learning stage, prior-language background). An atom can
> and often does appear in several facets and in several groups within a
> facet — per the new decision in PAGES_DESIGN.md §5 (#10), that's not a
> problem: a page just lists every group it belongs to. Facets past §1 only
> list names (no glosses) to avoid repeating §1 six times over — look the
> name up in §1 if you need the description.
>
> Think of §1 as the atom registry and §2–§6 as saved "views" over it —
> exactly like the left sidebar could offer a dropdown: "browse by Area /
> by Paradigm / by Task / by What's-Unique / by Level / by Background."

---

## 1. By Language Area (master list)

### 1.1 Ownership & Borrowing
- [x] Ownership — who owns a value and what that guarantees
- [x] Immutability by default — bindings are immutable unless `mut`
- [x] Borrowing (shared references) — read-only access without taking ownership
- [x] Mutable borrowing — exclusive, temporary write access
- [x] The borrow checker — how the compiler enforces the above at compile time
- [x] Move semantics — ownership transfer instead of implicit copying
- [x] Copy vs Clone — implicit bitwise copy vs explicit deep copy
- [x] Lifetimes — how long a reference is valid for
- [x] Lifetime elision — when you don't have to write lifetimes out
- [x] RAII & the `Drop` trait — resource cleanup tied to scope exit
- [x] Interior mutability (`Cell` & `RefCell`) — mutating through a shared reference safely
- [x] Shared ownership (`Rc` & `Arc`) — reference-counted multi-ownership
- [x] Weak references (`Weak<T>`) — non-owning pointers that break reference cycles
- [x] Smart pointers (`Box<T>`) — heap allocation with owned semantics
- [x] `Deref` & `DerefMut` coercion — smart pointers acting like their target
- [x] Stack vs heap allocation — where data lives and why it matters

### 1.2 Types & Data Modeling
- [x] Structs — named product types
- [x] Tuple structs — structs with positional, unnamed fields
- [x] Unit structs — zero-field marker types
- [x] Enums (algebraic data types) — sum types / tagged unions
- [x] Generics — types and functions parameterized over types
- [x] Type inference — how Rust figures out types without annotations
- [x] Type aliases — naming a type for readability
- [x] The newtype pattern — a one-field tuple struct wrapping another type
- [x] Zero-sized types & `PhantomData` — types that carry meaning, not bytes
- [x] Const generics — types parameterized over values, not just types
- [x] Associated types — a type slot attached to a trait implementation
- [x] Slices — a view into a contiguous sequence
- [x] Arrays vs `Vec` — fixed-size vs growable sequences
- [x] Recursive types (via `Box<T>`) — types that contain themselves, made possible by indirection
- [x] Numeric types & overflow behavior — integer width, over/underflow, checked/wrapping/saturating arithmetic

### 1.3 Traits & Polymorphism
- [x] Traits — shared behavior contracts
- [x] Default trait methods — behavior a trait provides out of the box
- [x] Trait bounds — constraining generics to types that implement a trait
- [x] Supertraits — a trait requiring another trait
- [x] Trait objects & dynamic dispatch (`dyn Trait`) — runtime polymorphism
- [x] Static dispatch & monomorphization — compile-time polymorphism
- [x] Operator overloading (`std::ops` traits) — giving meaning to `+`, `==`, `[]`, …
- [x] Derivable traits (`Debug`, `Clone`, `PartialEq`, …) — auto-generated impls
- [x] Marker traits (`Send`, `Sync`, `Sized`, `Copy`) — traits with no methods, only meaning
- [x] Blanket implementations — implementing a trait for every type matching a bound
- [x] The orphan rule & coherence — why you can't impl foreign traits on foreign types
- [x] Dependency injection via traits/generics — decoupling code from concrete types without a class hierarchy
- [x] Type erasure (`dyn Any` & downcasting) — recovering a concrete type from a trait object at runtime

### 1.4 Functions & Closures
- [ ] Functions — named, reusable blocks of code
- [ ] Expression-oriented language — almost everything evaluates to a value
- [ ] Closures & capturing — anonymous functions that borrow their environment
- [ ] `Fn` / `FnMut` / `FnOnce` — the three ways a closure can use its captures
- [ ] Higher-order functions — functions taking or returning functions
- [ ] Function pointers (`fn` types) — plain, non-capturing function values

### 1.5 Iterators
- [ ] The `Iterator` trait — the core abstraction for sequential access
- [ ] `IntoIterator` (`iter`/`iter_mut`/`into_iter`) — the three ways to start iterating
- [ ] Iterator adaptors (`map`, `filter`, `zip`, `enumerate`, `chain`, `flat_map`, `rev`, `scan`, `peekable`, `take`/`skip`, …) — lazy transformations
- [ ] Iterator consumers (`collect`, `sum`, `fold`, `reduce`, `count`, `for_each`, `any`/`all`, `min`/`max`) — driving iteration to a result
- [ ] Lazy evaluation — adaptors do nothing until consumed
- [ ] Custom iterators — implementing `Iterator` for your own type
- [ ] `FromIterator` & collect targets — how `.collect()` knows what to build

### 1.6 Error Handling
- [ ] `Option<T>` — an explicit "maybe absent" value
- [ ] `Result<T, E>` — an explicit "maybe failed" value
- [ ] The `?` operator (concept angle) — early-return propagation as control flow
- [ ] Panic & unwinding — unrecoverable errors and what happens when they occur
- [ ] Custom error types — designing your own `E` in `Result<T, E>`
- [ ] The `Error` trait — the standard interface error types should implement

### 1.7 Pattern Matching
- [ ] `match` expressions — exhaustive branching on structure
- [ ] `if let` / `while let` — matching a single pattern without full `match`
- [ ] Destructuring — pulling a value apart into its components
- [ ] Match guards — extra `if` conditions on a match arm
- [ ] Exhaustiveness checking — the compiler proving you handled every case

### 1.8 Modules, Crates & Visibility
- [ ] Modules — namespacing and code organization within a crate
- [ ] Crates — the unit of compilation and distribution
- [ ] Workspaces — multiple crates managed together
- [ ] Visibility & privacy (`pub` and friends) — what's exposed outside a module
- [ ] Cargo & `Cargo.toml` *(ecosystem/tooling, not core language)*
- [ ] Dependency management & semver *(ecosystem/tooling, not core language)*

### 1.9 Concurrency & Async
- [ ] Threads (`std::thread`) — OS-level parallel execution
- [ ] Message passing (channels / `mpsc`) — sharing data by sending it, not sharing it
- [ ] Shared-state concurrency (`Mutex`, `RwLock`) — sharing data by locking it
- [ ] `Send` & `Sync` — which types are safe to move/share across threads
- [ ] Async/await — cooperative, non-blocking concurrency
- [ ] Futures — values representing work not yet complete
- [ ] Async runtimes *(ecosystem note: tokio, async-std — not part of std)*

### 1.10 Memory & Unsafe
- [ ] Unsafe Rust — the subset where the compiler trusts you instead of checking
- [ ] Raw pointers (`*const T` / `*mut T`) — pointers without borrow-checker guarantees
- [ ] FFI (foreign function interface) — calling into/from other languages
- [ ] Memory layout & `repr` — controlling how types are laid out in memory
- [ ] The undefined-behavior boundary — what unsafe code must never do

### 1.11 Macros & Metaprogramming
- [ ] Declarative macros (`macro_rules!`) — pattern-matching code generation
- [ ] Procedural macros — macros written as compiler plugins (functions over token streams)
- [ ] Derive macros — `#[derive(...)]`-driven code generation
- [ ] Attribute-like macros — custom `#[attribute]`s that transform an item
- [ ] Function-like macros — proc-macros invoked like `ident!(...)`

### 1.12 Collections & Strings
- [ ] `Vec<T>` — a growable array
- [ ] `HashMap` & `HashSet` — unordered key-based collections
- [ ] `BTreeMap` & `BTreeSet` — ordered key-based collections
- [ ] `String` vs `&str` — owned, growable text vs a borrowed view of text
- [ ] String formatting (`Display`, `Debug`, `format!`) — turning values into text

### 1.13 Testing & Tooling *(ecosystem, flagged as non-core-language)*
- [ ] Unit tests
- [ ] Integration tests
- [ ] Doc tests
- [ ] Benchmarking
- [ ] Clippy & rustfmt
- [ ] Serialization (the `serde` ecosystem) — turning Rust values into/from external data formats

### 1.14 Design Patterns & Idioms *(sourced from the Rust Patterns book)*
- [ ] The builder pattern — constructing complex values step by step
- [ ] The visitor pattern
- [ ] The strategy pattern
- [ ] The command pattern
- [ ] The typestate pattern — encoding state machines in the type system
- [ ] Composition over inheritance
- [ ] On-stack dynamic dispatch — using `dyn Trait` without heap allocation
- [ ] Prefer small crates *(idiom)*
- [ ] Compose structs *(idiom — struct-of-structs over god-structs)*
- [ ] Contain unsafety in small modules *(idiom)*
- [ ] Constructor functions (`new()` convention) *(idiom)*
- [ ] The `Default` trait as idiom
- [ ] `mem::take` / `mem::replace` *(idiom)*
- [ ] Privacy for extensibility *(idiom)*
- [ ] Temporary mutability *(idiom)*
- [ ] Return consumed argument on error *(idiom)*
- [ ] Anti-pattern: cloning to satisfy the borrow checker
- [ ] Anti-pattern: `Deref` polymorphism (faking inheritance)
- [ ] Anti-pattern: `#[deny(warnings)]`

### 1.15 Rust Philosophy & Design Principles
- [ ] Zero-cost abstractions
- [ ] Fearless concurrency
- [ ] Memory safety without a garbage collector
- [ ] The edition system — opt-in breaking changes without fracturing the ecosystem
- [ ] "Make invalid states unrepresentable" — type-driven design

---

## 2. By Programming Paradigm

Rust borrows ideas from several traditions; this facet groups concepts by
*where they come from*, useful for readers arriving with a paradigm
background already in their head.

### 2.1 Functional Programming
- [ ] Closures & capturing
- [ ] Higher-order functions
- [ ] `Fn` / `FnMut` / `FnOnce`
- [ ] The `Iterator` trait
- [ ] Iterator adaptors
- [ ] Iterator consumers
- [ ] Lazy evaluation
- [ ] `FromIterator` & collect targets
- [ ] `Option<T>` / `Result<T, E>` (as near-monadic types)
- [ ] The `?` operator
- [ ] `match` expressions
- [ ] Destructuring
- [ ] Enums (algebraic data types)
- [ ] Immutability by default
- [ ] Expression-oriented language
- [ ] Generics as type classes
- [ ] Functional optics (lenses)

### 2.2 Object-Oriented-ish Patterns
- [ ] Traits (as interfaces)
- [ ] Default trait methods
- [ ] Trait objects & dynamic dispatch
- [ ] Structs (as objects/data)
- [ ] Visibility & privacy (encapsulation)
- [ ] Composition over inheritance
- [ ] The builder pattern
- [ ] The visitor pattern
- [ ] The strategy pattern
- [ ] The command pattern

### 2.3 Systems / Low-Level Programming
- [ ] Unsafe Rust
- [ ] Raw pointers
- [ ] Memory layout & `repr`
- [ ] Stack vs heap allocation
- [ ] Zero-cost abstractions
- [ ] RAII & the `Drop` trait
- [ ] FFI
- [ ] The undefined-behavior boundary

### 2.4 Declarative / Metaprogramming
- [ ] Declarative macros
- [ ] Procedural macros
- [ ] Derive macros
- [ ] Attribute-like macros
- [ ] Const generics
- [ ] "Make invalid states unrepresentable"

### 2.5 Concurrent / Message-Passing
- [ ] Message passing (channels)
- [ ] Shared-state concurrency (`Mutex`, `RwLock`)
- [ ] `Send` & `Sync`
- [ ] Async/await
- [ ] Futures
- [ ] Fearless concurrency

---

## 3. By Task ("what are you trying to do")

Practical, use-case-first grouping — good for a reader who knows *what*
they want to build but not what it's called.

### 3.1 Working with Collections
- [ ] `Vec<T>`
- [ ] `HashMap` & `HashSet`
- [ ] `BTreeMap` & `BTreeSet`
- [ ] Slices
- [ ] Arrays vs `Vec`
- [ ] `String` vs `&str`

### 3.2 Iterating & Transforming Data
- [ ] The `Iterator` trait
- [ ] `IntoIterator`
- [ ] Iterator adaptors
- [ ] Iterator consumers
- [ ] Lazy evaluation
- [ ] Custom iterators
- [ ] `FromIterator` & collect targets

### 3.3 Handling Errors & Failure
- [ ] `Option<T>`
- [ ] `Result<T, E>`
- [ ] The `?` operator
- [ ] Panic & unwinding
- [ ] Custom error types
- [ ] The `Error` trait
- [ ] Return consumed argument on error

### 3.4 Sharing & Mutating Data Safely
- [ ] Interior mutability (`Cell` & `RefCell`)
- [ ] Shared ownership (`Rc` & `Arc`)
- [ ] Weak references
- [ ] Smart pointers (`Box<T>`)
- [ ] Mutex & RwLock
- [ ] Send & Sync
- [ ] `mem::take` / `mem::replace`

### 3.5 Writing Concurrent & Parallel Code
- [ ] Threads
- [ ] Message passing (channels)
- [ ] Shared-state concurrency
- [ ] Send & Sync
- [ ] Fearless concurrency

### 3.6 Writing Async Code
- [ ] Async/await
- [ ] Futures
- [ ] Async runtimes

### 3.7 Structuring a Project
- [ ] Modules
- [ ] Crates
- [ ] Workspaces
- [ ] Visibility & privacy
- [ ] Cargo & Cargo.toml
- [ ] Dependency management & semver
- [ ] Prefer small crates

### 3.8 Testing & Documenting Code
- [ ] Unit tests
- [ ] Integration tests
- [ ] Doc tests
- [ ] Benchmarking
- [ ] Clippy & rustfmt

### 3.9 Interfacing with C / Other Languages
- [ ] FFI
- [ ] Unsafe Rust
- [ ] Raw pointers
- [ ] Memory layout & repr

### 3.10 Writing Generic & Reusable Code
- [ ] Generics
- [ ] Trait bounds
- [ ] Supertraits
- [ ] Const generics
- [ ] Associated types
- [ ] Blanket implementations
- [ ] Generics as type classes

### 3.11 Generating Code / Metaprogramming
- [ ] Declarative macros
- [ ] Procedural macros
- [ ] Derive macros
- [ ] Attribute-like macros
- [ ] Function-like macros

### 3.12 Designing Robust Data Models
- [ ] Structs
- [ ] Enums (algebraic data types)
- [ ] The newtype pattern
- [ ] Zero-sized types & PhantomData
- [ ] "Make invalid states unrepresentable"
- [ ] match expressions / destructuring / match guards
- [ ] The typestate pattern

---

## 4. Unique to Rust

Concepts that don't map cleanly onto a mainstream GC'd or manually-managed
language — the "why is Rust *different*" tour. Good landing page for
skeptical or curious newcomers.

- [ ] Ownership & the borrow checker
- [ ] Lifetimes as a first-class concept
- [ ] Move semantics by default
- [ ] Zero-cost abstractions
- [ ] No null — `Option<T>`
- [ ] No exceptions — `Result<T, E>` & the `?` operator
- [ ] Fearless concurrency (`Send` & `Sync`, compiler-enforced)
- [ ] Traits instead of classical inheritance
- [ ] Exhaustiveness checking in pattern matching
- [ ] RAII & `Drop` paired with no garbage collector
- [ ] Const generics & compile-time guarantees
- [ ] The newtype pattern & the orphan rule
- [ ] Macro hygiene
- [ ] Unsafe as an explicit, opt-in escape hatch
- [ ] The edition system
- [ ] Cargo & crates.io as integrated, language-level tooling
- [ ] Memory safety without a garbage collector
- [ ] "Make invalid states unrepresentable"

---

## 5. By Technique / Topic

Not a tutorial path — this is a reference wiki, not a course. This facet is
a flat set of specific, individually-named techniques and topics, the kind
of term you'd search for directly ("multithreading," "boxing,"
"decoupling") rather than a broad umbrella or a suggested reading order.
Groups here are intentionally narrow — some point at just one or two atoms
— because the point is precise, direct navigation to a named idea, not
a curated learning arc.

### 5.1 Multithreading
- [ ] Threads
- [ ] Send & Sync
- [ ] Shared-state concurrency (Mutex, RwLock)
- [ ] Fearless concurrency

### 5.2 Message Passing
- [ ] Message passing (channels)
- [ ] Threads

### 5.3 Async I/O
- [ ] Async/await
- [ ] Futures
- [ ] Async runtimes

### 5.4 Boxing
- [ ] Smart pointers (Box<T>)
- [ ] Stack vs heap allocation
- [ ] Recursive types (via Box<T>)

### 5.5 Reference Counting
- [ ] Shared ownership (Rc & Arc)
- [ ] Weak references

### 5.6 Interior Mutability
- [ ] Interior mutability (Cell & RefCell)
- [ ] mem::take / mem::replace

### 5.7 Decoupling
- [ ] Traits (as interfaces)
- [ ] Trait bounds
- [ ] Trait objects & dynamic dispatch
- [ ] Dependency injection via traits/generics
- [ ] Composition over inheritance
- [ ] Blanket implementations

### 5.8 Encapsulation
- [ ] Visibility & privacy
- [ ] Modules
- [ ] Privacy for extensibility

### 5.9 Composition
- [ ] Compose structs
- [ ] Composition over inheritance
- [ ] Structs

### 5.10 Polymorphism
- [ ] Trait objects & dynamic dispatch
- [ ] Static dispatch & monomorphization
- [ ] Generics
- [ ] Operator overloading

### 5.11 Type Erasure
- [ ] Type erasure (dyn Any & downcasting)
- [ ] Trait objects & dynamic dispatch

### 5.12 State Machines
- [ ] The typestate pattern
- [ ] Enums (algebraic data types)

### 5.13 Builders & Object Construction
- [ ] The builder pattern
- [ ] Constructor functions (new() convention)
- [ ] The Default trait as idiom

### 5.14 Error Propagation
- [ ] Result<T, E>
- [ ] The ? operator
- [ ] Custom error types
- [ ] The Error trait

### 5.15 Generic Programming
- [ ] Generics
- [ ] Trait bounds
- [ ] Const generics
- [ ] Associated types
- [ ] Generics as type classes

### 5.16 Macros & Code Generation
- [ ] Declarative macros
- [ ] Procedural macros
- [ ] Derive macros
- [ ] Attribute-like macros
- [ ] Function-like macros

### 5.17 FFI / Interop
- [ ] FFI
- [ ] Unsafe Rust
- [ ] Raw pointers
- [ ] Memory layout & repr

### 5.18 Testing
- [ ] Unit tests
- [ ] Integration tests
- [ ] Doc tests
- [ ] Benchmarking

### 5.19 Serialization
- [ ] Serialization (the serde ecosystem)
- [ ] Derivable traits

### 5.20 Move Semantics
- [ ] Move semantics
- [ ] Copy vs Clone

### 5.21 Lifetime Management
- [ ] Lifetimes
- [ ] Lifetime elision

### 5.22 Pattern Matching
- [ ] match expressions
- [ ] Destructuring
- [ ] Match guards
- [ ] Exhaustiveness checking

### 5.23 Recursive Data Structures
- [ ] Recursive types (via Box<T>)
- [ ] Enums (algebraic data types)

### 5.24 Numeric Safety
- [ ] Numeric types & overflow behavior

### 5.25 String Handling
- [ ] String vs &str
- [ ] String formatting

### 5.26 Collections
- [ ] Vec<T>
- [ ] HashMap & HashSet
- [ ] BTreeMap & BTreeSet
- [ ] Slices
- [ ] Arrays vs Vec

---

## 6. Coming From Another Language

A comparative facet: same atoms, framed as "here's the Rust answer to what
you already know." Lighter-weight than the others — likely one landing
page per source-language cluster rather than one page per bullet.

### 6.1 Coming from Python / JavaScript
- [ ] Ownership & the borrow checker (the biggest adjustment)
- [ ] No null — Option<T>
- [ ] Result<T, E> & `?` (vs try/except)
- [ ] Type inference (static typing without Python/JS dynamism)
- [ ] Immutability by default
- [ ] Iterator adaptors (vs list/array comprehensions)

### 6.2 Coming from Java / C#
- [ ] Traits instead of classical inheritance
- [ ] Trait objects & dynamic dispatch (vs interfaces)
- [ ] No null — Option<T>
- [ ] Ownership & the borrow checker (vs garbage collection)
- [ ] Enums (ADTs) (vs Java/C# enums being much weaker)
- [ ] Visibility & privacy (vs access modifiers)

### 6.3 Coming from C / C++
- [ ] Ownership & the borrow checker (compile-time enforced vs manual/RAII discipline)
- [ ] RAII & the `Drop` trait (shared heritage with C++)
- [ ] Move semantics by default (vs C++ copy-by-default + explicit `std::move`)
- [ ] Smart pointers (Box/Rc/Arc vs unique_ptr/shared_ptr)
- [ ] Unsafe Rust & raw pointers (the equivalent escape hatch)
- [ ] Zero-cost abstractions (shared philosophy)
- [ ] Const generics (vs templates)

### 6.4 Coming from Haskell / functional languages
- [ ] Traits as type classes
- [ ] Enums (ADTs)
- [ ] Option<T> / Result<T, E> as Maybe / Either
- [ ] Iterator adaptors as list functions
- [ ] Pattern matching & destructuring
- [ ] Generics as type classes
- [ ] The known gap: no higher-kinded types *(worth an honest page on this limitation)*

---

## Summary

| Facet | Groups | Notes |
|-------|--------|-------|
| 1. By Language Area | 15 | Master list — every atom defined here once |
| 2. By Paradigm | 5 | Functional / OO / Systems / Declarative / Concurrent |
| 3. By Task | 12 | Practical, use-case-first navigation |
| 4. Unique to Rust | 1 (flat) | Marketing-friendly "why Rust" tour |
| 5. By Technique / Topic | 26 (flat) | Specific, individually-named techniques — not a reading order |
| 6. Coming From Another Language | 4 | Lightweight, comparative, likely fewer dedicated pages |

**~104 unique concept atoms**, resliced across **6 facets / ~60 groups**
total. Every facet after §1 is a *view*, not new content — no atom should
need a new page just because it shows up in a new facet group. When a truly
new idea surfaces while filling in a facet (as happened with §1.14 Design
Patterns & §1.15 Philosophy, both pulled in while building the "Unique to
Rust" and "Functional Programming" facets), add it back to §1 first, then
reference it from wherever else it belongs.

This maps to PAGES_DESIGN.md §5 decision #10 (many-to-many group
membership) and feeds §4.7 (phasing) the same way SYNTAX_INVENTORY.md does
— pick a handful of atoms across a few facets for the first vertical slice
rather than trying to seed all ~100 at once.

---

*This is a living checklist. Check items off as pages are created; add rows
to §1 first if something new turns up while building out a facet.*
