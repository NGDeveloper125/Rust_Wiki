---
title: "regex"
version: "1.13.1"
publisher: "Andrew Gallant (BurntSushi), rust-lang-owner, regex-owners"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-18"
summary: "Regular expressions with a linear-time guarantee: matching can never blow up exponentially, because the engine deliberately omits backreferences and lookaround."
domain: "Text, parsing & Unicode"
categories: ["text-processing", "parsing", "regex"]
repository: "https://github.com/rust-lang/regex"
---

## Overview

`regex` is Rust's regular expression engine. The usual things work as expected:

```
use regex::Regex;

let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();

assert!(re.is_match("released 2026-09-18"));

let caps = re.captures("released 2026-09-18").unwrap();
assert_eq!(&caps[1], "2026");
assert_eq!(&caps[3], "18");
```

**What makes it different from the regex engine in most other languages is a
promise about time.** Matching is guaranteed linear in the length of the input
and the pattern — there is no input that makes it take exponential time. Perl,
Python, Java and JavaScript all use backtracking engines where a pattern like
`(a+)+b` against a long run of `a`s takes effectively forever, which is a real
denial-of-service vector when the pattern or the input comes from a user.

That guarantee has a price, and it is the thing to know before you start:

- **No backreferences.** `(\w)\1` — "the same character twice" — is not
  expressible.
- **No lookahead or lookbehind.** `(?=...)`, `(?<=...)` and their negations are
  absent.

Both are omitted because both require backtracking. If you genuinely need them,
`fancy-regex` wraps this crate and adds them, accepting the worst case; more
often the need disappears once you use `captures` and ordinary code instead of
pushing all the logic into the pattern.

**The other thing to get right is compiling once.** `Regex::new` parses and
builds an automaton, which is orders of magnitude more expensive than a match.
Doing it inside a loop or a function called per request is the single most common
performance mistake with this crate. Build it once — a `LazyLock`, a struct
field, a `OnceLock` — and reuse it.

```
use regex::Regex;
use std::sync::LazyLock;

static VERSION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"v\d+\.\d+").unwrap());

assert!(VERSION.is_match("v1.13"));
```

Unicode is on by default: `\w`, `\d` and `\b` follow Unicode definitions, and
patterns match on `&str`. The `regex::bytes` module is the same engine over
`&[u8]` for data that is not valid UTF-8. `regex-lite` is a drop-in with a much
smaller binary and no Unicode tables, for when compile time and size matter more
than throughput.

It is maintained by the author of ripgrep, requires Rust 1.65, and is fast
enough that hand-rolled string scanning usually loses to it.

## When to use it

### Use case: Pulling fields out of log lines

The classic job: a semi-structured line where the interesting parts are at
predictable places but not at fixed offsets.

```
use regex::Regex;

let line = r#"127.0.0.1 - - [18/Sep/2026:10:32:01] "GET /health HTTP/1.1" 200 1234"#;

let re = Regex::new(
    r#"^(?<ip>\S+) \S+ \S+ \[(?<time>[^\]]+)\] "(?<method>\w+) (?<path>\S+) [^"]+" (?<status>\d{3})"#,
).unwrap();

let caps = re.captures(line).unwrap();

assert_eq!(&caps["ip"], "127.0.0.1");
assert_eq!(&caps["method"], "GET");
assert_eq!(&caps["path"], "/health");
assert_eq!(&caps["status"], "200");
```

**Why it fits:** named groups make the pattern self-documenting, which matters
because a regex this size is otherwise unreadable six months later. For a format
you control, a real parser is better — but for someone else's log format, this is
the proportionate tool.

### Use case: Finding every match in a document

Scanning rather than validating: collect all the occurrences and do something
with each.

```
use regex::Regex;

let text = "See RFC 2616, RFC 7231 and RFC 9110 for details.";
let re = Regex::new(r"RFC (\d+)").unwrap();

let numbers: Vec<&str> = re
    .captures_iter(text)
    .map(|caps| caps.get(1).unwrap().as_str())
    .collect();

assert_eq!(numbers, ["2616", "7231", "9110"]);

// find_iter gives whole matches with positions, when you need offsets.
let first = re.find(text).unwrap();
assert_eq!(first.as_str(), "RFC 2616");
assert_eq!(first.start(), 4);
```

**Why it fits:** the iterators are lazy and allocate nothing per match beyond the
capture positions, so scanning a large document is cheap. Matches never overlap
and are returned left to right, which is what makes a single pass sufficient.

### Use case: Rewriting text with a computed replacement

Substitution where the new text depends on what matched — the case a plain
`str::replace` cannot do.

