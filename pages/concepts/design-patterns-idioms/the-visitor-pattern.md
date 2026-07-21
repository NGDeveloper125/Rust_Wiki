---
title: "The visitor pattern"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns", "Design Patterns & Idioms", "Object-Oriented-ish Patterns"]
related_syntax: [dyn]
see_also: ["Trait objects & dynamic dispatch (dyn Trait)", "Enums (algebraic data types)", "The strategy pattern", "On-stack dynamic dispatch"]
---

## Explanation

The visitor pattern separates an operation from the data it operates
over, so new operations can be added without touching the types being
operated on. In classic object-oriented form this needs double dispatch:
each element type gets an `accept` method that calls back into a
`visit_*` method on a visitor, and the visitor interface declares one
method per concrete element type. Rust expresses that same shape
directly with traits — an `Element`-like trait with an `accept` method,
and a `Visitor` trait with one method per concrete type, dispatched
dynamically through `&dyn Visitor` — with no abstract base class or
hand-wired vtable, since [trait objects](../traits-polymorphism/trait-objects-dynamic-dispatch.md)
already supply that machinery.

The pattern is far less essential in Rust than in Java or C++, though,
and that's worth internalizing before reaching for it. When the set of
element types is closed and known up front — the usual case for an AST,
a fixed set of message shapes, a small collection of node kinds — Rust
already has a better tool for "do something different per variant": an
[enum](../types-data-modeling/enums-algebraic-data-types.md) plus an
exhaustive `match`. A match arm *is* the visitor's `visit_*` method,
minus the trait, the indirection, and the vtable, and the compiler still
guarantees every variant gets handled. A trait-based visitor earns its
keep specifically when the element set is *open* — extensible across
crate boundaries, defined by plugins, or otherwise not enumerable as a
single enum — because only a trait, not an enum, can be implemented by a
type its author has never seen.

A second reason to reach for a real visitor even with a closed type set:
when there are many independent operations over the same small set of
types (serialize, validate, render, estimate cost), matching per
operation duplicates the "which variant is this" dispatch logic once per
operation. A visitor factors that dispatch into one `accept` method per
type, written once, while each operation becomes its own `Visitor`
implementation. Which axis — types or operations — changes more often in
practice is exactly the question worth asking before choosing between an
enum `match` and a trait-based visitor.

The mental model: `accept` says "call me back through whichever visitor
you hand me," and each `Visitor` implementation is one complete pass of
business logic over the whole element hierarchy, decoupled from any
other pass.

## Basic usage example

```
trait Visitor {
    fn visit_circle(&mut self, radius: f64);
    fn visit_square(&mut self, side: f64);
}

trait Shape {
    fn accept(&self, visitor: &mut dyn Visitor); // <- double dispatch: each shape calls back into the visitor
}

struct Circle { radius: f64 }
impl Shape for Circle {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_circle(self.radius);
    }
}

struct Square { side: f64 }
impl Shape for Square {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_square(self.side);
    }
}

struct AreaSummer { total: f64 }
impl Visitor for AreaSummer {
    fn visit_circle(&mut self, radius: f64) {
        self.total += std::f64::consts::PI * radius * radius;
    }
    fn visit_square(&mut self, side: f64) {
        self.total += side * side;
    }
}

let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle { radius: 1.0 }), Box::new(Square { side: 2.0 })];
let mut summer = AreaSummer { total: 0.0 };
for shape in &shapes {
    shape.accept(&mut summer);
}
println!("{}", summer.total);
```

## Best practices & deeper information

### Scenario: Runtime polymorphism

A document-export system lets third-party plugin crates define new node
types; an enum can't represent a type declared outside this crate, so
each node implements `accept` and calls back into whichever `Exporter`
it's handed.

```
trait DocNode {
    fn accept(&self, exporter: &mut dyn Exporter);
}

trait Exporter {
    fn export_heading(&mut self, text: &str);
    fn export_paragraph(&mut self, text: &str);
}

struct Heading(String);
impl DocNode for Heading {
    fn accept(&self, exporter: &mut dyn Exporter) {
        exporter.export_heading(&self.0); // <- visitor call-back, resolved at runtime
    }
}

struct Paragraph(String);
impl DocNode for Paragraph {
    fn accept(&self, exporter: &mut dyn Exporter) {
        exporter.export_paragraph(&self.0);
    }
}

struct HtmlExporter { out: String }
impl Exporter for HtmlExporter {
    fn export_heading(&mut self, text: &str) {
        self.out += &format!("<h1>{text}</h1>");
    }
    fn export_paragraph(&mut self, text: &str) {
        self.out += &format!("<p>{text}</p>");
    }
}

let nodes: Vec<Box<dyn DocNode>> = vec![Box::new(Heading("Title".into())), Box::new(Paragraph("Body".into()))];
let mut html = HtmlExporter { out: String::new() };
for node in &nodes {
    node.accept(&mut html); // <- works even for a DocNode type defined in a plugin crate this loop never sees
}
```

**Why this way:** a plugin crate can implement `DocNode` for its own type
and this loop still dispatches correctly, which an enum `match` couldn't
do without editing the enum itself — exactly the open-set case the
[Rust Design Patterns' visitor entry](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)
targets.

### Scenario: Branching on data (pattern matching)

A config file's node types are entirely owned by this crate and never
extended by anyone else, so an exhaustive `match` plays the visitor's
role with no trait or vtable at all.

```
enum ConfigNode {
    Section { name: String, children: Vec<ConfigNode> },
    KeyValue { key: String, value: String },
}

fn print_tree(node: &ConfigNode, depth: usize) {
    let indent = "  ".repeat(depth);
    match node { // <- exhaustive match plays the visitor's role here, with no trait or dyn dispatch needed
        ConfigNode::Section { name, children } => {
            println!("{indent}[{name}]");
            for child in children {
                print_tree(child, depth + 1);
            }
        }
        ConfigNode::KeyValue { key, value } => println!("{indent}{key} = {value}"),
    }
}
```

**Why this way:** the compiler forces every `ConfigNode` variant to be
handled here, a guarantee a hand-written visitor only keeps if every
`accept` implementation is kept in sync by hand — the
[Rust Book's `match` chapter](https://doc.rust-lang.org/book/ch06-02-match.html)
covers this exhaustiveness checking, which is why a closed, crate-local
type set rarely needs a visitor at all.

## Embedded Rust Notes

**Full support.** Both shapes shown here work identically in
`#![no_std]`: the enum-plus-`match` form has no runtime cost beyond an
ordinary tag check, and `&dyn Visitor`/`&mut dyn Visitor` dispatch needs
only a reference, no allocator. The one allocating detail is incidental
to the example, not the pattern: a `Vec<Box<dyn DocNode>>` collecting
heterogeneous nodes needs the `alloc` crate; visiting a single value
through `&dyn Visitor` does not.
