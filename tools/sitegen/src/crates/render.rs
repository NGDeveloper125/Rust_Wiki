//! Render the crates index (a searchable, sortable card grid) and one page per
//! crate, reusing the site's shell/sidebar/card styling.

use std::io;
use std::path::Path;

use super::domain::{self, Domain};
use super::{ApiGroup, Crate, UseCase, DEPTH};
use crate::model::Page;
use crate::nav::{render_sidebar, TopNav};
use crate::render::{abs_url, href_from, shell, shell_with_page_class, Head};
use crate::util::{fmt_date, html_escape, render_inline};

/// The repo's contributor guide, linked from the "document a crate" CTA.
const CONTRIBUTE_URL: &str =
    "https://github.com/NGDeveloper125/Rust_Wiki/blob/main/CONTRIBUTING.md#crates";

pub fn write_pages(
    docs_root: &Path,
    crates: &[Crate],
    intro_html: &str,
    pages: &[Page],
) -> io::Result<()> {
    let dir = docs_root.join("crates");
    std::fs::create_dir_all(&dir)?;

    let domains = domain::occupied(crates);
    std::fs::write(
        dir.join("index.html"),
        render_index(crates, intro_html, pages, &domains),
    )?;
    for c in crates {
        std::fs::write(
            dir.join(format!("{}.html", c.slug)),
            render_crate(c, pages, &domains),
        )?;
    }
    Ok(())
}

fn render_index(
    crates: &[Crate],
    intro_html: &str,
    pages: &[Page],
    domains: &[&'static Domain],
) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::Crates(None), domains);
    let home = href_from(DEPTH, "");

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

    let lead = r#"<p class="lead">A directory of the crates people actually reach for, filed by what they are for. Every page follows the same three sections &mdash; what the crate is, the situations it fits, and a map of its API with a small call example for each item &mdash; so you can look up an unfamiliar crate the same way every time. Crate pages are contributed as markdown pull requests.</p>"#;

    // The intro is prose about the ecosystem; the directory below it is the
    // reason most people are here. The skip link comes first so anyone who
    // already knows what a crate is sees the way past before the prose.
    let intro = if intro_html.trim().is_empty() {
        String::new()
    } else {
        format!(
            r##"<div class="crate-intro">
        <p class="crate-intro-skip"><a href="#directory">Skip to the directory &darr;</a></p>
        {intro_html}
      </div>"##
        )
    };

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
              <option value="section" selected>By type</option>
              <option value="name">A &ndash; Z</option>
              <option value="date">Newest</option>
              <option value="rating">Top rated</option>
            </select>
          </label>
        </div>"#;

        format!(
            "{toolbar}\n        <div class=\"crate-grid\" id=\"crate-grid\">\n        {cards}\n        </div>\n        <p class=\"article-nomatch\" id=\"crate-nomatch\" hidden>No crates match your filter.</p>",
            cards = render_sections(crates, domains),
        )
    };

    let main = format!(
        r#"      {breadcrumb}

      {page_head}

      {lead}

      <hr class="divider">

      {intro}

      <div id="directory"></div>

      {body}

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>Crate pages are contributed as markdown pull requests</span>
      </div>
"#
    );

    let head = Head {
        // Not "Rust - Crates - ...": that is the title of the Crates *concept*
        // page, and two pages sharing one title are indistinguishable in search
        // results, browser tabs and history.
        title: "Rust - Crate Directory - Rusty Yellow Pages".to_string(),
        description: "A directory of Rust crates — what each one is, when it's a good fit, and a map of its API with a call example for every item.".to_string(),
        canonical: abs_url("crates/"),
        og_type: "website",
        image: None,
    };
    // Wider than the site's reading measure: this page is a card grid.
    shell_with_page_class(&head, DEPTH, &sidebar, &main, "page-crate-index")
}

