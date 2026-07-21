---
title: "#[diagnostic::on_unimplemented] / #[diagnostic::do_not_recommend]"
kind: attribute
embedded_support: full
groups: ["Lints & Diagnostics", "Traits & Polymorphism"]
related_concepts: ["Traits", "Trait bounds"]
related_syntax: ["trait", "impl"]
see_also: []
---

## Explanation

Both attributes live under the `diagnostic` tool-attribute namespace
(written `#[diagnostic::name(...)]`, a path with a `::` in it rather than
a single identifier — see [`#[meta] / #![meta]`](attribute-syntax.md) for
attribute paths in general) and exist purely to make the compiler's error
messages more helpful. Neither one changes what compiles, what type-checks,
or how a program behaves — a crate that removed every `#[diagnostic::...]`
attribute from its source would compile to identical code; only the
wording of certain error messages would get worse.

**`#[diagnostic::on_unimplemented(message = "...", label = "...")]`**,
placed on a `trait` definition, customizes the error the compiler shows
when some type fails to implement that trait where the trait is required
— replacing (or supplementing) the generic "the trait bound `T: SomeTrait`
is not satisfied" message with wording specific to that trait. The
standard library uses this on several of its own most commonly-hit traits
— `Iterator`, `Fn`/`FnMut`/`FnOnce` — which is why forgetting to implement
`Iterator` correctly, or passing something that isn't callable where a
closure is expected, tends to produce an error message that names the
actual problem in plain language instead of the raw, generic trait-bound
wording. `message` replaces the top-line error text; `label` customizes
the inline caret annotation shown under the offending code span.

**`#[diagnostic::do_not_recommend]`**, placed on an `impl` block (most
often a blanket impl), tells the compiler not to *suggest* that impl as a
fix when some other trait bound fails to hold nearby — even though the
impl technically does apply. This matters specifically for blanket impls:
a broad `impl<T: SomeBound> Trait for T` can technically satisfy a
compiler's "here's an impl that would work" suggestion machinery in cases
where mentioning it would only confuse the reader (the real fix is
elsewhere, and the blanket impl is incidental, not the intended solution).
Suppressing it from *suggestions* doesn't stop the impl from existing or
working — code that already relies on it continues to compile exactly as
before; only the compiler's advice-generation ignores it as a candidate to
recommend.

## Usage examples

### Customizing the error message for an unimplemented trait

```
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be rendered to the screen",
    label = "missing `Renderable` implementation"
)]
trait Renderable {
    fn render(&self) -> String;
}
```

### Designing a public API

A UI framework's core trait is implemented by many types across an
application; a plain, generic trait-bound error doesn't tell a newcomer
*what* is missing or *why* — a custom `on_unimplemented` message speaks
directly to the framework's own vocabulary instead.

```
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a `Widget` — implement `Widget` to use it in a layout",
    label = "not a `Widget`"
)]
trait Widget {
    fn measure(&self) -> (u32, u32);
}

struct Label {
    text: String,
}

impl Widget for Label {
    fn measure(&self) -> (u32, u32) {
        (self.text.len() as u32 * 8, 16)
    }
}

fn add_to_layout<W: Widget>(_widget: W) {}

fn build_layout() {
    add_to_layout(Label { text: "Hello".to_string() }); // <- compiles: Label implements Widget
    // add_to_layout(42); // would show the custom message above instead of a generic trait-bound error
}
```

A generic "the trait bound `i32: Widget` is not
satisfied" tells a reader *that* something is wrong but not *what to do*
about it, whereas a message written in the framework's own terms points
directly at the fix; the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-diagnosticon_unimplemented-attribute)
documents this attribute as intended for exactly this kind of
domain-specific guidance on commonly-hit trait bounds, following the same
pattern the standard library uses on `Iterator` and the `Fn` traits.

### Implementing traits

A serialization trait has both specific, hand-written impls for certain
types and a broad blanket impl covering anything convertible through
`Display` — when a genuinely unrelated type fails to implement the
trait, suggesting the blanket impl as "the fix" would be more confusing
than helpful, since the real gap is that the type isn't `Display` either.

```
trait ToWireFormat {
    fn to_wire(&self) -> String;
}

#[diagnostic::do_not_recommend] // <- don't suggest this blanket impl as the fix for unrelated failures
impl<T: std::fmt::Display> ToWireFormat for T {
    fn to_wire(&self) -> String {
        self.to_string()
    }
}
```

Without `do_not_recommend`, a compiler error involving
some type that implements neither `Display` nor `ToWireFormat` directly
could suggest "implement `ToWireFormat`" by pointing at this blanket impl,
which is technically accurate but unhelpful — the underlying gap is
`Display`, not `ToWireFormat`; the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-diagnosticdo_not_recommend-attribute)
documents this exact "blanket impl technically applies but isn't the
useful thing to suggest" scenario as its intended use.

## Embedded Rust Notes

**Full support.** Both attributes are pure compile-time diagnostic
metadata with zero runtime footprint, so they behave identically in
`#![no_std]` crates — a `core`-only trait used heavily across a HAL
crate's public API benefits from a custom `on_unimplemented` message the
same way any hosted crate's trait would.
