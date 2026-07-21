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
- [x] `move` — force closure to take ownership *(primary; also Concurrency/Closures — cross-link)*
- [x] `ref` — bind by reference in a pattern

### Lifetimes
- [x] `'ident` — named lifetime / loop label
- [x] `'static` — the static lifetime (weak keyword)
- [x] `'a: 'b` — lifetime outlives bound
- [x] `'r#keyword` — raw lifetime (2021+ edition)

### Operators & Sigils
- [x] `&` — shared borrow / reference type
- [x] `&mut` — mutable borrow / reference type
- [x] `*` — dereference *(cross-link from Basics)*
- [x] `&raw const` — raw borrow (weak keyword `raw`)
- [x] `&raw mut` — raw mutable borrow

---

## 3. Types & Data Structures

### Keywords
- [x] `struct` — struct declaration
- [x] `enum` — enum declaration
- [x] `union` — union declaration (context keyword)
- [x] `type` — type alias
- [x] `as` — type casting

### Operators & Sigils
- [x] `::` — path/namespace separator *(primary here or Modules — pick one; heavily cross-linked either way)*
- [x] `<...>` — generic type parameters
- [x] `::<...>` — turbofish (generics in expression position)
- [x] `.` — field access
- [x] `.0` / `.1` — tuple indexing
- [x] `[...]` — array literal / type
- [x] `[T; N]` — fixed-size array type/literal
- [x] `expr[expr]` — indexing (`Index`/`IndexMut`)
- [x] `expr[..]`, `expr[a..]`, `expr[..b]`, `expr[a..b]` — slicing
- [x] `()` — unit type/value, tuple type/expression
- [x] `for<'a> type` — higher-ranked trait bounds (also Traits)
- [x] `type<ident=type>` — associated-type binding

### Attributes
- [x] `#[repr(...)]` — control type layout
- [x] `#[non_exhaustive]` — allow future fields/variants

---

## 4. Traits & Polymorphism

### Keywords
- [x] `trait` — trait declaration
- [x] `impl` — implementation block
- [x] `dyn` — dynamic trait object
- [x] `where` — trait-bound clause
- [x] `Self` — current type
- [x] `self` — current instance (receiver)

### Operators & Sigils
- [x] `:` — trait bound constraint (`T: U`)
- [x] `+` — compound trait bound (`Trait + Trait`, `'a + Trait`) *(cross-link from Basics)*
- [x] `?Sized` — relax implicit `Sized` bound
- [x] `for<'a> type` — HRTB *(cross-link from Types)*

### Attributes
- [x] `#[derive(...)]` — automatic trait impl generation
- [x] `#[automatically_derived]` — marker on derive-generated impls

---

## 5. Functions & Closures

### Keywords
- [x] `fn` *(cross-link from Basics)*
- [x] `move` *(cross-link from Ownership)*

### Operators & Sigils
- [x] `->` — return type *(cross-link from Basics)*
- [x] `|args| expr` — closure syntax
- [x] `||` — zero-argument closure form *(cross-link/disambiguation note vs. logical OR)*

---

## 6. Control Flow & Pattern Matching

### Keywords
- [x] `match` — pattern-match expression
- [x] `if let` — conditional pattern match
- [x] `while let` — loop while pattern matches
- [x] `let else` — refutable let with diverging else

### Operators & Sigils
- [x] `|` — pattern alternatives (`pat | pat`) *(distinct page from bitwise `|`, cross-linked)*
- [x] `@` — pattern binding (`ident @ pat`)
- [x] `..` — rest-of-pattern / range
- [x] `..=` — inclusive range pattern/expression
- [x] `...` — deprecated inclusive range pattern (historical note)
- [x] `_` — wildcard pattern
- [x] `=>` — match arm separator

---

## 7. Error Handling

### Operators & Sigils
- [x] `?` — error propagation operator

### Related macros *(pages live in §11, cross-linked here)*
- [x] `panic!`

---

## 8. Modules, Crates & Visibility

### Keywords
- [x] `mod` — module declaration
- [x] `use` — import declaration
- [x] `pub` — public visibility (incl. `pub(crate)`, `pub(super)`, `pub(in path)` forms)
- [x] `crate` — crate root
- [x] `self` — current module (in paths) *(cross-link from Traits, different sense)*
- [x] `super` — parent module
- [x] `extern crate` — extern crate declaration (2018+ largely implicit; still valid)

### Operators & Sigils
- [x] `::` *(cross-link from Types, or primary here — pick one)*
- [x] `as` — import renaming (`use foo as bar`) *(cross-link from Types)*

### Attributes
- [x] `#[path = "..."]` — explicit module file path

---

## 9. Concurrency & Async

### Keywords
- [x] `async` — asynchronous function/block
- [x] `await` — await an async result
- [x] `move` *(cross-link from Ownership — async blocks/closures)*

---

## 10. Memory & Unsafe / FFI

### Keywords
- [x] `unsafe` — unsafe code block/fn/trait
- [x] `extern` — external function/ABI block
- [x] `static` — static item / static storage duration
- [x] `union` *(cross-link from Types)*
- [x] `safe` — marks a safe fn/static inside an `extern` block (weak keyword)

