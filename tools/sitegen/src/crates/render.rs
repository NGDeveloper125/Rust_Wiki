//! Render the crates index (a searchable, sortable card grid) and one page per
//! crate, reusing the site's shell/sidebar/card styling.

use std::io;
use std::path::Path;

use super::{ApiGroup, Crate, UseCase, DEPTH};
use crate::model::Page;
use crate::nav::{render_sidebar, TopNav};
use crate::render::{abs_url, href_from, shell, Head};
use crate::util::{fmt_date, html_escape, render_inline};

/// The repo's contributor guide, linked from the "document a crate" CTA.
const CONTRIBUTE_URL: &str =
    "https://github.com/NGDeveloper125/Rust_Wiki/blob/main/CONTRIBUTING.md#crates";

pub fn write_pages(docs_root: &Path, crates: &[Crate], pages: &[Page]) -> io::Result<()> {
    let dir = docs_root.join("crates");
    std::fs::create_dir_all(&dir)?;

    std::fs::write(dir.join("index.html"), render_index(crates, pages))?;
    for c in crates {
        std::fs::write(dir.join(format!("{}.html", c.slug)), render_crate(c, pages))?;
    }
    Ok(())
}

fn render_index(crates: &[Crate], pages: &[Page]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::Crates);
    let home = href_from(DEPTH, "index.html");

    let breadcrumb = format!(
        r#"<nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">Crates</span>
      </nav>"#
    );

    let page_head = format!(
        r#"<div class="page-head">
        <div class="title-block">
          <h1 class="page-title">Crates</h1>
        </div>
        <a class="convo-cta" href="{CONTRIBUTE_URL}" target="_blank" rel="noopener">Document a crate &rarr;</a>
      </div>"#
    );

    let lead = r#"<p class="lead">A directory of the crates people actually reach for. Every page follows the same three sections &mdash; what the crate is, the situations it fits, and a map of its API with a small call example for each item &mdash; so you can look up an unfamiliar crate the same way every time. Crate pages are contributed as markdown pull requests.</p>"#;

    let body = if crates.is_empty() {
        format!(
            r#"<div class="card convo-empty">
        <p>No crate pages yet.</p>
        <p>Be the first &mdash; <a href="{CONTRIBUTE_URL}" target="_blank" rel="noopener">document a crate and open a pull request &rarr;</a></p>
      </div>"#
        )
    } else {
        let toolbar = r#"<div class="article-toolbar">
          <div class="article-search">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="7"/><path d="m21 21-3.6-3.6"/></svg>
            <input id="crate-search" type="text" placeholder="Filter crates by name, summary or category&hellip;" autocomplete="off" spellcheck="false" aria-label="Filter crates">
          </div>
          <label class="article-sort">
            <span>Sort</span>
            <select id="crate-sort" aria-label="Sort crates">
              <option value="name" selected>A &ndash; Z</option>
              <option value="date">Newest</option>
              <option value="rating">Top rated</option>
            </select>
          </label>
        </div>"#;

        let cards: String = crates
            .iter()
            .enumerate()
            .map(|(i, c)| render_card(c, i))
            .collect::<Vec<_>>()
            .join("\n        ");

        format!(
            "{toolbar}\n        <div class=\"crate-grid\" id=\"crate-grid\">\n        {cards}\n        </div>\n        <p class=\"article-nomatch\" id=\"crate-nomatch\" hidden>No crates match your filter.</p>"
        )
    };

    let main = format!(
        r#"      {breadcrumb}

      {page_head}

      {lead}

      <hr class="divider">

      {body}

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>Crate pages are contributed as markdown pull requests</span>
      </div>
"#
    );

    let head = Head {
        title: "Rust - Crates - Rusty Yellow Pages".to_string(),
        description: "A directory of Rust crates — what each one is, when it's a good fit, and a map of its API with a call example for every item.".to_string(),
        canonical: abs_url("crates/index.html"),
        og_type: "website",
        image: None,
    };
    shell(&head, DEPTH, &sidebar, &main)
}

