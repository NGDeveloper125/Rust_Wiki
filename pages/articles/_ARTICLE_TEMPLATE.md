<!--
  ARTICLE TEMPLATE — copy me, don't edit me.

  How to use:
    1. Copy this file in `pages/articles/` and rename it to your slug, e.g.
       `pages/articles/lifetimes-without-the-fear.md`. The file name becomes
       the URL (`articles/<slug>.html`), so keep it short, lowercase, and
       hyphenated.
    2. Fill in the frontmatter below (all fields except `tags` and `image`
       are required — the build errors and names any missing field).
    3. Replace the body with your article, then open a pull request.

  Files whose name starts with `_` (like this one) are skipped by the site
  generator, so this template never appears on the website.

  What articles are for: TECHNICAL, CODE-FIRST articles about how Rust works and
  how to implement things (error handling, custom iterators, lifetimes, …) —
  real, compiling code with the reasoning behind it. NOT opinion/think-pieces
  ("why you should use Rust", "where Rust will be in 10 years") — an article
  shows code and explains it. NOT crate/library write-ups (those get their own
  dedicated section). See CONTRIBUTING.md ("Articles") for the full guidelines.
-->
---
title: "Your article title"
author: "Your Name"                 # display name shown in the byline
github: "your-handle"               # your GitHub handle (a leading @ is fine)
date: "2026-01-01"                  # YYYY-MM-DD; the maintainer adjusts this at merge
summary: "One or two sentences shown on the article card and used for search. Wrap code tokens in backticks, e.g. the `?` operator, so they read as code."
tags: ["concept", "beginner"]       # small free list; `topics:` is also accepted
# image: "images/<your-slug>.png"   # optional lead image; name it after this article's slug and drop it in pages/articles/images/ (see CONTRIBUTING.md → Articles → Images)
---

<!--
  Below is free-form prose — there is NO fixed section structure. Use whatever
  headings fit your topic. The scaffold here is only a suggestion; delete or
  reshape it freely.

  Reminders:
  - Open with why the topic matters before the how.
  - Code fences are plain (no language tag) — all code is treated as Rust and
    highlighted automatically. Use `// <-` comments to point at key lines.
  - Link liberally into the wiki: link to a page's markdown file and it's
    rewritten to the right .html URL. Paths are relative to pages/articles/,
    so wiki pages are `../concepts/...` or `../syntax/...`.
-->

An opening paragraph or two: what problem or question does this article tackle,
and why should the reader care? Set the stage before diving into mechanics.

## A first heading

Explain the core idea. Reference other pages inline where it helps, for example
the [`?` operator](../syntax/operators/question-mark.md) or
[`Result<T, E>`](../concepts/error-handling/result.md).

```
fn example() -> Result<(), std::io::Error> {
    let data = std::fs::read_to_string("config.toml")?; // <- the line that matters
    println!("{data}");
    Ok(())
}
```

Walk through what the code shows, one idea at a time.

## Another heading

Build on the first section. Use lists where they read better than prose:

- A point worth calling out on its own.
- A trade-off, a gotcha, or a "when not to do this".

## Wrapping up

Land the plane: restate the one idea you most want the reader to keep.
