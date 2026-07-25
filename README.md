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

The aim is for this reference to grow with contributions from people writing Rust, not just the maintainer. There are three ways knowledge gets added, each through a normal pull request:

### 1. Approaches — *live now* ✅

There's rarely one right way to do something in Rust. The same problem can be an iterator chain, a pre-allocated buffer, an arena, a zero-mutation rewrite — each with its own trade-offs.

Every scenario on a concept page starts with the site's recommended **Classic** solution. Anyone can add an **Approach**: an alternative implementation of the *exact same scenario*, attributed to you. When a scenario has more than one, readers get an `Approach:` dropdown to switch between them and can 👍 the ones they find useful; higher-voted approaches sort to the top. Contributions are purely additive — you never touch the Classic content or anyone else's — so they're low-risk to write and easy to review.

Full walkthrough at the [bottom of this README](#-approaches-in-depth) and in **[CONTRIBUTING.md](CONTRIBUTING.md)**.

### 2. Articles — *on the way* 🚧

Technical, code-first articles on Rust and its ecosystem: how to implement something, how a crate works, how to solve a concrete problem — with real, compiling code in the article itself. This is not the place for opinion or think-pieces ("why you should use Rust", "where Rust will be in ten years"); an article shows code and explains it. Free-form prose, your byline, linked into the rest of the wiki.

### 3. Conversations — *on the way* 🚧

A community discussion area, mirrored into the site's own styling — a place to ask questions, compare approaches, and share what you know.

---

## 🛠️ Using the repo: add, report, suggest

### ➕ Add or improve content

Content is authored in markdown under `pages/`; the HTML under `docs/` is generated from it by the maintainer. Just edit the markdown — you don't need to build or touch `docs/`.

1. Edit or add the relevant file:
   - Syntax entries → `pages/syntax/<group>/<slug>.md`
   - Concept pages → `pages/concepts/<subgroup>/<slug>.md`
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

See [LICENSE](LICENSE).

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