/// One card in the index grid. `i` is the authored (A-Z) position, used by
/// site.js to restore alphabetical order after a different sort.
fn render_card(c: &Crate, i: usize) -> String {
    let api_count = c.api_count();
    let apis = match api_count {
        0 => String::new(),
        1 => " &middot; 1 API entry".to_string(),
        n => format!(" &middot; {n} API entries"),
    };

    format!(
        r#"<article class="crate-card" data-i="{i}" data-date="{iso}" data-search="{search}" data-vote-key="{vote_key}">
          <a class="crate-card-link" href="{href}">
            <div class="crate-card-body">
              <div class="crate-card-head">
                <h3 class="crate-card-title">{title}</h3>
                {version}
              </div>
              <p class="article-card-summary">{summary_html}</p>
            </div>
          </a>
          <div class="article-card-foot">
            <div class="article-card-meta">by {author}{apis}</div>
            {categories}
            <a class="article-like" hidden target="_blank" rel="noopener">&#128077; <span class="like-n"></span></a>
          </div>
        </article>"#,
        iso = html_escape(&c.date),
        search = html_escape(&search_text(c)),
        vote_key = html_escape(&c.vote_key()),
        // The index lives at `crates/index.html`, so link to a sibling crate
        // page by bare filename, not its site-root-relative `href`.
        href = html_escape(&format!("{}.html", c.slug)),
        title = html_escape(&c.title),
        version = render_version(c),
        summary_html = render_inline(&c.summary),
        author = html_escape(&c.author),
        categories = render_categories(c),
    )
}

/// The lowercase haystack the index's filter box matches against, baked into
/// the card so the client never has to scrape it back out of the DOM.
fn search_text(c: &Crate) -> String {
    format!(
        "{} {} {} {} {}",
        c.title,
        c.crate_name,
        c.summary,
        c.categories.join(" "),
        c.publisher.as_deref().unwrap_or_default(),
    )
    .replace('`', "")
    .to_lowercase()
}

fn render_version(c: &Crate) -> String {
    match &c.version {
        Some(v) if !v.trim().is_empty() => {
            format!("<span class=\"crate-version\">v{}</span>", html_escape(v))
        }
        _ => String::new(),
    }
}

/// "published by &lt;owner&gt;" — the crate's crates.io owner(s), linked to their
/// crates.io page when `publisher_url` is set. Renders nothing without a
/// `publisher`, since guessing an owner would be worse than omitting one.
fn render_publisher(c: &Crate) -> String {
    let name = match c.publisher.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => p,
        _ => return String::new(),
    };
    let inner = match c.publisher_url.as_deref().map(str::trim) {
        Some(url) if !url.is_empty() => format!(
            r#"<a href="{url}" target="_blank" rel="noopener">{name}</a>"#,
            url = html_escape(url),
            name = html_escape(name),
        ),
        _ => html_escape(name),
    };
    format!("<span>published by {inner}</span>")
}

/// `no_std: yes | optional | no` as the site's existing support badge, so a
/// crate's `no_std` story reads the same as a language feature's embedded
/// support. Any other value (or none) renders nothing.
fn render_no_std(c: &Crate) -> String {
    let (class, label) = match c.no_std.as_deref().map(str::trim) {
        Some("yes") => ("level-full", "no_std: yes"),
        Some("optional") => ("level-partial", "no_std: optional"),
        Some("no") => ("level-none", "no_std: no"),
        _ => return String::new(),
    };
    format!(
        r#"<span class="support-badge {class}">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
          {label}
        </span>"#
    )
}

fn render_categories(c: &Crate) -> String {
    if c.categories.is_empty() {
        return String::new();
    }
    let tags: String = c
        .categories
        .iter()
        .map(|t| format!("<span class=\"article-tag\">{}</span>", html_escape(t)))
        .collect::<Vec<_>>()
        .join("");
    format!("<div class=\"article-tags\">{tags}</div>")
}

