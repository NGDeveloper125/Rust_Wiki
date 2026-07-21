---
title: "Option<T>"
area: "Error Handling"
embedded_support: full
groups: ["Error Handling", "Functional Programming", "Handling Errors & Failure", "Unique to Rust", "Coming from Python / JavaScript", "Coming from Java / C#", "Coming from Haskell / functional languages"]
related_syntax: [match, "if let", "?"]
see_also: ["Result<T, E>", "The ? operator (concept angle)", "match expressions", "Enums (algebraic data types)"]
---

## Explanation

`Option<T>` is Rust's way of making the possibility of "nothing" a value
the type system knows about, rather than a special sentinel hiding inside
an otherwise-valid value. It's a plain enum with two variants —
`Some(T)`, holding a value of type `T`, and `None`, holding nothing —
defined in the standard library with no special-casing by the compiler
beyond what any other enum gets. Anywhere a value might legitimately be
absent — a lookup that might miss, a field the user left blank, the
result of subtracting past zero on an unsigned type — the function's
return type says so, as `Option<T>` instead of `T`.

The reason it exists is the absence of null. Languages that let any
reference secretly be null push the burden of remembering "which of these
could be missing" onto the programmer's memory, and get it wrong
constantly — famously called the "billion-dollar mistake" by its own
inventor. Rust has no null: a `String` is always a real string, never
secretly absent, so if a value can be missing, its type must say
`Option<String>`, and the compiler refuses to let code use the inner
value without first checking which variant is actually there.

The mental model worth keeping is "a box that holds zero or one items."
That's why `Option<T>` implements `Iterator` (yielding zero or one item),
why it has combinators like `map`, `filter`, and `unwrap_or` that mirror
iterator and [`Result`](result.md) methods, and why `?` (see
[the ? operator](the-question-mark-operator.md)) works on it just as it
does on `Result` — both are "maybe" types, differing only in whether the
empty case carries a reason.

`Option<T>` and [`Result<T, E>`](result.md) are siblings, not
competitors: reach for `Option<T>` when absence itself is the only
interesting fact ("was there a value at this key or not"), and reach for
`Result<T, E>` when a caller needs to know *why* something didn't work.
It's common to convert between them — `Option::ok_or` turns a `None` into
a specific `Err`, and `Result::ok` discards an error down to `None` —
exactly because the two express closely related ideas at different levels
of detail.

Because `Option<T>` is an ordinary enum, everything true of
[enums](../types-data-modeling/enums-algebraic-data-types.md) is true of
it: exhaustive `match`, destructuring, and the compiler refusing to
compile code that forgets a variant. That exhaustiveness is what makes
`Option<T>` load-bearing rather than decorative — there's no way to
"forget" to handle `None`, unlike a null check that's easy to skip.

## Basic usage example

```
fn find_user_email(id: u32) -> Option<String> { // <- explicit "maybe absent" return type
    if id == 1 {
        Some(String::from("admin@example.com"))
    } else {
        None
    }
}

match find_user_email(2) {
    Some(email) => println!("found: {email}"),
    None => println!("no email on file"),
}
```

## Best practices & deeper information

### Scenario: Working with collections

A warehouse inventory keyed by SKU may or may not have a given item, so
looking it up returns `Option<u32>` rather than panicking or returning a
sentinel like `-1`.

```
use std::collections::HashMap;

struct Stock {
    quantity: u32,
}

fn check_stock(inventory: &HashMap<String, Stock>, sku: &str) -> Option<u32> {
    inventory.get(sku).map(|stock| stock.quantity) // <- HashMap::get returns Option<&Stock>
}

let mut inventory = HashMap::new();
inventory.insert("SKU-100".to_string(), Stock { quantity: 42 });

match check_stock(&inventory, "SKU-404") {
    Some(qty) => println!("{qty} in stock"),
    None => println!("SKU not found"), // <- absence handled, not a crash
}
```

**Why this way:** `HashMap::get` returns `Option<&V>` instead of
panicking or making up a default, so a missing key is a normal, handled
case rather than a runtime error — see the
[std docs for `HashMap::get`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get).

### Scenario: Validating input

A signup form's middle name field is genuinely optional, so the parsed
struct stores it as `Option<String>` instead of an empty string standing
in for "not provided."

```
struct SignupForm {
    first_name: String,
    middle_name: Option<String>, // <- absence is explicit, not an empty-string convention
    last_name: String,
}

fn parse_middle_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None // <- blank field means "not provided," not an empty String
    } else {
        Some(trimmed.to_string())
    }
}

let form = SignupForm {
    first_name: "Ada".to_string(),
    middle_name: parse_middle_name(""),
    last_name: "Lovelace".to_string(),
};
```

**Why this way:** an empty string and "no value" are different facts a
plain `String` can't distinguish; modeling optional fields as `Option<T>`
makes the distinction explicit in the type, an application of
"parse, don't validate" from
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/).

### Scenario: Branching on data (pattern matching)

An order may or may not have a discount applied; branching on the
`Option` decides which price to display, with the compiler guaranteeing
both cases are handled.

```
struct Discount {
    percent_off: u8,
}

struct Order {
    subtotal_cents: u64,
    discount: Option<Discount>, // <- an order either has a discount applied or it doesn't
}

fn final_price_cents(order: &Order) -> u64 {
    match &order.discount {
        Some(discount) => { // <- exhaustive match forces both cases to be handled
            let off = order.subtotal_cents * discount.percent_off as u64 / 100;
            order.subtotal_cents - off
        }
        None => order.subtotal_cents,
    }
}
```

**Why this way:** matching exhaustively on `Option` means adding a new
case later (a stacked discount, say) is impossible to forget at every
call site — the compiler errors until it's addressed, one of the concrete
benefits of exhaustiveness checking the
[Rust Book](https://doc.rust-lang.org/book/ch06-02-match.html) highlights
for `match`.

## Embedded Rust Notes

**Full support.** `Option<T>` is `core::option::Option`, works identically
in `#![no_std]`, and needs no allocator — it's a plain enum. The compiler
also applies niche optimization to common cases (`Option<&T>`,
`Option<Box<T>>`, `Option<NonZeroU32>`, …), making it the same size as `T`
with no wasted tag byte, which matters on memory-constrained targets.
