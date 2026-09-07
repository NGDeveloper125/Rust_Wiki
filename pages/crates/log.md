---
title: "log"
version: "0.4.34"
publisher: "Huon Wilson (huonw), Steven Fackler (sfackler), Ashley Mannix (KodrAus), rust-lang-owner, log-owners"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-07"
summary: "The logging facade the ecosystem agrees on. Libraries call `info!` and `warn!` against it; the application chooses one implementation, and every library's output goes there."
categories: ["logging", "diagnostics", "no-std"]
repository: "https://github.com/rust-lang/log"
---

## Overview

If every library picked its own logging crate, an application would need to
configure all of them. `log` is the agreement that avoids that: it defines the
macros and a `Log` trait, and nothing else. Libraries call the macros; the
application installs exactly one implementation, and everything lands there.

```
use log::{debug, error, info, warn};

fn connect(host: &str) -> Result<(), String> {
    debug!("resolving {host}");
    info!("connected to {host}");
    warn!("using default timeout");
    if host.is_empty() {
        error!("no host given");
        return Err("no host".to_string());
    }
    Ok(())
}

assert!(connect("example.com").is_ok());
assert!(connect("").is_err());
```

**The thing that catches everyone: by default those calls do nothing.** With no
logger installed, every macro is a no-op — no output, no warning, no error. Code
that "isn't logging" is almost always code where the application never called an
implementation's init function. `log` is the façade; you also need one of
`env_logger`, `fern`, `simple_logger`, `tracing-subscriber` or similar, and the
choice belongs to the binary.

That split is the rule to follow:

- **A library depends on `log` only.** It has no business deciding where its
  output goes, and pulling in an implementation would force that on its users.
- **An application depends on `log` plus one implementation**, and initialises it
  early in `main`.

**`tracing` is the other answer to the same problem**, and worth knowing before
you commit. Where `log` records isolated events, `tracing` adds spans — a
request, a transaction — so a record carries the context it happened in, which
is what you want for anything concurrent or distributed. It can consume `log`
records too, so a `tracing` application still sees output from `log`-using
libraries. Pick `log` for simplicity and the widest library compatibility;
`tracing` when you need structure.

The crate is tiny with no required dependencies, requires Rust 1.71, and works
`no_std`. Two features are worth knowing: `kv` adds structured key-value pairs
to records, and the `max_level_*` and `release_max_level_*` features compile
lower-severity calls out entirely, so a `trace!` in a hot loop costs nothing in
release.

## When to use it

### Use case: A library that reports without deciding

The whole point of the facade. A library logs; where that goes is not its
concern.

```
use log::{debug, warn};

pub struct Cache {
    entries: Vec<(String, String)>,
    capacity: usize,
}

impl Cache {
    pub fn insert(&mut self, key: &str, value: &str) {
        if self.entries.len() >= self.capacity {
            warn!("cache full ({}), evicting oldest", self.capacity);
            self.entries.remove(0);
        }
        debug!("caching {key}");
        self.entries.push((key.to_string(), value.to_string()));
    }
}

let mut cache = Cache { entries: Vec::new(), capacity: 1 };
cache.insert("a", "1");
cache.insert("b", "2"); // <- logs the eviction
assert_eq!(cache.entries.len(), 1);
```

**Why it fits:** the library adds one small dependency and no policy. An
application using it with `env_logger` gets these records on stderr; one using
`tracing` gets them in its span tree; one that installs nothing sees nothing and
pays almost no cost.

### Use case: Installing a logger in an application

The binary's job, done once before anything else runs.

```
use log::{info, LevelFilter, Log, Metadata, Record};
use std::sync::{Mutex, OnceLock};

// A minimal logger: real applications use env_logger or similar.
struct Collector;

static LINES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

impl Log for Collector {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let lines = LINES.get_or_init(|| Mutex::new(Vec::new()));
            lines.lock().unwrap().push(format!("{} {}", record.level(), record.args()));
        }
    }
    fn flush(&self) {}
}

log::set_logger(&Collector).unwrap();
log::set_max_level(LevelFilter::Info);

info!("service started");
log::debug!("not recorded"); // <- below the max level

let recorded = LINES.get().unwrap().lock().unwrap().clone();
assert_eq!(recorded, ["INFO service started"]);
```

