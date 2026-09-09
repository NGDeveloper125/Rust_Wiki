---
title: "winnow"
version: "1.0.4"
publisher: "Ed Page (epage)"
publisher_url: "https://crates.io/users/epage"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-09"
summary: "A parser combinator library: build a parser from small functions that compose, instead of writing a state machine or reaching for a regex. Parsers are ordinary functions taking `&mut Stream`."
categories: ["parsing", "text-processing", "no-std"]
repository: "https://github.com/winnow-rs/winnow"
---

## Overview

Between "split on commas" and "write a real parser" is a wide gap. A regex stops
being readable the moment the grammar nests; a hand-written state machine works
but is a lot of code to get the error messages right.

`winnow` fills that gap with combinators. A parser is a function taking
`&mut Stream` and returning what it recognised, and larger parsers are built by
combining smaller ones:

```
use winnow::ascii::dec_uint;
use winnow::combinator::separated;
use winnow::prelude::*;

fn numbers(input: &mut &str) -> ModalResult<Vec<u32>> {
    separated(1.., dec_uint::<_, u32, _>, ',').parse_next(input)
}

assert_eq!(numbers.parse("1,2,30").unwrap(), vec![1, 2, 30]);
assert!(numbers.parse("1,,2").is_err());
```

The shape to internalise: **the input is a `&mut` reference, and a successful
parser advances it.** That is the design decision that separates winnow from
`nom`, which it forked from — nom threads the remaining input through return
values, winnow mutates a cursor. It makes parsers read like ordinary imperative
code, and it makes backtracking explicit, because a failing parser resets the
cursor for you.

**Two things decide whether it fits.**

The first is whether you need a parser at all. For a fixed, flat format, `split`
and `parse` are shorter and clearer. For a real grammar — nested expressions,
quoted strings with escapes, an indentation-sensitive config — combinators are
the point where they start paying off. And if your input is already JSON, TOML
or YAML, use that format's crate rather than parsing it yourself.

The second is which parser library. `winnow` and `nom` are close relatives with
similar power; winnow aims at a gentler learning curve and better error
messages, and is what `toml` and `cargo` use. `pest` takes a grammar file
instead of Rust code, which suits a formal grammar you want written down.
`chumsky` leans furthest into error recovery, for tooling that must keep going
after a mistake. Any of them beats a regex once nesting appears.

**Errors are the part worth planning for.** By default a parser that fails
*backtracks*: `alt` tries the next branch. `cut_err` says "past this point the
input is committed", which turns a vague "no alternative matched" into a message
naming what was actually wrong. Adding `.context()` at the right places is most
of the difference between a parser people can use and one they can't.

The crate has no required dependencies, requires Rust 1.65, and works `no_std`
with `alloc`.

## When to use it

### Use case: A key-value config line

The size of problem where a regex still fits but is already less readable than
combinators.

```
use winnow::ascii::{space0, till_line_ending};
use winnow::combinator::{delimited, separated_pair};
use winnow::prelude::*;
use winnow::token::take_while;

fn setting(input: &mut &str) -> ModalResult<(String, String)> {
    separated_pair(
        take_while(1.., |c: char| c.is_alphanumeric() || c == '_').map(String::from),
        delimited(space0, '=', space0),
        till_line_ending.map(str::trim).map(String::from),
    )
    .parse_next(input)
}

let (key, value) = setting.parse("max_retries = 5").unwrap();
assert_eq!(key, "max_retries");
assert_eq!(value, "5");

// A missing `=` is an error rather than a silent half-match.
assert!(setting.parse("max_retries 5").is_err());
```

**Why it fits:** each piece names what it recognises, so the grammar is legible
top to bottom. `separated_pair` discards the separator and keeps the two sides,
which is exactly the shape of the data.

### Use case: A nested expression grammar

Where regexes stop working entirely. Recursion is just a parser calling itself.

