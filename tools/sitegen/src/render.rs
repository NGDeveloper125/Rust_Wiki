use crate::links::{render_chip_row, LinkIndex};
use crate::model::{group_label, Page, Section};
use crate::nav::{render_sidebar, TopNav, CHEVRON_SVG};
use crate::util::html_escape;

/// Public base URL of the deployed site, used for absolute `<link rel="canonical">`
/// URLs, Open Graph URLs, the sitemap, and robots.txt. The site is served from
/// the `rustyyellowpages.dev` custom domain (see `docs/CNAME`), where the repo
/// lives at the domain root, so paths carry no `/Rust_Wiki/` prefix.
pub const SITE_BASE: &str = "https://rustyyellowpages.dev/";

/// The project's GitHub repository, linked from the landing page's call for
/// contributions.
pub const REPO_URL: &str = "https://github.com/NGDeveloper125/Rust_Wiki";

/// Site-root-relative path of the share card used for any page without a lead
/// image of its own, so links unfurl as a card rather than a bare text summary
/// wherever they are posted. Rendered from
/// `tools/sitegen/templates/og-card.html`; it is a committed asset rather than
/// a generated one, so the dimensions below must be kept in step with the file.
const DEFAULT_SHARE_IMAGE: &str = "assets/og-card.png";
const DEFAULT_SHARE_IMAGE_W: u32 = 1200;
const DEFAULT_SHARE_IMAGE_H: u32 = 630;
const DEFAULT_SHARE_IMAGE_ALT: &str =
    "Rusty Yellow Pages \u{2014} a free, open-source Rust reference";

/// The `<head>` metadata for a page. All string fields are raw (unescaped);
/// [`shell`] escapes them at render time.
pub struct Head {
    /// Full `<title>` text.
    pub title: String,
    /// `<meta name="description">` text.
    pub description: String,
    /// Absolute `<link rel="canonical">` URL.
    pub canonical: String,
    /// Open Graph `og:type` — `"website"` for most pages, `"article"` for articles.
    pub og_type: &'static str,
    /// Absolute URL of a share image, if the page has one (article lead images).
    pub image: Option<String>,
}

impl Head {
    /// The Open Graph / Twitter Card tag block for this page.
    fn social_meta(&self) -> String {
        // Every page carries an image: its own if it has one, otherwise the
        // site-wide card. Dimensions and alt text are only emitted for the
        // latter, since a page's own image has neither known to us here.
        let image = match &self.image {
            Some(own) => own.clone(),
            None => abs_url(DEFAULT_SHARE_IMAGE),
        };
        let mut s = format!(
            r#"<meta property="og:type" content="{og_type}">
<meta property="og:site_name" content="Rusty Yellow Pages">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:url" content="{canonical}">
<meta property="og:image" content="{image}">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="{title}">
<meta name="twitter:description" content="{description}">
<meta name="twitter:image" content="{image}">"#,
            og_type = self.og_type,
            title = html_escape(&self.title),
            description = html_escape(&self.description),
            canonical = html_escape(&self.canonical),
            image = html_escape(&image),
        );
        if self.image.is_none() {
            s.push_str(&format!(
                "\n<meta property=\"og:image:width\" content=\"{DEFAULT_SHARE_IMAGE_W}\">\
                 \n<meta property=\"og:image:height\" content=\"{DEFAULT_SHARE_IMAGE_H}\">\
                 \n<meta property=\"og:image:alt\" content=\"{alt}\">",
                alt = html_escape(DEFAULT_SHARE_IMAGE_ALT),
            ));
        }
        s
    }
}

/// Build an absolute site URL from a site-root-relative path
/// (e.g. `"syntax/operators/ampersand.html"`). An empty path yields the base.
pub fn abs_url(site_relative_path: &str) -> String {
    format!("{SITE_BASE}{site_relative_path}")
}

/// Depth marker for a page that can be served from any URL: the 404 page,
/// which GitHub Pages returns for every missing path while the browser keeps
/// the address that was asked for. A relative href would resolve against that
/// address rather than against `/404.html`, so such a page links root-absolute
/// instead. Passing this to [`href_from`] switches it into that mode.
pub const ANY_URL: usize = usize::MAX;

/// Build a relative href from a page `depth` directories below the site root
/// to a site-root-relative `target`.
///
/// Index pages are addressed as directories (`""` for the site root,
/// `"articles/"`) rather than by filename, so that the URL a visitor lands on
/// is the one `<link rel="canonical">` and the sitemap advertise. The empty
/// target needs the explicit `./` at depth 0: an empty href would resolve to
/// the current page rather than to the site root.
pub fn href_from(depth: usize, target: &str) -> String {
    if depth == ANY_URL {
        format!("/{target}")
    } else if depth == 0 {
        if target.is_empty() {
            "./".to_string()
        } else {
            target.to_string()
        }
    } else {
        "../".repeat(depth) + target
    }
}

fn topbar(depth: usize) -> String {
    let home = href_from(depth, "");
    format!(
        r##"<header class="topbar">
  <button class="hamburger" id="hamburger" aria-label="Toggle navigation">
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M4 6h16M4 12h16M4 18h16"/></svg>
  </button>

  <a class="wordmark" href="{home}">
    <div class="mark">R</div>
    <div class="name">RUSTY <span class="lo">YELLOW PAGES</span></div>
  </a>

  <div class="search" id="search">
    <div class="search-field">
      <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="7"/><path d="m21 21-3.6-3.6"/></svg>
      <input id="search-input" type="text" placeholder="Search tokens &amp; concepts&hellip;  &nbsp;try &quot;borrow&quot;, &quot;&amp;&quot;, &quot;lifetime&quot;" autocomplete="off" spellcheck="false">
      <span class="kbd">/</span>
    </div>
    <div class="search-dropdown" id="search-dropdown"></div>
  </div>

  <button class="theme-toggle" id="theme-toggle" aria-label="Toggle light and dark mode">
    <span class="ic-moon" style="display:inline-flex"><svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"/></svg></span>
    <span class="ic-sun"><svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg></span>
    <span class="lbl" id="theme-label">Dark</span>
  </button>
</header>

<div class="backdrop" id="backdrop"></div>"##
    )
}

