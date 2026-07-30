# Contributing to the Rust Wiki

Thanks for wanting to contribute! The main way to contribute is adding an
**alternative approach** to a best-practice scenario on a concept page — a
different way to implement the *exact same scenario*. The site's own
recommended way stays as the default **"Classic"** entry; your approach
appears as an extra option in the scenario's `Approach:` dropdown, with your
name on it. You can also propose a brand-new scenario, or write a full
long-form **[article](#articles)** (see below).

Contribution is entirely PR-based markdown — no forms, no backend. You edit a
page file, open a pull request, and a maintainer reviews and merges it.

**Licensing of contributions:** by contributing, you agree that the content you
submit (prose, code examples, articles, crate pages, approaches, and any images) is published
under the project's content license, **CC BY 4.0**, with attribution to you
preserved; any code you contribute to the tooling is under the **MIT License** —
the same terms as the rest of the project. See
[LICENSE](LICENSE) and [LICENSE-CONTENT](LICENSE-CONTENT).

## What you can contribute

Every concept page (`pages/concepts/<subgroup>/<slug>.md`) has a
`## Best practices & deeper information` section made of
`### Scenario: <name>` blocks. Each scenario shows one recommended way to do
something — the Classic approach. If you know a different, defensible way to
implement the same scenario (a different data structure, an iterator-based
formulation, an arena, whatever), you can add it as an approach. An approach
does **not** change what the scenario is about — it is another implementation
of exactly the same situation.

Name your approach after the idea (`The 0 mutation`, `Arena-based`,
`Iterator-chain`) or after yourself (`The JamesHill approach`) — the name
becomes its entry in the scenario's `Approach:` dropdown. Don't use `::` in
the name (it's reserved for the vote-issue naming scheme below).

## Liking approaches

Readers can vote for the approach they prefer: each community approach has a
👍 chip next to its attribution that links to a GitHub issue — react with
👍 on that issue to like the approach. The like count is shown on the chip
itself, and higher-liked approaches are listed first in the `Approach:`
dropdown (Classic always stays the default). Counts are fetched live from the
GitHub API on each page load, so a reload reflects the current votes.

## Where to add your approach

1. Find the page: `pages/concepts/<subgroup>/<slug>.md`
   (e.g. `pages/concepts/collections-strings/vec.md`).
2. Find the `### Scenario:` block you're targeting, inside either:
   - `## Best practices & deeper information` — the normal (Classic-flavor) section, and/or
   - `## Best practices & deeper information (Embedded)` — the embedded-Rust variant.
3. Append your `#### Approach:` block at the **end of that scenario block**
   (after its `**Why this way:**` paragraph, before the next `### Scenario:`
   heading or the next `## ` section).

Your approach shows up only in the flavor(s) you add it to: add it under the
normal section and it appears when the page is in Classic view; add it under
the `(Embedded)` section and it appears in Embedded view; add it under both
(same or adapted content) and it appears in both.

**Never modify the existing Classic content or another contributor's
approach** — your diff should be purely additive.

## Format

````markdown
#### Approach: <short title — it becomes the dropdown entry>

*Contributed by [@your-handle](https://github.com/your-handle)*

A paragraph or two explaining the approach and when it beats (or trades off
against) the Classic one.

```
fn example() {
    // your code, with `// <-` comments pointing at the key lines
}
```

**Why this way:** optional closing rationale. If present it must start with
exactly `**Why this way:**` and be the final paragraph of your block.
````

Rules (the site generator enforces the structural ones and prints warnings
during the build):

- The heading must be exactly `#### Approach: ` followed by a short title —
  it becomes the dropdown entry, so keep it to a few words (e.g.
  `Iterator-chain`, `Arena-based`, `The JamesHill approach`).
- The **attribution line is mandatory** and must be the first paragraph of
  your block: `*Contributed by [@your-handle](https://github.com/your-handle)*`.
  It links to your GitHub profile and is shown at the top of your approach.
  A display name is fine too: `*Contributed by [Jane Doe](https://github.com/janedoe)*`.
- Code fences are plain (untagged) — all code on this site is Rust and gets
  highlighted client-side.
- The `**Why this way:**` rationale is optional, but if present it must be
  the last paragraph of your block.
- Don't use `#### ` headings inside your approach body — the next
  `#### Approach: ` line starts the next approach.
- Internal links to other wiki pages work: link to the markdown file
  (e.g. `[Vec<T>](../collections-strings/vec.md)`) and the generator rewrites
  it to the right HTML page.

## Review criteria

A maintainer will check that:

- **The code compiles** on stable Rust (paste it into the
  [Rust Playground](https://play.rust-lang.org/) to check). Embedded-section
  approaches may use the crates the surrounding embedded examples already use
  (e.g. `heapless`).
- **It's idiomatic — or the deviation is argued.** If your approach does
  something unusual, the `**Why this way:**` rationale should justify it.
- **It fits the scenario.** It must solve the same problem the scenario
  describes, not a related-but-different one.
- **It's genuinely an alternative**, not a small variation of the Classic
  code or of an existing approach.

## Proposing a brand-new scenario

If a concept page is missing a situation worth covering, you can propose a
whole new scenario instead of an approach. Add a new `### Scenario: <name>`
block at the end of the `## Best practices & deeper information` section
(and/or its `(Embedded)` counterpart), following the same format as the
existing ones on the page: a 1–2 sentence setup, one code block with
`// <-` comments pointing at the key lines, and a closing
`**Why this way:**` paragraph.

The content you write becomes the scenario's Classic approach; others can
later contribute alternative approaches to it. New scenarios are reviewed
more strictly than approaches — they must cover a genuinely distinct
situation the page doesn't already handle, so it's worth opening an issue
to discuss it first.

## Preview locally (optional)

You don't need to build anything to contribute. A markdown-only PR (your edit
under `pages/`) is all that's needed — the maintainer regenerates
`docs/` from your markdown and republishes it on merge, so **don't include
`docs/` changes in your PR.**

If you'd like to see your change rendered before opening the PR, and you have
a Rust toolchain, the site is generated by a small Rust tool:

```
cd tools/sitegen
cargo run
```

- Watch the console: the build prints a warning if your attribution line is
  missing or a section is malformed. A clean build of your page = no new
  warnings.
- Open `docs/concepts/<subgroup>/<slug>.html` in a browser and pick your
  approach from the scenario's `Approach:` dropdown: Classic should stay the
  default, selecting yours should switch the content, and your attribution
  should be visible and link to your profile.
- This is just a local check — leave the regenerated `docs/` output out of the
  PR.

## For maintainers: wiring up voting for a merged approach

After merging an approach PR, create its vote issue so the like button and
count appear on the site. The issue title must be exactly
`<page-path>::<scenario title>::<approach title>` (page path = the page's
docs path without `.html`), with the `approach-vote` label:

```
# once per repo:
gh label create approach-vote --description "Vote issue for a community approach" --color F5C518

# once per merged approach:
gh issue create \
  --title "concepts/collections-strings/vec::Creating a new object::Collect from an iterator" \
  --label approach-vote \
  --body "React with a 👍 to vote for this approach. See it on the page: https://rustyyellowpages.dev/concepts/collections-strings/vec.html"
```

Notes:

- The title must match the markdown exactly (scenario and approach titles are
  case- and punctuation-sensitive) — a mismatch just means no count shows.
- One issue per approach, even if it appears in both the normal and
  `(Embedded)` sections — votes are shared.
- If an approach is renamed, rename its issue title to match.
- The site reads the first 100 open `approach-vote` issues in one API call;
  revisit (pagination) if we ever approach that many.

## Articles

Beyond per-scenario approaches, you can contribute a **full article** — a
long-form, **technical, code-first** piece about how Rust works or how to
implement something (say, a deep dive into error handling, or how to build a
custom iterator). Articles are free-form prose: unlike concept and syntax pages
they are **not** forced into the `Explanation` / `Basic usage` /
`Best practices` structure. Write it however the topic wants to be written — but
every article is about code and implementation: it shows real, compiling Rust
and explains it.

Two things articles are *not*: opinion or think-pieces with no code ("why you
should use Rust", "where Rust will be in ten years"), and crate/library
write-ups — those have their own section, [Crates](#crates). Articles are for
the language and how to build with it.

Like everything else here, an article is a markdown pull request: you add one
file, open a PR, and a maintainer reviews and merges it. It then appears on the
[Articles page](https://rustyyellowpages.dev/articles/).

### Where the file goes

One markdown file, flat, under `pages/articles/`:

```
pages/articles/<your-slug>.md
```

The file name becomes the URL (`articles/<your-slug>.html`), so pick a short,
descriptive, hyphenated slug (e.g. `a-tour-of-the-question-mark-operator.md`).

**Start from the template.** Copy
[`pages/articles/_ARTICLE_TEMPLATE.md`](pages/articles/_ARTICLE_TEMPLATE.md),
rename the copy to your slug, and fill it in — the frontmatter and a suggested
structure are already there. Files whose name starts with `_` (like the
template) are skipped by the generator, so the template never appears on the
site; the same trick works for a work-in-progress draft (`_my-draft.md`).

### Frontmatter

Every article starts with this YAML block. All fields except `tags` and
`image` are **required** — the build prints a clear error naming the file and
the missing field if one is absent, and skips the article until it's fixed.

```yaml
---
title: "A tour of the ? operator: error handling that gets out of your way"
author: "Your Name"                # display name for the byline
github: "your-handle"              # your GitHub handle (a leading @ is fine)
date: "2026-07-25"                 # YYYY-MM-DD — see the note below
summary: "One or two sentences shown in listings and used for search."
tags: ["error-handling", "result", "beginner"]   # small free list; `topics:` also accepted
image: "images/optional-lead.png"  # optional; see "Images" below
---

Your article body starts here...
```

**About `date`:** set it to the day you open the PR; the **maintainer sets or
adjusts it at merge** so the publication date reflects when the article
actually went live. Articles are listed newest-first by this date.

**Inline code in the `summary`:** wrap code tokens in backticks — e.g.
``summary: "How the `?` operator works"`` — and they render as distinct
monospace on the card, so an operator like `?` reads as code instead of stray
punctuation.

### Writing the body

- Use any structure you like: `##` and `###` headings, paragraphs, lists,
  block quotes, tables, and code blocks.
- Code fences are plain (untagged) — all code on this site is treated as Rust
  and highlighted client-side. Use `// <-` comments to point at key lines,
  matching the rest of the site.
- **Link liberally into the wiki.** Link to another page's markdown file and
  the generator rewrites it to the right `.html` URL, exactly like scenario
  bodies do:

  ```markdown
  The [`?` operator](../syntax/operators/question-mark.md) unwraps a
  [`Result<T, E>`](../concepts/error-handling/result.md) or returns early.
  ```

  Paths are relative to `pages/articles/`, so wiki pages are `../concepts/...`
  or `../syntax/...`. A link with no matching page prints a build warning.

### Images (optional)

To include an image, drop the file in `pages/articles/images/` and reference it
as `images/<file>` — either as the frontmatter `image:` (a lead image shown on
the card and at the top of the article) or inline in the body with normal
markdown. An external `https://…` URL works in `image:` too. Keep images small
and relevant; the site ships them as-is.

**Name assets after the article they belong to.** Use the article's slug (the
markdown file name without `.md`) as the image name, so it's obvious which
article an asset serves:

```
pages/articles/
  a-tour-of-the-question-mark-operator.md
  images/
    a-tour-of-the-question-mark-operator.png        # the lead image
    a-tour-of-the-question-mark-operator-fig-1.png  # extra images: slug + a suffix
```

Then the frontmatter is `image: images/a-tour-of-the-question-mark-operator.png`.
This keeps each article's assets self-documenting and avoids name collisions in
the shared `images/` folder. A lead image reads best as a wide banner (about a
**2:1** aspect ratio).

### Review criteria

A maintainer will check that the article is:

- **Technically accurate**, and any code **compiles** on stable Rust (paste it
  into the [Rust Playground](https://play.rust-lang.org/) to check).
- **On-topic** — a Rust concept, language feature, or the standard library
  (crate/library write-ups belong in [Crates](#crates), not here).
- **Original content** you wrote (or have the right to contribute), not a
  copy of an existing post or the docs.
- **Civil and constructive** in tone. Opinion and "here's how I'd approach it"
  pieces are welcome; put-downs of people or projects are not.

### For maintainers: enabling the like/rating on a merged article

Articles use the same 👍 mechanism as approaches. Readers "like" an article by
reacting to its GitHub issue, the count shows on the article's card and byline,
and the Articles page can sort by it (**Sort → Top rated**). After merging,
create the article's vote issue — title exactly `article::<slug>` (the file
stem), with the `article-vote` label:

```
# once per repo:
gh label create article-vote --description "Vote issue for a community article" --color F5C518

# once per merged article:
gh issue create \
  --title "article::a-tour-of-the-question-mark-operator" \
  --label article-vote \
  --body "React with a 👍 to like this article. Read it: https://rustyyellowpages.dev/articles/a-tour-of-the-question-mark-operator.html"
```

Until the issue exists the article simply shows no like chip and sorts as 0
under "Top rated" — nothing breaks. The site reads the first 100 open
`article-vote` issues in one anonymous API call.

## Crates

A **crate page** documents one crate from the ecosystem: what it is, the
situations it's a good fit for, and a map of its API with a small call example
for every item. It's the section you reach for when you've heard of a crate and
want to know, in one page, whether it solves your problem and how to call it.

Where an article is deliberately free-form, a crate page is the opposite: every
crate page has the **same three sections, in the same order**, so looking up an
unfamiliar crate always works the same way.

| Section | What goes in it |
| --- | --- |
| `## Overview` | What the crate is and the problem it solves — plus what decides whether it belongs in a project (maturity, dependency weight, `no_std`, the obvious alternatives). |
| `## When to use it` | Two to four `### Use case:` blocks: a concrete situation, real code, and a `**Why it fits:**` line. |
| `## API map` | The API, grouped under `###` headings, with one `####` entry per item: what it does, a small call example, and a `**When to use it:**` line. |

Like everything else here, a crate page is a markdown pull request: you add one
file, open a PR, and a maintainer reviews and merges it. It then appears on the
[Crates page](https://rustyyellowpages.dev/crates/).

### Where the file goes

One markdown file, flat, under `pages/crates/`, **named after the crate on
crates.io**:

```
pages/crates/<crate-name>.md
```

The file name becomes the URL (`crates/<crate-name>.html`) and the default
crates.io and docs.rs links, so `pages/crates/serde_json.md` is right and
`pages/crates/serde-json-guide.md` is not. (If the name really has to differ,
set `crate:` in the frontmatter.)

**Start from the template.** Copy
[`pages/crates/_CRATE_TEMPLATE.md`](pages/crates/_CRATE_TEMPLATE.md), rename the
copy to the crate's name, and fill it in — the frontmatter and all three
sections are already scaffolded. Files whose name starts with `_` are skipped by
the generator, so the template never appears on the site (and the same trick
parks a work-in-progress draft as `_my-draft.md`).

### Frontmatter

`title`, `author`, `github`, `date` and `summary` are **required** — the build
prints a clear error naming the file and the missing field, and skips that page
until it's fixed. Everything else is optional.

```yaml
---
title: "anyhow"                    # usually just the crate's name
crate: "anyhow"                    # optional; defaults to the file name
version: "1.0.104"                 # optional; the exact release you wrote against
publisher: "David Tolnay (dtolnay)"               # optional; the crates.io owner(s)
publisher_url: "https://crates.io/users/dtolnay"  # optional; link for the above
no_std: "optional"                 # optional; yes | optional | no
author: "Your Name"                # display name for the byline
github: "your-handle"              # your GitHub handle (a leading @ is fine)
date: "2026-07-29"                 # YYYY-MM-DD — the maintainer sets this at merge
summary: "One or two sentences shown in listings and used for search."
categories: ["error-handling"]     # small free list; `tags:` also accepted
repository: "https://github.com/dtolnay/anyhow"   # optional
docs: "https://docs.rs/anyhow"     # optional; defaults to docs.rs/<crate>
---
```

**`version` and `publisher` are facts about someone else's software — copy them
from the crate's crates.io page, don't guess.** `version` is the exact release
you wrote the page against (`1.0.104`, not `1.0`), because it's the claim a
reader relies on when they check whether the API map still applies. `publisher`
is the crate's owner as crates.io lists it; owners can be users or teams and a
crate can have several, so it's free text — a single user reads best as
`Name (login)` with `publisher_url` pointing at their crates.io page.

The page keeps these upstream facts visually separate from your byline: the
crate line says who publishes the *crate*, the byline below says who wrote the
*page*.

`no_std` renders as the same support badge language features use, so a reader
can tell at a glance whether a crate works on bare metal. Inline code in the
`summary` works exactly as it does for articles: backtick a token and it renders
as monospace on the card.

**Keeping a page current.** Crates release; pages go stale. The maintainer runs
a local `crate-sync` check that compares every page against crates.io and
reports drift — a new release, a changed owner, a moved repository. Link and
owner fixes are applied automatically; a version bump only happens after someone
has re-checked the API map, since the `version` field is a promise about what
the entries below it document.

### Writing the body

- **Keep the three `##` headings exactly as they are** — `Overview`,
  `When to use it`, `API map`. A missing one prints a build warning and the page
  publishes without that section.
- **Use cases** are `### Use case: <short title>` blocks. End each one with a
  `**Why it fits:**` paragraph — it's rendered as a callout, the same as the
  `**Why this way:**` line on a concept page's scenario.
- **API entries** are `#### <item>` under a `###` group. Group by what a reader
  is trying to *do* ("Creating an error", "Reading a file"), not by module path.
  End each entry with a `**When to use it:**` paragraph — including what to
  reach for instead when the answer is "not this".
- **Every entry gets a call example.** One short, compiling snippet showing the
  call is the point of the section; a prose tour that covers half the API isn't.
- Code fences are plain (untagged) — all code on this site is treated as Rust
  and highlighted client-side. Use `// <-` comments to point at key lines.
- **Link liberally into the wiki.** Paths are relative to `pages/crates/`, so
  wiki pages are `../concepts/...` or `../syntax/...`, and the generator
  rewrites the `.md` target to the right `.html` URL.

### Review criteria

A maintainer will check that the page is:

- **Accurate for the stated `version`**, and that every snippet **compiles**
  against it.
- **Complete enough to be useful** — the API map should cover what someone
  actually needs to use the crate, not three cherry-picked functions.
- **Fair.** A crate page is a reference entry, not a pitch: say what the crate
  is bad at and when to use something else. Comparisons to alternatives are
  welcome; put-downs of other projects or their authors are not.
- **Original content** you wrote (or have the right to contribute) — not the
  crate's README or docs.rs pages pasted in.

### For maintainers: enabling the like/rating on a merged crate page

Crate pages use the same 👍 mechanism as articles and approaches, and the
Crates page can sort by it (**Sort → Top rated**). After merging, create the
page's vote issue — title exactly `crate::<slug>` (the file stem), with the
`crate-vote` label:

```
# once per repo:
gh label create crate-vote --description "Vote issue for a community crate page" --color F5C518

# once per merged crate page:
gh issue create \
  --title "crate::anyhow" \
  --label crate-vote \
  --body "React with a 👍 to like this crate page. Read it: https://rustyyellowpages.dev/crates/anyhow.html"
```

Until the issue exists the page simply shows no like chip and sorts as 0 under
"Top rated" — nothing breaks. The site reads the first 100 open `crate-vote`
issues in one anonymous API call.

## Conversations & questions (no PR needed)

Not every contribution is a pull request. For questions, comparing approaches,
ecosystem chat, or feedback on the site, use the repo's
[**GitHub Discussions**](https://github.com/NGDeveloper125/Rust_Wiki/discussions).
Those threads are mirrored, read-only, onto the site's
[Conversations page](https://rustyyellowpages.dev/conversations/) in the site's
own styling and refreshed automatically — so a good discussion becomes a
browsable part of the reference.

- Posting and replying happen on GitHub (a GitHub account is all you need); the
  site never accepts writes, and there's nothing to build or edit.
- Pick the category that fits: **Approaches & Idioms**, **Ecosystem**, **Q&A**,
  or **Site feedback**.
- It's the right place for open-ended discussion; use a **pull request** (above)
  when you want to actually add an approach or scenario to a page, and an
  **issue** to report something wrong or broken.
