---
title: "Crates"
area: "Modules, Crates & Visibility"
embedded_support: full
groups: ["Modules, Crates & Visibility", "Structuring a Project"]
related_syntax: [crate, extern, use, pub]
see_also: ["Modules", "Workspaces", "Cargo & Cargo.toml"]
---

## Explanation

A crate is the smallest unit the Rust compiler treats as a single
compilation — one call to `rustc` in, one library or executable out — and
also the unit Cargo publishes to crates.io. A crate is either a *binary
crate*, rooted at `src/main.rs`, which produces a runnable executable, or
a *library crate*, rooted at `src/lib.rs`, which produces something other
crates can depend on. Where [modules](modules.md) organize the *inside*
of a crate, the crate itself is the boundary the compiler and the
package ecosystem both actually operate on.

Every crate has exactly one root module — the top of `main.rs` or
`lib.rs` — and that root is itself addressable with the `crate` keyword:
`crate::some_function` always names a path starting from this crate's
root, no matter how deeply nested the code writing it is. That gives a
crate an unambiguous way to refer to "my own top level" that doesn't
depend on where in the module tree the reference happens to be written.

The crate boundary is also the outer limit of one particular kind of
privacy: an item marked `pub(crate)` is visible everywhere inside the
crate but nowhere outside it, which is why [visibility &
privacy](visibility-and-privacy.md) and the crate concept are so tightly
paired — a crate is the largest scope something can be shared across
while still being completely absent from that crate's public API.

Crates are also the unit dependency management operates on: each entry
under `[dependencies]` in a crate's [Cargo.toml](cargo-and-cargo-toml.md)
names another crate, Cargo compiles it once as its own unit and links it
in, and a single crate carries exactly one name, one
[semver version](dependency-management-and-semver.md), and one published
identity on crates.io. A large project made of several crates that are
still developed together, rather than published as independent
dependencies, is what a [workspace](workspaces.md) is for.

## Basic usage example

```
// src/lib.rs — this file is the crate root
pub fn greet(name: &str) -> String {   // <- part of this crate's public API
    format!("Hello, {name}!")
}

pub mod inventory {
    pub fn describe() -> String {
        crate::greet("inventory")      // <- `crate::` names this crate's root, from anywhere inside it
    }
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A library crate's `lib.rs` is the one file whose top-level items define
the crate's entire public surface — everything a caller can reach starts
there, however the implementation is organized underneath.

```
// src/lib.rs
mod client;                       // <- private: an implementation detail
mod config;                       // <- private: an implementation detail

pub use client::WeatherClient;    // <- the crate's public API is curated here, not scattered
pub use config::Config;

pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

**Why this way:** keeping the internal modules private and re-exporting a
short, deliberate list from the crate root means the crate's actual
public surface can be read at a glance from one file, matching the
curation the
[API Guidelines' future-proofing chapter](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends for a stable public API.

### Scenario: Testing

Integration tests in a crate's `tests/` directory are each compiled as
their own separate crate that depends on the library crate the same way
an external user would — so they can only exercise what's actually
`pub`.

```
// src/lib.rs
pub fn discount_price(price_cents: u32, percent_off: u8) -> u32 {
    price_cents - (price_cents * percent_off as u32 / 100)
}

fn round_to_nearest_cent(value: f64) -> u32 { // <- private: invisible outside this crate
    value.round() as u32
}

// tests/pricing.rs — compiled as its own crate, linked against weather_api's public API
use weather_api::discount_price;              // <- only `pub` items are reachable here

#[test]
fn applies_discount() {
    assert_eq!(discount_price(2000, 10), 1800);
}
```

**Why this way:** because each file under `tests/` links against the
library crate exactly like any outside consumer, integration tests double
as a check that the crate's public API is actually usable on its own,
per the
[Rust Book's chapter on test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests).

## Explanation (Embedded)

The crate-as-compilation-unit mechanism is unchanged for embedded
targets; what's genuinely distinctive is the layered crate ecosystem
embedded projects converge on. A peripheral access crate (PAC) — usually
generated from the chip vendor's SVD register-description file by
`svd2rust` — gives raw, unsafe, register-level access with no semantics
attached beyond "this bit lives at this address." A hardware abstraction
layer crate (HAL) depends on the PAC and wraps it in a safe API: types
like `Gpio` or `Spi` whose methods enforce that registers get poked in
valid sequences. A board support crate (BSP) depends on the HAL and adds
one more layer: pin mappings and peripheral names specific to one
physical development board, so application code can say
`board.user_led` instead of naming a raw GPIO pin. The application crate,
finally, depends on some combination of these and is where actual
firmware logic lives. Each layer is an ordinary crate with its own
`Cargo.toml`, its own semver version, and its own privacy boundary,
exactly per the general crate model described above — the layering is a
convention the ecosystem follows, not a different kind of crate.

## Basic usage example (Embedded)

```
[dependencies]
stm32f4 = "0.15"              # <- PAC: svd2rust-generated raw register access
stm32f4xx-hal = "0.21"        # <- HAL: safe wrapper over the PAC
# an application crate depends on the HAL directly;
# a board support crate would sit between them, pin-mapped to one board
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A HAL crate's whole purpose is to turn a PAC's raw register access into
a public API that can't be driven into an invalid hardware state, so its
`lib.rs` deliberately keeps the PAC import out of its own public
surface.

```
// hal/src/lib.rs
use stm32f4::stm32f411 as pac;   // <- private: the PAC is an implementation detail of the HAL

pub struct Gpio {
    // ...
}

impl Gpio {
    pub fn set_high(&mut self) {
        // unsafe register write happens here, behind a safe method
    }
}

pub use self::Gpio as Pin;   // <- curated public name, independent of the PAC's own naming
```

**Why this way:** hiding the PAC dependency behind the HAL's own public
types means the HAL's version can evolve, or even its underlying PAC be
swapped, without breaking application code, matching the crate-curation
practice the
[API Guidelines' future-proofing chapter](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends generally, applied here to a hardware boundary.

### Scenario: Layering a hardware crate stack

A new board support crate needs to depend on both the chip's HAL (for
the safe peripheral API) and, transitively, its PAC — but application
code should only ever need to depend on the BSP.

```
// bsp/Cargo.toml
[dependencies]
stm32f4xx-hal = "0.21"    # <- BSP depends on the HAL, not the PAC directly

// bsp/src/lib.rs
pub use stm32f4xx_hal::gpio::Pin;   // <- re-exports what applications actually need

pub struct Board {
    pub user_led: Pin,     // <- pin already mapped to this specific board's LED
}
```

**Why this way:** an application depending only on the BSP, not reaching
past it to the HAL or PAC, keeps board-specific wiring (which pin the
LED is actually on) in one place instead of duplicated across every
application crate that targets the same board — the same "small crate,
curated dependency direction" argument the
[Rust Design Patterns' "Prefer small crates"](https://rust-unofficial.github.io/patterns/idioms/prefer-small-crates.html)
idiom makes generally.