/// The crates.io / docs.rs / repository row shown under the byline.
fn render_links(c: &Crate) -> String {
    let mut links = vec![
        format!(
            r#"<a class="crate-link" href="{url}" target="_blank" rel="noopener">crates.io</a>"#,
            url = html_escape(&c.crates_io_url()),
        ),
        format!(
            r#"<a class="crate-link" href="{url}" target="_blank" rel="noopener">docs.rs</a>"#,
            url = html_escape(&c.docs),
        ),
    ];
    if let Some(repo) = c.repository.as_deref().filter(|r| !r.trim().is_empty()) {
        links.push(format!(
            r#"<a class="crate-link" href="{url}" target="_blank" rel="noopener">Repository</a>"#,
            url = html_escape(repo),
        ));
    }
    format!("<div class=\"crate-links\">{}</div>", links.join(""))
}

fn render_use_cases(use_cases: &[UseCase]) -> String {
    use_cases
        .iter()
        .map(|u| {
            let fit = u
                .fit_html
                .as_ref()
                .map(|f| format!("<div class=\"rationale\">{f}</div>"))
                .unwrap_or_default();
            format!(
                r#"<div class="card crate-use-case">
            <div class="scen-tag">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
              Use case
            </div>
            <h3 class="scenario-title">{title}</h3>
            {body}
            {fit}
          </div>"#,
                title = html_escape(&u.title),
                body = u.body_html,
            )
        })
        .collect::<Vec<_>>()
        .join("\n        ")
}

fn render_api_groups(groups: &[ApiGroup]) -> String {
    groups
        .iter()
        .map(|g| {
            let entries: String = g
                .entries
                .iter()
                .map(|e| {
                    let when = e
                        .when_html
                        .as_ref()
                        .map(|w| format!("<div class=\"rationale\">{w}</div>"))
                        .unwrap_or_default();
                    format!(
                        r#"<div class="card api-entry">
            <h4 class="api-sig">{sig}</h4>
            {body}
            {when}
          </div>"#,
                        sig = render_inline(&e.signature),
                        body = e.body_html,
                    )
                })
                .collect::<Vec<_>>()
                .join("\n        ");
            format!(
                r#"<div class="api-group">
        <h3 class="api-group-title">{title}<span class="api-group-count">{count}</span></h3>
        {intro}
        <div class="scenarios crate-scenarios">
        {entries}
        </div>
      </div>"#,
                title = html_escape(&g.title),
                count = g.entries.len(),
                intro = g.intro_html,
            )
        })
        .collect::<Vec<_>>()
        .join("\n      ")
}