/// The grid's contents: every section that has a page, each one a heading
/// followed by its crates A-Z, then anything the taxonomy didn't catch.
///
/// `crates` arrives sorted A-Z, so filtering it per section preserves that
/// order inside each one for free. The running index handed to `render_card`
/// is the position in *this* sequence, which is what site.js restores when
/// the reader sorts one way and then back.
fn render_sections(crates: &[Crate], domains: &[&'static Domain]) -> String {
    let mut items: Vec<String> = Vec::with_capacity(crates.len() + domains.len() + 2);
    let mut i = 0;

    for d in domains {
        items.push(render_section_head(d));
        for c in crates.iter().filter(|c| c.domain == Some(*d)) {
            items.push(render_card(c, i, d.slug));
            i += 1;
        }
    }

    // A page whose `domain:` was missing or unrecognised is still a page, and
    // hiding it here would make it reachable only through search. The build
    // already warned about each one by name; this is the visible half of that
    // warning.
    let unfiled: Vec<&Crate> = crates.iter().filter(|c| c.domain.is_none()).collect();
    if !unfiled.is_empty() {
        items.push(render_unfiled_head());
        for c in unfiled {
            items.push(render_card(c, i, UNFILED_SLUG));
            i += 1;
        }
    }

    items.join("\n        ")
}

/// The section a page lands in when its `domain:` was missing or unrecognised.
const UNFILED_SLUG: &str = "unfiled";

/// A section heading spanning the full width of the grid. site.js hides these
/// when the reader sorts by anything other than type, or filters every crate
/// out of a section.
fn render_section_head(d: &Domain) -> String {
    format!(
        r#"<div class="crate-section" data-section="{slug}" id="{slug}">
          <h2 class="crate-section-title">{label}</h2>
          <p class="crate-section-blurb">{blurb}</p>
        </div>"#,
        slug = d.slug,
        label = html_escape(d.label),
        blurb = html_escape(d.blurb),
    )
}

fn render_unfiled_head() -> String {
    format!(
        r#"<div class="crate-section" data-section="{UNFILED_SLUG}" id="{UNFILED_SLUG}">
          <h2 class="crate-section-title">Everything else</h2>
          <p class="crate-section-blurb">Pages that haven&rsquo;t been filed under a type yet.</p>
        </div>"#
    )
}

/// One card in the index grid. `i` is the authored position, used by site.js
/// to restore the by-type order after a different sort; `section` is the slug
/// of the heading it sits under, so that heading can be hidden when the
/// filter empties it.
fn render_card(c: &Crate, i: usize, section: &str) -> String {
    let api_count = c.api_count();
    let apis = match api_count {
        0 => String::new(),
        1 => " &middot; 1 API entry".to_string(),
        n => format!(" &middot; {n} API entries"),
    };

    format!(
        r#"<article class="crate-card" data-i="{i}" data-section="{section}" data-name="{name}" data-date="{iso}" data-search="{search}" data-vote-key="{vote_key}">
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
            {links}
            <a class="article-like" hidden target="_blank" rel="noopener">&#128077; <span class="like-n"></span></a>
          </div>
        </article>"#,
        section = html_escape(section),
        name = html_escape(&c.title.to_lowercase()),
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
        links = render_card_links(c),
    )
}

/// Outbound links on an index card: crates.io always, the source repository
/// when the page records one. They sit in the card foot, outside the anchor
/// wrapping the card body, so they aren't nested inside another link.
fn render_card_links(c: &Crate) -> String {
    let mut links = vec![format!(
        r#"<a href="{url}" target="_blank" rel="noopener">crates.io</a>"#,
        url = html_escape(&c.crates_io_url()),
    )];
    if let Some(repo) = c.repository.as_deref().filter(|r| !r.trim().is_empty()) {
        links.push(format!(
            r#"<a href="{url}" target="_blank" rel="noopener">repo</a>"#,
            url = html_escape(repo),
        ));
    }
    format!("<div class=\"crate-card-links\">{}</div>", links.join(""))
}

/// The lowercase haystack the index's filter box matches against, baked into
/// the card so the client never has to scrape it back out of the DOM.
fn search_text(c: &Crate) -> String {
    format!(
        "{} {} {} {} {} {}",
        c.title,
        c.crate_name,
        c.summary,
        c.categories.join(" "),
        // Typing a section's name should find its crates, the same way
        // clicking it in the sidebar does.
        c.domain.map(|d| d.label).unwrap_or_default(),
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
          {icon}
          {label}
        </span>"#,
        icon = crate::render::support_badge_icon(class),
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

fn render_crate(c: &Crate, pages: &[Page], domains: &[&'static Domain]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::Crates(c.domain), domains);
    let home = href_from(DEPTH, "");
    let index = "./";

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
            domain: domain::lookup("Error handling"),
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
        let html = render_card(&sample(), 0, "error-handling");
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

    /// The check mark is an assertion that the crate supports `no_std`, so
    /// `no_std: no` must not carry one.
    #[test]
    fn only_supported_levels_get_a_check_mark() {
        const CHECK: &str = "M20 6 9 17l-5-5";
        let mut c = sample();

        c.no_std = Some("no".into());
        let none = render_no_std(&c);
        assert!(!none.contains(CHECK), "no_std: no rendered a check mark: {none}");
        assert!(none.contains("M18 6 6 18"), "no_std: no should render a cross");

        c.no_std = Some("yes".into());
        assert!(render_no_std(&c).contains(CHECK));
        c.no_std = Some("optional".into());
        assert!(render_no_std(&c).contains(CHECK));
    }

    /// site.js pairs a card with its heading by this attribute, so a card
    /// without one would go missing from its section on every re-sort.
    #[test]
    fn a_card_carries_the_slug_of_its_section() {
        let html = render_card(&sample(), 0, "error-handling");
        assert!(html.contains(r#"data-section="error-handling""#));
    }

    /// The A-Z sort reorders cards in the browser, so the key it sorts on has
    /// to be on the card and has to be case-folded — otherwise `Serde JSON`
    /// sorts before `anyhow`.
    #[test]
    fn a_card_carries_a_lowercased_sort_name() {
        let mut c = sample();
        c.title = "Serde JSON".into();
        assert!(render_card(&c, 0, "serialization-data-formats")
            .contains(r#"data-name="serde json""#));
    }

    /// Every page reaches the grid: one that named no section, or named one
    /// the taxonomy doesn't have, still gets a card under "Everything else".
    #[test]
    fn an_unfiled_page_still_gets_a_card() {
        let mut c = sample();
        c.domain = None;
        let html = render_sections(std::slice::from_ref(&c), &[]);
        assert!(html.contains(r#"data-section="unfiled""#), "{html}");
        assert!(html.contains("Everything else"));
    }

    /// A section the reader can reach from the sidebar has to exist as an
    /// anchor on the page, or the link lands at the top of the index.
    #[test]
    fn a_section_heading_is_an_anchor_target() {
        let c = sample();
        let d = domain::lookup("Error handling").unwrap();
        let html = render_sections(std::slice::from_ref(&c), &[d]);
        assert!(html.contains(r#"id="error-handling""#), "{html}");
        assert!(html.contains("Error handling"));
    }

    #[test]
    fn repository_link_is_omitted_when_absent() {
        let mut c = sample();
        assert!(!render_links(&c).contains("Repository"));
        c.repository = Some("https://github.com/dtolnay/anyhow".into());
        assert!(render_links(&c).contains("Repository"));
    }
}