/// Wrap `sidebar_html` + `main_html` in the full document shell.
pub fn shell(head: &Head, depth: usize, sidebar_html: &str, main_html: &str) -> String {
    shell_with_page_class(head, depth, sidebar_html, main_html, "")
}

/// [`shell`], plus an extra class on the `<article class="page">` wrapper.
/// Used by the landing page (`page-landing`), which drops the prose measure
/// so its hero band can run the full width of the content column.
pub fn shell_with_page_class(
    head: &Head,
    depth: usize,
    sidebar_html: &str,
    main_html: &str,
    extra_page_class: &str,
) -> String {
    let page_class = if extra_page_class.is_empty() {
        String::new()
    } else {
        format!(" {extra_page_class}")
    };
    let css = href_from(depth, "assets/site.css");
    let favicon = href_from(depth, "favicon.svg");
    let search_index_js = href_from(depth, "assets/search-index.js");
    let site_js = href_from(depth, "assets/site.js");
    format!(
        r#"<!doctype html>
<html lang="en" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<meta name="description" content="{description}">
<link rel="canonical" href="{canonical}">
{social_meta}
<link rel="icon" href="{favicon}" type="image/svg+xml">
<link rel="stylesheet" href="{css}">
</head>
<body>

{topbar}

<div class="shell">
  <aside class="sidebar" id="sidebar">{sidebar_html}
  </aside>

  <main class="content">
    <article class="page{page_class}">
{main_html}
    </article>
  </main>
</div>

<script>window.SITE_ROOT = "{site_root}";</script>
<script src="{search_index_js}"></script>
<script src="{site_js}"></script>
</body>
</html>
"#,
        title = html_escape(&head.title),
        description = html_escape(&head.description),
        canonical = html_escape(&head.canonical),
        social_meta = head.social_meta(),
        topbar = topbar(depth),
        // Prefix the search's result links, so it has to follow the same
        // root-absolute rule as every other link on an ANY_URL page.
        site_root = if depth == ANY_URL {
            "/".to_string()
        } else {
            "../".repeat(depth)
        },
    )
}

fn embedded_badge(support: &str) -> &'static str {
    match support {
        "full" => "Full",
        "partial" => "Partial",
        _ => "None",
    }
}

fn render_examples(examples: &[crate::model::Example]) -> String {
    examples
        .iter()
        .map(|ex| {
            format!(
                r#"<div class="card">
            <h3 class="scenario-title">{title}</h3>
            {body}
          </div>"#,
                title = html_escape(&ex.title),
                body = ex.body_html,
            )
        })
        .collect::<Vec<_>>()
        .join("\n        ")
}

fn render_scenarios(scenarios: &[crate::model::Scenario], vote_prefix: &str) -> String {
    scenarios
        .iter()
        .map(|s| {
            if s.approaches.is_empty() {
                render_single_scenario(s)
            } else {
                render_multi_scenario(s, vote_prefix)
            }
        })
        .collect::<Vec<_>>()
        .join("\n        ")
}

/// Render a scenario with only the Classic content — the exact markup the
/// site produced before approaches existed, so approach-less pages stay
/// byte-identical.
fn render_single_scenario(s: &crate::model::Scenario) -> String {
    let rationale = s
        .rationale_html
        .as_ref()
        .map(|r| format!("<div class=\"rationale\">{r}</div>"))
        .unwrap_or_default();
    format!(
        r#"<div class="card">
            <div class="scen-tag">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="9" cy="7" r="3"/><path d="M2 21v-1a5 5 0 0 1 5-5h4a5 5 0 0 1 5 5v1M16 3.1a3 3 0 0 1 0 5.8M22 21v-1a5 5 0 0 0-3-4.6"/></svg>
              Scenario
            </div>
            <h3 class="scenario-title">{title}</h3>
            {body}
            {rationale}
          </div>"#,
        title = html_escape(&s.title),
        body = s.body_html,
    )
}

