# Rust Syntax Inventory

> Master checklist of every syntax page to build (§4.1 of
> [PAGES_DESIGN.md](PAGES_DESIGN.md) — "everything" means every keyword,
> operator, sigil, punctuation mark, attribute, and literal form). Each
> checked item below becomes exactly one syntax page.
>
> **Sources:** [Rust Reference — Keywords](https://doc.rust-lang.org/reference/keywords.html),
> [Rust Reference — Tokens](https://doc.rust-lang.org/reference/tokens.html),
> [Rust Reference — Attributes](https://doc.rust-lang.org/reference/attributes.html),
> [The Book — Appendix B: Operators and Symbols](https://doc.rust-lang.org/book/appendix-02-operators.html).
>
> **Grouping convention:** big groups below mirror the *concept* areas from
> §3 of PAGES_DESIGN.md (Basics, Ownership, Types, Traits, …) rather than
> pure token-kind. Sub-groups inside each (Keywords / Operators & Sigils /
> Punctuation / Literals / Attributes) are the token-kind split. Many tokens
> are relevant to more than one concept (`&` is both a "basic operator" and
> the core of borrowing); each token gets **one primary home** here (its page
> lives there) and is *cross-linked* from anywhere else it's relevant, per
> the no-duplication rule in §4.2. Primary-home calls below are a first pass
> — reshuffle freely.

---

## 1. Basics

Everything needed before any of the deeper concepts make sense.

### Keywords
- [x] `let` — variable binding
- [x] `mut` — mutable binding
- [x] `const` — compile-time constant
- [x] `fn` — function declaration
- [x] `if` — conditional
- [x] `else` — conditional alternative
- [x] `while` — conditional loop
- [x] `loop` — infinite loop
- [x] `for` — iterator loop
- [x] `in` — loop iterator binding
- [x] `break` — exit loop
- [x] `continue` — skip to next iteration
- [x] `return` — return from function
- [x] `true` — boolean literal
- [x] `false` — boolean literal

### Operators
- [x] `+` — arithmetic addition
- [x] `-` — arithmetic subtraction / unary negation
- [x] `*` — arithmetic multiplication *(also Ownership: dereference — cross-link)*
- [x] `/` — arithmetic division
- [x] `%` — arithmetic remainder
- [x] `==` — equality comparison
- [x] `!=` — inequality comparison
- [x] `<` / `<=` / `>` / `>=` — ordering comparisons
- [x] `&&` — short-circuiting logical AND
- [x] `||` — short-circuiting logical OR *(also Closures: empty-capture closure syntax — cross-link)*
- [x] `!` (prefix) — logical/bitwise complement *(also Macros: `!` invocation — cross-link)*
- [x] `=` — assignment
- [x] `+=` `-=` `*=` `/=` `%=` — arithmetic compound assignment
- [x] `&` `|` `^` — bitwise AND / OR / XOR *(`&` also Ownership: borrow — cross-link)*
- [x] `<<` `>>` — bitwise shifts
- [x] `&=` `|=` `^=` `<<=` `>>=` — bitwise compound assignment

### Punctuation & Delimiters
- [x] `;` — statement terminator
- [x] `,` — argument/element separator
- [x] `:` — type/constraint annotation
- [x] `{ }` — block expression
- [x] `( )` — grouping / tuple *(also Types: tuple expression — cross-link)*
- [x] `[ ]` — array literal/index *(also Types: array/slice — cross-link)*
- [x] `->` — function return type *(also Closures — cross-link)*

### Literals
- [x] Decimal integer literal (`123`, `123_456`)
- [x] Hexadecimal integer literal (`0xff`)
- [x] Octal integer literal (`0o77`)
- [x] Binary integer literal (`0b1010`)
- [x] Integer suffixes (`u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize`)
- [x] Floating-point literal (`1.0`, `1.0E+10`, `2.`)
- [x] Float suffixes (`f32 f64`)
- [x] String literal (`"..."`)
- [x] Raw string literal (`r"..."`, `r#"..."#`)
- [x] Byte literal (`b'H'`)
- [x] Byte string literal (`b"..."`)
- [x] Raw byte string literal (`br"..."`, `br#"..."#`)
- [x] C string literal (`c"..."`)
- [x] Raw C string literal (`cr"..."`, `cr#"..."#`)
- [x] Character literal (`'H'`)
- [x] Escape sequences (`\n \r \t \\ \0 \' \" \xNN \u{NNNN}`)
- [x] Digit separator `_` in numeric literals

### Comments
- [x] `//` — line comment
- [x] `/* */` — block comment
- [x] `///` — outer line doc comment
- [x] `//!` — inner line doc comment
- [x] `/** */` — outer block doc comment
- [x] `/*! */` — inner block doc comment

---

## 2. Ownership & Borrowing

### Keywords
- [ ] `move` — force closure to take ownership *(primary; also Concurrency/Closures — cross-link)*
- [ ] `ref` — bind by reference in a pattern

### Lifetimes
- [ ] `'ident` — named lifetime / loop label
- [ ] `'static` — the static lifetime (weak keyword)
- [ ] `'a: 'b` — lifetime outlives bound
- [ ] `'r#keyword` — raw lifetime (2021+ edition)

### Operators & Sigils
- [ ] `&` — shared borrow / reference type
- [ ] `&mut` — mutable borrow / reference type
- [ ] `*` — dereference *(cross-link from Basics)*
- [ ] `&raw const` — raw borrow (weak keyword `raw`)
- [ ] `&raw mut` — raw mutable borrow

---

## 3. Types & Data Structures

### Keywords
- [ ] `struct` — struct declaration
- [ ] `enum` — enum declaration
- [ ] `union` — union declaration (context keyword)
- [ ] `type` — type alias
- [ ] `as` — type casting

### Operators & Sigils
- [ ] `::` — path/namespace separator *(primary here or Modules — pick one; heavily cross-linked either way)*
- [ ] `<...>` — generic type parameters
- [ ] `::<...>` — turbofish (generics in expression position)
- [ ] `.` — field access
- [ ] `.0` / `.1` — tuple indexing
- [ ] `[...]` — array literal / type
- [ ] `[T; N]` — fixed-size array type/literal
- [ ] `expr[expr]` — indexing (`Index`/`IndexMut`)
- [ ] `expr[..]`, `expr[a..]`, `expr[..b]`, `expr[a..b]` — slicing
- [ ] `()` — unit type/value, tuple type/expression
- [ ] `for<'a> type` — higher-ranked trait bounds (also Traits)
- [ ] `type<ident=type>` — associated-type binding

### Attributes
- [ ] `#[repr(...)]` — control type layout
- [ ] `#[non_exhaustive]` — allow future fields/variants

---

## 4. Traits & Polymorphism

### Keywords
- [ ] `trait` — trait declaration
- [ ] `impl` — implementation block
- [ ] `dyn` — dynamic trait object
- [ ] `where` — trait-bound clause
- [ ] `Self` — current type
- [ ] `self` — current instance (receiver)

### Operators & Sigils
- [ ] `:` — trait bound constraint (`T: U`)
- [ ] `+` — compound trait bound (`Trait + Trait`, `'a + Trait`) *(cross-link from Basics)*
- [ ] `?Sized` — relax implicit `Sized` bound
- [ ] `for<'a> type` — HRTB *(cross-link from Types)*

### Attributes
- [ ] `#[derive(...)]` — automatic trait impl generation
- [ ] `#[automatically_derived]` — marker on derive-generated impls

---

## 5. Functions & Closures

### Keywords
- [ ] `fn` *(cross-link from Basics)*
- [ ] `move` *(cross-link from Ownership)*

### Operators & Sigils
- [ ] `->` — return type *(cross-link from Basics)*
- [ ] `|args| expr` — closure syntax
- [ ] `||` — zero-argument closure form *(cross-link/disambiguation note vs. logical OR)*

---

## 6. Control Flow & Pattern Matching

### Keywords
- [ ] `match` — pattern-match expression
- [ ] `if let` — conditional pattern match
- [ ] `while let` — loop while pattern matches
- [ ] `let else` — refutable let with diverging else

### Operators & Sigils
- [ ] `|` — pattern alternatives (`pat | pat`) *(distinct page from bitwise `|`, cross-linked)*
- [ ] `@` — pattern binding (`ident @ pat`)
- [ ] `..` — rest-of-pattern / range
- [ ] `..=` — inclusive range pattern/expression
- [ ] `...` — deprecated inclusive range pattern (historical note)
- [ ] `_` — wildcard pattern
- [ ] `=>` — match arm separator

---

## 7. Error Handling

### Operators & Sigils
- [ ] `?` — error propagation operator

### Related macros *(pages live in §11, cross-linked here)*
- [ ] `panic!`

---

## 8. Modules, Crates & Visibility

### Keywords
- [ ] `mod` — module declaration
- [ ] `use` — import declaration
- [ ] `pub` — public visibility (incl. `pub(crate)`, `pub(super)`, `pub(in path)` forms)
- [ ] `crate` — crate root
- [ ] `self` — current module (in paths) *(cross-link from Traits, different sense)*
- [ ] `super` — parent module
- [ ] `extern crate` — extern crate declaration (2018+ largely implicit; still valid)

### Operators & Sigils
- [ ] `::` *(cross-link from Types, or primary here — pick one)*
- [ ] `as` — import renaming (`use foo as bar`) *(cross-link from Types)*

### Attributes
- [ ] `#[path = "..."]` — explicit module file path

---

## 9. Concurrency & Async

### Keywords
- [ ] `async` — asynchronous function/block
- [ ] `await` — await an async result
- [ ] `move` *(cross-link from Ownership — async blocks/closures)*

---

## 10. Memory & Unsafe / FFI

### Keywords
- [ ] `unsafe` — unsafe code block/fn/trait
- [ ] `extern` — external function/ABI block
- [ ] `static` — static item / static storage duration
- [ ] `union` *(cross-link from Types)*
- [ ] `safe` — marks a safe fn/static inside an `extern` block (weak keyword)

### Operators & Sigils
- [ ] `*const T` — raw immutable pointer type
- [ ] `*mut T` — raw mutable pointer type
- [ ] `&raw const` / `&raw mut` *(cross-link from Ownership)*

### Attributes
- [ ] `#[no_mangle]`
- [ ] `#[link(...)]`
- [ ] `#[link_name = "..."]`
- [ ] `#[link_ordinal(...)]`
- [ ] `#[link_section = "..."]`
- [ ] `#[no_link]`
- [ ] `#[export_name = "..."]`
- [ ] `#[used]`
- [ ] `#[crate_type = "..."]`
- [ ] `#[crate_name = "..."]`
- [ ] `#[no_main]`
- [ ] `#[naked]`
- [ ] `#[no_builtins]`
- [ ] `#[target_feature(...)]`
- [ ] `#[instruction_set(...)]`
- [ ] `#[panic_handler]`
- [ ] `#[global_allocator]`
- [ ] `#[windows_subsystem = "..."]`
- [ ] `#[no_std]`
- [ ] `#[no_implicit_prelude]`
- [ ] `#[cold]`
- [ ] `#[track_caller]`

---

## 11. Macros & Metaprogramming

### Keywords
- [ ] `macro_rules` — declarative macro definition (weak keyword)
- [ ] `macro` — reserved for future macro 2.0 syntax

### Operators & Sigils
- [ ] `!` — macro invocation marker (`ident!(...)`) *(cross-link from Basics)*
- [ ] `ident!(...)` / `ident!{...}` / `ident![...]` — the three invocation delimiter forms
- [ ] `$ident` — macro substitution variable
- [ ] `$ident:kind` — macro metavariable with fragment specifier
- [ ] `$(...)…` — macro repetition

### Attributes
- [ ] `#[macro_export]`
- [ ] `#[macro_use]`
- [ ] `#[proc_macro]`
- [ ] `#[proc_macro_derive(...)]`
- [ ] `#[proc_macro_attribute]`

### Standard macros (worth their own pages even though library, not language, items)
- [ ] `println!` / `print!` / `eprintln!` / `eprint!`
- [ ] `format!`
- [ ] `vec!`
- [ ] `panic!` *(cross-linked to Error Handling)*
- [ ] `assert!` / `assert_eq!` / `assert_ne!`
- [ ] `todo!` / `unimplemented!` / `unreachable!`
- [ ] `matches!`
- [ ] `write!` / `writeln!`
- [ ] `cfg!`
- [ ] `include!` / `include_str!` / `include_bytes!`
- [ ] `env!` / `option_env!`
- [ ] `concat!` / `stringify!` / `line!` / `column!` / `file!` / `module_path!`

---

## 12. Attributes (core syntax + remaining categories)

### Core syntax
- [ ] `#[meta]` — outer attribute
- [ ] `#![meta]` — inner attribute

### Conditional compilation
- [ ] `#[cfg(...)]`
- [ ] `#[cfg_attr(...)]`

### Testing
- [ ] `#[test]`
- [ ] `#[ignore]`
- [ ] `#[should_panic]`

### Diagnostics
- [ ] `#[allow(...)]`
- [ ] `#[expect(...)]`
- [ ] `#[warn(...)]`
- [ ] `#[deny(...)]`
- [ ] `#[forbid(...)]`
- [ ] `#[deprecated]`
- [ ] `#[must_use]`
- [ ] `#[diagnostic::on_unimplemented]`
- [ ] `#[diagnostic::do_not_recommend]`

### Documentation
- [ ] `#[doc = "..."]` (and its relation to `///`/`//!`)

### Limits
- [ ] `#[recursion_limit = "N"]`
- [ ] `#[type_length_limit = "N"]`

### Features
- [ ] `#[feature(...)]`

### Debugger
- [ ] `#[debugger_visualizer(...)]`
- [ ] `#[collapse_debuginfo]`

*(`derive`/`automatically_derived` → §4 Traits; `repr`/`non_exhaustive` → §3 Types;
`path` → §8 Modules; FFI/codegen attributes → §10 Memory & Unsafe; macro attributes
→ §11 Macros. Listed there as primary homes, cross-linked back to this index page.)*

---

## 13. Reserved / Future-Use Keywords

Not usable yet, but still syntax elements per §4.1 ("nothing is too small to get
its own page") — these get short stub pages explaining *why* they're reserved
and what they're expected to become.

- [ ] `abstract`
- [ ] `become`
- [ ] `box`
- [ ] `do`
- [ ] `final`
- [ ] `gen` — reserved 2024 edition (generator functions)
- [ ] `override`
- [ ] `priv`
- [ ] `try` — reserved 2018 edition (`try`/catch-style blocks)
- [ ] `typeof`
- [ ] `unsized`
- [ ] `virtual`
- [ ] `yield`

---

## 14. Edition-specific reserved syntax (footnote, likely one shared page)

Not individual tokens so much as reserved *patterns* — worth a single "Reserved
syntax & edition gotchas" page rather than one page each:

- [ ] Reserved prefixes (2021+): `ident#`, `ident'` (except `b'`), `ident"..."`
      (except `b"` `c"` `r"` `br"` `cr"`), `'ident#`
- [ ] Reserved string guards (2024+): `#"string"`, `##`

---

## Summary

| # | Group | Approx. token count |
|---|-------|---------------------|
| 1 | Basics | ~50 |
| 2 | Ownership & Borrowing | ~10 |
| 3 | Types & Data Structures | ~20 |
| 4 | Traits & Polymorphism | ~10 |
| 5 | Functions & Closures | ~3 |
| 6 | Control Flow & Pattern Matching | ~11 |
| 7 | Error Handling | ~2 |
| 8 | Modules, Crates & Visibility | ~9 |
| 9 | Concurrency & Async | ~3 |
| 10 | Memory & Unsafe / FFI | ~25 |
| 11 | Macros & Metaprogramming | ~20 |
| 12 | Attributes (remaining) | ~20 |
| 13 | Reserved / Future-Use Keywords | 13 |
| 14 | Edition-specific reserved syntax | 1 page (multi-item) |

**Total: ~200 syntax pages** for the first pass. This feeds directly into
§4.7 (phasing) and §4.11 (slug table) of PAGES_DESIGN.md — nothing here is
final; re-slot items between groups as the concept pages (§3) get fleshed out.

---

*This is a living checklist. Check items off as pages are created; add rows
if something was missed (the Rust Reference/Book sources above are the
tie-breaker for "did we miss a token").*