fn render_crate(c: &Crate, pages: &[Page]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::Crates);
    let home = href_from(DEPTH, "index.html");
    let index = "index.html";

    let breadcrumb = format!(
        r#"<nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <a href="{index}">Crates</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">{title}</span>
      </nav>"#,
        title = html_escape(&c.title),
    );

    let page_head = format!(
        r#"<div class="page-head">
        <div class="title-block">
          <h1 class="page-title">{title}<span class="kind">Crate</span></h1>
        </div>
        <a class="convo-cta" href="{url}" target="_blank" rel="noopener">Open on crates.io &rarr;</a>
      </div>"#,
        title = html_escape(&c.title),
        url = html_escape(&c.crates_io_url()),
    );

    // Two separate facts, deliberately not mixed: who publishes the *crate*
    // (upstream, from crates.io) and who wrote this *page* (a contributor here).
    // The crates.io name only earns a line of its own when it isn't already the
    // page title (e.g. a page titled "Serde JSON" documenting `serde_json`).
    let crate_name = if c.crate_name == c.title {
        String::new()
    } else {
        format!(
            "<span class=\"crate-name\">{}</span>",
            html_escape(&c.crate_name)
        )
    };
    let crate_meta = format!(
        r#"<div class="crate-meta">
        {crate_name}
        {version}
        {publisher}
        {no_std}
      </div>"#,
        version = render_version(c),
        publisher = render_publisher(c),
        no_std = render_no_std(c),
    );

    let byline = format!(
        r#"<div class="crate-byline" data-vote-key="{vote_key}">
        <span>page written by <a class="article-author" href="{gh}" target="_blank" rel="noopener">{author}</a> &middot; {date}</span>
        <a class="article-like" hidden target="_blank" rel="noopener">&#128077; <span class="like-n"></span> &mdash; like this page on GitHub</a>
      </div>"#,
        vote_key = html_escape(&c.vote_key()),
        gh = html_escape(&c.github_url()),
        author = html_escape(&c.author),
        date = fmt_date(&c.date),
    );

    let summary = format!("<p class=\"lead\">{}</p>", render_inline(&c.summary));

    let categories = render_categories(c);

    let tabs = r#"<nav class="section-tabs" id="section-tabs">
        <button class="tab on" data-target="overview">Overview</button>
        <button class="tab" data-target="use-cases">When to use it</button>
        <button class="tab" data-target="api">API map</button>
      </nav>"#;

    let main = format!(
        r#"      {breadcrumb}

      {page_head}

      {crate_meta}

      {byline}

      {links}

      {summary}

      {categories}

      <hr class="divider">

      {tabs}

      <section class="doc" data-tab="overview">
        <h2 class="section-title">Overview</h2>
        {overview}
      </section>

      <section class="doc" data-tab="use-cases">
        <h2 class="section-title">When to use it</h2>
        {use_cases_intro}
        <div class="scenarios crate-scenarios">
        {use_cases}
        </div>
      </section>

      <section class="doc" data-tab="api">
        <h2 class="section-title">API map</h2>
        {api_intro}
        {api_groups}
      </section>

      <div class="convo-actions convo-actions-foot">
        <a class="convo-open" href="{index}">&#8592; Back to Crates</a>
      </div>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>{version_note}</span>
      </div>
"#,
        links = render_links(c),
        overview = c.overview_html,
        use_cases_intro = c.use_cases_intro_html,
        use_cases = render_use_cases(&c.use_cases),
        api_intro = c.api_intro_html,
        api_groups = render_api_groups(&c.api_groups),
        version_note = match &c.version {
            Some(v) if !v.trim().is_empty() => format!(
                "Written against {} {}",
                html_escape(&c.crate_name),
                html_escape(v)
            ),
            _ => format!("Written against {}", html_escape(&c.crate_name)),
        },
    );

    let head = Head {
        title: format!("Rust - {} crate - Rusty Yellow Pages", c.title),
        description: c.summary.replace('`', ""),
        canonical: abs_url(&c.href),
        og_type: "article",
        image: None,
    };
    shell(&head, DEPTH, &sidebar, &main)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Crate {
        Crate {
            title: "anyhow".into(),
            crate_name: "anyhow".into(),
            version: Some("1.0.104".into()),
            publisher: Some("David Tolnay (dtolnay)".into()),
            publisher_url: Some("https://crates.io/users/dtolnay".into()),
            no_std: Some("optional".into()),
            author: "A".into(),
            github: "handle".into(),
            date: "2026-07-29".into(),
            summary: "Flexible errors.".into(),
            categories: vec!["error-handling".into()],
            repository: None,
            docs: "https://docs.rs/anyhow".into(),
            slug: "anyhow".into(),
            href: "crates/anyhow.html".into(),
            overview_html: String::new(),
            use_cases_intro_html: String::new(),
            use_cases: vec![],
            api_intro_html: String::new(),
            api_groups: vec![],
        }
    }

    #[test]
    fn card_links_to_a_sibling_page_not_the_site_root_href() {
        let html = render_card(&sample(), 0);
        assert!(html.contains(r#"href="anyhow.html""#));
        assert!(!html.contains(r#"href="crates/anyhow.html""#));
    }

    #[test]
    fn no_std_maps_to_the_support_badge_levels() {
        let mut c = sample();
        assert!(render_no_std(&c).contains("level-partial"));
        c.no_std = Some("yes".into());
        assert!(render_no_std(&c).contains("level-full"));
        c.no_std = None;
        assert!(render_no_std(&c).is_empty());
    }

    #[test]
    fn repository_link_is_omitted_when_absent() {
        let mut c = sample();
        assert!(!render_links(&c).contains("Repository"));
        c.repository = Some("https://github.com/dtolnay/anyhow".into());
        assert!(render_links(&c).contains("Repository"));
    }
}
