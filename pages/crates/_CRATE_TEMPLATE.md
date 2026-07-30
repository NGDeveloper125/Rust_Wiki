<!--
  CRATE PAGE TEMPLATE — copy me, don't edit me.

  How to use:
    1. Copy this file in `pages/crates/` and rename it to the crate's name on
       crates.io, e.g. `pages/crates/serde_json.md`. The file name becomes the
       URL (`crates/<name>.html`) and the default crates.io / docs.rs links.
    2. Fill in the frontmatter below (title, author, github, date and summary
       are required — the build errors and names any missing field). Take
       `version` and `publisher` from the crate's crates.io page so they are
       real, verified values, not guesses.
    3. Fill in the three sections. Unlike an article, a crate page has a FIXED
       structure: every crate page has these same three `##` headings, in this
       order, so a reader can look up any crate the same way every time.
    4. Open a pull request.

  Files whose name starts with `_` (like this one) are skipped by the site
  generator, so this template never appears on the website.

  What a crate page is for: helping someone decide whether a crate fits their
  problem, and then actually use it — what it is, the situations it's a good
  fit for, and a map of its API with a small call example for every item. See
  CONTRIBUTING.md ("Crates") for the full guidelines.
-->
---
title: "crate_name"                 # usually just the crate's name
# crate: "crate_name"               # only if it differs from this file's name
version: "1.0.0"                    # the exact release you wrote the page against
publisher: "Their Name (handle)"    # the crate's owner(s) as shown on crates.io
publisher_url: "https://crates.io/users/handle"   # optional link for the above
no_std: "optional"                  # yes | optional | no — omit if unknown
author: "Your Name"                 # display name shown in the byline
github: "your-handle"               # your GitHub handle (a leading @ is fine)
date: "2026-01-01"                  # YYYY-MM-DD; the maintainer adjusts this at merge
summary: "One or two sentences shown on the crate card and used for search. Wrap code tokens in backticks, e.g. the `?` operator, so they read as code."
categories: ["error-handling"]      # small free list; `tags:` is also accepted
repository: "https://github.com/owner/repo"   # optional
# docs: "https://docs.rs/crate_name"          # optional; defaults to docs.rs/<crate>
---

## Overview

What the crate is, in a paragraph or two: the problem it solves, the shape of
its API, and anything that decides whether it belongs in a project — maturity,
dependency weight, `no_std` support, how it relates to the obvious alternatives.

Link into the wiki wherever it helps: paths are relative to `pages/crates/`, so
wiki pages are `../concepts/...` or `../syntax/...`, e.g.
[`Result<T, E>`](../concepts/error-handling/result.md).

## When to use it

An optional sentence or two framing the situations below, then one
`### Use case:` block per situation. Two to four is a good number — each should
be a real situation with real code, not a restatement of the overview.

### Use case: A short, concrete situation

Set up the situation in a sentence, then show it.

```
fn main() {
    println!("real, compiling code");
}
```

**Why it fits:** one or two sentences on what the crate bought you here — the
last block of a use case, rendered as a callout.

### Use case: Another situation

Same shape as above.

```
fn example() {}
```

**Why it fits:** ...

## API map

An optional sentence or two, then the crate's API grouped under `###`
headings, with one `####` entry per item. Group by what a reader is trying to
do ("Creating errors", "Reading a file") rather than by module path.

### A group of related items

Optional prose about the group as a whole.

#### `some_function`

One or two sentences on what it does.

```
let value = some_function("input");
```

**When to use it:** when you'd reach for it, and what to use instead when you
wouldn't — the last block of an entry, rendered as a callout.

#### `SomeType::method`

Same shape as above. Aim to cover the API a reader actually needs; a map with
every item and a one-line example beats a prose tour that covers half of them.

```
let out = SomeType::new().method();
```

**When to use it:** ...
