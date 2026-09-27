---
title: "clap"
version: "4.6.7"
publisher: "Kevin K. (kbknapp), Maintainers, Admins"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-27"
summary: "Command-line parsing from a struct definition: derive `Parser`, and the fields become flags and arguments with validation, `--help`, `--version` and error messages generated for you."
domain: "CLI & terminal"
categories: ["cli", "arguments", "derive"]
repository: "https://github.com/clap-rs/clap"
---

## Overview

Parsing `std::env::args()` by hand starts easy and stops being easy the moment
anyone expects `--help`. Short and long forms, `--` to end options, `-vvv`
stacking, defaults, required arguments, subcommands, a usage line that stays
accurate — all of it is work nobody wants to do twice.

`clap` generates it from a struct:

```
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "serve", version, about = "Run the server")]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Print more detail
    #[arg(short, long)]
    verbose: bool,
}

// try_parse_from takes the arguments explicitly, which is what makes a CLI
// testable. In main you call Cli::parse() instead.
let cli = Cli::try_parse_from(["serve", "--port", "9000", "-v"]).unwrap();

assert_eq!(cli.port, 9000);
assert!(cli.verbose);
```

The doc comment becomes the help text, the field name becomes `--port`, the
type decides how the value is parsed, and `--help` is written for you.

**There are two APIs, and the derive is the one to use.** The builder
(`Command::new("serve").arg(Arg::new("port")...)`) came first and is still
there; it earns its place when the arguments are decided at run time — a plugin
adding its own flags, a spec loaded from a file. For everything else the derive
says the same thing in a quarter of the space, and keeps the parsed result a
typed struct rather than a bag of strings.

**The cost is compile time, and it is the crate's real trade-off.** `clap` with
the derive pulls `clap_builder`, `clap_derive`, [`syn`](syn.md),
[`quote`](quote.md) and [`proc-macro2`](proc-macro2.md), and adds meaningful time
to a cold build and a few hundred KiB to the binary. For a large tool that is
nothing against what it saves. For a small utility with three flags it can
dominate, and the alternatives exist for exactly that case:

- **`lexopt`** — tiny, no derive, you write the match loop yourself.
- **`argh`** and **`pico-args`** — small derive and small imperative options.
- **`bpaf`** — comparable power, smaller, with a combinator flavour.

Pick `clap` when the interface is more than a couple of flags, when you want
`--help` to be good without writing it, or when you need subcommands. Its
ubiquity is a feature in itself: `cargo`, `ripgrep` and most of the ecosystem
use it, so its conventions are what users already expect.

**`derive` is not a default feature.** `clap = { version = "4", features =
["derive"] }` is the line you want, and `env` is a separate one again. It
requires Rust 1.85.

## When to use it

### Use case: A tool with subcommands

Anything shaped like `git commit` or `cargo build`. Each subcommand gets its own
arguments, and the enum makes handling them exhaustive.

```
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "todo", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Add a new item
    Add(AddArgs),
    /// Remove an item by name
    Remove { name: String },
    /// List everything
    List,
}

#[derive(Args)]
struct AddArgs {
    /// What to add
    name: String,
    /// Add even if it already exists
    #[arg(short, long)]
    force: bool,
}

fn run(cli: Cli) -> String {
    match cli.command {
        Command::Add(args) if args.force => format!("force-added {}", args.name),
        Command::Add(args) => format!("added {}", args.name),
        Command::Remove { name } => format!("removed {name}"),
        Command::List => "listing".to_string(),
    }
}

let cli = Cli::try_parse_from(["todo", "add", "milk", "--force"]).unwrap();
assert_eq!(run(cli), "force-added milk");

let cli = Cli::try_parse_from(["todo", "remove", "milk"]).unwrap();
assert_eq!(run(cli), "removed milk");
```

**Why it fits:** the `match` is exhaustive, so adding a subcommand is a compile
error everywhere it needs handling. A struct variant suits a couple of fields; a
tuple variant wrapping an `Args` struct keeps a large subcommand's arguments
together and reusable.

### Use case: Validating input at the boundary

