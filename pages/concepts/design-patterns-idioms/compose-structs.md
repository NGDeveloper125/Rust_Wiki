---
title: "Compose structs"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Composition"]
related_syntax: []
see_also: ["Composition over inheritance", "The builder pattern", "Structs"]
---

## Explanation

"Compose structs" is the data-modeling half of
[composition over inheritance](composition-over-inheritance.md): break a
struct that has accumulated many only loosely related fields — a "god
struct" — into several smaller structs grouped by what actually changes
together, then compose the original type back out of them as fields. A
`ServerConfig` with ten flat fields (`host`, `port`, `log_level`,
`log_file`, `max_connections`, `db_url`, `db_pool_size`, …) reads as one
undifferentiated blob; the same data as `ServerConfig { network:
NetworkConfig, logging: LoggingConfig, database: DatabaseConfig }` reads
as three clearly named concerns, each small enough to understand,
construct, and pass around on its own.

The payoff isn't just readability. A smaller sub-struct can carry its
own `impl` block with methods that only make sense for that concern
(`NetworkConfig::bind_address(&self)`), can implement `Default`
independently so unrelated concerns don't need to agree on one shared
default, and — most importantly — can be constructed, validated, and
reused on its own: a `DatabaseConfig` extracted this way can be shared
between the server binary and a separate migration tool, without either
one depending on `ServerConfig` as a whole. A flat god struct forces
every consumer to either depend on the entire thing or duplicate the
fields it actually needs.

This is the same instinct behind normalizing a database schema — group
fields by what changes together, not just by what happens to exist on
the same conceptual "thing" — applied to a Rust type instead of a table.
It's a distinct concern from
[composition over inheritance](composition-over-inheritance.md): that
page is about sharing *behavior* across types via traits, while this
idiom is about grouping *fields* into smaller, independently meaningful
[structs](../types-data-modeling/structs.md). The two are usually
applied together, since a well-factored sub-struct is also a natural
place to attach its own methods.

## Basic usage example

```
struct NetworkConfig {
    host: String,
    port: u16,
}

struct LoggingConfig {
    level: String,
    file: Option<String>,
}

struct ServerConfig { // <- composed of two focused structs instead of four flat, unrelated fields
    network: NetworkConfig,
    logging: LoggingConfig,
}

let config = ServerConfig {
    network: NetworkConfig { host: "0.0.0.0".to_string(), port: 8080 },
    logging: LoggingConfig { level: "info".to_string(), file: None },
};
println!("{}:{}", config.network.host, config.network.port);
```

## Best practices & deeper information

### Scenario: Designing a public API

A payment-processing config has grown flat fields for both the merchant
account and the retry policy; splitting them into their own structs lets
each be reused and tested independently once a second payment provider
needs the same retry-policy shape.

```
struct MerchantAccount {
    id: String,
    api_key: String,
}

struct RetryPolicy {
    max_attempts: u8,
    backoff_ms: u64,
}

struct PaymentConfig { // <- two focused structs instead of four unrelated flat fields
    merchant: MerchantAccount,
    retry: RetryPolicy,
}

impl RetryPolicy {
    fn should_retry(&self, attempt: u8) -> bool { // <- a method that only makes sense on this one concern
        attempt < self.max_attempts
    }
}

let retry = RetryPolicy { max_attempts: 3, backoff_ms: 200 };
assert!(retry.should_retry(1));
```

**Why this way:** `RetryPolicy` can now be reused verbatim by a second
payment provider's config without dragging `MerchantAccount` along with
it, and `should_retry` has an obvious, single home instead of sitting on
a struct with a dozen unrelated fields — grouping by concern rather than
by "everything this feature happens to need" keeps each piece both
readable and reusable on its own.

### Scenario: Creating a new object

Composed sub-structs each get their own sensible `Default`, so the outer
struct's constructor doesn't have to hardcode every leaf value by hand.

```
#[derive(Default)]
struct NetworkConfig {
    host: String,
    port: u16,
}

#[derive(Default)]
struct LoggingConfig {
    level: String,
    file: Option<String>,
}

#[derive(Default)]
struct ServerConfig { // <- derives Default by combining each component's own Default
    network: NetworkConfig,
    logging: LoggingConfig,
}

let config = ServerConfig::default();
println!("{}", config.network.port);
```

