//! Render the conversations index + per-thread pages, reusing the site's
//! existing shell, sidebar, and card/chip styling so the mirror looks native.

use std::io;
use std::path::Path;

use super::sanitize;
use super::{Author, Conversation, Snapshot, REPO_NAME, REPO_OWNER};
use crate::model::Page;
use crate::nav::{render_sidebar, TopNav};
use crate::render::{abs_url, href_from, shell, Head};
use crate::util::html_escape;

/// Depth of every conversations page (`docs/conversations/<file>.html`).
const DEPTH: usize = 1;

pub fn write_pages(docs_root: &Path, snap: &Snapshot, pages: &[Page]) -> io::Result<()> {
    let dir = docs_root.join("conversations");
    std::fs::create_dir_all(&dir)?;

    std::fs::write(dir.join("index.html"), render_index(snap, pages))?;
    for c in &snap.conversations {
        std::fs::write(dir.join(thread_filename(c)), render_thread(c, pages))?;
    }
    Ok(())
}

fn new_discussion_url() -> String {
    format!("https://github.com/{REPO_OWNER}/{REPO_NAME}/discussions/new/choose")
}

fn render_index(snap: &Snapshot, pages: &[Page]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::Conversations);
    let home = href_from(DEPTH, "index.html");

    let breadcrumb = format!(
        r#"<nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">Conversations</span>
      </nav>"#
    );

    let page_head = format!(
        r#"<div class="page-head">
        <div class="title-block">
          <h1 class="page-title">Conversations</h1>
        </div>
        <a class="convo-cta" href="{new}" target="_blank" rel="noopener">Start a conversation &rarr;</a>
      </div>"#,
        new = new_discussion_url(),
    );

    let lead = r#"<p class="lead">A read-only mirror of the project's <strong>GitHub Discussions</strong>. Browse threads here; posting, replying, and moderation all happen on GitHub. This page is a snapshot, refreshed periodically &mdash; near-live, not live.</p>"#;

    let body = if snap.conversations.is_empty() {
        format!(
            r#"<div class="card convo-empty">
        <p>No conversations to show yet.</p>
        <p>Be the first &mdash; <a href="{new}" target="_blank" rel="noopener">start a conversation on GitHub &rarr;</a></p>
      </div>"#,
            new = new_discussion_url(),
        )
    } else {
        let filters = render_filters(&snap.conversations);
        let items: String = snap
            .conversations
            .iter()
            .map(render_index_item)
            .collect::<Vec<_>>()
            .join("\n        ");
        format!("{filters}\n        <div class=\"convo-list\">\n        {items}\n        </div>")
    };

    let main = format!(
        r#"      {breadcrumb}

      {page_head}

      {lead}

      <hr class="divider">

      {body}

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>Conversations mirror the repo's GitHub Discussions</span>
      </div>
"#
    );

    let head = Head {
        title: "Rust - Conversations - Rusty Yellow Pages".to_string(),
        description: "A read-only mirror of the project's GitHub Discussions — ask questions, compare approaches, and share what you know about Rust.".to_string(),
        canonical: abs_url("conversations/index.html"),
    };
    shell(&head, DEPTH, &sidebar, &main)
}

/// Distinct category filter chips (client-side filtering via site.js).
fn render_filters(convos: &[Conversation]) -> String {
    let mut cats: Vec<&str> = Vec::new();
    for c in convos {
        if !c.category.is_empty() && !cats.contains(&c.category.as_str()) {
            cats.push(&c.category);
        }
    }
    if cats.len() < 2 {
        return String::new();
    }
    let mut chips = String::from(
        r#"<button class="convo-filter on" data-cat="*">All</button>"#,
    );
    for cat in cats {
        chips.push_str(&format!(
            "\n          <button class=\"convo-filter\" data-cat=\"{cat}\">{cat}</button>",
            cat = html_escape(cat),
        ));
    }
    format!("<div class=\"convo-filters\" role=\"group\" aria-label=\"Filter by category\">\n          {chips}\n        </div>")
}

fn render_index_item(c: &Conversation) -> String {
    let href = thread_filename(c);
    let comment_count = format!(
        "{} comment{}",
        c.total_comment_count,
        if c.total_comment_count == 1 { "" } else { "s" }
    );

    // Preview: the most recent reply, or the original post if there are none.
    let (preview_label, preview_body) = match c.comments.last() {
        Some(last) => (
            format!(
                "Latest reply &middot; by {} &middot; {}",
                author_html(&last.author),
                fmt_date(&last.created_at)
            ),
            render_body(&last.body_md),
        ),
        None => (
            "Original post".to_string(),
            render_body(&c.body_md),
        ),
    };

    format!(
        r#"<article class="card convo-item" data-cat="{cat_attr}">
          <div class="convo-item-head">
            <a class="convo-title" href="{href}">{title}</a>
            {cat_badge}
          </div>
          <div class="convo-meta">by {author} &middot; {date} &middot; {count}</div>
          <div class="convo-preview">
            <div class="convo-preview-label">{preview_label}</div>
            {preview_body}
          </div>
          <button class="convo-expand" aria-expanded="false">Expand full conversation &#9662;</button>
          <div class="convo-full" hidden>
            {full}
          </div>
          <div class="convo-actions">
            <a class="convo-open" href="{href}">Open full thread &#8599;</a>
            <a class="convo-github" href="{gh}" target="_blank" rel="noopener">Add a comment on GitHub &rarr;</a>
          </div>
        </article>"#,
        cat_attr = html_escape(&c.category),
        title = html_escape(&c.title),
        cat_badge = category_badge(c),
        author = author_html(&c.author),
        date = fmt_date(&c.created_at),
        count = comment_count,
        full = render_thread_body(c),
        gh = html_escape(&c.url),
    )
}