Rejecting bad input before it reaches your code, with a message that says what
was wrong.

```
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Port to bind (1-65535, 0 is not a real port to listen on)
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    /// How many workers
    #[arg(long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(1..=64))]
    workers: u8,
}

let cli = Cli::try_parse_from(["app", "--port", "8080"]).unwrap();
assert_eq!((cli.port, cli.workers), (8080, 4));

// Out of range, so parsing fails rather than the server failing later.
assert!(Cli::try_parse_from(["app", "--port", "0"]).is_err());
assert!(Cli::try_parse_from(["app", "--port", "80", "--workers", "0"]).is_err());

// So does a value that isn't a number at all.
assert!(Cli::try_parse_from(["app", "--port", "http"]).is_err());
```

**Why it fits:** the field is a `u16` in a valid range by the time your code
sees it, so there is no "did someone validate this?" further in. clap reports
the failure with the argument name and the accepted range, which is a better
error than anything you would write by hand for each flag.

### Use case: Testing the interface without running the program

`try_parse_from` makes the argument surface ordinary testable code.

```
use clap::Parser;

#[derive(Parser, Debug, PartialEq)]
struct Cli {
    #[arg(short, long)]
    name: String,
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

// The happy path.
let parsed = Cli::try_parse_from(["greet", "--name", "ada", "-c", "3"]).unwrap();
assert_eq!(parsed, Cli { name: "ada".to_string(), count: 3 });

// Short and long forms are the same argument.
assert_eq!(
    Cli::try_parse_from(["greet", "-n", "ada"]).unwrap(),
    Cli::try_parse_from(["greet", "--name", "ada"]).unwrap(),
);

// A missing required argument is an error with a specific kind.
let err = Cli::try_parse_from(["greet"]).unwrap_err();
assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
```

**Why it fits:** a CLI is an interface, and interfaces deserve tests. Asserting
on `ErrorKind` rather than the message text keeps the test stable when clap
improves its wording, which it does between versions.

## API map

The derive is four macros — `Parser`, `Subcommand`, `Args`, `ValueEnum` — plus
attributes on the type and its fields. Those attributes are most of the surface,
so the entries below are grouped by what they configure.

### The derives

#### `#[derive(Parser)]`

Turns a struct into the program's argument parser.

```
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// The file to read
    path: String,
}

let cli = Cli::try_parse_from(["tool", "notes.txt"]).unwrap();
assert_eq!(cli.path, "notes.txt");
```

**When to use it:** once, on the top-level struct. In `main` you call
`Cli::parse()`, which reads the real arguments and exits with a formatted error
and the right status code on failure — that exit behaviour is why `parse` is
right for `main` and `try_parse_from` is right for tests.

#### `#[derive(Subcommand)]`

An enum whose variants are subcommands.

```
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, PartialEq, Debug)]
enum Command {
    Build { release: bool },
    Clean,
}

let cli = Cli::try_parse_from(["tool", "clean"]).unwrap();
assert_eq!(cli.command, Command::Clean);
```

**When to use it:** whenever the tool does more than one thing. Make the field
`Option<Command>` if running with no subcommand is valid — otherwise clap
requires one, which is usually what you want.

#### `#[derive(Args)]`

A group of arguments reusable across commands, or flattened into a parent.

```
use clap::{Args, Parser};

#[derive(Args, Debug)]
struct Common {
    /// Print more detail
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(flatten)]
    common: Common,
    name: String,
}

let cli = Cli::try_parse_from(["tool", "-v", "thing"]).unwrap();
assert!(cli.common.verbose);
assert_eq!(cli.name, "thing");
```

**When to use it:** shared options — verbosity, config path, output format —
that several subcommands need. `flatten` inlines them so the user sees one flat
set of flags while your code keeps them grouped.

#### `#[derive(ValueEnum)]`

Restricts an argument to a fixed set of values, checked by clap.