**Why it fits:** it shows both halves of installation, which is where the
mistakes are. `set_logger` alone is not enough — `set_max_level` gates the
macros before they even build a record, and leaving it at the default `Off`
means a correctly installed logger that still prints nothing.

### Use case: Skipping work that only logging needs

When producing the message is expensive, check whether anyone is listening
first.

```
use log::{log_enabled, trace, Level};

fn summarise(rows: &[u32]) -> String {
    // Pretend this is expensive.
    rows.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(",")
}

fn process(rows: &[u32]) -> u32 {
    if log_enabled!(Level::Trace) {
        trace!("processing rows: {}", summarise(rows)); // <- only built when enabled
    }
    rows.iter().sum()
}

assert_eq!(process(&[1, 2, 3]), 6);
```

**Why it fits:** the macros already avoid formatting when the level is disabled,
but they cannot avoid evaluating an argument you computed beforehand.
`log_enabled!` is for the case where the *argument* is the expensive part — a
serialisation, a database round trip, a `Debug` of something huge.

## API map

Five macros, a trait to implement, and two functions to install it. Most code
uses only the macros.

### The macros

#### `error!` and `warn!`

The two levels that mean something went wrong.

```
use log::{error, warn};

fn parse(input: &str) -> Option<u32> {
    match input.parse() {
        Ok(n) => Some(n),
        Err(e) => {
            error!("could not parse {input:?}: {e}");
            None
        }
    }
}

warn!("running with defaults");
assert_eq!(parse("12"), Some(12));
assert_eq!(parse("x"), None);
```

**When to use it:** `error!` when the operation failed and someone must know;
`warn!` when it continued but the outcome is degraded. Neither should fire on
the normal path — a warning that appears every run trains people to ignore all
of them.

#### `info!`, `debug!` and `trace!`

The three levels for reporting what happened rather than what went wrong.

```
use log::{debug, info, trace};

fn handle(id: u32) {
    trace!("entered handle({id})");        // <- per-call detail
    debug!("looking up record {id}");      // <- developer-facing
    info!("served request {id}");          // <- operator-facing
}

handle(7);
```

**When to use it:** `info!` for events an operator watching production cares
about — started, connected, finished. `debug!` for what a developer diagnosing a
problem wants. `trace!` for volume you would only enable while hunting something
specific. The default for a released binary is usually `info`.

#### Targets

Every record carries a target, defaulting to the module path, and it is how
implementations filter by component.

```
use log::info;

info!("default target is this module's path");
info!(target: "metrics", "requests={}", 42);
```

**When to use it:** grouping records that cross module boundaries — all metrics,
all requests — so a user can enable one area without the rest. `RUST_LOG=metrics=debug`
in `env_logger` filters on exactly this.

#### Structured fields

With the `kv` feature, records can carry typed key-value pairs alongside the
message.

```
use log::info;

let user = 42;
info!(user, attempt = 2; "login failed");
```

**When to use it:** when the log is consumed by a machine rather than a person —
a JSON collector that indexes the fields. Without `kv` these values can only be
interpolated into the message, which means whatever reads them has to parse the
sentence back apart.

### Levels

#### `Level` and `LevelFilter`

`Level` is the severity of a record; `LevelFilter` is a threshold, with the extra
`Off` value.

```
use log::{Level, LevelFilter};

assert!(Level::Error < Level::Warn); // <- ordered by severity, most severe first
assert!(Level::Debug > Level::Info);

let filter = LevelFilter::Info;
assert!(Level::Warn <= filter); // <- would be recorded
assert!(Level::Debug > filter); // <- would not

assert_eq!("debug".parse::<LevelFilter>().unwrap(), LevelFilter::Debug);
```

**When to use it:** comparing severities, and parsing a level from configuration.
The ordering is the thing to get right — `Error` is the *smallest*, so
"is this enabled" is `record_level <= filter`, which reads backwards until you
know it.

