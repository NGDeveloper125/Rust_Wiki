# Community Approaches

> This README covers **one feature** of the wiki — the "Approaches" system on
> concept pages. It does not describe the rest of the project.

## What it is

Every concept page has a **Best practices & deeper information** section made
up of *scenarios* — concrete situations like *"Creating a new object"* or
*"Working with collections"*. Each scenario shows the site's own recommended
way of handling it, labelled **Classic**.

An **approach** is an alternative way to implement the *exact same scenario*,
contributed by someone in the community. The scenario doesn't change — only
the implementation does. So a single scenario can carry several approaches
side by side: the Classic one, plus `The 0-mutation approach`, plus
`Arena-based`, plus whatever else people contribute.

When a scenario has more than one approach, it shows an **`Approach:`
dropdown**. Pick an entry and the code, explanation, and rationale below it
swap to that approach. Scenarios that only have the Classic way look exactly
as they always have — no dropdown, no extra UI.

## How it works

The whole thing is static — there is no server and no database.

- Approaches are plain markdown living inside the page file, right next to the
  Classic content. When the site is generated, each approach becomes an entry
  in the scenario's `Approach:` dropdown and a matching content panel, all
  baked into the HTML at build time.
- Switching approaches happens entirely in your browser — selecting a
  different entry just shows the corresponding panel. Nothing is loaded from
  anywhere.
- The default selection is always **Classic**.

## How to add a new approach

You add an approach by editing the page's markdown and opening a pull request
— no forms, no accounts beyond GitHub, no backend. Inside the scenario you
want to extend, append a block like this:

```markdown
#### Approach: The 0-mutation approach

*Contributed by [@your-handle](https://github.com/your-handle)*

A sentence or two on what this approach does and when it's a good fit.

```
fn example() {
    // your Rust code, with `// <-` comments on the key lines
}
```

**Why this way:** an optional closing note explaining the trade-off.
```

- The `#### Approach:` title becomes the dropdown entry, so keep it short.
- The `*Contributed by ...*` line is your **attribution** — it links to your
  GitHub profile and is shown with your approach, so you get credit.
- You never touch the Classic content or anyone else's approach; your change
  is purely additive, which makes it easy to review and merge.

A maintainer reviews the PR (the code must compile and genuinely fit the
scenario) and merges it. That's it — your approach is now live on the page.

## How to like an approach

Readers can show which approaches they find most useful.

- Each contributed approach has a **👍 like button** next to its attribution.
- Clicking it takes you to a small GitHub issue for that approach; react with
  a **👍** there to cast your like. (A GitHub account is all you need — which
  also keeps the votes honest.)
- The page reads those 👍 counts live from GitHub each time it loads. The
  like button shows the current count, and approaches are **sorted by likes**
  in the dropdown, so the community's favourites rise to the top. Classic
  always stays first as the default.

## Why this is helpful

- **There's rarely one "right" way in Rust.** A problem can be solved with an
  iterator chain, a pre-allocated buffer, an arena, and more — each with
  different trade-offs. Showing them together, on the same scenario, teaches
  far more than a single blessed answer.
- **Credit stays with the author.** Every approach carries its contributor's
  name and profile link, so sharing what you know is recognised.
- **The best ideas surface themselves.** Likes let readers, not just
  maintainers, signal which alternatives are genuinely useful, and the
  ordering reflects that automatically.
- **Contributing is low-friction and safe.** Because an approach is an
  additive markdown block reviewed through a normal pull request, anyone can
  share a technique without risk of breaking the existing content.

Together these turn each scenario from a one-voice recommendation into a
small, curated collection of community knowledge — which is exactly how a
living reference should grow.
