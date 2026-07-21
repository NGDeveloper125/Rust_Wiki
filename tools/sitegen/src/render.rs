use crate::links::{render_chip_row, LinkIndex};
use crate::model::{group_label, Page, Section};
use crate::nav::render_sidebar;
use crate::util::html_escape;

pub fn href_from(depth: usize, target: &str) -> String {
    if depth == 0 {
        target.to_string()
    } else {
        "../".repeat(depth) + target
    }
}

fn topbar(depth: usize) -> String {
    let home = href_from(depth, "index.html");
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
pub fn shell(title: &str, depth: usize, sidebar_html: &str, main_html: &str) -> String {
    let css = href_from(depth, "assets/site.css");
    let search_index_js = href_from(depth, "assets/search-index.js");
    let site_js = href_from(depth, "assets/site.js");
    format!(
        r#"<!doctype html>
<html lang="en" data-theme="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} &mdash; Rusty Yellow Pages</title>
<link rel="stylesheet" href="{css}">
</head>
<body>

{topbar}

<div class="shell">
  <aside class="sidebar" id="sidebar">{sidebar_html}
  </aside>

  <main class="content">
    <article class="page">
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
        topbar = topbar(depth),
        site_root = "../".repeat(depth),
    )
}

fn embedded_badge(support: &str) -> &'static str {
    match support {
        "full" => "Full",
        "partial" => "Partial",
        _ => "None",
    }
}

pub fn render_content_page(page: &Page, pages: &[Page], index: &LinkIndex) -> String {
    let depth = page.href.matches('/').count();
    let home = href_from(depth, "index.html");
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
    let embedded_disabled = if support == "none" { " disabled" } else { "" };

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
          <button id="seg-embedded" role="tab" aria-selected="false"{embedded_disabled}>
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

    let (tabs_html, body_sections_html, core_section_word) = if page.section == Section::Syntax {
        let examples_html: String = page
            .usage_examples
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
            .join("\n        ");

        let tabs = r#"<nav class="section-tabs" id="section-tabs">
        <button class="tab on" data-target="explanation">Explanation</button>
        <button class="tab" data-target="examples">Usage examples</button>
      </nav>"#
            .to_string();

        let sections = format!(
            r#"<section class="doc" id="explanation">
        <h2 class="section-title">Explanation</h2>
        {explanation}
      </section>

      <section class="doc" id="examples">
        <h2 class="section-title">Usage examples</h2>
        <div class="scenarios">
        {examples}
        </div>
      </section>"#,
            explanation = page.explanation_html,
            examples = examples_html,
        );
        (tabs, sections, "two")
    } else {
        let scenarios_html: String = page
            .scenarios
            .iter()
            .map(|s| {
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
            })
            .collect::<Vec<_>>()
            .join("\n        ");

        let tabs = r#"<nav class="section-tabs" id="section-tabs">
        <button class="tab on" data-target="explanation">Explanation</button>
        <button class="tab" data-target="basic">Basic usage example</button>
        <button class="tab" data-target="best">Best practices &amp; deeper information</button>
      </nav>"#
            .to_string();

        let sections = format!(
            r#"<section class="doc" id="explanation">
        <h2 class="section-title">Explanation</h2>
        {explanation}
      </section>

      <section class="doc" id="basic">
        <h2 class="section-title">Basic usage example</h2>
        {basic_usage}
      </section>

      <section class="doc" id="best">
        <h2 class="section-title">Best practices &amp; deeper information</h2>
        {intro}
        <div class="scenarios">
        {scenarios}
        </div>
      </section>"#,
            explanation = page.explanation_html,
            basic_usage = page.basic_usage_html,
            intro = page.best_practices_intro_html,
            scenarios = scenarios_html,
        );
        (tabs, sections, "three")
    };

    format!(
        r#"      {breadcrumb}

      {page_head}

      {related}

      <hr class="divider">

      {tabs_html}

      {body_sections_html}

      <section class="doc" id="embedded">
        <h2 class="section-title">Embedded Rust Notes</h2>
        <span class="support-badge">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
          Embedded support: {support_label}
        </span>
        {embedded}
        <p class="emb-hint">Embedded view active &mdash; this section is highlighted. The {core_section_word} core sections above stay written for hosted <code>std</code> Rust and are unchanged.</p>
      </section>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>Targets current stable Rust &middot; edition 2021</span>
      </div>
"#,
        embedded = page.embedded_notes_html,
        support_label = embedded_badge(support),
    )
}

pub fn render_landing_page(pages: &[Page]) -> String {
    let depth = 0;
    let sidebar = render_sidebar(pages, None, depth);

    let mut groups_html = String::new();
    for section in [Section::Syntax, Section::Concepts] {
        for (folder, label) in crate::model::group_order(section) {
            let mut group_pages: Vec<&Page> = pages
                .iter()
                .filter(|p| p.section == section && p.subgroup == *folder)
                .collect();
            if group_pages.is_empty() {
                continue;
            }
            group_pages.sort_by(|a, b| a.front.title.cmp(&b.front.title));
            let links: String = group_pages
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
                .join("\n            ");
            groups_html.push_str(&format!(
                "\n          <div class=\"related-row\">\n            <span class=\"related-label\">{label} ({count})</span>\n            {links}\n          </div>\n",
                count = group_pages.len(),
            ));
        }
    }

    let main = format!(
        r#"      <div class="page-head">
        <div class="title-block">
          <h1 class="page-title">Rusty Yellow Pages</h1>
        </div>
      </div>
      <p class="lead">A free, open-source, deep reference for the Rust programming language &mdash; a directory you look things up in. Every syntax element and every language concept gets its own page, densely cross-linked.</p>

      <hr class="divider">

      <section class="doc">
        <h2 class="section-title">Browse</h2>
        <div class="related">
        {groups}
        </div>
      </section>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>Targets current stable Rust &middot; edition 2021</span>
      </div>
"#,
        groups = groups_html,
    );

    shell("Rusty Yellow Pages", depth, &sidebar, &main)
}

pub fn render_page_document(page: &Page, pages: &[Page], index: &LinkIndex) -> String {
    let depth = page.href.matches('/').count();
    let sidebar = render_sidebar(pages, Some(page), depth);
    let main = render_content_page(page, pages, index);
    shell(&page.front.title, depth, &sidebar, &main)
}