/// Render a scenario that has community approaches: same card chrome plus an
/// "Approach:" picker row (a native dropdown, "Classic" first and always the
/// default) and one panel per approach, switched client-side by site.js.
/// Options and panels pair by explicit value/data-idx (not DOM position),
/// so site.js can reorder options by like-count without breaking pairing.
/// Each approach option carries a `data-vote-key` matching the title of its
/// GitHub vote issue (`<page-path>::<scenario>::<approach>`); site.js uses
/// it to show 👍 reaction counts fetched from the GitHub API.
fn render_multi_scenario(s: &crate::model::Scenario, vote_prefix: &str) -> String {
    let mut options = String::from(r#"<option value="0" selected>Classic</option>"#);
    for (i, a) in s.approaches.iter().enumerate() {
        let vote_key = format!("{vote_prefix}::{}::{}", s.title, a.title);
        options.push_str(&format!(
            "\n                <option value=\"{idx}\" data-vote-key=\"{key}\">{title}</option>",
            idx = i + 1,
            key = html_escape(&vote_key),
            title = html_escape(&a.title)
        ));
    }

    let classic_rationale = s
        .rationale_html
        .as_ref()
        .map(|r| format!("<div class=\"rationale\">{r}</div>"))
        .unwrap_or_default();
    let mut panels = format!(
        r#"<div class="approach-panel on" data-idx="0">
            {body}
            {rationale}
            </div>"#,
        body = s.body_html,
        rationale = classic_rationale,
    );
    for (i, a) in s.approaches.iter().enumerate() {
        let like_chip = concat!(
            r#"<a class="approach-like" hidden target="_blank" rel="noopener">"#,
            r#"&#128077; <span class="like-n"></span> &mdash; like this approach on GitHub</a>"#
        );
        let byline = if a.attribution_html.is_empty() {
            format!("<div class=\"approach-byline\">{like_chip}</div>")
        } else {
            format!(
                "<div class=\"approach-byline\">{}{like_chip}</div>",
                a.attribution_html
            )
        };
        let rationale = a
            .rationale_html
            .as_ref()
            .map(|r| format!("<div class=\"rationale\">{r}</div>"))
            .unwrap_or_default();
        panels.push_str(&format!(
            r#"
            <div class="approach-panel" data-idx="{idx}">
            {byline}
            {body}
            {rationale}
            </div>"#,
            idx = i + 1,
            byline = byline,
            body = a.body_html,
            rationale = rationale,
        ));
    }

    format!(
        r#"<div class="card">
            <div class="scen-tag">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="9" cy="7" r="3"/><path d="M2 21v-1a5 5 0 0 1 5-5h4a5 5 0 0 1 5 5v1M16 3.1a3 3 0 0 1 0 5.8M22 21v-1a5 5 0 0 0-3-4.6"/></svg>
              Scenario
            </div>
            <h3 class="scenario-title">{title}</h3>
            <div class="approach-picker">
              <span class="approach-label">Approach:</span>
              <select class="approach-select" aria-label="Approach for this scenario">
                {options}
              </select>
            </div>
            {panels}
          </div>"#,
        title = html_escape(&s.title),
        options = options,
        panels = panels,
    )
}

pub fn render_content_page(page: &Page, pages: &[Page], index: &LinkIndex) -> String {
    let depth = page.href.matches('/').count();
    let home = href_from(depth, "");
    let group_lbl = group_label(page.section, &page.subgroup);
    let title_html = if page.section == Section::Syntax {
        format!("<span class=\"tok\">{}</span>", html_escape(&page.front.title))
    } else {
        html_escape(&page.front.title)
    };

    let breadcrumb = format!(
        r#"<nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">{section}</a><span class="sep">&rsaquo;</span>
        <span>{group}</span><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">{title}</span>
      </nav>"#,
        section = page.section.label(),
        group = html_escape(&group_lbl),
        title = title_html,
    );

    let support = page.embedded_support();

    let page_head = format!(
        r#"<div class="page-head">
        <div class="title-block">
          <h1 class="page-title">{title}<span class="kind">{kind}</span></h1>
        </div>
        <div class="segmented" role="tablist" aria-label="Rust flavor">
          <button id="seg-classic" class="on" role="tab" aria-selected="true">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 18l6-6-6-6M8 6l-6 6 6 6"/></svg>
            Classic Rust
          </button>
          <button id="seg-embedded" role="tab" aria-selected="false">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><path d="M9 2v2M15 2v2M9 20v2M15 20v2M2 9h2M2 15h2M20 9h2M20 15h2"/></svg>
            Embedded Rust
          </button>
        </div>
      </div>"#,
        title = title_html,
        kind = html_escape(&page.kind_badge()),
    );

    let concepts_row = render_chip_row(
        index,
        pages,
        depth,
        "Related concepts",
        &page.front.related_concepts,
        false,
    );
    let syntax_row = render_chip_row(
        index,
        pages,
        depth,
        "Related syntax",
        &page.front.related_syntax,
        true,
    );
    let related = if concepts_row.is_empty() && syntax_row.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"related\">\n        {concepts_row}\n        {syntax_row}\n      </div>"
        )
    };

    let badge_class = match support {
        "none" => "level-none",
        "partial" => "level-partial",
        _ => "level-full",
    };
    let support_badge_html = format!(
        r#"<span class="support-badge {badge_class}">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
          Embedded support: {support_label}
        </span>"#,
        support_label = embedded_badge(support),
    );

    let tabs_class = if support == "none" { " no-embedded-tabs" } else { "" };

    let (tabs_html, classic_sections_html, embedded_sections_html) = if page.section == Section::Syntax
    {
        let tabs = format!(
            r#"<nav class="section-tabs{tabs_class}" id="section-tabs">
        <button class="tab on" data-target="explanation">Explanation</button>
        <button class="tab" data-target="examples">Usage examples</button>
      </nav>"#
        );

        let classic = format!(
            r#"<section class="doc" data-tab="explanation">
        <h2 class="section-title">Explanation</h2>
        {explanation}
      </section>

      <section class="doc" data-tab="examples">
        <h2 class="section-title">Usage examples</h2>
        <div class="scenarios">
        {examples}
        </div>
      </section>"#,
            explanation = page.explanation_html,
            examples = render_examples(&page.usage_examples),
        );

        let embedded = if support == "none" {
            format!(
                r#"<div class="unsupported-note">
        {support_badge_html}
        {notes}
      </div>"#,
                notes = page.embedded_notes_html,
            )
        } else {
            format!(
                r#"<section class="doc" data-tab="explanation">
        <h2 class="section-title">Explanation</h2>
        {support_badge_html}
        {explanation}
      </section>

      <section class="doc" data-tab="examples">
        <h2 class="section-title">Usage examples</h2>
        <div class="scenarios">
        {examples}
        </div>
      </section>"#,
                explanation = page.embedded_explanation_html,
                examples = render_examples(&page.embedded_usage_examples),
            )
        };

        (tabs, classic, embedded)
    } else {
        let tabs = format!(
            r#"<nav class="section-tabs{tabs_class}" id="section-tabs">
        <button class="tab on" data-target="explanation">Explanation</button>
        <button class="tab" data-target="basic">Basic usage example</button>
        <button class="tab" data-target="best">Best practices &amp; deeper information</button>
      </nav>"#
        );

        let classic = format!(
            r#"<section class="doc" data-tab="explanation">
        <h2 class="section-title">Explanation</h2>
        {explanation}
      </section>

      <section class="doc" data-tab="basic">
        <h2 class="section-title">Basic usage example</h2>
        {basic_usage}
      </section>

      <section class="doc" data-tab="best">
        <h2 class="section-title">Best practices &amp; deeper information</h2>
        {intro}
        <div class="scenarios">
        {scenarios}
        </div>
      </section>"#,
            explanation = page.explanation_html,
            basic_usage = page.basic_usage_html,
            intro = page.best_practices_intro_html,
            scenarios = render_scenarios(&page.scenarios, page.href.trim_end_matches(".html")),
        );

        let embedded = if support == "none" {
            format!(
                r#"<div class="unsupported-note">
        {support_badge_html}
        {notes}
      </div>"#,
                notes = page.embedded_notes_html,
            )
        } else {
            format!(
                r#"<section class="doc" data-tab="explanation">
        <h2 class="section-title">Explanation</h2>
        {support_badge_html}
        {explanation}
      </section>

      <section class="doc" data-tab="basic">
        <h2 class="section-title">Basic usage example</h2>
        {basic_usage}
      </section>

      <section class="doc" data-tab="best">
        <h2 class="section-title">Best practices &amp; deeper information</h2>
        {intro}
        <div class="scenarios">
        {scenarios}
        </div>
      </section>"#,
                explanation = page.embedded_explanation_html,
                basic_usage = page.embedded_basic_usage_html,
                intro = page.embedded_best_practices_intro_html,
                scenarios =
                    render_scenarios(&page.embedded_scenarios, page.href.trim_end_matches(".html")),
            )
        };

        (tabs, classic, embedded)
    };

    format!(
        r#"      {breadcrumb}

      {page_head}

      {related}

      <hr class="divider">

      {tabs_html}

      <div class="flavor flavor-classic">
      {classic_sections_html}
      </div>

      <div class="flavor flavor-embedded">
      {embedded_sections_html}
      </div>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>Targets current stable Rust &middot; edition 2024</span>
      </div>
"#,
    )
}