```
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum Format {
    Json,
    Text,
    /// Rendered as "pretty-json" on the command line
    PrettyJson,
}

#[derive(Parser)]
struct Cli {
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

assert_eq!(Cli::try_parse_from(["t"]).unwrap().format, Format::Text);
assert_eq!(
    Cli::try_parse_from(["t", "--format", "pretty-json"]).unwrap().format,
    Format::PrettyJson,
);

// Anything else is rejected, and the help lists the valid values.
assert!(Cli::try_parse_from(["t", "--format", "xml"]).is_err());
```

**When to use it:** any argument with a closed set of choices. Variant names are
converted to kebab-case automatically, the possible values appear in `--help`,
and a typo produces a "did you mean" suggestion rather than a runtime match on a
string.

### Describing arguments

#### `#[arg(short, long)]`

Gives a field a `-p` and `--port` form, named from the field.

```
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    port: u16,
    /// Only a long form
    #[arg(long)]
    dry_run: bool,
    /// An explicit short, when the derived letter collides
    #[arg(short = 'j', long)]
    jobs: Option<u8>,
}

let cli = Cli::try_parse_from(["t", "-p", "80", "--dry-run", "-j", "4"]).unwrap();
assert_eq!(cli.port, 80);
assert!(cli.dry_run);
assert_eq!(cli.jobs, Some(4));
```

**When to use it:** on every optional argument. Note `dry_run` becoming
`--dry-run` — underscores become hyphens, which is the convention users expect.
Two fields starting with the same letter is a runtime panic at startup, not a
compile error, so give one of them an explicit `short = 'x'`.

#### Positional arguments

A field with no `short` or `long` is positional, in declaration order.

```
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Where to read from
    source: String,
    /// Where to write to
    destination: String,
    /// Any number of extra files
    extra: Vec<String>,
}

let cli = Cli::try_parse_from(["cp", "a.txt", "b.txt", "c.txt", "d.txt"]).unwrap();
assert_eq!(cli.source, "a.txt");
assert_eq!(cli.destination, "b.txt");
assert_eq!(cli.extra, ["c.txt", "d.txt"]);
```

**When to use it:** for the thing the command is *about* — the file, the target,
the query. A `Vec` field takes the remainder, so it must come last; an `Option`
makes a positional optional.

#### `default_value_t` and `Option`

Two different ways of saying an argument may be absent.

```
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Always has a value; the default if unspecified
    #[arg(long, default_value_t = 3)]
    retries: u8,
    /// Absent is distinguishable from any value
    #[arg(long)]
    output: Option<String>,
}

let cli = Cli::try_parse_from(["t"]).unwrap();
assert_eq!(cli.retries, 3);
assert_eq!(cli.output, None);

let cli = Cli::try_parse_from(["t", "--retries", "9", "--output", "log.txt"]).unwrap();
assert_eq!(cli.retries, 9);
assert_eq!(cli.output.as_deref(), Some("log.txt"));
```

**When to use it:** `default_value_t` when there is a sensible default and the
code should not care whether it was given — it needs `Display` on the type and
shows the default in `--help`. `Option` when "not specified" is genuinely
different from any value, such as deciding whether to fall back to a config
file.

#### Counting and repetition

Repeated flags, either counted or collected.

```
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Repeat for more detail: -v, -vv, -vvv
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// May be given several times
    #[arg(short = 'D', long)]
    define: Vec<String>,
}

let cli = Cli::try_parse_from(["t", "-vvv", "-D", "a=1", "-D", "b=2"]).unwrap();
assert_eq!(cli.verbose, 3);
assert_eq!(cli.define, ["a=1", "b=2"]);
```

**When to use it:** `Count` for verbosity and similar dials. A `Vec` for
anything the user may supply more than once — include paths, defines, headers —
where each occurrence adds rather than replaces.

#### `env`

Falls back to an environment variable when the flag is absent. Needs the `env`
feature.

```
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(long, env = "APP_ENDPOINT", default_value = "http://localhost")]
    endpoint: String,
}

// The flag wins when given.
let cli = Cli::try_parse_from(["t", "--endpoint", "http://prod"]).unwrap();
assert_eq!(cli.endpoint, "http://prod");

// Otherwise the environment, then the default.
let cli = Cli::try_parse_from(["t"]).unwrap();
assert!(cli.endpoint.starts_with("http://"));
```