**Why this way:** `#[derive(Default)]` on the composed struct only works
because each field's own type already implements `Default` — exactly
the kind of small, independently-defaultable component this idiom
produces, per the
[standard library's `Default` trait docs](https://doc.rust-lang.org/std/default/trait.Default.html).

## Explanation (Embedded)

This idiom fits embedded driver design about as directly as any pattern
in this catalog: a peripheral driver is almost always, structurally, a
composition of a HAL handle plus whatever configuration or state the
driver layers on top of it. A `Sensor` struct isn't one flat bag of
fields — it's naturally `Sensor { i2c: I2c, config: SensorConfig,
calibration: Calibration }`, where `i2c` is an owned HAL peripheral
handle, `config` is the sensor's own settings (sample rate, resolution),
and `calibration` is data read back from the device itself. Splitting
these into their own small structs gives each one an obvious owner and
an obvious `impl` block: `SensorConfig` can validate its own sample-rate
range independently of anything the I2C bus is doing, and `Calibration`
can be swapped out or reset without touching the bus handle at all.

The same instinct scales up to a whole board: a `Board` struct composed
of `Led`, `Button`, and `Uart` fields — each itself perhaps composed
further (a `Led` wrapping an `OutputPin`) — reads far more clearly than
one flat struct holding every raw pin and peripheral register the board
exposes, and each composed piece can be handed off independently to the
part of the firmware that actually owns that concern (the button to an
input task, the UART to a logging task), which a single monolithic board
struct would make far more awkward.

## Basic usage example (Embedded)

```
struct I2cHandle; // stands in for a HAL I2c peripheral type

struct SensorConfig {
    sample_rate_hz: u16,
}

struct Sensor { // <- composed of a HAL handle and its own config, not one flat struct
    i2c: I2cHandle,
    config: SensorConfig,
}

let sensor = Sensor {
    i2c: I2cHandle,
    config: SensorConfig { sample_rate_hz: 100 },
};
println!("{} Hz", sensor.config.sample_rate_hz);
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A temperature-and-humidity sensor driver is composed from the I2C handle
it talks over and its own settings, so the settings can be validated and
reused independently of the specific bus instance the sensor happens to
be wired to.

```
struct I2cHandle; // stands in for embedded-hal's I2c trait type

struct SensorConfig {
    sample_rate_hz: u16,
    oversampling: u8,
}

impl SensorConfig {
    fn validated(sample_rate_hz: u16, oversampling: u8) -> Option<Self> {
        if sample_rate_hz == 0 || oversampling > 8 {
            return None; // <- a method that only makes sense on this one focused concern
        }
        Some(Self { sample_rate_hz, oversampling })
    }
}

struct TempHumiditySensor { // <- composed of two focused pieces instead of one flat struct
    i2c: I2cHandle,
    config: SensorConfig,
}

impl TempHumiditySensor {
    fn new(i2c: I2cHandle, config: SensorConfig) -> Self {
        Self { i2c, config }
    }
}

let config = SensorConfig::validated(100, 4).expect("valid sample config");
let sensor = TempHumiditySensor::new(I2cHandle, config);
println!("{} Hz", sensor.config.sample_rate_hz);
```

**Why this way:** `SensorConfig` can be constructed, validated, and unit
tested with no I2C bus present at all, and a second sensor on a
different bus can reuse the exact same `SensorConfig` type — grouping
the driver's own settings separately from the HAL handle it happens to
be wired to is the same "group by what changes together" reasoning the
classic page applies to a `ServerConfig`, just with a peripheral handle
standing in for a database URL.

### Scenario: Sharing data with multiple references

A board-support struct composed of independent peripheral fields lets
each one be borrowed and handed to a different part of firmware without
those parts needing to know about each other or about the board struct
itself.

```
struct Led;
struct Button;
struct Uart;

struct Board { // <- composed of independent peripherals, not one flat register-access struct
    status_led: Led,
    user_button: Button,
    debug_uart: Uart,
}

fn poll_button(button: &Button) -> bool {
    let _ = button;
    false // stands in for a real GPIO read
}

fn log_line(uart: &Uart, message: &str) {
    let _ = (uart, message); // stands in for a real UART write
}

let board = Board { status_led: Led, user_button: Button, debug_uart: Uart };
let pressed = poll_button(&board.user_button); // <- borrows only the field this task needs
log_line(&board.debug_uart, "boot complete");
println!("{pressed}");
```

**Why this way:** because each peripheral is its own field, one task can
hold `&board.user_button` while another holds `&board.debug_uart` at the
same time with no borrow conflict — a flat struct doesn't change what's
possible here, but composing by peripheral makes each task's actual
dependency explicit in its function signature instead of "the whole
board."
