//! Community long-form articles — technical, code-first pieces about how Rust
//! works and how to implement things.
//!
//! Unlike syntax/concept pages (`crate::model::Page`), articles are **free-form
//! prose** — they are NOT forced into the `Explanation`/`Basic usage`/`Best
//! practices` H2 structure. So rather than adding a third `Section` variant
//! (which would drag articles through `parse::build_page`'s fixed layout and
//! every exhaustive `match` on `Section`), articles get their own small,
//! parallel model — the same shape Feature 2 used for the conversations mirror.
//!
//! Contribution is PR-based markdown: an author drops a file under
//! `pages/articles/`, opens a PR, and the maintainer reviews and merges. The
//! site stays 100% static; the body is trusted (maintainer-reviewed), so it is
//! rendered with `crate::markdown::to_html` like our other authored pages.

mod render;

use std::path::Path;

use serde::Deserialize;

use crate::bodylinks;
use crate::markdown;
use crate::model::Page;
use crate::parse::split_frontmatter;

/// Every article page lives directly under `docs/articles/`, so its depth
/// (number of `/`-separated segments before the file) is 1.
const DEPTH: usize = 1;

#[derive(Debug, Deserialize)]
struct FrontMatter {
    title: String,
    /// Display name for the byline.
    author: String,
    /// GitHub handle, used to build the attribution link.
    github: String,
    /// Publication date, `YYYY-MM-DD`. Set/adjusted by the maintainer at merge.
    date: String,
    /// One-to-two sentences for listings and search.
    summary: String,
    /// Small free list of topics. `topics:` is accepted as an alias.
    #[serde(default, alias = "topics")]
    tags: Vec<String>,
    /// Optional lead image: an external URL, or a repo-local path like
    /// `images/foo.png` (authors drop the file in `pages/articles/images/`,
    /// which is copied verbatim into `docs/articles/images/`).
    #[serde(default)]
    image: Option<String>,
}

pub struct Article {
    pub title: String,
    pub author: String,
    pub github: String,
    pub date: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub image: Option<String>,
    /// File stem, e.g. `the-question-mark-operator`.
    pub slug: String,
    /// Site-root-relative output path, e.g. `articles/the-question-mark-operator.html`.
    pub href: String,
    /// Rendered (and body-link-rewritten) HTML of the whole article body.
    pub body_html: String,
}

impl Article {
    /// The GitHub profile URL for the byline link.
    pub fn github_url(&self) -> String {
        let handle = self.github.trim_start_matches('@');
        format!("https://github.com/{handle}")
    }

    /// The `data-vote-key` / vote-issue title for this article's like chip,
    /// mirroring the approach-vote scheme (`article::<slug>`).
    pub fn vote_key(&self) -> String {
        format!("article::{}", self.slug)
    }
}

/// Load every `pages/articles/*.md` (flat — no subgroup level), newest first.
/// Files whose name starts with `_` (e.g. `_ARTICLE_TEMPLATE.md`) are treated
/// as templates/drafts and skipped.
///
/// A file with missing/invalid required frontmatter prints a clear
/// `error:` line naming the file and the offending field and is skipped, so
/// the rest of the site still builds — this is the maintainer's PR-review
/// safety net, not a hard build stopper.
pub fn load(pages_root: &Path) -> Vec<Article> {
    let dir = pages_root.join("articles");
    let mut articles = Vec::new();

    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return articles, // no articles yet — fine.
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("article")
            .to_string();
        // `_`-prefixed files are templates/drafts, not published articles
        // (e.g. `_ARTICLE_TEMPLATE.md`). Skip them silently.
        if slug.starts_with('_') {
            continue;
        }

        let raw = match std::fs::read_to_string(&path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: {} — could not read file: {e}", path.display());
                continue;
            }
        };
        let (yaml, body) = match split_frontmatter(&raw) {
            Ok(parts) => parts,
            Err(e) => {
                eprintln!("error: {} — {e}", path.display());
                continue;
            }
        };
        let front: FrontMatter = match serde_yaml::from_str(yaml) {
            Ok(f) => f,
            Err(e) => {
                // serde_yaml's message already names the missing/invalid field
                // (e.g. "missing field `author`").
                eprintln!("error: {} — invalid frontmatter: {e}", path.display());
                continue;
            }
        };

        if !looks_like_iso_date(&front.date) {
            eprintln!(
                "  warning: {} — date \"{}\" is not YYYY-MM-DD; listings sort and display it as-is.",
                path.display(),
                front.date
            );
        }

        let href = format!("articles/{slug}.html");
        articles.push(Article {
            title: front.title,
            author: front.author,
            github: front.github,
            date: front.date,
            summary: front.summary,
            tags: front.tags,
            image: front.image,
            slug,
            href,
            body_html: markdown::to_html(body),
        });
    }

    // Newest first. ISO `YYYY-MM-DD` sorts chronologically as plain strings;
    // ties fall back to title for a stable order.
    articles.sort_by(|a, b| {
        b.date
            .cmp(&a.date)
            .then_with(|| a.title.cmp(&b.title))
    });
    articles
}

/// Rewrite relative `.md` links in each article body to the correct `.html`
/// URL, so articles can link liberally into the wiki (mirrors `bodylinks` for
/// pages). Article sources live at `pages/articles/<slug>.md`, so their link
/// base directory is `articles/`.
pub fn rewrite_body_links(articles: &mut [Article], pages: &[Page]) {
    let known = bodylinks::known_hrefs(pages);
    for a in articles.iter_mut() {
        a.body_html = bodylinks::rewrite_links_in(&a.body_html, "articles", DEPTH, &known);
    }
}

/// Copy any repo-local article images and write the index + article pages.
pub fn build(pages_root: &Path, docs_root: &Path, articles: &[Article], pages: &[Page]) {
    copy_images(pages_root, docs_root);
    if let Err(e) = render::write_pages(docs_root, articles, pages) {
        eprintln!("articles: could not write pages: {e}");
    } else {
        println!("articles: rendered index + {} article page(s)", articles.len());
    }
}

/// Copy `pages/articles/images/` verbatim into `docs/articles/images/` so
/// repo-local lead images resolve. External-URL images need no copying.
fn copy_images(pages_root: &Path, docs_root: &Path) {
    let src = pages_root.join("articles").join("images");
    let dst = docs_root.join("articles").join("images");
    let entries = match std::fs::read_dir(&src) {
        Ok(e) => e,
        Err(_) => return, // no images folder — fine.
    };
    if let Err(e) = std::fs::create_dir_all(&dst) {
        eprintln!("articles: could not create {}: {e}", dst.display());
        return;
    }
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() {
            if let Some(name) = p.file_name() {
                if let Err(e) = std::fs::copy(&p, dst.join(name)) {
                    eprintln!("articles: could not copy image {}: {e}", p.display());
                }
            }
        }
    }
}

fn looks_like_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..].iter().all(u8::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_date_check() {
        assert!(looks_like_iso_date("2026-07-25"));
        assert!(!looks_like_iso_date("2026-7-25"));
        assert!(!looks_like_iso_date("July 25, 2026"));
        assert!(!looks_like_iso_date(""));
    }

    #[test]
    fn github_url_strips_at() {
        let a = Article {
            title: "t".into(),
            author: "A".into(),
            github: "@handle".into(),
            date: "2026-01-01".into(),
            summary: "s".into(),
            tags: vec![],
            image: None,
            slug: "t".into(),
            href: "articles/t.html".into(),
            body_html: String::new(),
        };
        assert_eq!(a.github_url(), "https://github.com/handle");
        assert_eq!(a.vote_key(), "article::t");
    }
}