### Operators & Sigils
- [x] `*const T` — raw immutable pointer type
- [x] `*mut T` — raw mutable pointer type
- [x] `&raw const` / `&raw mut` *(cross-link from Ownership)*

### Attributes
- [x] `#[no_mangle]`
- [x] `#[link(...)]`
- [x] `#[link_name = "..."]`
- [x] `#[link_ordinal(...)]`
- [x] `#[link_section = "..."]`
- [x] `#[no_link]`
- [x] `#[export_name = "..."]`
- [x] `#[used]`
- [x] `#[crate_type = "..."]`
- [x] `#[crate_name = "..."]`
- [x] `#[no_main]`
- [x] `#[naked]`
- [x] `#[no_builtins]`
- [x] `#[target_feature(...)]`
- [x] `#[instruction_set(...)]`
- [x] `#[panic_handler]`
- [x] `#[global_allocator]`
- [x] `#[windows_subsystem = "..."]`
- [x] `#[no_std]`
- [x] `#[no_implicit_prelude]`
- [x] `#[cold]`
- [x] `#[track_caller]`

---

## 11. Macros & Metaprogramming

### Keywords
- [x] `macro_rules` — declarative macro definition (weak keyword)
- [x] `macro` — reserved for future macro 2.0 syntax

### Operators & Sigils
- [x] `!` — macro invocation marker (`ident!(...)`) *(cross-link from Basics)*
- [x] `ident!(...)` / `ident!{...}` / `ident![...]` — the three invocation delimiter forms
- [x] `$ident` — macro substitution variable
- [x] `$ident:kind` — macro metavariable with fragment specifier
- [x] `$(...)…` — macro repetition

### Attributes
- [x] `#[macro_export]`
- [x] `#[macro_use]`
- [x] `#[proc_macro]`
- [x] `#[proc_macro_derive(...)]`
- [x] `#[proc_macro_attribute]`

### Standard macros (worth their own pages even though library, not language, items)
- [x] `println!` / `print!` / `eprintln!` / `eprint!`
- [x] `format!`
- [x] `vec!`
- [x] `panic!` *(cross-linked to Error Handling)*
- [x] `assert!` / `assert_eq!` / `assert_ne!`
- [x] `todo!` / `unimplemented!` / `unreachable!`
- [x] `matches!`
- [x] `write!` / `writeln!`
- [x] `cfg!`
- [x] `include!` / `include_str!` / `include_bytes!`
- [x] `env!` / `option_env!`
- [x] `concat!` / `stringify!` / `line!` / `column!` / `file!` / `module_path!`

---

## 12. Attributes (core syntax + remaining categories)

### Core syntax
- [x] `#[meta]` — outer attribute
- [x] `#![meta]` — inner attribute

### Conditional compilation
- [x] `#[cfg(...)]`
- [x] `#[cfg_attr(...)]`

### Testing
- [x] `#[test]`
- [x] `#[ignore]`
- [x] `#[should_panic]`

### Diagnostics
- [x] `#[allow(...)]`
- [x] `#[expect(...)]`
- [x] `#[warn(...)]`
- [x] `#[deny(...)]`
- [x] `#[forbid(...)]`
- [x] `#[deprecated]`
- [x] `#[must_use]`
- [x] `#[diagnostic::on_unimplemented]`
- [x] `#[diagnostic::do_not_recommend]`

### Documentation
- [x] `#[doc = "..."]` (and its relation to `///`/`//!`)

### Limits
- [x] `#[recursion_limit = "N"]`
- [x] `#[type_length_limit = "N"]`

### Features
- [x] `#[feature(...)]`

### Debugger
- [x] `#[debugger_visualizer(...)]`
- [x] `#[collapse_debuginfo]`

*(`derive`/`automatically_derived` → §4 Traits; `repr`/`non_exhaustive` → §3 Types;
`path` → §8 Modules; FFI/codegen attributes → §10 Memory & Unsafe; macro attributes
→ §11 Macros. Listed there as primary homes, cross-linked back to this index page.)*

---

## 13. Reserved / Future-Use Keywords

Not usable yet, but still syntax elements per §4.1 ("nothing is too small to get
its own page") — these get short stub pages explaining *why* they're reserved
and what they're expected to become.

- [x] `abstract`
- [x] `become`
- [x] `box`
- [x] `do`
- [x] `final`
- [x] `gen` — reserved 2024 edition (generator functions)
- [x] `override`
- [x] `priv`
- [x] `try` — reserved 2018 edition (`try`/catch-style blocks)
- [x] `typeof`
- [x] `unsized`
- [x] `virtual`
- [x] `yield`

---

## 14. Edition-specific reserved syntax (footnote, likely one shared page)

Not individual tokens so much as reserved *patterns* — worth a single "Reserved
syntax & edition gotchas" page rather than one page each:

- [x] Reserved prefixes (2021+): `ident#`, `ident'` (except `b'`), `ident"..."`
      (except `b"` `c"` `r"` `br"` `cr"`), `'ident#`
- [x] Reserved string guards (2024+): `#"string"`, `##`

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
