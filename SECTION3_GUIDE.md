# Section 3 Guide — Scenario-Based Best Practices

> This document defines how **Section 3** ("Best practices & deeper
> information", per [PAGES_DESIGN.md](PAGES_DESIGN.md) §1.3/§2.1) is written
> for every page. It is the working spec for the model/author doing the
> writing: the scenario catalog, the per-page relevance rules, the example
> format, the source list, and the QA checklist all live here.
>
> Read this whole file before writing any Section 3 content.

---

## 1. What Section 3 is (and isn't)

Sections 1–2 answer *"what is this and what does it look like?"* Section 3
answers *"how do I use this well in real code?"* — by showing the page's
syntax/concept doing its job inside **recognizable, real-world scenarios**
(spawning threads, serving an endpoint, querying a database, building an
object, mutating through references, implementing a trait, …).

Each page's Section 3 is a set of **scenario blocks**. Every block is one
scenario from the fixed catalog in §3 below, containing one best-practice
example where the page's own syntax/concept is load-bearing and marked
inline — same marking convention as Section 2.

What Section 3 is **not**:

- Not a rehash of Section 2 with more lines. Section 2 is the minimal
  "show me the syntax" example; Section 3 examples earn their length by
  being situated in a scenario.
- Not a tour of *other* pages' concepts. If an example touches a sibling
  concept, link to its page instead of re-explaining it (cross-reference
  rule, PAGES_DESIGN.md §2.2 and §4.2's angle split).
- Not exhaustive. A page gets only the scenarios where it genuinely
  matters (§4). Forcing an irrelevant scenario is a defect, not coverage.
- Not the place for embedded/no_std content — that stays in the
  **Embedded Rust Notes** block (PAGES_DESIGN.md §2.5).

---

## 2. Format contract

Every page gains exactly one new H2, placed **after** "## Basic usage
example" and **before** "## Embedded Rust Notes":

````markdown
## Best practices & deeper information

### Scenario: <exact catalog title>

<1–2 sentence setup: what the situation is and what role THIS page's
syntax/concept plays in it.>

```
<code, ~10–25 lines, page's item marked with // <- or //  ^^^ comments>
```

**Why this way:** <1–3 sentences of best-practice rationale. Cite a
source from §6 when the claim is opinionated or non-obvious.>

### Scenario: <next scenario>
...
````

Hard rules (all learned/proven during the Section 2 pass):

1. **Plain code fences** — three backticks, no `rust` language tag
   (repo-wide convention).
2. **H3 titles must match the catalog exactly** — `### Scenario: ` prefix
   plus the exact title string from §3. This is deliberate: identical
   titles across pages let search/tooling later answer "show me every
   page's Multi-threading example." Never invent a new scenario title
   inline; if a needed scenario is missing, add it to this file first.
3. **Mark the page's item inline** in every code block (`// <- ...` or
   `//  ^^^ ...`). An example where the reader can't instantly find the
   page's own syntax/concept has failed its purpose.
4. **Blank line** between a closing fence and any following `**...:**`
   paragraph.
5. **Anchored edits only** — insert by replacing the
   `## Embedded Rust Notes` heading; everything else in the file stays
   byte-for-byte untouched.
6. **2–4 scenario blocks** is the target per page (minimum 1, hard max 6).
   Order blocks most-central-scenario first.
7. Each block is one code example. A good/bad contrast inside one block is
   allowed (and encouraged where the best practice is "avoid X, prefer Y")
   — mark the halves with `// AVOID:` and `// PREFER:` comments.
8. Realistic domain nouns (`sensor`, `order`, `config`, `user`), never
   `foo`/`bar`.

### Compile policy

PAGES_DESIGN.md §2.3 requires examples to compile (current stable, edition
2024, stated site-wide). For Section 3 this means:

- **Std-only scenarios:** the snippet must be a complete item (fn/impl/
  block) that compiles as-is, or inside an obvious `fn main`.
- **Crate-backed scenarios** (see crate policy, §5): the snippet must be a
  complete, correct item that compiles inside a project with the stated
  dependency. Start the code block with a comment naming it, e.g.
  `// [dependencies] axum = "0.8", tokio = { version = "1", features = ["full"] }`.
  Elide only genuinely irrelevant plumbing, and only with `// ...` — never
  elide the lines the scenario exists to show.

---

## 3. The scenario catalog

Stable IDs (kebab-case, for planning/tracking) and **exact H3 title
strings** (what appears on pages). Grouped for navigation; the grouping
itself doesn't appear on pages.

Source keys in the last column refer to §6.

### 3.1 Ownership & data flow

| ID | H3 title | The example shows | Sources |
|----|----------|-------------------|---------|
| construct | Creating a new object | `new()` constructor convention, builder for many-field types, `Default` | API-GL, PATTERNS |
| modify | Modifying an existing object | `&mut self` methods vs public fields, temporary mutability, `mem::take`/`replace` | PATTERNS, STD |
| share-read | Sharing data with multiple references | passing `&T` around, several live `&T` at once, iterating while reading | BOOK, STD |
| exclusive-mut | Mutating through a reference | `&mut` discipline, borrow-scope shaping, splitting borrows (fields/`split_at_mut`) | BOOK, STD |
| transfer | Transferring ownership | moving into functions/collections/threads, returning owned values, consuming `self` | BOOK, API-GL |
| clone-copy | Cloning and copying | when `.clone()` is right vs a borrow-checker workaround; deriving `Copy` correctly | PATTERNS, CLIPPY |
| boxing | Boxing and heap allocation | `Box` for large values, recursion, `Box<dyn Trait>`; when NOT to box | STD, PATTERNS |
| shared-own | Shared ownership | `Rc`/`Arc` for genuinely multi-owner data (caches, graphs, config) | BOOK, STD |
| interior-mut | Interior mutability | `Cell`/`RefCell` single-threaded; pointing to `Mutex` for threads | BOOK, STD |
| raii | Managing resources (RAII) | guard types, `Drop` for files/locks/connections, scope-driven cleanup | BOOK, PATTERNS |

### 3.2 Abstraction & APIs

| ID | H3 title | The example shows | Sources |
|----|----------|-------------------|---------|
| impl-traits | Implementing traits | derive vs manual impl; which std traits to implement (`Debug`, `Display`, `PartialEq`, …) | API-GL, STD |
| generic-fn | Writing generic code | generic fns/types with bounds, `where` clauses, `impl Trait` in argument/return position | BOOK, API-GL |
| dyn-poly | Runtime polymorphism | heterogeneous `Vec<Box<dyn Trait>>`, plugin-style dispatch, `&dyn` without allocation | BOOK, PATTERNS |
| conversions | Converting between types | `From`/`Into`, `TryFrom` for fallible, `AsRef` in APIs | API-GL, STD |
| api-design | Designing a public API | newtypes for domain safety, privacy for invariants, extensibility (`#[non_exhaustive]`, sealed traits) | API-GL, PATTERNS, EFF-RUST |

### 3.3 Robustness

| ID | H3 title | The example shows | Sources |
|----|----------|-------------------|---------|
| errors | Handling and propagating errors | `Result` + `?` pipelines, custom error types (`thiserror` for libs, `anyhow` for apps) | BOOK, API-GL, EFF-RUST |
| validation | Validating input | parse-don't-validate, constructors returning `Result`, invalid states unrepresentable | PATTERNS, EFF-RUST |
| matching | Branching on data (pattern matching) | exhaustive `match` on enums, `if let`/`let else`, state-machine dispatch | BOOK, REF |

### 3.4 Data processing

| ID | H3 title | The example shows | Sources |
|----|----------|-------------------|---------|
| collections | Working with collections | `Vec`/`HashMap` usage, iterator chains (`map`/`filter`/`collect`), entry API | STD, COOKBOOK |
| text | Working with text | `String` vs `&str` in signatures, formatting, parsing, building strings | STD, API-GL |
| numeric | Numeric computation | checked/wrapping/saturating arithmetic, choosing widths, float pitfalls | STD, CLIPPY |
| bit-ops | Bit manipulation and flags | masks, shifts, flag sets, register/protocol byte packing | STD, RBE |

### 3.5 Concurrency & async

| ID | H3 title | The example shows | Sources |
|----|----------|-------------------|---------|
| threads | Multi-threading | `thread::spawn`/`join`, `thread::scope` for borrowing, `Send`/`Sync` implications | BOOK, STD |
| shared-state | Sharing state across threads | `Arc<Mutex<T>>`/`RwLock`, lock-scope hygiene, poisoning | BOOK, STD |
| channels | Message passing between threads | `mpsc` pipelines, worker pools, ownership transfer through a channel | BOOK, STD |
| async | Async tasks | `async fn`/`.await`, spawning tasks, why blocking in async is a bug | TOKIO |

### 3.6 Application contexts (crate-backed)

| ID | H3 title | The example shows | Sources |
|----|----------|-------------------|---------|
| endpoint | Serving a web endpoint | an axum handler: extractors, shared state, typed responses | AXUM, TOKIO |
| database | Querying a database | sqlx query + row-to-struct mapping, connection pool as shared state | SQLX |
| serialization | Serializing and deserializing | serde derive, JSON/TOML config, field attributes | SERDE |
| testing | Testing | unit tests with `#[test]`, doc tests, testing via trait seams (mock impls) | BOOK, API-GL |
| docs | Documenting an API | rustdoc conventions, runnable examples in docs, intra-doc links | API-GL, RUSTDOC |
| ffi | Crossing an FFI boundary | *(reserved — used when Memory & Unsafe pages are written; not for current groups)* | NOMICON |

---

## 4. Relevance rules — which scenarios a page gets

1. **Only scenarios where the page's item is load-bearing.** Test: could
   you delete the page's syntax/concept from the example and lose the
   point? If the item is just incidentally present, the scenario is
   irrelevant for this page.
2. **Target 2–4 blocks** per page. Some pages honestly support only 1
   (e.g. `true`/`false`); a rare few support 5–6 (e.g. `&`, Ownership).
   Never pad; never force. If a page can't sustain even one honest
   scenario, don't invent one — leave a `TODO` note in the commit message
   and flag it for review.
3. **Differentiate near-duplicate pages.** Sibling pages (the four
   comparison operators; the compound-assignment family; shared vs
   mutable borrowing) must not receive four copies of the same example.
   Either pick different scenarios per sibling, or — for tight families
   like `+=`/`-=`/`&=` — give the *family representative* the full
   treatment and give siblings one scenario plus a "see also" link to the
   representative.
4. **Respect the syntax/concept angle split** (PAGES_DESIGN.md §4.2):
   on a *syntax* page, the scenario example demonstrates writing the
   token well (form, placement, pitfalls); on a *concept* page it
   demonstrates deciding/designing well (when, why, tradeoffs). `&` shows
   how borrow expressions read in a threaded context; the Borrowing
   concept page shows how to *structure* code so borrows stay short.
5. Embedded/no_std relevance goes to **Embedded Rust Notes**, never to a
   Section 3 block.

### Starter map — current 8 page groups

Suggestions, not quotas — the author still applies rule 1 per page.

| Directory | Likely scenarios |
|-----------|-----------------|
| syntax/comments | docs, testing, api-design |
| syntax/keywords | per keyword: `let`→construct/share-read; `mut`→modify/exclusive-mut; `const`→numeric/bit-ops/api-design; `fn`→generic-fn/errors/api-design; `if`/`else`→validation/matching/errors; `for`/`in`→collections/channels; `while`/`loop`→channels/threads (polling, worker loops); `break`/`continue`→collections/matching; `return`→errors/validation; `true`/`false`→validation/matching |
| syntax/literals | integer bases & suffixes→bit-ops/numeric; digit-separator→numeric; float forms→numeric; string/escape/raw→text/serialization; byte & byte-string→bit-ops/text (protocol bytes); c-string & raw-c-string→ffi is reserved, so use text with an FFI-flavored mention only if honest — otherwise one scenario |
| syntax/operators | arithmetic→numeric/collections; comparison→matching/validation/collections (sorting); `==`/`!=`→validation/testing; bitwise & shifts→bit-ops; compound assignment→modify/numeric (family-representative rule); `&`→share-read/exclusive-mut/threads; `*`→boxing/exclusive-mut; `!`→validation/matching; `&&`/`\|\|`→validation/errors |
| concepts/ownership-borrowing | the §3.1 family, plus threads/shared-state/channels where the page is `Arc`-adjacent, endpoint/database for Ownership & Borrowing (state sharing in handlers is *the* real-world borrow scenario) |
| concepts/traits-polymorphism | impl-traits, generic-fn, dyn-poly, conversions, api-design, testing (trait seams for mocking), endpoint (DI via traits) |
| concepts/types-data-modeling | construct, validation, matching, serialization, api-design, collections, numeric, database (row-to-struct mapping for structs/enums) |
| *(future groups)* | Functions & Closures→generic-fn/collections/errors; Iterators→collections/text; Error Handling→errors/validation/endpoint; Concurrency & Async→§3.5 + endpoint/database; Macros→api-design/testing |

---

## 5. Crate policy

Default is **std-only**. Non-std crates are allowed only in these
scenarios, from this blessed list, with the dependency comment required by
§2's compile policy:

| Scenario | Allowed crates |
|----------|---------------|
| async | `tokio` |
| endpoint | `axum`, `tokio` |
| database | `sqlx`, `tokio` |
| serialization | `serde`, `serde_json`, `toml` |
| errors | `thiserror` (libraries), `anyhow` (applications) — but show the std-only custom error where the page is about the mechanism itself |

Everything else: std only. No other crates without updating this table
first. (Embedded substitutes like `heapless` belong in Embedded Rust
Notes per PAGES_DESIGN.md §2.5, not here.)

Version pins to state in dependency comments (bump repo-wide, not
per-page, when they age): `tokio = "1"`, `axum = "0.8"`, `sqlx = "0.8"`,
`serde = "1"`, `thiserror = "2"`, `anyhow = "1"`.

---

## 6. Sources

Cite these in **Why this way** lines whenever a claim is opinionated,
surprising, or numeric. Key → source:

| Key | Source | Use it for |
|-----|--------|-----------|
| BOOK | [The Rust Programming Language](https://doc.rust-lang.org/book/) | canonical explanations, ownership/concurrency idioms |
| RBE | [Rust by Example](https://doc.rust-lang.org/rust-by-example/) | compact runnable example shapes to model after |
| STD | [Standard library docs](https://doc.rust-lang.org/std/) | authoritative method-level guidance; std's own examples are best-practice gold |
| REF | [The Rust Reference](https://doc.rust-lang.org/reference/) | precise semantics backing any "Restriction"-grade claim |
| API-GL | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) | naming, conversions, builders, trait-impl checklists (cite the C-XXXX item) |
| PATTERNS | [Rust Design Patterns](https://rust-unofficial.github.io/patterns/) | patterns, idioms, anti-patterns (already a §1.14 source) |
| CLIPPY | [Clippy lint list](https://rust-lang.github.io/rust-clippy/master/) | each lint is a citable, named best-practice statement |
| NOMICON | [The Rustonomicon](https://doc.rust-lang.org/nomicon/) | unsafe/FFI scenarios only |
| COOKBOOK | [Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/) | app-level recipe shapes |
| TOKIO | [Tokio tutorial](https://tokio.rs/tokio/tutorial) | async scenario ground truth |
| AXUM | [axum docs & examples](https://docs.rs/axum/) | endpoint scenario ground truth |
| SQLX | [sqlx docs](https://docs.rs/sqlx/) | database scenario ground truth |
| SERDE | [serde.rs](https://serde.rs/) | serialization scenario ground truth |
| RUSTDOC | [rustdoc book](https://doc.rust-lang.org/rustdoc/) | docs scenario conventions |
| EFF-RUST | [Effective Rust](https://effective-rust.com/) | item-based best-practice arguments |

Citation form: an inline markdown link inside the **Why this way**
sentence — e.g. `…which the API Guidelines codify as
[C-CONV](https://rust-lang.github.io/api-guidelines/interoperability.html)`.
Uncited opinions are allowed only when uncontroversial.

---

## 7. Execution workflow

Work **group by group, commit per group** — same cadence as the Section 2
pass (comments → punctuation → keywords → literals → operators →
ownership-borrowing → traits-polymorphism → types-data-modeling).

Per group:

1. **Plan first, write second.** Before editing any file in a group, draft
   the scenario assignment for every page in the group (page → list of
   scenario IDs) applying §4 — especially the sibling-differentiation
   rule, which can only be checked group-wide. Keep the plan in the
   session (or scratchpad), not in the repo.
2. Write each page's Section 3 per the §2 contract. Read the whole page
   first — the example must not contradict or duplicate the page's
   Explanation/Section 2 content.
3. Verify (greps below), review a sample diff by eye, then commit the
   group: `Add Section 3 (Best practices & deeper information) to <group> pages`.

Verification greps (run per group before committing):

```
# every page has exactly one Section 3 heading
grep -c "## Best practices & deeper information" pages/<group>/*.md   # all :1

# heading order per file: Explanation < Basic usage < Best practices < Embedded
grep -n "^## " pages/<group>/<file>.md

# every scenario title is a legal catalog title
grep -rh "^### Scenario: " pages/<group>/ | sort -u   # compare against §3 titles

# scenario count per page within 1–6
grep -c "^### Scenario: " pages/<group>/*.md

# no rust-tagged fences
grep -rl '```rust' pages/<group>/

# blank line before bold-paragraphs after fences (spot-check)
grep -rB1 "^\*\*Why this way" pages/<group>/ | grep -A0 '```'   # should be empty
```

Sub-agent guidance (if the writing is delegated): batch ≤15 pages per
agent; give each agent this file's §2–§6 verbatim plus the group's
scenario-assignment plan from step 1; require the agent to read each page
fully before editing; verify everything yourself before committing —
agents drift on formatting details (Section 2 experience: fence tags,
blank lines) and on sibling-differentiation.

---

## 8. Worked reference example

The shape every scenario block should aspire to — this one would live on
the **`mut` keyword** page (syntax angle: writing the token well):

````markdown
### Scenario: Sharing state across threads

`mut` disappears from the signature when state is shared across threads —
the mutability moves into the lock, and bindings of the `Arc` itself stay
immutable.

```
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0)); // note: no `mut` — the Mutex provides mutability
let mut handles = Vec::new();          // <- `mut` needed here: the Vec itself is grown

for _ in 0..4 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        let mut n = counter.lock().unwrap(); // <- `mut` binds the guard for write access
        *n += 1;
    }));
}
```

**Why this way:** declaring the `Arc` binding `mut` would be misleading —
interior mutability means the binding is never reassigned; Clippy flags
unneeded `mut` bindings via
[`unused_mut`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-mut).
````

Note what makes it work: the scenario is real, the page's token is marked
at every load-bearing occurrence, the example teaches something a Section
2 reader doesn't know yet, and the opinion is cited.

---

*Living document. Extend the catalog (§3) and crate table (§5) here first;
never invent scenario titles or crates inline on a page.*