#### `set_max_level` and `max_level`

The global threshold the macros check before doing anything.

```
use log::{max_level, set_max_level, LevelFilter};

set_max_level(LevelFilter::Warn);
assert_eq!(max_level(), LevelFilter::Warn);

// Below the threshold: the macro doesn't even format its arguments.
log::info!("skipped entirely");

set_max_level(LevelFilter::Trace);
assert_eq!(max_level(), LevelFilter::Trace);
```

**When to use it:** at startup, and again if a runtime control changes verbosity.
It defaults to `Off`, which is the second half of the "my logging does nothing"
problem — most implementations' init functions call it for you, but a hand-rolled
logger must.

#### `log_enabled!`

Whether a record at this level would be handled.

```
use log::{log_enabled, Level, LevelFilter, Log, Metadata, Record};

struct Everything;
impl Log for Everything {
    fn enabled(&self, _: &Metadata) -> bool { true }
    fn log(&self, _: &Record) {}
    fn flush(&self) {}
}
static LOGGER: Everything = Everything;

// With no logger installed it is false whatever the max level says.
log::set_max_level(LevelFilter::Trace);
assert!(!log_enabled!(Level::Info));

log::set_logger(&LOGGER).unwrap();
log::set_max_level(LevelFilter::Info);

assert!(log_enabled!(Level::Info));
assert!(!log_enabled!(Level::Debug)); // <- above the max level
```

**When to use it:** guarding an expensive computation that exists only to be
logged. Don't wrap ordinary calls in it — the macros are already cheap when
disabled, and the guard just adds noise. Note what it checks: the max level
*and* the installed logger's own `enabled`, so with no logger it is always
false — which is the same reason an uninstalled facade prints nothing.

### Implementing a logger

#### The `Log` trait

Three methods: a filter, a handler, and a flush.

```
use log::{Level, Log, Metadata, Record};

struct StderrLogger;

impl Log for StderrLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("[{}] {}: {}", record.level(), record.target(), record.args());
        }
    }

    fn flush(&self) {}
}

let logger = StderrLogger;
assert!(logger.enabled(&log::Metadata::builder().level(Level::Warn).build()));
assert!(!logger.enabled(&log::Metadata::builder().level(Level::Trace).build()));
```

**When to use it:** writing an implementation — which most people never do, since
`env_logger` and friends exist. It matters to know the shape because `log` does
*not* call `enabled` for you before `log`; an implementation that forgets to
check will print everything the global max level allows.

#### `Record` and `Metadata`

What a logger receives: the message plus where it came from.

```
use log::{Level, Record};

let record = Record::builder()
    .level(Level::Warn)
    .target("my_app::db")
    .args(format_args!("slow query: {}ms", 250))
    .module_path(Some("my_app::db"))
    .line(Some(42))
    .build();

assert_eq!(record.level(), Level::Warn);
assert_eq!(record.target(), "my_app::db");
assert_eq!(record.line(), Some(42));
assert_eq!(record.args().to_string(), "slow query: 250ms");
```

**When to use it:** inside a `Log` implementation, and in tests of one. The
message arrives as `fmt::Arguments`, not a `String` — it has not been formatted
yet, which is what lets a logger discard a record without paying for it.

#### `set_logger` and `set_boxed_logger`

Installs the implementation, once per process.

```
use log::{LevelFilter, Log, Metadata, Record};

struct Noop;
impl Log for Noop {
    fn enabled(&self, _: &Metadata) -> bool { false }
    fn log(&self, _: &Record) {}
    fn flush(&self) {}
}

static LOGGER: Noop = Noop;

log::set_logger(&LOGGER).unwrap();
log::set_max_level(LevelFilter::Info);

// A second install fails rather than replacing the first.
assert!(log::set_logger(&LOGGER).is_err());
```

**When to use it:** once, early in `main`. `set_logger` takes a `&'static dyn Log`
and so allocates nothing; `set_boxed_logger` takes a `Box` for a logger built at
runtime from configuration. The second call failing is deliberate — a library
that installed a logger would silently break the application's own choice, which
is why libraries must not.
