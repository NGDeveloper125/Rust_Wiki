//! Render the articles index (a searchable, sortable 2-column card grid) and
//! one page per article, reusing the site's shell/sidebar/card styling.

use std::io;
use std::path::Path;

use super::{Article, DEPTH};
use crate::model::Page;
use crate::nav::{render_sidebar, TopNav};
use crate::render::{abs_url, href_from, shell, Head};
use crate::util::html_escape;

/// The repo's contributor guide, linked from the "write an article" CTA.
const CONTRIBUTE_URL: &str =
    "https://github.com/NGDeveloper125/Rust_Wiki/blob/main/CONTRIBUTING.md#articles";

pub fn write_pages(docs_root: &Path, articles: &[Article], pages: &[Page]) -> io::Result<()> {
    let dir = docs_root.join("articles");
    std::fs::create_dir_all(&dir)?;

    std::fs::write(dir.join("index.html"), render_index(articles, pages))?;
    for a in articles {
        std::fs::write(dir.join(format!("{}.html", a.slug)), render_article(a, pages))?;
    }
    Ok(())
}

fn render_index(articles: &[Article], pages: &[Page]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::Articles);
    let home = href_from(DEPTH, "index.html");

    let breadcrumb = format!(
        r#"<nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">Articles</span>
      </nav>"#
    );

    let page_head = format!(
        r#"<div class="page-head">
        <div class="title-block">
          <h1 class="page-title">Articles</h1>
        </div>
        <a class="convo-cta" href="{CONTRIBUTE_URL}" target="_blank" rel="noopener">Write an article &rarr;</a>
      </div>"#
    );

    let lead = r#"<p class="lead">Community-written, technical articles about how Rust works and how to implement things &mdash; real, compiling code with the reasoning behind it, linked into the rest of the wiki. Articles are contributed as markdown pull requests.</p>"#;

    let body = if articles.is_empty() {
        format!(
            r#"<div class="card convo-empty">
        <p>No articles yet.</p>
        <p>Be the first &mdash; <a href="{CONTRIBUTE_URL}" target="_blank" rel="noopener">write one and open a pull request &rarr;</a></p>
      </div>"#
        )
    } else {
        let toolbar = r#"<div class="article-toolbar">
          <div class="article-search">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="7"/><path d="m21 21-3.6-3.6"/></svg>
            <input id="article-search" type="text" placeholder="Filter articles by title or summary&hellip;" autocomplete="off" spellcheck="false" aria-label="Filter articles">
          </div>
          <label class="article-sort">
            <span>Sort</span>
            <select id="article-sort" aria-label="Sort articles">
              <option value="date" selected>Newest</option>
              <option value="rating">Top rated</option>
            </select>
          </label>
        </div>"#;

        let cards: String = articles
            .iter()
            .enumerate()
            .map(|(i, a)| render_card(a, i))
            .collect::<Vec<_>>()
            .join("\n        ");

        format!(
            "{toolbar}\n        <div class=\"article-grid\" id=\"article-grid\">\n        {cards}\n        </div>\n        <p class=\"article-nomatch\" id=\"article-nomatch\" hidden>No articles match your filter.</p>"
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
        <span>Articles are contributed as markdown pull requests</span>
      </div>
"#
    );

    let head = Head {
        title: "Rust - Articles - Rusty Yellow Pages".to_string(),
        description: "Community-written, technical articles about how Rust works and how to implement things — real, compiling code with the reasoning behind it.".to_string(),
        canonical: abs_url("articles/index.html"),
    };
    shell(&head, DEPTH, &sidebar, &main)
}

/// One card in the index grid. `i` is the authored (newest-first) position,
/// used by site.js to restore "Newest" order after a "Top rated" sort.
fn render_card(a: &Article, i: usize) -> String {
    let image = match &a.image {
        Some(src) if !src.trim().is_empty() => format!(
            r#"<div class="article-card-img"><img src="{src}" alt="" loading="lazy"></div>"#,
            src = html_escape(src),
        ),
        _ => String::new(),
    };

    format!(
        r#"<article class="article-card" data-i="{i}" data-vote-key="{vote_key}">
          <a class="article-card-link" href="{href}">
            {image}
            <div class="article-card-body">
              <h3 class="article-card-title">{title}</h3>
              <p class="article-card-summary">{summary_html}</p>
            </div>
          </a>
          <div class="article-card-foot">
            <div class="article-card-meta">by {author} &middot; {date}</div>
            {tags}
            <a class="article-like" hidden target="_blank" rel="noopener">&#128077; <span class="like-n"></span></a>
          </div>
        </article>"#,
        vote_key = html_escape(&a.vote_key()),
        // The index lives at `articles/index.html`, so link to a sibling
        // article by bare filename, not its site-root-relative `href`.
        href = html_escape(&format!("{}.html", a.slug)),
        title = html_escape(&a.title),
        summary_html = render_inline(&a.summary),
        author = html_escape(&a.author),
        date = fmt_date(&a.date),
        tags = render_tags(a),
    )
}

