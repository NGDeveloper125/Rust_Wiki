# Rust Wiki — Pages Design & Guidelines

> This document is the single source of truth for how the Rust Wiki is
> structured, how pages are written, and how they link together. It also
> records open problems we need to decide on before/while building.
> Everything here is a guideline we agree to follow so hundreds of pages
> stay consistent.

---

## 1. Vision (as described)

A free, open-source, deep Rust reference deployed as static pages on GitHub
Pages. It aims to cover the language **top to bottom** with far more depth and
examples than the official book, in a format we control.

### 1.1 Layout — three columns

```
+-------------------------------------------------------------------+
|                     TOP BAR: [ search... ]  (centered)            |
+----------------+--------------------------------------------------+
|                |                                                  |
|   LEFT         |               MAIN CONTENT                       |
|   SIDEBAR      |                                                  |
|                |   The page we are currently viewing.             |
|  Expandable    |                                                  |
|  area headers  |   3 sections per page:                           |
|  (nav tree)    |   1. Explanation                                 |
|                |   2. Basic usage example                         |
|                |   3. Best practices / deep                       |
|                |                                                  |
+----------------+--------------------------------------------------+
```

- **Top bar (centered) search:** a search box at the top-center of the page.
  Typing surfaces a live list of pages where the text appears; clicking a
  result opens that page in the main column.
- **Left sidebar:** expandable headers grouping the areas of the wiki (the
  navigation tree). Collapsed by default at the top level; expand to drill in.
- **Main (center):** the current page, given the full remaining width.

### 1.2 Two page groups

Every page belongs to exactly one of two groups:

1. **Syntax pages** — one page per *syntax element* of the language. This means
   **everything** — every keyword, operator, sigil, punctuation mark, attribute,
   and literal form (§4.1). Each syntax page **references the concept pages** it
   participates in, if any.
2. **Concept pages** — one page per language concept (borrowing, boxing,
   functions, traits, ownership, lifetimes, closures, etc.). Each concept page
   **references the syntax pages** for any syntax words shown on it.

The two groups **cross-link bidirectionally** so a reader can jump from a
concept to the syntax it uses, and from a syntax word to the concepts it
belongs to.

### 1.3 Page anatomy — three fixed sections

Every page (both groups) has the same three sections, in order:

1. **Explanation** — the pure, plain explanation of what it is and what it does.
2. **Basic usage example** — the simplest representative, runnable example.
3. **Best practices & deeper information** — idioms, gotchas, scenario-specific
   guidance, performance notes, and anything else worth knowing. Structured as
   scenario blocks from a fixed catalog — see [SECTION3_GUIDE.md](SECTION3_GUIDE.md).

---

## 2. Content model & conventions

These are the rules that keep pages uniform.

### 2.1 Page template

Every page starts from the same skeleton (fields TBD once we pick a stack):

```
Title:            <the syntax word or concept name>
Group:            syntax | concept
Groups:           [sidebar groups this page appears under — many-to-many;
                   a page can belong to multiple groups at once, each shown
                   here rather than picking one "primary" home]
Embedded support: full | partial | none   (drives the Classic/Embedded
                   toggle — see §2.5; disabled in the UI when `none`)
Related concepts: [links]   (on syntax pages)
Related syntax:   [links]   (on concept pages)
See also:         [links to sibling pages]

## Explanation
## Basic usage example
## Best practices & deeper information
## Embedded Rust Notes   (short delta note, see §2.5 — not a rewritten page)
```

### 2.2 Cross-reference rule

- A link between a syntax page and a concept page should exist **on both ends**
  (bidirectional). If page A links to page B, B should link back to A.
- To keep this from rotting (see §4.4), links should be generated/validated
  rather than hand-maintained where possible.

### 2.3 Examples

- Every example must **compile** on a stated Rust edition (see §4.6).
- Prefer minimal examples in the "Basic usage" section; save elaborate ones for
  "Best practices."
- Consider linking each example to the **Rust Playground** so readers can run it.

### 2.4 Tone & sourcing

- "Best practices" is opinionated by nature. Where a claim is non-obvious, cite
  it (std docs, RFCs, Rust reference, clippy lint) so the wiki stays trustworthy
  and reviewable.

### 2.5 Embedded Rust toggle — DECIDED: lightweight delta notes, per-page support status

Every page carries a **Classic Rust / Embedded Rust** toggle in the UI.
Toggling does **not** swap out the three core sections (Explanation / Basic
usage / Best practices) — those stay written for hosted, `std`-available
Rust, unchanged. Instead, each page gets one additional block, **Embedded
Rust Notes**, shown when the toggle is switched to Embedded. This avoids
duplicating explanations for the (large) majority of syntax/concepts that
behave identically in `#![no_std]` — see the "lightweight vs full parallel
page" tradeoff this resolves.

**Front-matter field:** `embedded_support: full | partial | none`
- `full` — behaves identically in `#![no_std]`; toggle enabled.
- `partial` — available with a caveat (typically: needs the `alloc` crate
  plus a `#[global_allocator]`, or an idiomatic embedded substitute like
  `heapless`); toggle enabled.
