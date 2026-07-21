---
title: "( )"
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Functions]
related_syntax: [","]
see_also: [","]
---

## Explanation

`( )` serves several distinct roles depending on context:

- **Grouping:** `(a + b) * c` — overrides normal precedence, same as
  in arithmetic notation generally.
- **Tuple expression/type:** `(1, "a", true)` is a 3-tuple value;
  `(i32, &str, bool)` is its type. `()` with nothing inside is the
  **unit** value/type — Rust's "no meaningful value" type, distinct from
  `void` in that it's a real, first-class, zero-sized type you can bind,
  pass, and return.
- **Single-element tuple:** `(x,)` — the trailing comma is mandatory (see
  [`,`](comma.md)); without it, `(x)` is just `x` grouped, not a tuple.
- **Function call / tuple-struct or enum-variant construction:**
  `f(a, b)`, `Point(1, 2)`, `Some(x)`.

Which meaning applies depends on position and content: `(` directly
after any expression (a name, a call's result, an indexed element) is a
call or construction; `(` starting an expression is grouping or a tuple —
and between those two, the presence of a comma inside is what makes it a
tuple.

## Usage examples

### Grouping, calling, and building tuples

```
fn add(a: i32, b: i32) -> i32 { a + b } // <- `( )` groups the parameter list
let sum = add(1, 2); // <- `( )` here is the call, passing the arguments
let pair = (1, "a"); // <- `( )` here builds a tuple value
```

### Creating a new object

Tuple structs and enum variants are constructed by calling their name
like a function — the same `( )` syntax as any other function call,
which is what makes `Some`/`Ok`/a tuple struct usable directly as a
function value in iterator adaptors.

```
struct Meters(f64);

let distance = Meters(4.2); // <- `( )` here constructs a tuple struct
let readings: Vec<Option<f64>> = vec![1.0, 2.0]
    .into_iter()
    .map(Some) // <- `Some` used directly as a fn value, thanks to this same `( )` construction rule
    .collect();
```

Because tuple-struct/variant construction really is a
function call under the hood, the constructor can be passed anywhere a
closure is expected (`.map(Some)` instead of `.map(|x| Some(x))`) — one
less closure to write and read.

### Branching on data (pattern matching)

The same `( )` used to construct a tuple struct or variant is used, in
reverse, to destructure one inside a `match` arm or `if let`.

```
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

let shape = Shape::Circle(2.0); // <- `( )` constructs the variant...
let area = match shape {
    Shape::Circle(r) => std::f64::consts::PI * r * r, // <- ...and `( )` destructures it again here
    Shape::Rectangle(w, h) => w * h,
};
```

Matching gives each field a name at the point of use
(`r`, `w`, `h`) instead of reaching into the value with `.0`/`.1`
afterward — the
[Rust Book](https://doc.rust-lang.org/book/ch06-02-match.html) covers
this destructuring-in-match pattern for working with enum payloads.

## Explanation (Embedded)

`( )` means exactly the same thing under `#![no_std]` — grouping, tuples,
and calls are core grammar with no `std` dependency, and the unit type
`()` is exactly as zero-cost on an embedded target as anywhere else.
Firmware startup code leans on the call and tuple forms constantly: a
device's peripherals are taken through an `Option`-returning call that
gets unwrapped, and splitting a configuration value into several pieces
is usually returned and destructured as a tuple.

## Usage examples (Embedded)

### Taking peripherals and constructing a driver

```
let dp = pac::Peripherals::take().unwrap(); // <- `( )` here is the `.take()` call
let mut led = Led::new(dp.GPIOA); // <- `( )` here constructs the driver
```

### Building and destructuring a timer config tuple

```
fn timer_config() -> (u32, u8) {
    (16_000_000, 8) // <- `( )` builds a tuple: (clock_hz, prescaler)
}

let (clock_hz, prescaler) = timer_config(); // <- `( )` destructures it back into two bindings
```