```
use winnow::ascii::{dec_int, multispace0};
use winnow::combinator::{alt, delimited, repeat};
use winnow::prelude::*;

// term := int | '(' expr ')'
// expr := term (('+' | '-') term)*
fn expr(input: &mut &str) -> ModalResult<i64> {
    let init = term.parse_next(input)?;
    repeat(0.., (delimited(multispace0, alt(('+', '-')), multispace0), term))
        .fold(
            move || init,
            |acc, (op, val): (char, i64)| if op == '+' { acc + val } else { acc - val },
        )
        .parse_next(input)
}

fn term(input: &mut &str) -> ModalResult<i64> {
    alt((
        dec_int,
        delimited(('(', multispace0), expr, (multispace0, ')')),
    ))
    .parse_next(input)
}

assert_eq!(expr.parse("1 + 2 - 3").unwrap(), 0);
assert_eq!(expr.parse("(1 + 2) - (3 - 4)").unwrap(), 4);
```

**Why it fits:** `term` calls `expr` and `expr` calls `term`, which is the
grammar written down. Doing this with a regex is impossible — nesting is not a
regular language — and doing it by hand means a stack and a loop you have to get
right.

### Use case: Errors that say what was expected

A parser that only says "failed" is a parser people will avoid. `cut_err` and
`context` turn a backtrack into a diagnosis.

```
use winnow::ascii::dec_uint;
use winnow::combinator::{cut_err, preceded};
use winnow::prelude::*;

fn port(input: &mut &str) -> ModalResult<u16> {
    preceded(
        "port=",
        // Past "port=" the input is committed: don't backtrack, report.
        cut_err(dec_uint::<_, u16, _>).context(winnow::error::StrContext::Label("port number")),
    )
    .parse_next(input)
}

assert_eq!(port.parse("port=8080").unwrap(), 8080);

let err = port.parse("port=abc").unwrap_err();
let rendered = err.to_string();
assert!(rendered.contains("port number"), "got: {rendered}");
```

**Why it fits:** without `cut_err` the whole parser just backtracks and the
caller learns only that nothing matched. With it, the error points at the offset
after `port=` and says what belonged there. Put a cut wherever the input has
committed to one interpretation.

## API map

A parser is anything implementing `Parser` — usually a plain
`fn(&mut Stream) -> ModalResult<T>`. `use winnow::prelude::*;` brings in the
trait and the result type. The entries below split into the parsers that consume
input, the combinators that join them, and the adapters that reshape a result.

One signature detail to know up front: a parser returning a borrowed slice needs
the input's lifetime named, as `fn p<'i>(input: &mut &'i str) -> ModalResult<&'i str>`.
Writing `&mut &str` there compiles for parsers returning owned values and fails
confusingly for ones returning `&str`.

### Running a parser

#### `Parser::parse`

Runs a parser over a whole input, requiring it to consume everything.

```
use winnow::ascii::alpha1;
use winnow::prelude::*;

assert_eq!(alpha1::<_, winnow::error::ContextError>.parse("abc").unwrap(), "abc");

// Trailing input is an error: parse means "parse all of this".
assert!(alpha1::<_, winnow::error::ContextError>.parse("abc123").is_err());
```

**When to use it:** at the top level, once. It produces a `ParseError` carrying
the offset, which is what you render for a user. Everything inside your parser
uses `parse_next` instead.

#### `Parser::parse_next`

Runs a parser against a mutable stream, advancing it past what matched.

```
use winnow::ascii::digit1;
use winnow::prelude::*;

let mut input = "123abc";
let digits = digit1::<_, winnow::error::ContextError>.parse_next(&mut input).unwrap();

assert_eq!(digits, "123");
assert_eq!(input, "abc"); // <- the cursor moved
```

**When to use it:** inside a parser, which is where nearly all your calls are.
The mutation is the whole model — after a success the stream points at what is
left, and after a failure it is back where it started.

#### `Parser::parse_peek`

Runs a parser without committing the cursor, returning the remainder alongside
the output.

```
use winnow::ascii::digit1;
use winnow::prelude::*;

let (rest, matched) = digit1::<_, winnow::error::ContextError>.parse_peek("42x").unwrap();

assert_eq!(matched, "42");
assert_eq!(rest, "x");
```

**When to use it:** tests, and the rare place you want to inspect a result
before deciding. In a grammar prefer `peek`, which is the combinator form and
keeps the code in the same style as everything around it.

### Recognising tokens

#### `literal`

Matches an exact string or byte sequence. A bare `&str` or `char` works as a
parser directly.

```
use winnow::prelude::*;
use winnow::token::literal;

fn verb<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    literal("GET").parse_next(input)
}

