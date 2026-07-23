# Contributing to the Rust Wiki

Thanks for wanting to contribute! The main way to contribute is adding an
**alternative approach** to a best-practice scenario on a concept page — a
different way to implement the *exact same scenario*. The site's own
recommended way stays as the default **"Classic"** entry; your approach
appears as an extra option in the scenario's `Approach:` dropdown, with your
name on it. You can also propose a brand-new scenario (see below).

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

## Build & preview locally

The site is generated by a small Rust tool and served straight from `docs/`
on `main` — there is no CI, so the generated output ships with your PR.

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
- Include the regenerated `docs/` output for your page in the PR, along with
  your edit to the markdown under `pages/`.

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
  --body "React with a 👍 to vote for this approach. See it on the page: https://ngdeveloper125.github.io/Rust_Wiki/concepts/collections-strings/vec.html"
```

Notes:

- The title must match the markdown exactly (scenario and approach titles are
  case- and punctuation-sensitive) — a mismatch just means no count shows.
- One issue per approach, even if it appears in both the normal and
  `(Embedded)` sections — votes are shared.
- If an approach is renamed, rename its issue title to match.
- The site reads the first 100 open `approach-vote` issues in one API call;
  revisit (pagination) if we ever approach that many.
