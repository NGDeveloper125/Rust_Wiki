# Design Prompt — "Rusty Yellow Pages"

> Paste the block below into Claude (an artifact/design chat) to generate
> the site design. It's self-contained. Tweak the hex values or the sample
> page if you want a different starting point.

---

Design a single, self-contained, responsive HTML mockup for a documentation
website called **Rusty Yellow Pages**. Output one HTML file with all CSS (and
any minimal JS) inlined — no external assets, fonts, or CDNs. It must render
correctly offline and work in both a **dark mode** and a **light mode** with
a visible toggle.

## What Rusty Yellow Pages is (the goal)

Rusty Yellow Pages is a free, open-source, deep reference for the Rust
programming language — think of it as a *directory you look things up in*
(hence "Yellow Pages"): comprehensive, alphabetized-by-topic, one page per
thing, densely cross-linked. It aims to cover the language **top to bottom**
with far more depth and worked examples than the official book, in a clean,
fast, professional format.

There are exactly **two kinds of pages**, and they cross-link to each other:

- **Syntax pages** — one page for *every* syntactic element of the language:
  every keyword (`fn`, `let`, `mut`, `loop`…), operator and sigil (`&`, `*`,
  `+=`, `<<`, `?`…), punctuation (`{ }`, `->`, `;`…), literal form (hex,
  byte strings, raw strings…), and comment form. Nothing is too small to get
  its own page.
- **Concept pages** — one page per language idea: ownership, borrowing,
  lifetimes, traits, generics, enums, smart pointers, error handling,
  concurrency, and so on.

A syntax page links to the concepts it participates in; a concept page links
to the syntax it uses. The design should make this "jump between the token
and the idea" feel effortless.

## Layout — three regions

```
+---------------------------------------------------------------+
|  TOP BAR:   [ RUST YELLOW PAGES logo ]    [ search... ]   [☀/☾]|
+----------------+----------------------------------------------+
|                |                                              |
|  LEFT SIDEBAR  |            MAIN CONTENT                       |
|                |                                              |
|  Nav tree:     |   The current page. Fixed section order:     |
|  collapsible   |     1. Explanation                           |
|  area headers, |     2. Basic usage example                   |
|  drilling into |     3. Best practices & deeper information    |
|  page links    |     4. Embedded Rust Notes                   |
|                |                                              |
|                |   [ Classic Rust | Embedded Rust ] toggle    |
+----------------+----------------------------------------------+
```

- **Top bar (full width):** the "Rusty Yellow Pages" wordmark on the left, a
  prominent **centered search box** (typing shows a live dropdown of matching
  pages), and a **light/dark toggle** on the right.
- **Left sidebar:** an expandable/collapsible navigation tree. Top-level
  groups (Syntax → Keywords / Operators / Literals / Punctuation / Comments;
  Concepts → Ownership & Borrowing / Types & Data Modeling / Traits &
  Polymorphism / …). Collapsed at the top level by default; clicking a group
  expands it to reveal individual page links. Show a clear "active page"
  highlight. On narrow screens this collapses into a hamburger menu.
- **Main content:** the current page, given the full remaining width. Comfortable
  reading measure (~70–80ch max), generous line-height, syntax-highlighted
  code blocks.

## Color system (this is the core of the brand — get it exact)

Two independent color relationships, and they invert between modes:

**Chrome — the top bar and the left sidebar — is metallic dark grey + yellow:**
- **Dark mode:** metallic **dark grey background**, **yellow** text, icons,
  logo, and active highlights. Give the grey a subtle brushed-metal / soft
  vertical gradient + a faint 1px highlight edge so it reads "metallic," not
  flat.
- **Light mode:** the two **swap** — **yellow background**, **metallic dark
  grey** text/icons/logo.

**Main content pane is dark turquoise + white:**
- **Dark mode:** deep **dark turquoise background**, **white / off-white**
  body text.