// The shorthand: a string literal is itself a parser.
fn other_verb<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    "POST".parse_next(input)
}

let mut input = "GET /path";
assert_eq!(verb.parse_next(&mut input).unwrap(), "GET");
assert_eq!(input, " /path"); // <- cursor advanced

assert_eq!(other_verb.parse("POST").unwrap(), "POST");
```

**When to use it:** keywords, punctuation, fixed markers. The shorthand is what
you will usually write — `'('` and `"=>"` inside a combinator are literals
without naming the function.

#### `take_while` and `take_till`

Consume input while, or until, a predicate holds.

```
use winnow::prelude::*;
use winnow::token::{take_till, take_while};

let mut input = "name42=value";
let ident = take_while::<_, _, winnow::error::ContextError>(1.., |c: char| c.is_alphanumeric())
    .parse_next(&mut input)
    .unwrap();
assert_eq!(ident, "name42");

let mut line = "key: value";
let key = take_till::<_, _, winnow::error::ContextError>(1.., ':')
    .parse_next(&mut line)
    .unwrap();
assert_eq!(key, "key");
```

**When to use it:** the workhorses for identifiers, numbers and runs of
anything. The range is the important argument — `0..` allows empty, `1..` does
not, and using `0..` inside `repeat` is how you write an infinite loop by
accident.

#### `one_of` and `any`

A single token from a set, or any single token.

```
use winnow::prelude::*;
use winnow::token::{any, one_of};

let mut input = "+5";
let sign = one_of::<_, _, winnow::error::ContextError>(['+', '-']).parse_next(&mut input).unwrap();
assert_eq!(sign, '+');

let mut other = "xy";
assert_eq!(any::<_, winnow::error::ContextError>.parse_next(&mut other).unwrap(), 'x');
```

**When to use it:** operators, sign characters, single-character flags.
`none_of` is the inverse, for "any character except these", which is how the
body of a quoted string is usually written.

#### The `ascii` parsers

Ready-made parsers for the common character classes and numbers.

```
use winnow::ascii::{alpha1, dec_int, digit1, multispace0};
use winnow::prelude::*;

type E = winnow::error::ContextError;

assert_eq!(alpha1::<_, E>.parse("abc").unwrap(), "abc");
assert_eq!(digit1::<_, E>.parse("123").unwrap(), "123");
assert_eq!(dec_int::<_, i32, E>.parse("-42").unwrap(), -42);
assert_eq!(multispace0::<_, E>.parse("  \n\t").unwrap(), "  \n\t");
```

**When to use it:** rather than writing the predicate yourself. `dec_int` and
`dec_uint` parse straight to an integer type, handling the sign and overflow —
which `digit1` followed by `parse` does not, so prefer them for numbers.

### Combining parsers

#### `alt`

Tries each alternative in order, taking the first that matches.

```
use winnow::combinator::alt;
use winnow::prelude::*;

fn boolean(input: &mut &str) -> ModalResult<bool> {
    alt(("true".value(true), "false".value(false))).parse_next(input)
}

assert!(boolean.parse("true").unwrap());
assert!(!boolean.parse("false").unwrap());
assert!(boolean.parse("maybe").is_err());
```

**When to use it:** any "one of these" in the grammar. Order matters: a
shorter alternative that is a prefix of a longer one must come second, or it
wins and leaves the rest unparsed. Failures backtrack, which is what makes
trying alternatives safe.

#### Sequences with tuples

A tuple of parsers runs them in order and yields a tuple of results.

```
use winnow::ascii::dec_uint;
use winnow::prelude::*;

fn version(input: &mut &str) -> ModalResult<(u32, u32)> {
    // Four parsers in order; the tuple yields all four results.
    let (_, major, _, minor) =
        ("v", dec_uint::<_, u32, _>, '.', dec_uint::<_, u32, _>).parse_next(input)?;
    Ok((major, minor))
}

