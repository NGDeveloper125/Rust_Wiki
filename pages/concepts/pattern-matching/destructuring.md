---
title: "Destructuring"
area: "Pattern Matching"
embedded_support: full
groups: ["Pattern Matching", "Functional Programming", "Designing Robust Data Models", "Coming from Haskell / functional languages"]
related_syntax: ["let", ".. / ..= / ...", "@", "|"]
see_also: ["match expressions", "Match guards", "Structs", "Tuple structs", "Enums (algebraic data types)"]
---

## Explanation

Destructuring pulls a compound value apart into its components in a
single step, binding names directly to the pieces instead of reaching in
field-by-field afterward. `let Point { x, y } = point;` binds `x` and `y`
directly from a struct; `let (width, height) = dimensions();` does the
same for a tuple. It's the same pattern syntax used in
[`match`](match-expressions.md) arms and
[`if let`/`while let`](if-let-and-while-let.md), just applied at an
ordinary `let` binding, in a function parameter list, or in a `for` loop
variable — anywhere a pattern is legal, not only inside a match.

The appeal is directness: without destructuring, extracting several
fields from a struct or tuple means several separate `.field` or `.0`
accesses, each one a small chance to typo a name or grab the wrong
index. Destructuring states the shape once and gets every piece named
correctly in that same step, which reads closer to how the data is
actually described elsewhere in the code (a struct definition, a
tuple's declared order).

Destructuring composes with the rest of Rust's data model:
[structs](../types-data-modeling/structs.md) destructure by field name,
[tuple structs](../types-data-modeling/tuple-structs.md) by position, and
[enum](../types-data-modeling/enums-algebraic-data-types.md) variants by
whichever shape they carry — and a pattern can nest arbitrarily, so a
struct field that is itself an enum variant can be destructured in the
same pattern that destructures the outer struct. `..` fills in for "the
rest of the fields, which I don't need here," and `@` binds a name to a
value while also testing it against a sub-pattern, for the rarer case
where both the whole value and confirmation of its shape are needed at
once.

Because a pattern can fail to match — a tuple destructures unconditionally,
but an enum variant might not be the one you assumed — irrefutable
destructuring (the kind usable directly in a `let`) is restricted to
patterns that always succeed; anything that might not match belongs in a
`match` arm or an `if let`/`while let` instead, which is why those forms
share this same pattern syntax rather than inventing a separate one.

## Basic usage example

```
struct Point {
    x: f64,
    y: f64,
}

let point = Point { x: 3.0, y: 4.0 };
let Point { x, y } = point; // <- destructures the struct into two bindings in one step
println!("{x}, {y}");

let dimensions = (1920, 1080);
let (width, height) = dimensions; // <- destructures the tuple positionally
println!("{width}x{height}");
```

## Best practices & deeper information

### Scenario: Creating a new object

A configuration struct has several fields, but a setup function only
needs two of them — destructuring at the binding site, with `..` for the
rest, pulls out exactly what's needed without a chain of `.field`
accesses.

```
struct ServerConfig {
    host: String,
    port: u16,
    max_connections: u32,
    timeout_secs: u32,
}

fn bind_address(config: &ServerConfig) -> String {
    let ServerConfig { host, port, .. } = config; // <- only host and port are needed here
    format!("{host}:{port}")
}

let config = ServerConfig {
    host: "0.0.0.0".to_string(),
    port: 8080,
    max_connections: 100,
    timeout_secs: 30,
};
println!("{}", bind_address(&config));
```

**Why this way:** naming only the fields a function actually uses, with
`..` standing in for the rest, keeps the binding self-documenting about
what's relevant — the
[Rust Reference on struct patterns](https://doc.rust-lang.org/reference/patterns.html#struct-patterns)
covers `..` as the form for ignoring remaining fields explicitly rather
than binding names nothing uses.

### Scenario: Working with collections

Reporting on a season's results means walking a list of paired values —
destructuring each tuple directly in the `for` loop's variable avoids a
separate `.0`/`.1` lookup inside the loop body.

```
let scores = vec![("Alice", 92), ("Bob", 87), ("Cara", 95)];

for (name, score) in &scores { // <- destructures each &(&str, i32) tuple as it's produced
    println!("{name}: {score}");
}
```

**Why this way:** destructuring in the loop head names each piece once,
at the point it's introduced, instead of leaving every use inside the
body to re-derive which position meant what — the same pattern-in-`for`
form the
[Rust Book](https://doc.rust-lang.org/book/ch03-05-control-flow.html#looping-through-a-collection-with-for)
uses when iterating tuples.

### Scenario: Branching on data (pattern matching)

An order confirmation needs the customer's name and, only when the order
has shipped, its tracking number — nesting a struct pattern inside an
enum pattern pulls both out in the same `match` arm.

```
struct Customer {
    name: String,
}

enum OrderStatus {
    Placed,
    Shipped { tracking: String },
}

struct Order {
    customer: Customer,
    status: OrderStatus,
}

fn summarize(order: &Order) -> String {
    match order {
        Order { customer: Customer { name }, status: OrderStatus::Shipped { tracking } } => {
            // <- destructures two levels deep: the Customer and the Shipped variant, in one pattern
            format!("{name}'s order shipped, tracking {tracking}")
        }
        Order { customer: Customer { name }, status: OrderStatus::Placed } => {
            format!("{name}'s order is still being prepared")
        }
    }
}
```

**Why this way:** nesting the patterns mirrors how the data is nested,
so the match arm reads as "an order whose customer is *this* and whose
status is *that*" rather than a flat check followed by several separate
field accesses — the
[Rust Reference on patterns](https://doc.rust-lang.org/reference/patterns.html)
allows arbitrary nesting for exactly this reason.

## Explanation (Embedded)

Destructuring is core-language and allocator-free: it compiles to direct
field/offset access, identical to what a manual `.field` access would
generate, so there's no runtime cost specific to writing the pattern
form. It's a natural fit for a register block or a decoded protocol
frame — once the raw bits behind a peripheral are represented as a
struct (one field per register, or one field per header value), pulling
several of them out in a single `let` or `match` arm reads the same way
the datasheet's own register table does, instead of a run of separate
`.field` accesses at the point of use.

The same restriction to irrefutable patterns applies unchanged: a
register block or a frame struct with a fixed shape destructures freely
at a plain `let`, but a frame whose shape depends on a decoded message
kind — an enum wrapping different payload structs per variant — needs
its destructuring inside a `match` arm or an `if let`, exactly as on the
host.

## Basic usage example (Embedded)

```
struct GpioBlock {
    input: u32,
    output: u32,
    direction: u32,
}

let regs = GpioBlock { input: 0x0000_00F0, output: 0x0000_0001, direction: 0x0000_00FF };
let GpioBlock { input, output, direction } = regs; // <- binds all three register words in one step

let pin3_set = input & (1 << 3) != 0;
let _ = (output, direction, pin3_set);
```

## Best practices & deeper information (Embedded)

### Scenario: Bit manipulation and flags

A UART peripheral's 8-bit status register packs several independent
flags into one word; the register is decoded once into a named struct
so the flags can be destructured in a single step instead of masking
each one out separately every time they're checked.

```
struct UartStatus {
    rx_ready: bool,
    tx_empty: bool,
    framing_error: bool,
}

fn decode_status(raw: u8) -> UartStatus {
    UartStatus {
        rx_ready: raw & 0b001 != 0,
        tx_empty: raw & 0b010 != 0,
        framing_error: raw & 0b100 != 0,
    }
}

let UartStatus { rx_ready, tx_empty, framing_error } = decode_status(0b011); // <- all three flags named in one step

if framing_error {
    // clear the error and resynchronize before reading further
} else if rx_ready {
    let _ = tx_empty; // a byte is waiting
}
```

**Why this way:** decoding the raw register once into a named struct and
then destructuring it keeps every later flag check readable by name
instead of a repeated `raw & MASK != 0` at each use site, while
compiling to the same direct bit tests a hand-written mask chain would
produce — pattern matching costs nothing here beyond what the
equivalent, less readable, hand-rolled version would take.

### Scenario: Branching on data (pattern matching)

A CAN bus driver receives frames tagged by kind; destructuring the
frame's payload inside the same match arm that identifies the frame
kind pulls it out in the one step the kind is determined, mirroring how
the classic page nests a struct pattern inside an enum pattern.

```
struct CanFrame {
    id: u16,
    payload: [u8; 8],
}

enum Message {
    Telemetry(CanFrame),
    Command(CanFrame),
}

fn first_byte(msg: &Message) -> u8 {
    match msg {
        Message::Telemetry(CanFrame { payload, .. }) => payload[0], // <- destructures the CanFrame while matching Telemetry
        Message::Command(CanFrame { payload, .. }) => payload[0],
    }
}
```

**Why this way:** nesting the `CanFrame` pattern inside the `Message`
pattern names `payload` directly at the point the frame kind is known,
rather than matching the kind first and then reaching into `.payload`
separately afterward — the same one-step nesting the classic page uses
for an order's customer and shipping data, just with bus-frame nouns.