**When to use it:** configuration that differs per deployment — endpoints,
tokens, log levels — where a flag is awkward in a container. The precedence is
flag, then environment, then default, which is the order people expect. Be
careful with secrets: clap will print the variable's *name* in `--help`, which
is fine, but a value passed as a flag is visible in the process list.

#### `value_parser`

Parsing and validating a value into a real type.

```
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// Ranges are checked by clap
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=65535))]
    port: u16,
    /// Any FromStr type works, including std ones
    #[arg(long)]
    config: Option<PathBuf>,
    /// Or your own function, returning Result
    #[arg(long, value_parser = parse_pair)]
    size: Option<(u32, u32)>,
}

fn parse_pair(s: &str) -> Result<(u32, u32), String> {
    let (w, h) = s.split_once('x').ok_or("expected WIDTHxHEIGHT")?;
    Ok((
        w.parse().map_err(|_| "bad width".to_string())?,
        h.parse().map_err(|_| "bad height".to_string())?,
    ))
}

let cli = Cli::try_parse_from(["t", "--port", "80", "--size", "640x480"]).unwrap();
assert_eq!(cli.size, Some((640, 480)));

// The custom parser's error message reaches the user.
assert!(Cli::try_parse_from(["t", "--port", "80", "--size", "640"]).is_err());
```

**When to use it:** whenever a `String` would immediately be parsed anyway.
Moving it into the parser means the failure is reported with the argument's name
and usage, instead of a panic or a hand-written error three layers in.

### Relationships and metadata

#### `conflicts_with` and `requires`

Expressing that arguments interact.

```
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(long, conflicts_with = "quiet")]
    verbose: bool,
    #[arg(long)]
    quiet: bool,
    /// Only meaningful alongside --output
    #[arg(long, requires = "output")]
    overwrite: bool,
    #[arg(long)]
    output: Option<String>,
}

assert!(Cli::try_parse_from(["t", "--verbose"]).is_ok());

// Mutually exclusive.
assert!(Cli::try_parse_from(["t", "--verbose", "--quiet"]).is_err());

// --overwrite without --output is rejected.
assert!(Cli::try_parse_from(["t", "--overwrite"]).is_err());
assert!(Cli::try_parse_from(["t", "--overwrite", "--output", "f"]).is_ok());
```

**When to use it:** when a combination is meaningless rather than merely
unusual. clap rejects it with a message naming both arguments, which is better
than accepting it and picking one silently — a habit that makes a tool feel
unpredictable.

#### `#[command(...)]` metadata

Name, version, and the text around the help.

```
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "widget",
    version = "2.1.0",
    about = "Manage widgets",
    long_about = "Manage widgets, including creating, listing and destroying them.",
)]
#[derive(Debug)] // <- so unwrap_err() below can report the Ok value
struct Cli {
    #[arg(long)]
    all: bool,
}

// --version and --help exit rather than returning a parsed struct.
let err = Cli::try_parse_from(["widget", "--version"]).unwrap_err();
assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
assert!(err.to_string().contains("2.1.0"));
```

**When to use it:** on every top-level parser. `version` with no value takes it
from `CARGO_PKG_VERSION`, so it cannot drift from the crate. Note that `--help`
and `--version` come back as an `Err` whose kind says "display this and exit
zero" — `parse()` handles that for you, which is why it is what `main` uses.

#### `Error::kind` and `exit`

Handling a parse failure yourself.

```
use clap::Parser;
use clap::error::ErrorKind;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    name: String,
}

let err = Cli::try_parse_from(["t", "--nope"]).unwrap_err();
assert_eq!(err.kind(), ErrorKind::UnknownArgument);

let err = Cli::try_parse_from(["t"]).unwrap_err();
assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);

// In main, `err.exit()` prints and exits with the conventional status.
```

**When to use it:** when you need to do something before exiting — flush a log,
emit a machine-readable error. Match on `kind` rather than the message; the
wording is not a stable interface and `DisplayHelp` and `DisplayVersion` are
successes wearing an `Err`, so treating every error as a failure gets those
wrong.