/// One-line characterisation of a section, shown next to its page count in
/// the landing page's Browse header.
fn section_blurb(section: Section) -> &'static str {
    match section {
        Section::Syntax => "a dictionary, one page per token",
        Section::Concepts => "a wiki with scenarios and approaches",
    }
}

/// The landing page's Browse column for one section: a compact row per group
/// (name, then page count) over the hidden panel of page chips it expands.
///
/// The previous layout gave each of the 23 groups an equal-sized card and
/// wrapped them into a field that had to be read in both directions. A single
/// column of rows reads top to bottom, lets Syntax and Concepts sit side by
/// side, and takes roughly a third of the height. Returns the rendered column
/// plus the section's page and group counts.
fn render_browse_column(pages: &[Page], section: Section, depth: usize) -> (String, usize, usize) {
    let section_name = match section {
        Section::Syntax => "syntax",
        Section::Concepts => "concepts",
    };
    let mut rows = String::new();
    let mut page_count = 0usize;
    let mut group_count = 0usize;

    for (folder, label) in crate::model::group_order(section) {
        let mut group_pages: Vec<&Page> = pages
            .iter()
            .filter(|p| p.section == section && p.subgroup == *folder)
            .collect();
        if group_pages.is_empty() {
            continue;
        }
        group_pages.sort_by(|a, b| a.front.title.cmp(&b.front.title));
        page_count += group_pages.len();
        group_count += 1;

        let panel_id = format!("group-{section_name}-{folder}");
        let chips: String = group_pages
            .iter()
            .map(|p| {
                let href = href_from(depth, &p.href);
                let label_html = if p.section == Section::Syntax {
                    format!("<span class=\"tok\">{}</span>", html_escape(&p.front.title))
                } else {
                    html_escape(&p.front.title)
                };
                format!("<a class=\"chip\" href=\"{href}\">{label_html}</a>")
            })
            .collect::<Vec<_>>()
            .join("\n              ");

        rows.push_str(&format!(
            r#"
            <button type="button" class="lp-grow" aria-expanded="false" aria-controls="{panel_id}">
              {CHEVRON_SVG}<span class="lp-grow-name">{label}</span><span class="lp-grow-n">{count}</span>
            </button>
            <div class="lp-gpanel" id="{panel_id}" hidden>
              {chips}
            </div>
"#,
            count = group_pages.len(),
        ));
    }

    let html = format!(
        r#"
          <div class="lp-browse-col is-{section_name}">
            <div class="lp-col-head">
              <h3 class="lp-col-title">{label}</h3>
              <span class="lp-col-note">{page_count} pages &middot; {blurb}</span>
            </div>
            <div class="lp-glist">{rows}
            </div>
          </div>
"#,
        label = section.label(),
        blurb = section_blurb(section),
    );
    (html, page_count, group_count)
}