```
use regex::Regex;

let re = Regex::new(r"\b(\d+)px\b").unwrap();
let css = "margin: 16px; padding: 8px;";

// A closure gets the captures and returns the replacement.
let rem = re.replace_all(css, |caps: &regex::Captures| {
    let px: f64 = caps[1].parse().unwrap();
    format!("{}rem", px / 16.0)
});

assert_eq!(rem, "margin: 1rem; padding: 0.5rem;");

// A string replacement can reference groups by number or name.
let doubled = re.replace_all(css, "$1 pixels");
assert_eq!(doubled, "margin: 16 pixels; padding: 8 pixels;");
```

**Why it fits:** `replace_all` returns a `Cow`, so a haystack with no matches is
returned borrowed with no allocation at all. The closure form is what lets the
replacement be computed rather than templated.

## API map

Nearly everything is a method on `Regex`. The entries below group by what you
are asking: does it match, where, what did the groups capture, and what should
replace it.

### Building

#### `Regex::new`

Compiles a pattern, returning an error for invalid syntax.

```
use regex::Regex;

let re = Regex::new(r"^\d+$").unwrap();
assert!(re.is_match("123"));
assert!(!re.is_match("12a"));

// Invalid syntax is an error rather than a panic.
assert!(Regex::new(r"(unclosed").is_err());

// So are the constructs this engine deliberately lacks.
assert!(Regex::new(r"(\w)\1").is_err());      // <- backreference
assert!(Regex::new(r"foo(?=bar)").is_err());  // <- lookahead
```

**When to use it:** once per pattern, at startup or in a lazy static. Use raw
strings (`r"..."`) so backslashes reach the regex engine rather than being eaten
by Rust's string escapes — this is the most common source of confusing "invalid
pattern" errors.

#### `RegexBuilder`

Compilation options: case-insensitivity, multi-line mode, size limits.

```
use regex::RegexBuilder;

let re = RegexBuilder::new(r"^error")
    .case_insensitive(true)
    .multi_line(true) // <- ^ matches at each line start, not just the input start
    .build()
    .unwrap();

let log = "ok\nERROR: disk full\nok";
assert_eq!(re.find_iter(log).count(), 1);
```

**When to use it:** flags you would otherwise embed as `(?im)` in the pattern —
the builder is clearer. `size_limit` is the one that matters for untrusted
patterns: it caps the compiled automaton so a hostile pattern cannot exhaust
memory, which is the remaining resource risk once time is bounded.

### Testing and locating

#### `is_match`

Whether the pattern occurs anywhere in the haystack.

```
use regex::Regex;

let re = Regex::new(r"\bcat\b").unwrap();

assert!(re.is_match("the cat sat"));
assert!(!re.is_match("concatenate")); // <- \b stops it matching inside a word
```

**When to use it:** validation and filtering, where you only need yes or no. It
is the fastest method because it can stop at the first match and never records
positions — prefer it over `find(..).is_some()`.

#### `find` and `find_iter`

The location of the first match, or of every match.

```
use regex::Regex;

let re = Regex::new(r"\d+").unwrap();
let text = "a1 bb22 c333";

let first = re.find(text).unwrap();
assert_eq!(first.as_str(), "1");
assert_eq!(first.range(), 1..2);

let all: Vec<&str> = re.find_iter(text).map(|m| m.as_str()).collect();
assert_eq!(all, ["1", "22", "333"]);
```

**When to use it:** when you need the matched text or its offsets but no
sub-groups. A `Match` borrows the haystack, so there is no copying — `as_str`,
`start`, `end` and `range` are all views into the original.

#### `split` and `splitn`

Splitting on a pattern rather than a fixed string.

```
use regex::Regex;

let re = Regex::new(r"[,;]\s*").unwrap();
let fields: Vec<&str> = re.split("a, b;c,  d").collect();

assert_eq!(fields, ["a", "b", "c", "d"]);

// splitn caps the number of pieces, leaving the rest intact.
let two: Vec<&str> = re.splitn("a, b, c", 2).collect();
assert_eq!(two, ["a", "b, c"]);
```

**When to use it:** separators that vary — mixed delimiters, runs of whitespace,
optional spacing. For a single fixed separator `str::split` is faster and
clearer; this is for when the separator itself needs a pattern.

### Captures

#### `captures`

The sub-groups of the first match.

```
use regex::Regex;

let re = Regex::new(r"(\w+)@(\w+)\.com").unwrap();
let caps = re.captures("mail ada@example.com now").unwrap();

assert_eq!(&caps[0], "ada@example.com"); // <- group 0 is the whole match
assert_eq!(&caps[1], "ada");
assert_eq!(&caps[2], "example");

// get returns Option, for groups that may not have participated.
assert!(caps.get(3).is_none());
```

**When to use it:** whenever you want the parts rather than the whole. Indexing
with `[n]` panics on a missing group, so use `get` when a group is inside `?` or
an alternation and might not have matched.