fn render_thread(c: &Conversation, pages: &[Page]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::Conversations);
    let home = href_from(DEPTH, "index.html");
    let index = "index.html";

    let breadcrumb = format!(
        r#"<nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <a href="{index}">Conversations</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">{title}</span>
      </nav>"#,
        title = html_escape(&c.title),
    );

    let page_head = format!(
        r#"<div class="page-head">
        <div class="title-block">
          <h1 class="page-title">{title}</h1>
        </div>
        <a class="convo-cta" href="{gh}" target="_blank" rel="noopener">Add a comment on GitHub &rarr;</a>
      </div>"#,
        title = html_escape(&c.title),
        gh = html_escape(&c.url),
    );

    let meta = format!(
        r#"<div class="convo-meta">by {author} &middot; {date} &middot; {count} comment{plural} {badge}</div>"#,
        author = author_html(&c.author),
        date = fmt_date(&c.created_at),
        count = c.total_comment_count,
        plural = if c.total_comment_count == 1 { "" } else { "s" },
        badge = category_badge(c),
    );

    let main = format!(
        r#"      {breadcrumb}

      {page_head}

      {meta}

      <hr class="divider">

      {body}

      <div class="convo-actions convo-actions-foot">
        <a class="convo-open" href="{index}">&#8592; Back to Conversations</a>
        <a class="convo-github" href="{gh}" target="_blank" rel="noopener">Add a comment on GitHub &rarr;</a>
      </div>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>Mirrored read-only from GitHub Discussions</span>
      </div>
"#,
        body = render_thread_body(c),
        gh = html_escape(&c.url),
    );

    let head = Head {
        title: format!("Rust - {} - Rusty Yellow Pages", c.title),
        description: format!(
            "{} — a community discussion in the Rusty Yellow Pages Rust reference.",
            c.title
        ),
        canonical: abs_url(&format!("conversations/{}", thread_filename(c))),
    };
    shell(&head, DEPTH, &sidebar, &main)
}

/// The full conversation body (original post + every fetched comment).
/// Shared by the inline "expand" on the index and the standalone thread page.
fn render_thread_body(c: &Conversation) -> String {
    let mut out = format!(
        "<div class=\"comment convo-op\">\n            <div class=\"comment-head\">Original post &middot; by {author} &middot; {date}</div>\n            {body}\n          </div>\n",
        author = author_html(&c.author),
        date = fmt_date(&c.created_at),
        body = render_body(&c.body_md),
    );

    let n = c.comments.len();
    if n > 0 {
        out.push_str(&format!(
            "          <h2 class=\"section-title convo-replies-title\">{n} repl{}</h2>\n",
            if n == 1 { "y" } else { "ies" }
        ));
        for cm in &c.comments {
            out.push_str(&format!(
                "          <div class=\"comment\">\n            <div class=\"comment-head\">by {author} &middot; {date}</div>\n            {body}\n          </div>\n",
                author = author_html(&cm.author),
                date = fmt_date(&cm.created_at),
                body = render_body(&cm.body_md),
            ));
        }
    }

    if c.total_comment_count as usize > n {
        out.push_str(&format!(
            "          <p class=\"convo-more\">Showing the first {n} of {total} comments. <a href=\"{url}\" target=\"_blank\" rel=\"noopener\">Read the rest on GitHub &rarr;</a></p>\n",
            total = c.total_comment_count,
            url = html_escape(&c.url),
        ));
    }
    out
}

fn render_body(md: &str) -> String {
    format!("<div class=\"convo-body\">{}</div>", sanitize::to_html(md))
}

fn author_html(a: &Option<Author>) -> String {
    match a {
        Some(a) if !a.url.is_empty() => format!(
            "<a class=\"convo-author\" href=\"{}\" target=\"_blank\" rel=\"noopener\">@{}</a>",
            html_escape(&a.url),
            html_escape(&a.login)
        ),
        Some(a) => format!("<span class=\"convo-author\">@{}</span>", html_escape(&a.login)),
        None => "<span class=\"convo-author\">a former member</span>".to_string(),
    }
}

fn category_badge(c: &Conversation) -> String {
    if c.category.is_empty() {
        return String::new();
    }
    // GitHub's `emoji` field can return a shortcode (":pray:") rather than a
    // glyph; only show it when it's an actual emoji character.
    let emoji = if !c.category_emoji.is_empty() && !c.category_emoji.contains(':') {
        format!("{} ", html_escape(&c.category_emoji))
    } else {
        String::new()
    };
    format!(
        "<span class=\"chip convo-cat\">{emoji}{}</span>",
        html_escape(&c.category)
    )
}

fn thread_filename(c: &Conversation) -> String {
    let slug = slugify(&c.title);
    if slug.is_empty() {
        format!("{}.html", c.number)
    } else {
        format!("{}-{}.html", c.number, slug)
    }
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !out.is_empty() && !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    let out = out.trim_matches('-');
    // Keep filenames tidy (all ASCII, so byte slicing is safe).
    if out.len() > 60 {
        out[..60].trim_end_matches('-').to_string()
    } else {
        out.to_string()
    }
}

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
    fn slugify_basics() {
        assert_eq!(slugify("How should I structure errors?"), "how-should-i-structure-errors");
        assert_eq!(slugify("  Spaces  &  symbols!!  "), "spaces-symbols");
        assert_eq!(slugify("日本語"), "");
    }

    #[test]
    fn fmt_date_iso() {
        assert_eq!(fmt_date("2026-07-20T10:00:00Z"), "Jul 20, 2026");
        assert_eq!(fmt_date("garbage"), "garbage");
    }

    #[test]
    fn ghost_author_renders_safely() {
        assert!(author_html(&None).contains("former member"));
    }
}