/// The scenario icon, shared by the landing page's inline specimen. The two
/// scenario renderers above still carry their own copy; they are left alone so
/// this change cannot alter a single concept page's output.
const SCENARIO_ICON: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="9" cy="7" r="3"/><path d="M2 21v-1a5 5 0 0 1 5-5h4a5 5 0 0 1 5 5v1M16 3.1a3 3 0 0 1 0 5.8M22 21v-1a5 5 0 0 0-3-4.6"/></svg>"#;

/// The concept page the landing page shows a real excerpt of.
///
/// "What a page looks like" and "Approaches" are not mock-ups: they render this
/// page's own parsed markdown through the same code path a concept page uses,
/// so the homepage cannot drift from the page it is advertising.
const SPECIMEN_HREF: &str = "concepts/design-patterns-idioms/mem-take-and-mem-replace.html";
const SPECIMEN_SCENARIO: &str = "Modifying an existing object";

/// The scenario the Approaches section demonstrates, which must be a
/// *different* one from [`SPECIMEN_SCENARIO`]: the picker opens on its Classic
/// panel, so pointing both sections at one scenario would print the same card
/// twice on the same page.
const APPROACHES_SCENARIO: &str = "Interior mutability";

/// Find a scenario on `page` by title, falling back to its first one.
///
/// A renamed scenario should cost the homepage its intended excerpt, not a
/// whole section, so this degrades rather than giving up — and says so.
fn pick_scenario<'a>(page: &'a Page, title: &str) -> Option<&'a crate::model::Scenario> {
    if let Some(found) = page.scenarios.iter().find(|s| s.title == title) {
        return Some(found);
    }
    let fallback = page.scenarios.first()?;
    eprintln!(
        "  warning: landing scenario \"{title}\" not found on {SPECIMEN_HREF}; \
         fell back to \"{}\"",
        fallback.title
    );
    Some(fallback)
}

/// The page the landing page excerpts, plus the two distinct scenarios its
/// demo sections render.
fn find_specimen<'a>(
    pages: &'a [Page],
) -> Option<(
    &'a Page,
    &'a crate::model::Scenario,
    Option<&'a crate::model::Scenario>,
)> {
    let Some(page) = pages.iter().find(|p| p.href == SPECIMEN_HREF) else {
        eprintln!(
            "  warning: landing specimen page {SPECIMEN_HREF} not found; \
             \"What a page looks like\" and \"Approaches\" were left off the homepage"
        );
        return None;
    };
    let specimen = pick_scenario(page, SPECIMEN_SCENARIO)?;
    // Only offer the approaches scenario if it really is a different card.
    let approaches = page
        .scenarios
        .iter()
        .find(|s| s.title == APPROACHES_SCENARIO)
        .filter(|s| s.title != specimen.title);
    Some((page, specimen, approaches))
}