assert_eq!(version.parse("v2.7").unwrap(), (2, 7));
assert!(version.parse("2.7").is_err()); // <- the "v" is required
```

**When to use it:** whenever things must appear in order. It is the most common
combinator and needs no function call — the tuple itself implements `Parser`,
which is why grammars read as nested tuples.

#### `preceded`, `terminated` and `delimited`

Sequences that discard the parts you don't want.

```
use winnow::ascii::alpha1;
use winnow::combinator::{delimited, preceded, terminated};
use winnow::prelude::*;

type E = winnow::error::ContextError;

assert_eq!(preceded::<_, _, _, E, _, _>("$", alpha1).parse("$name").unwrap(), "name");
assert_eq!(terminated::<_, _, _, E, _, _>(alpha1, ";").parse("name;").unwrap(), "name");
assert_eq!(delimited::<_, _, _, _, E, _, _, _>('"', alpha1, '"').parse("\"name\"").unwrap(), "name");
```

**When to use it:** brackets, quotes, prefixes and separators — anything
structural you must match but don't want in the output. `delimited` is the one
you reach for constantly, for every kind of bracketed thing.

#### `repeat` and `separated`

Zero or more of something, optionally with separators between.

```
use winnow::ascii::dec_uint;
use winnow::combinator::{repeat, separated};
use winnow::prelude::*;

fn numbers(input: &mut &str) -> ModalResult<Vec<u32>> {
    repeat(1.., dec_uint::<_, u32, _>).parse_next(input)
}

fn csv(input: &mut &str) -> ModalResult<Vec<u32>> {
    separated(0.., dec_uint::<_, u32, _>, ',').parse_next(input)
}

// dec_uint is greedy: "123" is one number, not three.
assert_eq!(numbers.parse("123").unwrap(), vec![123]);

assert_eq!(csv.parse("1,2,3").unwrap(), vec![1, 2, 3]);
assert_eq!(csv.parse("").unwrap(), Vec::<u32>::new());
```

**When to use it:** lists, sequences of statements, repeated fields. Both take a
range, so `1..` requires at least one and `0..=3` caps the count. Watch the
greediness — each iteration takes as much as it can, so repeating `dec_uint`
over `"123"` yields one number rather than three; a separator is usually what
makes the boundaries real, which is what `separated` is for. The inner parser
must also consume input or the repetition never terminates, and winnow detects
that and errors rather than hanging.

#### `opt`

Makes a parser optional, yielding `Option`.

```
use winnow::ascii::dec_uint;
use winnow::combinator::opt;
use winnow::prelude::*;

fn signed(input: &mut &str) -> ModalResult<i64> {
    let sign = opt('-').parse_next(input)?;
    let value = dec_uint::<_, u64, _>.parse_next(input)? as i64;
    Ok(if sign.is_some() { -value } else { value })
}

assert_eq!(signed.parse("42").unwrap(), 42);
assert_eq!(signed.parse("-42").unwrap(), -42);
```

**When to use it:** optional prefixes, trailing separators, anything the grammar
allows to be absent. It never fails — a non-match yields `None` and leaves the
cursor alone.

### Shaping the result

#### `Parser::map` and `Parser::value`

Transform what a parser produced, or replace it with a constant.

```
use winnow::ascii::digit1;
use winnow::prelude::*;

// Reshape the matched text.
fn digit_len(input: &mut &str) -> ModalResult<usize> {
    digit1.map(|s: &str| s.len()).parse_next(input)
}

// Discard it in favour of a value.
fn flag(input: &mut &str) -> ModalResult<bool> {
    "yes".value(true).parse_next(input)
}

assert_eq!(digit_len.parse("12345").unwrap(), 5);
assert!(flag.parse("yes").unwrap());
```

**When to use it:** `map` to build your own types as you parse, so the parser
returns an AST rather than strings. `value` for keywords that stand for a
constant — an enum variant, a boolean — where the matched text itself is noise.

#### `Parser::parse_to` and `Parser::try_map`

Convert the matched text into another type, fallibly.

```
use winnow::ascii::digit1;
use winnow::prelude::*;