- `none` — fundamentally requires `std`/an OS (`std::thread`, `std::fs`, a
  hosted async runtime, …); **toggle rendered disabled/grayed out** in the
  UI. The Embedded Rust Notes block is still present even when disabled —
  it explains *why* the toggle is off, so a disabled toggle isn't a dead
  end for the reader.

**Content rule:** the Embedded Rust Notes block is short — a few sentences,
not a rewritten page. It states the support level, the reason for any
caveat, and the idiomatic embedded alternative if one exists
(`heapless::Vec` instead of `alloc::vec::Vec`, a `critical-section`-based
mutex instead of `std::sync::Mutex`, etc.).

---

## 3. Information architecture (draft)

A first-pass grouping for the left sidebar. Not final — a scaffolding to react to.

- **Syntax**
  - Keywords (`fn`, `let`, `mut`, `impl`, `match`, `move`, `dyn`, `async`, …)
  - Operators & sigils (`&`, `*`, `?`, `->`, `=>`, `::`, `..`, `!`, …)
  - Punctuation & delimiters
  - Literals (numeric, string, char, byte)
  - Attributes (`#[derive]`, `#[cfg]`, …)
  - Macros-as-syntax (`macro_rules!`, `println!` form)
- **Concepts**
  - Ownership & borrowing (ownership, borrowing, lifetimes, moves)
  - Types (structs, enums, generics, trait objects, `Box`/smart pointers)
  - Traits & polymorphism
  - Functions & closures
  - Error handling (`Result`, `Option`, `?`, panics)
  - Pattern matching
  - Modules, crates & visibility
  - Concurrency & async
  - Memory & unsafe
  - Macros & metaprogramming

---

## 4. Flagged problems / open questions

These are the things I think could bite us. Each needs a decision; none is a
blocker to starting, but the earlier we settle them the less rework later.

### 4.1 What counts as a "syntax word" — DECIDED: everything
**Decision:** the syntax group covers **every** syntactic element of the
language — every keyword, operator, sigil, punctuation mark, delimiter,
attribute, literal form, and any other character or token that is part of Rust.
Nothing is too small to get its own page. Categories (see §3) are just for
organizing the sidebar, not for excluding anything.

Practical consequence to handle (see §4.11): many tokens can't be filenames or
URLs directly (`?`, `&`, `::`, `#`), so each needs a stable human-readable slug.

### 4.2 Syntax vs. concept — DECIDED: cover both sides in full, by angle
Both groups get the **full, deep 3-section treatment** — we are NOT making
syntax pages shallow stubs. The difference is the **angle**, and the two pages
**point at each other** so the reader can jump between them:

- **Syntax page (e.g. `fn`)** — the token in code: how you write it, where it is
  legal, its exact grammar/forms, what it desugars to, syntax-level gotchas.
  Example items: `fn`, `&`/`&mut`, `move`, `dyn`, `?`.
- **Concept page (e.g. Functions)** — the idea in full: the mental model, why it
  exists, design guidance, tradeoffs, all the deeper info about using it well.

Both pages carry Explanation + Basic usage + Best practices, written from their
own angle. `fn` ⇄ Functions, `&` ⇄ Borrowing, `move` ⇄ Move semantics,
`dyn` ⇄ Trait objects — each pair cross-links both ways.

**Watch-out (still real):** because both sides are deep, we must avoid
*accidental duplication* of the same paragraphs. Guideline: if content is about
**how to write the token**, it lives on the syntax page; if it's about
**the idea/when/why**, it lives on the concept page. When in doubt, put it on
the concept page and link to it from the syntax page. **Status: decided —
enforce the angle split in review.**