/// "What a page looks like": one real scenario inside a frame that reproduces
/// the concept page's own chrome, so a first-time visitor can see the shape of
/// a page without leaving the homepage. The Classic panel is the scenario
/// exactly as its page renders it; the Embedded panel carries that page's
/// embedded basic-usage example behind the same segmented control, since the
/// parallel embedded content is the thing most visitors never discover.
fn render_specimen(page: &Page, scenario: &crate::model::Scenario, depth: usize) -> String {
    let rationale = scenario
        .rationale_html
        .as_ref()
        .map(|r| format!("<div class=\"rationale\">{r}</div>"))
        .unwrap_or_default();

    // Prefer the embedded treatment of the *same* scenario, so the two tabs
    // are the one topic handled twice rather than two unrelated excerpts —
    // which is the claim the section makes. Falling back to another embedded
    // scenario, and only then to the basic-usage example, keeps the panel
    // populated for pages whose embedded half is arranged differently.
    let embedded_scenario = page
        .embedded_scenarios
        .iter()
        .find(|s| s.title == scenario.title)
        .or_else(|| page.embedded_scenarios.first());

    // The section's claim is that the page is written twice, so the two tabs
    // have to be the same topic for it to demonstrate anything. Both fall-back
    // branches still render, and a weaker section looks exactly like a correct
    // one, so say it out loud — across the concept set only about two thirds of
    // scenarios have an embedded counterpart under the same title, and four
    // pages have no embedded half at all.
    match embedded_scenario {
        Some(es) if es.title == scenario.title => {}
        Some(es) => eprintln!(
            "  warning: landing specimen: {} has no Embedded scenario titled \"{}\", so the \
             Embedded tab shows \"{}\" — the two tabs are no longer the same topic",
            page.href, scenario.title, es.title
        ),
        None if !page.embedded_basic_usage_html.is_empty() => eprintln!(
            "  warning: landing specimen: {} has no Embedded scenarios; the Embedded tab falls \
             back to its basic-usage example",
            page.href
        ),
        None => eprintln!(
            "  warning: landing specimen: {} has no Embedded content at all, so the Embedded tab \
             shows only the callout and the \"written twice\" claim above it is false for it",
            page.href
        ),
    }

    let embedded_example = match embedded_scenario {
        Some(es) => {
            let es_rationale = es
                .rationale_html
                .as_ref()
                .map(|r| format!("<div class=\"rationale\">{r}</div>"))
                .unwrap_or_default();
            format!(
                r#"<div class="card">
                  <div class="scen-tag">{SCENARIO_ICON}Scenario</div>
                  <h3 class="scenario-title">{title}</h3>
                  {body}
                  {es_rationale}
                </div>"#,
                title = html_escape(&es.title),
                body = es.body_html,
            )
        }
        None if !page.embedded_basic_usage_html.is_empty() => format!(
            r#"<div class="card">
                  <div class="scen-tag">{SCENARIO_ICON}Basic usage example (Embedded)</div>
                  {body}
                </div>"#,
            body = page.embedded_basic_usage_html,
        ),
        None => String::new(),
    };

    format!(
        r#"      <div class="lp-band">
        <div class="lp-inner">
          <div class="lp-head">
            <h2>What a page looks like</h2>
            <span class="lp-head-note">Live excerpt &middot; {section} &rarr; {group}</span>
          </div>
          <p class="lp-intro">A concept page doesn&rsquo;t stop at an explanation. <strong>Best practices &amp; deeper information</strong> breaks the topic into concrete scenarios &mdash; &ldquo;creating a new object&rdquo;, &ldquo;working with collections&rdquo; &mdash; each with a recommended way to handle it, the code, and the reasoning.</p>

          <p class="lp-intro lp-intro-embedded">And nearly every concept page is written <strong>twice</strong>: once in the Classic Rust below, and again for <code>no_std</code> and bare-metal work, with its own explanation, its own examples and its own scenarios. Switch to it by pressing the <strong>Embedded</strong> button.</p>

          <div class="lp-legend">
            <span class="lp-legend-item"><b>1</b> the scenario</span>
            <span class="lp-legend-item"><b>2</b> the key line, marked <code>// &lt;-</code></span>
            <span class="lp-legend-item"><b>3</b> why this way, not another</span>
          </div>

          <div class="lp-specimen">
            <div class="lp-specimen-bar">
              <div class="lp-specimen-crumb">
                <span>{section}</span><span class="sep">/</span><span>{group}</span><span class="sep">/</span><span class="here">{title}</span>
              </div>
              <div class="segmented" role="tablist" aria-label="Content flavour">
                <button type="button" class="spec-seg on" data-flavor="classic" role="tab" aria-selected="true">Classic</button>
                <button type="button" class="spec-seg" data-flavor="embedded" role="tab" aria-selected="false">Embedded</button>
              </div>
            </div>

            <div class="lp-specimen-body">
              <div class="spec-flavor" data-flavor="classic">
                <div class="card">
                  <div class="scen-tag">{SCENARIO_ICON}Scenario</div>
                  <h3 class="scenario-title">{scen_title}</h3>
                  {body}
                  {rationale}
                </div>
              </div>
              <div class="spec-flavor" data-flavor="embedded" hidden>
                <div class="lp-embedded">
                  <span class="support-badge">Embedded: {badge}</span>
                  <p><strong>This is the Embedded half of the same page</strong> &mdash; written for <code>no_std</code> and bare-metal work, with its own explanation, its own examples and its own scenarios. Every page states its support level up front, as above.</p>
                </div>
                {embedded_example}
              </div>
            </div>

            <div class="lp-specimen-foot">
              <span>Excerpt from a real page &mdash; nothing here is a mock-up.</span>
              <a href="{href}">Open the full page &rarr;</a>
            </div>
          </div>
        </div>
      </div>"#,
        section = page.section.label(),
        group = html_escape(&group_label(page.section, &page.subgroup)),
        title = html_escape(&page.front.title),
        scen_title = html_escape(&scenario.title),
        badge = embedded_badge(page.embedded_support()),
        href = href_from(depth, &page.href),
        body = scenario.body_html,
    )
}

/// The Approaches section: the same scenario again, this time through
/// [`render_multi_scenario`] — the picker concept pages use, panels and vote
/// keys included, so the demo is the production component rather than a
/// look-alike. Skipped when the scenario has no approaches yet, since a
/// one-entry dropdown would demonstrate nothing.
fn render_approaches_section(page: &Page, scenario: &crate::model::Scenario) -> String {
    if scenario.approaches.is_empty() {
        return String::new();
    }
    let card = render_multi_scenario(scenario, page.href.trim_end_matches(".html"));
    format!(
        r#"      <div class="lp-band lp-band-deep">
        <div class="lp-inner">
          <div class="lp-head">
            <h2>Approaches</h2>
            <span class="lp-head-note">Community-contributed &middot; live on the site today</span>
          </div>
          <p class="lp-intro">There is rarely one right way to do something in Rust. Every scenario starts with the site&rsquo;s recommended <strong>Classic</strong> solution; anyone can add an alternative implementation of the <em>exact same scenario</em>, attributed to them. Switch below &mdash; the code, the explanation and the byline all change together.</p>
          {card}
        </div>
      </div>"#
    )
}

