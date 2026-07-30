<div align="center">

# 🦀 Rusty Yellow Pages

**A Rust reference for people writing Rust.**

Part dictionary, part wiki — meant to be kept open in a second tab while you code.

[**Open the site →**](https://rustyyellowpages.dev/)

</div>

---

## What this is

Rusty Yellow Pages is a free, open-source Rust reference. It's built for the moment you're mid-code and need one specific thing — what `?` desugars to, whether `Rc` is thread-safe, how a `match` guard behaves — and want the answer, a snippet that compiles, and a short note on *why*, not a chapter.

The site is two things:

- **📖 A dictionary — for syntax.** Every operator, keyword, attribute, literal, macro, and bit of punctuation has its own short, look-it-up page. Around **180 entries**.
- **🧠 A wiki — for concepts.** Ownership, lifetimes, traits, iterators, async, unsafe, error handling, design patterns, the language's philosophy — the parts you have to understand, not just recall. Around **126 concept pages** across 15 topic groups.

Concept pages go past a plain explanation. Each carries a **Best practices & deeper information** section made of concrete *scenarios* — "creating a new object", "working with collections" — each showing a recommended way to handle it. Many also ship an **Embedded** variant for `no_std` / bare-metal work.

The site is fully static: HTML generated from markdown, no server, no database, no tracking, no accounts.

> **Where the content comes from:** every page is distilled from the **official Rust documentation** (the Book, the Reference, the Nomicon, the API guidelines, `std` docs) and the mainstream Rust books. It's curated, not invented. If you spot something wrong, outdated, or misleading, that's exactly the kind of feedback worth sending — see [below](#found-something-wrong-tell-me).

---

## 🌱 Community

The aim is for this reference to grow with contributions from people writing Rust, not just the maintainer. There are four ways knowledge is added and shared — three through pull requests, one through discussion:

### 1. Approaches — *live now* ✅

There's rarely one right way to do something in Rust. The same problem can be an iterator chain, a pre-allocated buffer, an arena, a zero-mutation rewrite — each with its own trade-offs.

Every scenario on a concept page starts with the site's recommended **Classic** solution. Anyone can add an **Approach**: an alternative implementation of the *exact same scenario*, attributed to you. When a scenario has more than one, readers get an `Approach:` dropdown to switch between them and can 👍 the ones they find useful; higher-voted approaches sort to the top. Contributions are purely additive — you never touch the Classic content or anyone else's — so they're low-risk to write and easy to review.

Full walkthrough at the [bottom of this README](#-approaches-in-depth) and in **[CONTRIBUTING.md](CONTRIBUTING.md)**.

### 2. Articles — *live now* ✅

Community-written, **technical, code-first articles** — how something in Rust actually works, how to implement a feature, how to solve a concrete problem — with real, compiling code in the article itself and links into the rest of the wiki. Free-form prose, your byline on it, and readers can 👍 the ones they find useful. It's *not* the place for opinion or think-pieces ("why you should use Rust", "where Rust will be in ten years"); an article shows code and explains it. Crate and library write-ups have their own section (below) — articles are for the language and how to build with it.

Full walkthrough at the [bottom of this README](#-articles-in-depth) and in **[CONTRIBUTING.md](CONTRIBUTING.md#articles)**.

### 3. Crates — *live now* ✅

The wiki covers the language; **Crates** covers the ecosystem. Each page documents one crate with the *same three sections every time* — what it is, the situations it's a good fit for, and a map of its API with a small call example, an explanation, and a "when to use it" for every item. That fixed shape is the point: looking up an unfamiliar crate works the same way every time, and you can skim to the one entry you need instead of reading a README end to end.

Full walkthrough at the [bottom of this README](#-crates-in-depth) and in **[CONTRIBUTING.md](CONTRIBUTING.md#crates)**.

### 4. Conversations — *live now* ✅

A community discussion area, **mirrored into the site's own styling** from the repo's [GitHub Discussions](https://github.com/NGDeveloper125/Rust_Wiki/discussions) — a place to ask questions, compare approaches, and share what you know.

Threads and replies live on GitHub Discussions; the site fetches them at build time and renders a read-only, styled copy at the [Conversations page](https://rustyyellowpages.dev/conversations/). To start a thread or reply you click through to GitHub — the site never accepts writes, so posting stays accountable and moderation uses GitHub's native tools. Unlike Approaches and Articles, **this isn't a pull request**: you just post on GitHub and it appears on the site at the next rebuild (a near-live snapshot, refreshed on a schedule and when discussions change).

Full picture at the [bottom of this README](#-conversations-in-depth).

---

## 🛠️ Using the repo: add, report, suggest

### ➕ Add or improve content

Content is authored in markdown under `pages/`; the HTML under `docs/` is generated from it by the maintainer. Just edit the markdown — you don't need to build or touch `docs/`.

1. Edit or add the relevant file:
   - Syntax entries → `pages/syntax/<group>/<slug>.md`
   - Concept pages → `pages/concepts/<subgroup>/<slug>.md`
   - Articles → `pages/articles/<slug>.md` (copy `_ARTICLE_TEMPLATE.md` to start)
   - Crate pages → `pages/crates/<crate-name>.md` (copy `_CRATE_TEMPLATE.md` to start)
2. Open a pull request describing what you changed and why.

The site is regenerated and republished when the PR is merged, so a markdown-only PR is all that's needed. The most common contribution — adding an **Approach** — is a purely additive markdown block. **[CONTRIBUTING.md](CONTRIBUTING.md)** has the exact format and the review criteria.

### 🐛 Report an issue

Found a bug on the site, a page that won't render, a broken link, or content that's wrong? [**Open an issue**](https://github.com/NGDeveloper125/Rust_Wiki/issues/new) with the page, what's wrong (a link or screenshot helps), and what it should say if you know. Wrong or outdated content is the highest-priority kind of report.

### 💡 Suggest a change or addition

Missing page, a scenario worth covering, or a feature idea? [**Open an issue**](https://github.com/NGDeveloper125/Rust_Wiki/issues/new). For anything larger than a small fix — a new concept page or scenario — it's worth discussing in an issue before writing it.

---

## 🔧 Preview locally (optional)

You don't need to build anything to contribute — a markdown PR is enough, and the maintainer regenerates the site on merge. But if you want to see your change rendered before opening the PR, and you have a Rust toolchain:

```sh
cd tools/sitegen
cargo run
```

That reads `pages/` and writes HTML into `docs/`. The generator prints a warning if a page is malformed or an attribution line is missing, so a clean build of your page is a good sign. Open the generated file (e.g. `docs/concepts/ownership-borrowing/ownership.html`) in a browser to check it. There's no need to commit the regenerated `docs/` — the maintainer rebuilds it.

---

## 📬 Contact

Contributions of any size are welcome — a typo fix, an approach, an article, or just an idea.

- 🐙 **GitHub:** open an [issue](https://github.com/NGDeveloper125/Rust_Wiki/issues) or a PR — best for anything about the content.
- 📧 **Email:** `RustyYellowPages@outlook.com` — for anything else, or to reach me directly.

Two notes:

- I maintain this on my own, so a new PR or issue may sit a **day or two** before I get to it. I'll do my best to keep the turnaround quick.
- The content is curated from the official Rust docs and the mainstream Rust books. If a page looks wrong, imprecise, or out of date, let me know — corrections take priority.

### Found something wrong? Tell me.

A reference is only worth trusting if it gets corrected. Open an issue, send an email, or open a PR — whichever you prefer.

---

## 📄 License

This project is split into two parts, licensed separately:

- **Code** — the site generator (`tools/`) and the site's JavaScript, CSS, and
  HTML templates — is under the **MIT License**. See [LICENSE](LICENSE).
- **Content** — the reference material under `pages/` (syntax and concept pages,
  community articles, crate pages, and approaches) — is under **Creative Commons Attribution
  4.0 International (CC BY 4.0)**. See [LICENSE-CONTENT](LICENSE-CONTENT). You're
  free to reuse and adapt it, including commercially, as long as you credit
  Rusty Yellow Pages.

---

<a id="-approaches-in-depth"></a>

## 📚 Approaches, in depth

This is the flagship community feature and the one that's live today. Here's the full picture.

### What an approach is

Every concept page has a **Best practices & deeper information** section made up of *scenarios* — concrete situations like *"Creating a new object"* or *"Working with collections"*. Each scenario shows the site's own recommended way of handling it, labelled **Classic**.

An **approach** is an alternative way to implement the *exact same scenario*, contributed by someone in the community. The scenario doesn't change — only the implementation does. So a single scenario can carry several approaches side by side: the Classic one, plus `The 0-mutation approach`, plus `Arena-based`, plus whatever else people contribute.

When a scenario has more than one approach, it shows an **`Approach:` dropdown**. Pick an entry and the code, explanation, and rationale below it swap to that approach. Scenarios that only have the Classic way look exactly as they always have — no dropdown, no extra UI.

### How it works

The whole thing is static — there is no server and no database.

- Approaches are plain markdown living inside the page file, right next to the Classic content. When the site is generated, each approach becomes an entry in the scenario's `Approach:` dropdown and a matching content panel, all baked into the HTML at build time.
- Switching approaches happens entirely in your browser — selecting a different entry just shows the corresponding panel. Nothing is loaded from anywhere.
- The default selection is always **Classic**.

### How to add a new approach

You add an approach by editing the page's markdown and opening a pull request — no forms, no accounts beyond GitHub, no backend. Inside the scenario you want to extend, append a block like this:

````markdown
#### Approach: The 0-mutation approach

*Contributed by [@your-handle](https://github.com/your-handle)*

A sentence or two on what this approach does and when it's a good fit.

```
fn example() {
    // your Rust code, with `// <-` comments on the key lines
}
```

**Why this way:** an optional closing note explaining the trade-off.
````

- The `#### Approach:` title becomes the dropdown entry, so keep it short.
- The `*Contributed by ...*` line is your **attribution** — it links to your GitHub profile and is shown with your approach, so you get credit.
- You never touch the Classic content or anyone else's approach; your change is purely additive, which makes it easy to review and merge.

A maintainer reviews the PR (the code must compile and genuinely fit the scenario) and merges it. That's it — your approach is now live on the page.

### How to like an approach

Readers can show which approaches they find most useful.

- Each contributed approach has a **👍 like button** next to its attribution.
- Clicking it takes you to a small GitHub issue for that approach; react with a **👍** there to cast your like. (A GitHub account is all you need — which also keeps the votes honest.)
- The page reads those 👍 counts live from GitHub each time it loads. The like button shows the current count, and approaches are **sorted by likes** in the dropdown, so the community's favourites rise to the top. Classic always stays first as the default.

### Why this is helpful

- **There's rarely one "right" way in Rust.** A problem can be solved with an iterator chain, a pre-allocated buffer, an arena, and more — each with different trade-offs. Showing them together, on the same scenario, teaches far more than a single blessed answer.
- **Credit stays with the author.** Every approach carries its contributor's name and profile link, so sharing what you know is recognised.
- **The best ideas surface themselves.** Likes let readers, not just maintainers, signal which alternatives are genuinely useful, and the ordering reflects that automatically.
- **Contributing is low-friction and safe.** Because an approach is an additive markdown block reviewed through a normal pull request, anyone can share a technique without risk of breaking the existing content.

Together these turn each scenario from a one-voice recommendation into a small, curated collection of community knowledge — which is exactly how a living reference should grow.

---

<a id="-articles-in-depth"></a>

## 📝 Articles, in depth

Articles are community-written, **technical, code-first** pieces about how Rust works and how to build with it, rendered in the site's own styling and cross-linked into the rest of the wiki.

### What an article is

Where a concept page is a look-it-up reference and an approach is one scenario's alternative, an **article** is a longer, standalone, technical piece that takes a single thing — what the `?` operator really does under the hood, how to implement a custom iterator, how lifetimes flow through a function — and works through it with real, compiling code and the reasoning behind it.

Articles are **free-form**: unlike concept and syntax pages they have no fixed section structure, so use whatever headings fit the topic. What they are *not*: opinion or think-pieces with no code ("why you should use Rust", "where Rust will be in ten years"), or crate/library write-ups (those go in [Crates](#-crates-in-depth)). Every article is about code and implementation.

### The goal

- **Go deeper than a reference entry can.** Some things need more room than a scenario or a look-up page — how something works under the hood, or how to actually build it — and an article gives that room without cluttering the dictionary.
- **Grow the reference with the community's knowledge, credited.** Every article carries its author's byline and GitHub link, so sharing what you know is recognised.
- **Stay technical and code-first.** Every article shows real, compiling Rust and explains it — how something works or how to implement it — so readers leave with something they can use, not an opinion to agree or disagree with.

### How it works

The whole thing is static — there is no server and no database.

- Each article is a single markdown file under `pages/articles/`. At build time it's rendered to `articles/<slug>.html` and added to the **Articles index** — a card grid showing each article's title, summary, byline, tags, and an optional image.
- The index has a **search box** (filter the cards by words in the title or summary) and a **sort toggle**: *Newest* (default) or *Top rated*.
- Article bodies link into the wiki: a relative link to a concept or syntax page's markdown file is rewritten to the right `.html` URL at build time, so articles sit naturally alongside the reference.

### How to add a new article

You add an article by dropping a markdown file in `pages/articles/` and opening a pull request — no forms, no accounts beyond GitHub, no backend.

1. Copy [`pages/articles/_ARTICLE_TEMPLATE.md`](pages/articles/_ARTICLE_TEMPLATE.md) and rename the copy to your slug — the file name becomes the URL, e.g. `pages/articles/lifetimes-without-the-fear.md`. Files whose name starts with `_` are skipped by the generator, so the template itself never publishes (and the same trick parks a work-in-progress draft as `_my-draft.md`).
2. Fill in the frontmatter — `title`, `author`, `github`, `date`, `summary`, plus a few `tags` (everything except `tags` and an optional `image` is required). If a required field is missing the build prints a clear error naming the file and the field.
3. Write the body in free-form markdown, with real compiling code and `// <-` comments on the key lines. Link into the wiki wherever it helps.
4. Open the PR. A maintainer reviews it (code compiles, on-topic for the language/its concepts, original, civil), **sets the publication `date` at merge**, and merges — and your article is live on the [Articles page](https://rustyyellowpages.dev/articles/).

The full frontmatter reference and style guidance are in **[CONTRIBUTING.md](CONTRIBUTING.md#articles)**.

### How to like an article

Just like approaches, readers can show which articles they find most useful.

- Each article has a **👍 like button** on its card in the index and in its byline.
- Clicking it opens a small GitHub issue for that article; react with a **👍** there to cast your like. (A GitHub account is all you need — which keeps the count honest.)
- The site reads those counts live from GitHub on each load, shows the current number on the button, and the Articles page's **Top rated** sort orders by them — so the community's favourites rise to the top.

*(For maintainers: enabling the like button on a merged article means creating its `article-vote` issue — the exact step is in [CONTRIBUTING.md](CONTRIBUTING.md#articles).)*

---

<a id="-crates-in-depth"></a>

## 📦 Crates, in depth

The wiki documents the language. **Crates** documents the ecosystem — one page per crate, all built the same way.

### What a crate page is

A crate page answers two questions in one place: *should I use this crate?* and *how do I call it?* It's aimed at the moment you've seen a crate name in someone's `Cargo.toml` or a blog post and want a straight answer without reading a README, a docs.rs index, and three issues.

The defining feature is that **every crate page has the same three sections, in the same order** — the opposite of an article, which is deliberately free-form:

1. **Overview** — what the crate is and the problem it solves, plus the things that decide whether it belongs in your project: maturity, dependency weight, `no_std` support, how it compares to the obvious alternatives.
2. **When to use it** — two to four concrete situations the crate is a good fit for, each with real code and a short *why it fits*.
3. **API map** — the crate's API, grouped by what you're trying to do, with **one entry per item**: what it does, a small call example, and a *when to use it* (including what to reach for instead when the answer is "not this").

That fixed shape is the whole point. Once you've read one crate page you know exactly where to look in the next one, and the API map is skimmable — you can jump straight to the one function you needed a snippet for.

### How it works

The whole thing is static — there is no server and no database.

- Each crate page is a single markdown file under `pages/crates/`, named after the crate on crates.io. At build time it's rendered to `crates/<crate-name>.html` and added to the **Crates index** — a card grid showing each crate's name, version, summary, categories, and how many API entries its map covers.
- Every page states the **exact release it documents** and **who publishes the crate** (its crates.io owner), taken from crates.io rather than guessed — kept visually separate from the byline crediting whoever wrote the page. It links out to **crates.io**, **docs.rs**, and the crate's repository, and shows a `no_std` badge using the same language as the wiki's embedded-support badges.
- Pages are kept honest by a maintainer-side `crate-sync` check that diffs every page against crates.io: owner and link changes are corrected automatically, while a new release is flagged for a human to re-check the API map before the documented version is bumped.
- The index has a **search box** (filter by name, summary, or category) and a **sort toggle**: *A–Z* (default), *Newest*, or *Top rated*.
- Crate pages link into the wiki: a relative link to a concept or syntax page's markdown file is rewritten to the right `.html` URL at build time, so a crate page can point at the language concepts it builds on.

### How to add a new crate page

Same as everything else here: drop a markdown file in `pages/crates/` and open a pull request.

1. Copy [`pages/crates/_CRATE_TEMPLATE.md`](pages/crates/_CRATE_TEMPLATE.md) and rename the copy to the crate's name on crates.io — the file name becomes the URL and the default crates.io/docs.rs links, e.g. `pages/crates/anyhow.md`. Files whose name starts with `_` are skipped by the generator.
2. Fill in the frontmatter — `title`, `author`, `github`, `date` and `summary` are required; `version`, `no_std`, `categories`, `repository` and `docs` are optional. A missing required field prints a clear build error naming the file and the field.
3. Fill in the three sections, keeping the headings exactly as the template has them. Every API entry gets a compiling call example and a *when to use it*.
4. Open the PR. A maintainer reviews it (accurate for the stated version, snippets compile, the API map is complete enough to be useful, and the page is *fair* — a reference entry, not a pitch), **sets the publication `date` at merge**, and merges — and the page is live on the [Crates page](https://rustyyellowpages.dev/crates/).

The full frontmatter reference and section-by-section guidance are in **[CONTRIBUTING.md](CONTRIBUTING.md#crates)**.

### How to like a crate page

Identical to articles: each page has a **👍 like button** on its index card and in its byline, which opens a small GitHub issue for that page — react with a **👍** there to cast your like. Counts are read live from GitHub on each load, and the Crates page's **Top rated** sort orders by them.

*(For maintainers: enabling the like button on a merged crate page means creating its `crate-vote` issue — the exact step is in [CONTRIBUTING.md](CONTRIBUTING.md#crates).)*

---

<a id="-conversations-in-depth"></a>

## 💬 Conversations, in depth

Conversations is a **read-only mirror of the repo's GitHub Discussions**, rendered in the site's own look so discussion lives right next to the reference.

### How it works

The whole thing is static — there is no server and no database.

- **Storage is GitHub Discussions.** Every thread and reply is a real GitHub Discussion in this repo. The site never stores or accepts messages.
- **The site is a build-time snapshot.** When the site is generated it fetches all discussions via the GitHub API and renders a styled, read-only copy: an index of threads (title, author, date, category, reply count, a preview of the latest reply, and an **Expand** button that opens the whole thread inline) plus a dedicated page per thread.
- **Writing happens on GitHub.** Every *"Start a conversation"* and *"Add a comment"* button links out to GitHub Discussions. A GitHub account is all you need — which keeps posting accountable and lets maintainers moderate with GitHub's native tools.
- **It stays fresh automatically.** A GitHub Actions workflow rebuilds and republishes the site on a schedule (every ~6 hours), on each push, and whenever a discussion or comment changes — so new posts appear within minutes. It's a near-live snapshot, not live.
- **Untrusted content is sanitized.** Discussion markdown is rendered safely: raw HTML is stripped and unsafe links are neutralized, so nothing posted on GitHub can run as code on the site.

### How to take part

1. Open the [Conversations page](https://rustyyellowpages.dev/conversations/) on the site (also reachable from the sidebar on every page), or go straight to the repo's [Discussions tab](https://github.com/NGDeveloper125/Rust_Wiki/discussions).
2. Browse threads on the site; click through to GitHub to start one or reply.
3. Post in the category that fits — **Approaches & Idioms**, **Ecosystem**, **Q&A**, or **Site feedback**.

There's nothing to build, edit, or open a PR for — post on GitHub Discussions and it shows up on the site at the next rebuild.