#### Named groups

`(?<name>...)` in the pattern, `caps["name"]` to read it.

```
use regex::Regex;

let re = Regex::new(r"(?<key>\w+)\s*=\s*(?<value>.+)").unwrap();
let caps = re.captures("timeout = 30s").unwrap();

assert_eq!(&caps["key"], "timeout");
assert_eq!(&caps["value"], "30s");

// The names are introspectable, which is useful for generic handling.
let names: Vec<&str> = re.capture_names().flatten().collect();
assert_eq!(names, ["key", "value"]);
```

**When to use it:** any pattern with more than two groups. Numbered groups
renumber themselves the moment someone adds a parenthesis in the middle, and the
resulting bug is silent — named groups do not.

#### `captures_iter`

Sub-groups for every match.

```
use regex::Regex;

let re = Regex::new(r"(\w+):(\d+)").unwrap();
let text = "alpha:1 beta:22";

let pairs: Vec<(String, u32)> = re
    .captures_iter(text)
    .map(|c| (c[1].to_string(), c[2].parse().unwrap()))
    .collect();

assert_eq!(pairs, [("alpha".to_string(), 1), ("beta".to_string(), 22)]);
```

**When to use it:** extracting structured data from a document — every key-value
pair, every reference, every entry. It is the workhorse for turning
semi-structured text into typed values.

#### `Captures::extract`

Destructures a fixed number of groups into an array, checked at compile time.

```
use regex::Regex;

let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
let caps = re.captures("2026-09-18").unwrap();

let (whole, [year, month, day]) = caps.extract();

assert_eq!(whole, "2026-09-18");
assert_eq!((year, month, day), ("2026", "09", "18"));
```

**When to use it:** patterns with a known group count, which is most of them. The
array size is checked against the pattern at run time on the first call, and
binding by name in a `let` reads far better than three indexed accesses.

### Replacing

#### `replace` and `replace_all`

Substitution, with `$1` and `$name` referring to groups.

```
use regex::Regex;

let re = Regex::new(r"(?<first>\w+) (?<last>\w+)").unwrap();

assert_eq!(re.replace("Ada Lovelace", "$last, $first"), "Lovelace, Ada");

let re = Regex::new(r"\s+").unwrap();
assert_eq!(re.replace_all("a   b \n c", " "), "a b c");
```

**When to use it:** reformatting text. `replace` does the first match only,
`replace_all` every one. Both return a `Cow`, so a haystack with no match costs
no allocation — worth knowing when you run a substitution over many strings that
mostly don't match.

#### Replacement closures

A function computing each replacement from its captures.

```
use regex::{Captures, Regex};

let re = Regex::new(r"\{(\w+)\}").unwrap();

let template = "Hello {name}, you are {age}";
let filled = re.replace_all(template, |caps: &Captures| match &caps[1] {
    "name" => "Ada".to_string(),
    "age" => "36".to_string(),
    other => format!("{{{other}}}"), // <- leave unknown placeholders alone
});

assert_eq!(filled, "Hello Ada, you are 36");
```

**When to use it:** when the replacement is computed — a lookup, a conversion, a
counter. This is the form that makes `replace_all` a small templating engine, and
the fallback arm shows the useful habit of leaving unrecognised input untouched.

### Matching many patterns

#### `RegexSet`

Tests a haystack against many patterns in a single pass.

```
use regex::RegexSet;

let set = RegexSet::new([
    r"^\d+$",           // all digits
    r"^[a-z]+$",        // all lowercase
    r"^\w+$",           // word characters
]).unwrap();

let matches: Vec<usize> = set.matches("abc").into_iter().collect();
assert_eq!(matches, [1, 2]); // <- patterns 1 and 2, not 0

assert!(set.is_match("123"));
assert!(!set.is_match("!!"));
```

**When to use it:** routing and classification — deciding which of many rules
apply. One pass over the haystack tests all the patterns, where a loop of
individual regexes is one pass each. The limitation is that it reports *which*
patterns matched but not *where*, so re-run the individual regex if you need
positions.

### Bytes

#### `regex::bytes`

The same engine over `&[u8]`, for data that may not be UTF-8.

```
use regex::bytes::Regex;

let re = Regex::new(r"(?-u)\x00(\w+)\x00").unwrap();
let data: &[u8] = b"junk\x00name\x00more";

let caps = re.captures(data).unwrap();
assert_eq!(&caps[1], b"name");
```

**When to use it:** binary formats, log files with invalid UTF-8, network
payloads. Note `(?-u)` turning Unicode mode off — with it on, the engine
refuses patterns that could match invalid UTF-8, which is a safety property for
`&str` and an obstacle here.