pub fn render_landing_page(pages: &[Page]) -> String {
    let depth = 0;
    let sidebar = render_sidebar(pages, None, depth, TopNav::None);

    let (syntax_html, syntax_pages, syntax_groups) =
        render_browse_column(pages, Section::Syntax, depth);
    let (concepts_html, concepts_pages, concepts_groups) =
        render_browse_column(pages, Section::Concepts, depth);
    let total_pages = syntax_pages + concepts_pages;
    let total_groups = syntax_groups + concepts_groups;

    let (specimen, approaches) = match find_specimen(pages) {
        Some((page, specimen_scenario, approaches_scenario)) => (
            render_specimen(page, specimen_scenario, depth),
            approaches_scenario
                .map(|s| render_approaches_section(page, s))
                .unwrap_or_default(),
        ),
        None => (String::new(), String::new()),
    };

    let main = format!(
        r#"      <section class="hero">
        <div class="hero-inner">
          <div class="hero-lead">
            <h1 class="hero-title">A <span class="hl-syntax">Rust</span> reference<br>for people <span class="hl-concepts">writing Rust</span>.</h1>
            <p class="lp-subhead">Part dictionary, part wiki &mdash; meant to be kept open in a second tab while you code.</p>
            <div class="hero-actions">
              <a class="btn btn-primary" href="{contributing}">How to contribute</a>
              <a class="btn" href="{repo}">GitHub repository</a>
            </div>
          </div>
          <div class="hero-notes">
            <div class="hero-note">
              <div class="eyebrow">Built for the moment you&rsquo;re mid-code</div>
              <ul class="lp-lookups">
                <li class="lp-lookup">what <code>?</code> desugars to</li>
                <li class="lp-lookup">whether <code>Rc</code> is thread-safe</li>
                <li class="lp-lookup">how a <code>match</code> guard behaves</li>
              </ul>
              <p>You want the answer, a snippet that compiles, and a short note on <em>why</em> &mdash; not a chapter.</p>
            </div>
          </div>
        </div>
      </section>

      <div class="lp-band lp-band-quiet lp-band-tight">
        <div class="lp-trust">
          <div>
            <div class="eyebrow">Where the content comes from</div>
            <p>Every page is distilled from the <strong>official Rust documentation</strong> &mdash; the Book, the Reference, the Nomicon, the API guidelines, <code>std</code> docs &mdash; and the mainstream Rust books. It is curated, not invented. If something is wrong, outdated or misleading, that is exactly the kind of feedback worth sending.</p>
          </div>
        </div>
      </div>

{specimen}

{approaches}

      <div class="lp-band">
        <div class="lp-inner">
          <div class="lp-head">
            <h2>Browse</h2>
            <span class="lp-head-note">{total_pages} pages &middot; {total_groups} groups &middot; press <code>/</code> to search</span>
          </div>
          <div class="lp-browse">{syntax}{concepts}
          </div>
        </div>
      </div>

      <div class="lp-band">
        <div class="lp-inner">
          <div class="lp-head"><h2>Beyond the reference</h2></div>
          <div class="lp-features">
            <a class="lp-feature" href="articles/">
              <div class="eyebrow">Articles</div>
              <h3>Code-first write-ups &mdash; no think-pieces</h3>
              <p>Community articles that show real, compiling Rust and explain it: how something works under the hood, or how to build it. Opinion pieces don&rsquo;t qualify.</p>
              <span class="lp-feature-go">Read the articles &rarr;</span>
            </a>
            <a class="lp-feature" href="crates/">
              <div class="eyebrow">Crates</div>
              <h3>The same three sections, every crate</h3>
              <p>Overview, when to use it, API map &mdash; in that order, every time. The fixed shape is the point: the second crate page you read is faster than the first.</p>
              <span class="lp-feature-go">Look up a crate &rarr;</span>
            </a>
            <a class="lp-feature" href="conversations/">
              <div class="eyebrow">Conversations</div>
              <h3>GitHub Discussions, in the site&rsquo;s own styling</h3>
              <p>Threads and replies live on GitHub Discussions; the site renders a read-only, near-live mirror, so discussion sits right next to the reference.</p>
              <span class="lp-feature-go">Browse the threads &rarr;</span>
            </a>
          </div>
        </div>
      </div>

      <div class="lp-band lp-band-deep">
        <div class="lp-inner lp-help">
          <div>
            <div class="lp-head"><h2>How to help</h2></div>
            <p>The aim is for this reference to grow with contributions from people writing Rust, not just the maintainer. Contributions of all kinds are welcome &mdash; pointing out wrong information, reporting bugs, flagging what is missing, adding an article, covering a crate, or just taking part in the conversations. <em>Enjoy your coding!</em></p>
            <div class="hero-actions lp-help-actions">
              <a class="btn btn-primary" href="{contributing}">Read CONTRIBUTING.md</a>
              <a class="btn" href="{issues}">Open an issue</a>
            </div>
          </div>
          <ul class="lp-ways">
            <li class="lp-way"><span class="lp-way-k">Approach</span><span class="lp-way-v"><strong>The smallest useful PR.</strong> An additive markdown block on a scenario you know a better way through. You never touch anyone else&rsquo;s content.</span></li>
            <li class="lp-way"><span class="lp-way-k">Article</span><span class="lp-way-v">A technical, code-first piece under <code>pages/articles/</code>, with your byline on it.</span></li>
            <li class="lp-way"><span class="lp-way-k">Crate page</span><span class="lp-way-v">One crate, the three fixed sections, one entry per API item.</span></li>
            <li class="lp-way"><span class="lp-way-k">Correction</span><span class="lp-way-v"><strong>Highest priority of all.</strong> A reference is only worth trusting if it gets corrected.</span></li>
            <li class="lp-way"><span class="lp-way-k">Conversation</span><span class="lp-way-v">No PR needed &mdash; post on GitHub Discussions and it appears at the next rebuild.</span></li>
          </ul>
        </div>
      </div>

      <div class="lp-inner">
        <div class="footer-note">
          <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
          <span>Targets current stable Rust &middot; edition 2024</span>
        </div>
      </div>
"#,
        contributing = format!("{REPO_URL}/blob/main/CONTRIBUTING.md"),
        issues = format!("{REPO_URL}/issues/new"),
        repo = REPO_URL,
        syntax = syntax_html,
        concepts = concepts_html,
    );

    let head = Head {
        // Leads with the term people search and ends with the site name, the
        // same shape every content page uses. "Home" said nothing, and this
        // string is also the og:title, so it is the headline on every shared
        // link as well as the one in search results.
        title: "Rust reference for people writing Rust - Rusty Yellow Pages".to_string(),
        // What the project is for, not how much of it there is or how it is
        // arranged: page counts and per-page structure are both things that
        // change, and neither says why the site exists. The three things that
        // do are named here — mapping the language technically, giving the
        // reasoning alongside the practice, and getting the community to
        // share and compare. Kept under ~155 characters, which is roughly
        // where search results stop showing it; it is also the
        // og:description, so it is what a shared link says too.
        description: "Mapping Rust in technical detail — examples and best practices with the \
                      reasoning behind them, and a community sharing knowledge and comparing \
                      approaches."
            .to_string(),
        canonical: abs_url(""),
        og_type: "website",
        image: None,
    };
    shell_with_page_class(&head, depth, &sidebar, &main, "page-landing")
}