/// HTML-escape `s`, but render inline `` `code` `` spans (backtick-delimited)
/// as a distinct monospace token. Lets a summary mark an operator or ident
/// (e.g. `` `?` ``) so it reads as code rather than stray punctuation. If the
/// backticks are unbalanced, they're left as literal characters.
fn render_inline(s: &str) -> String {
    if s.matches('`').count() < 2 {
        return html_escape(s);
    }
    let balanced = s.matches('`').count() % 2 == 0;
    let mut out = String::new();
    let mut in_code = false;
    for (i, seg) in s.split('`').enumerate() {
        if i > 0 {
            if balanced {
                in_code = !in_code;
            } else {
                out.push('`'); // unbalanced — keep backticks literal
            }
        }
        if in_code {
            out.push_str(&format!(
                "<code class=\"tok-inline\">{}</code>",
                html_escape(seg)
            ));
        } else {
            out.push_str(&html_escape(seg));
        }
    }
    out
}

fn render_article(a: &Article, pages: &[Page]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::Articles);
    let home = href_from(DEPTH, "index.html");
    let index = "index.html";

    let breadcrumb = format!(
        r#"<nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <a href="{index}">Articles</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">{title}</span>
      </nav>"#,
        title = html_escape(&a.title),
    );

    let page_head = format!(
        r#"<div class="page-head">
        <div class="title-block">
          <h1 class="page-title">{title}</h1>
        </div>
      </div>"#,
        title = html_escape(&a.title),
    );

    let byline = format!(
        r#"<div class="article-byline" data-vote-key="{vote_key}">
        <span>by <a class="article-author" href="{gh}" target="_blank" rel="noopener">{author}</a> &middot; {date}</span>
        {tags}
        <a class="article-like" hidden target="_blank" rel="noopener">&#128077; <span class="like-n"></span> &mdash; like this article on GitHub</a>
      </div>"#,
        vote_key = html_escape(&a.vote_key()),
        gh = html_escape(&a.github_url()),
        author = html_escape(&a.author),
        date = fmt_date(&a.date),
        tags = render_tags(a),
    );

    let lead_image = match &a.image {
        Some(src) if !src.trim().is_empty() => format!(
            r#"<div class="article-hero"><img src="{src}" alt="" loading="lazy"></div>"#,
            src = html_escape(src),
        ),
        _ => String::new(),
    };

    let main = format!(
        r#"      {breadcrumb}

      {page_head}

      {byline}

      <hr class="divider">

      {lead_image}

      <div class="article-body">
      {body}
      </div>

      <div class="convo-actions convo-actions-foot">
        <a class="convo-open" href="{index}">&#8592; Back to Articles</a>
      </div>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>Targets current stable Rust &middot; edition 2021</span>
      </div>
"#,
        body = a.body_html,
    );

    let head = Head {
        title: format!("Rust - {} - Rusty Yellow Pages", a.title),
        description: a.summary.clone(),
        canonical: abs_url(&a.href),
    };
    shell(&head, DEPTH, &sidebar, &main)
}

fn render_tags(a: &Article) -> String {
    if a.tags.is_empty() {
        return String::new();
    }
    let tags: String = a
        .tags
        .iter()
        .map(|t| format!("<span class=\"article-tag\">{}</span>", html_escape(t)))
        .collect::<Vec<_>>()
        .join("");
    format!("<div class=\"article-tags\">{tags}</div>")
}

/// `YYYY-MM-DD` -> `Mon D, YYYY`; anything else is shown verbatim.
fn fmt_date(iso: &str) -> String {
    let d = iso.get(..10).unwrap_or(iso);
    let parts: Vec<&str> = d.split('-').collect();
    if parts.len() == 3 {
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        if let (Ok(m), Ok(day)) = (parts[1].parse::<usize>(), parts[2].parse::<u32>()) {
            if (1..=12).contains(&m) {
                return format!("{} {}, {}", MONTHS[m - 1], day, parts[0]);
            }
        }
    }
    d.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_date_iso() {
        assert_eq!(fmt_date("2026-07-25"), "Jul 25, 2026");
        assert_eq!(fmt_date("whenever"), "whenever");
    }
}