### 4.3 Cross-links are not always 1:1 — DECIDED: template allows many/one/none
Clarified: some concepts have no matching keyword ("Ownership" isn't a token),
and some tokens have no concept (`;`). So a page's "Related" section may list
**many** links, **one**, or **none**. Not a problem — the page template simply
allows the Related sections to be empty or hold multiple links. No further
decision needed.

### 4.4 Bidirectional links rot at scale
With hundreds of pages, hand-maintained back-links *will* drift (A links to B,
B forgets A; a page gets renamed and links 404). **Mitigation:** use a build
step / link-checker that validates every internal link and ideally generates the
back-links from a single declaration. This nudges us toward a static-site
generator over hand-written HTML (see §4.8).

### 4.5 Responsive layout on small screens
Layout is now top-center search + left nav + main (search moved out of a right
column — see §1.1). Still need a mobile plan: collapse the left sidebar to a
hamburger on narrow screens; the top-center search stays as a search field/icon.
**Decision needed:** exact mobile breakpoints/behavior (low priority; desktop
first).

### 4.6 Rust edition & version drift — DECIDED: track current + Updates area
**Decision:** target the **current/latest** Rust edition and stable release,
stated site-wide. To manage drift instead of fighting it, the site gets a
first-class **"Updates" area** (see §6) that records what changed with each new
Rust version, so readers see the history of changes per release. Later, a
**maintenance skill** will run periodically, detect changes in Rust
(new releases/editions/stdlib), update affected pages, and append entries to the
Updates area automatically. Until that skill exists, updates are manual.
**Decision needed later:** exact design of the update skill.

### 4.7 Scope is very large — needs phasing
"Cover everything top to bottom" is a big multi-hundred-page effort. Without a
phased plan it stalls. **Proposal:** define a "vertical slice" first — build the
full experience (layout + search + cross-links) for a *small* set of pages
(e.g. `fn`, `let`, `&`, plus Functions, Borrowing) end to end, then mass-produce
once the template is proven.

### 4.8 Raw HTML vs. generated HTML (authoring decision still open)
The three flagged items above (§4.4 link integrity, §4.2 avoiding duplication,
consistency across hundreds of pages) all get *easier* with a static-site
generator (one layout template, generated nav, built-in search, link checking)
and *harder* with hand-written HTML (boilerplate duplicated per page, manual
links, manual search wiring). The output is HTML either way. **Decision still
open — not yet made.**

### 4.9 Search UI — DECIDED: build it properly, no shortcuts
Search now lives top-center (§1.1), which is the conventional, well-supported
place — a search field that opens a live results list of matching pages, click
to open in main. **Decision:** this is a flagship feature of the site; we build
a proper, high-quality search experience rather than settling for a minimal
default. Underlying full-text indexing (e.g. Pagefind/Lunr-class) is fine; the
UI around it is ours to polish.

### 4.11 Tokens can't be filenames/URLs — need a slug map
Because §4.1 includes every operator/sigil, many syntax pages have titles that
are illegal or ugly as file paths and URLs (`?`, `&`, `*`, `::`, `->`, `#[]`,
`|`, `..`). We need a **stable slug table** mapping each token to a readable
URL segment (e.g. `?` → `question-mark`, `&` → `ampersand-reference`,
`::` → `path-separator`, `->` → `arrow-return-type`). The page still *displays*
the real token; only the URL/filename uses the slug. This table must be defined
once and reused so links stay stable. **Decision needed:** own the slug table
early (before mass page creation).

### 4.10 Consistency enforcement
With two authors-worth of pages sharing a strict 3-section shape, we need a
lint/checklist (every page has all 3 sections, a group, valid links, a compiling
example) or drift is inevitable. **Proposal:** a simple page checklist + CI
check.

---

## 5. Decisions log

| # | Question | Decision | Date |
|---|----------|----------|------|
| 1 | Syntax-word taxonomy (§4.1) | **Everything** — every keyword, operator, sigil, punctuation, attribute, literal; nothing excluded | 2026-07-18 |
| 2 | Syntax-vs-concept boundary (§4.2) | **Both sides in full**, differentiated by angle (token vs idea), cross-linked both ways | 2026-07-18 |
| 3 | Search placement (§1.1) | **Top-center**, not a right column | 2026-07-18 |
| 4 | Target Rust edition (§4.6) | **Current/latest**, plus an Updates area + future auto-update skill | 2026-07-18 |
| 5 | Search quality (§4.9) | **Build it properly** — flagship feature, no minimal default | 2026-07-18 |
| 6 | Mobile/responsive behavior (§4.5) | _open (low priority, desktop first)_ | |
| 7 | Authoring: raw HTML vs. generator (§4.8) | _open_ | |
| 8 | Token→slug table (§4.11) | _open (needed before mass page creation)_ | |
| 9 | First vertical slice page set (§4.7) | _open_ | |
| 10 | Sidebar/group membership | **Many-to-many** — a page may belong to multiple groups at once (e.g. a syntax token appearing under two concept areas, or a concept appearing under several taxonomies); no forced single "primary" group. Group names are listed on the page itself | 2026-07-18 |
| 11 | Embedded Rust toggle (§2.5) | **Lightweight delta notes** — the 3 core sections stay hosted-Rust-only; one added "Embedded Rust Notes" block per page, driven by an `embedded_support: full/partial/none` field. `none` disables the toggle in the UI but the block still explains why | 2026-07-18 |
| 12 | Section 3 structure (§1.3) | **Scenario-based** — a fixed catalog of real-world scenarios with stable titles (`### Scenario: …`); each page gets 2–4 blocks for only the scenarios where its item is load-bearing. Catalog, crate policy, sources, and QA rules in [SECTION3_GUIDE.md](SECTION3_GUIDE.md) | 2026-07-19 |

---

## 6. Site sections (top-level areas)

Beyond Syntax and Concepts, the site has these first-class areas:

- **Syntax** — a page per token (§4.1).
- **Concepts** — a page per concept (§1.2).
- **Updates** — the version history area (§4.6). One entry per Rust
  release/edition capturing what changed, so readers can browse the evolution of
  the language and see which wiki pages were affected. Eventually fed by the
  maintenance skill.

---

*This is a living document. Update the decisions log and sections as we lock
choices in.*