type E = winnow::error::ContextError;

// parse_to uses FromStr.
let n: u32 = digit1::<_, E>.parse_to().parse("123").unwrap();
assert_eq!(n, 123);

// try_map takes the conversion explicitly and reports its error.
let m = digit1::<_, E>.try_map(|s: &str| s.parse::<u8>()).parse("42").unwrap();
assert_eq!(m, 42);

// Out of range for u8, so the parse fails rather than wrapping.
assert!(digit1::<_, E>.try_map(|s: &str| s.parse::<u8>()).parse("999").is_err());
```

**When to use it:** turning text into numbers, dates or enums. The failure
becomes a parse error at the right offset, which is the advantage over parsing
the string later — by then you have lost where it came from.

#### `Parser::verify`

Rejects a result that doesn't satisfy a predicate.

```
use winnow::ascii::digit1;
use winnow::prelude::*;

fn year<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    digit1.verify(|s: &str| s.len() == 4).parse_next(input)
}

assert_eq!(year.parse("2026").unwrap(), "2026");
assert!(year.parse("26").is_err());
```

**When to use it:** constraints the grammar can't express directly — a fixed
width, a value in range, an identifier that isn't a reserved word. It fails as a
normal backtrack, so `alt` can still try another branch.

#### `Parser::take`

Returns the input the parser consumed, rather than what it produced.

```
use winnow::ascii::{alpha1, digit1};
use winnow::prelude::*;

// The tuple produces ("abc", "123"); take gives the whole matched slice.
fn ident<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    (alpha1, digit1).take().parse_next(input)
}

assert_eq!(ident.parse("abc123").unwrap(), "abc123");
```

**When to use it:** when the structure matters for matching but you want the raw
text — an identifier with an internal shape, a number with separators you plan
to clean up later. It avoids reassembling the pieces you just took apart.

### Errors

#### `cut_err`

Stops backtracking: past this point a failure is an error, not a rejected
alternative.

```
use winnow::ascii::dec_uint;
use winnow::combinator::{alt, cut_err, preceded};
use winnow::prelude::*;

fn value(input: &mut &str) -> ModalResult<u32> {
    alt((
        preceded("num:", cut_err(dec_uint::<_, u32, _>)),
        "zero".value(0),
    ))
    .parse_next(input)
}

assert_eq!(value.parse("num:7").unwrap(), 7);
assert_eq!(value.parse("zero").unwrap(), 0);

// After "num:" the alternative is committed, so this does not fall through
// to the "zero" branch — it reports a bad number.
assert!(value.parse("num:xyz").is_err());
```

**When to use it:** immediately after the token that identifies which
alternative you are in. This is the single biggest improvement you can make to a
combinator parser's error messages, and it costs one function call.

#### `Parser::context`

Attaches a label to a parser, so the error can say what was expected.

```
use winnow::ascii::dec_uint;
use winnow::error::StrContext;
use winnow::prelude::*;

let mut parser = dec_uint::<_, u32, winnow::error::ContextError>
    .context(StrContext::Label("timeout in seconds"));

assert_eq!(parser.parse("30").unwrap(), 30);

let err = parser.parse("soon").unwrap_err();
assert!(err.to_string().contains("timeout in seconds"));
```

**When to use it:** on the named things in your grammar rather than on every
parser — a field, a value, a whole clause. `StrContext::Expected` is the other
variant, for describing the specific token rather than the construct.

#### `ParseError` and offsets

What `parse` returns on failure: the input, the position, and the cause.

```
use winnow::ascii::alpha1;
use winnow::prelude::*;

let err = alpha1::<_, winnow::error::ContextError>.parse("abc123").unwrap_err();

assert_eq!(err.offset(), 3); // <- where it stopped
assert_eq!(err.input(), &"abc123");
assert!(err.to_string().contains('^')); // <- renders a caret under the offset
```

**When to use it:** rendering a failure for a human. The `Display` impl already
produces a pointed-at-the-offset message, so printing it is usually enough;
`offset` is there when you are building your own diagnostic with line numbers.