/// The page GitHub Pages serves for any URL that doesn't resolve.
///
/// Rendered at [`ANY_URL`] depth, so the sidebar, top navigation and search all
/// keep working from whatever address the reader actually asked for — which is
/// the point of replacing the default: someone who mistyped a URL or followed a
/// stale link lands somewhere they can search from rather than on a dead end.
///
/// Deliberately absent from `sitemap.xml`, and nothing links to it. It needs no
/// `noindex`: GitHub Pages serves it with a genuine 404 status, which is what
/// keeps it out of search results.
pub fn render_not_found_page(pages: &[Page]) -> String {
    let depth = ANY_URL;
    let sidebar = render_sidebar(pages, None, depth, TopNav::None);

    let main = format!(
        r#"      <section class="doc">
        <div class="browse-head">
          <h2 class="section-title">This page isn&rsquo;t here</h2>
          <span class="browse-count">Error 404</span>
        </div>

        <p>The address you asked for doesn&rsquo;t match any page on this site.
        It may have been renamed since the link you followed was written, or
        the URL may have picked up a typo on the way here.</p>

        <p>Every page is in the search box at the top &mdash; press
        <code>/</code> to jump straight to it. It matches tokens as
        well as words, so <code>?</code>, <code>&amp;</code> and
        <code>impl</code> all find their own pages. The full contents are in the
        sidebar too.</p>

        <hr class="divider">

        <div class="community-grid">
          <a class="community-card" href="{home}">
            <span class="eyebrow">Start over</span>
            <span class="community-title">Home</span>
            <span class="community-desc">Browse every group in Syntax and Concepts.</span>
          </a>
          <a class="community-card" href="{articles}">
            <span class="eyebrow">Read</span>
            <span class="community-title">Articles</span>
            <span class="community-desc">Community deep dives into Rust concepts.</span>
          </a>
          <a class="community-card" href="{crates}">
            <span class="eyebrow">Look up a crate</span>
            <span class="community-title">Crates</span>
            <span class="community-desc">A directory of the crates people reach for.</span>
          </a>
          <a class="community-card" href="{repo}/issues">
            <span class="eyebrow">Something we broke?</span>
            <span class="community-title">Report a dead link</span>
            <span class="community-desc">If a link on this site sent you here, we&rsquo;d like to know.</span>
          </a>
        </div>

        <div class="footer-note">
          <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
          <span>Targets current stable Rust &middot; edition 2024</span>
        </div>
      </section>
"#,
        home = href_from(depth, ""),
        articles = href_from(depth, "articles/"),
        crates = href_from(depth, "crates/"),
        repo = REPO_URL,
    );

    let head = Head {
        title: "Page not found - Rusty Yellow Pages".to_string(),
        description: "That page doesn't exist on Rusty Yellow Pages. Search the full reference, or start again from the contents.".to_string(),
        canonical: abs_url("404.html"),
        og_type: "website",
        image: None,
    };
    shell(&head, depth, &sidebar, &main)
}

/// Singular noun for a syntax subgroup, used to make bare-symbol pages
/// (`?`, `&`, `match`) into descriptive, searchable `<title>` text.
fn syntax_group_noun(subgroup: &str) -> &'static str {
    match subgroup {
        "keywords" => "Keyword",
        "operators" => "Operator",
        "lifetimes" => "Lifetime",
        "literals" => "Literal",
        "punctuation" => "Punctuation",
        "comments" => "Comment",
        "attributes" => "Attribute",
        "macros" => "Macro",
        _ => "Syntax",
    }
}

/// The `<title>` text for a content page, tuned for search: it leads with
/// "Rust" and the topic so a query like `rust traits` matches, and keeps the
/// site name at the end. Syntax pages get their group noun appended so a bare
/// symbol like `?` reads as `Rust - ? Operator - Rusty Yellow Pages`.
/// Returned raw; [`shell`] escapes it.
fn page_title(page: &Page) -> String {
    let t = &page.front.title;
    match page.section {
        Section::Syntax => format!(
            "Rust - {t} {group} - Rusty Yellow Pages",
            group = syntax_group_noun(&page.subgroup)
        ),
        Section::Concepts => format!("Rust - {t} - Rusty Yellow Pages"),
    }
}

/// A search-friendly `<meta name="description">` for a content page. Returned
/// raw; [`shell`] escapes it.
fn page_description(page: &Page) -> String {
    let t = &page.front.title;
    match page.section {
        Section::Syntax => format!(
            "{t} — a Rust {noun}. What it means and how to use it, with quick examples.",
            noun = syntax_group_noun(&page.subgroup).to_lowercase()
        ),
        Section::Concepts => format!(
            "{t} in Rust — explanation, examples, and best practices, densely cross-linked."
        ),
    }
}

/// The full `<head>` metadata for a content page.
fn page_head(page: &Page) -> Head {
    Head {
        title: page_title(page),
        description: page_description(page),
        canonical: abs_url(&page.href),
        og_type: "website",
        image: None,
    }
}

pub fn render_page_document(page: &Page, pages: &[Page], index: &LinkIndex) -> String {
    let depth = page.href.matches('/').count();
    let sidebar = render_sidebar(pages, Some(page), depth, TopNav::None);
    let main = render_content_page(page, pages, index);
    shell(&page_head(page), depth, &sidebar, &main)
}
