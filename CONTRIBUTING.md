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
showcases — those get their own dedicated section. Articles are for the
language and how to build with it.

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
  (crate/library showcases belong in the upcoming crates section, not here).
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