- **Light mode:** the two **swap** — **white / off-white background**, **dark
  turquoise** text.

Yellow is the shared brand accent — use it for links, the active nav item,
and section-heading accents inside the content area too, so the chrome and
the content feel like one system.

Suggested starting palette (refine for contrast, don't treat as final):
- Metallic dark grey: `#2b2f36` base with a `#3a4048`→`#23262b` gradient, hairline highlight `#565c66`
- Yellow: a confident, professional gold — `#F5C518` (not neon/lemon)
- Dark turquoise: `#0d3b3d` (dark-mode bg) / `#0a2c2e` deeper for code blocks
- Off-white text: `#eef3f2`

**Accessibility is non-negotiable:** every text/background pair must meet
WCAG AA (4.5:1). Yellow-on-grey and white-on-turquoise are fine; watch
yellow-on-white in light mode (darken the grey text, not the yellow bg) and
turquoise-on-white contrast.

## Page content structure (build the mockup around a real page)

Every page has the same four sections in this order. Populate the mockup with
this **real example page** so it looks authentic, not lorem-ipsum:

---

**Page title:** `&` (the reference / borrow operator)
*Breadcrumb:* Syntax › Operators › `&`
*Related concepts (chips/links):* Borrowing (shared references) · Mutable borrowing · Operator overloading
*Related syntax:* `&mut` · `*` · `&&`

**## Explanation**
`&` has two unrelated meanings, separated by position. As a prefix, `&expr`
produces a shared reference to a value without taking ownership of it; as a
binary operator, `a & b` is bitwise AND. The prefix/borrow use is by far the
more common in everyday Rust.

```rust
let x = 5;
let r = &x; // `&` borrows `x`, producing a shared reference `&i32`
```

**## Basic usage example**
```rust
fn print_len(s: &str) {   // borrows, doesn't take ownership
    println!("{}", s.len());
}
let owned = String::from("hello");
print_len(&owned);        // `&` creates the reference passed in
println!("{owned}");      // still usable — borrowing never moved it
```

**## Best practices & deeper information**
*(This section is a set of "scenario" cards — real situations, each with a
worked example and a short "why this way" rationale. Show two or three:)*

- **Scenario: Sharing data with multiple references** — any number of `&T`
  shared borrows can coexist, so pass `&T` around instead of cloning.
- **Scenario: Multi-threading** — `std::thread::scope` lets threads borrow
  stack data with plain `&`, no `Arc` needed.
- **Scenario: Designing a public API** — accept `&T` rather than an owned
  `T` when a function only needs to read its argument.

**## Embedded Rust Notes**
Full support for both meanings. Borrowing is core-language; bitwise AND lives
in `core::ops` and register-mask work (`status & FLAG_BIT`) is one of the
most common operations in embedded code.

---

Render syntax-highlighted code blocks (dark turquoise pages want slightly
darker code panels with a subtle border). Style the "scenario" items as
distinct cards or clearly delimited blocks. Style the related-concept/syntax
links as small pill/chip links.

## Interactions to show

- **Light/dark toggle** in the top bar that flips both color relationships at
  once (chrome grey↔yellow, content turquoise↔white). Make the toggle itself
  obvious (sun/moon).
- **Sidebar** groups expand/collapse on click; current page highlighted.
- **Search box** shows a live results dropdown on focus/typing (mock 3–4
  results is fine).
- **Classic Rust / Embedded Rust toggle** near the top of the content: a
  two-option segmented control. In "Embedded" state, visually emphasize the
  "Embedded Rust Notes" section (the other three sections stay the same).

## Quality bar

Clean, calm, and professional — a reference tool people trust, closer to
well-made developer documentation than a flashy marketing site. Confident use
of whitespace, a clear typographic hierarchy, a readable monospace font for
code, and smooth, restrained transitions on the mode toggle and sidebar. No
clutter. It should look great in both modes with no contrast or legibility
compromises.
