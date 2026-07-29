//! Community crate pages — a directory of the ecosystem's libraries, one page
//! per crate.
//!
//! Where an article (`crate::articles`) is deliberately free-form prose, a
//! crate page is the opposite: every one has the **same three sections**, so a
//! reader can jump between crates and always find the same thing in the same
//! place.
//!
//! 1. `## Overview` — what the crate is, what problem it solves, how it's built.
//! 2. `## When to use it` — concrete situations the crate is a good fit for,
//!    each a `### Use case:` block with real code.
//! 3. `## API map` — the crate's API, grouped by `###` headings, with one
//!    `####` entry per item: a small call example, what it does, and when
//!    you'd reach for it.
//!
//! Like articles, crate pages are contributed as markdown pull requests and
//! the body is trusted (maintainer-reviewed), so it renders through
//! `crate::markdown::to_html` rather than the conversations sanitizer.

mod render;

use std::path::Path;

use serde::Deserialize;

use crate::bodylinks;
use crate::markdown;
use crate::model::Page;
use crate::parse::split_frontmatter;

/// Every crate page lives directly under `docs/crates/`, so its depth
/// (number of `/`-separated segments before the file) is 1.
const DEPTH: usize = 1;

#[derive(Debug, Deserialize)]
struct FrontMatter {
    title: String,
    /// The crates.io name, when it differs from the file name. Defaults to the
    /// slug, which is why pages are named after the crate they document.
    #[serde(default, rename = "crate")]
    crate_name: Option<String>,
    /// The release the page was written against, e.g. `"1.0"`.
    #[serde(default)]
    version: Option<String>,
    /// `yes` / `optional` / `no` — whether the crate works without `std`.
    #[serde(default)]
    no_std: Option<String>,
    /// Display name for the byline.
    author: String,
    /// GitHub handle, used to build the attribution link.
    github: String,
    /// Publication date, `YYYY-MM-DD`. Set/adjusted by the maintainer at merge.
    date: String,
    /// One-to-two sentences for listings and search.
    summary: String,
    /// Small free list of topics. `tags:` is accepted as an alias.
    #[serde(default, alias = "tags")]
    categories: Vec<String>,
    /// Source repository. Defaults to no link.
    #[serde(default)]
    repository: Option<String>,
    /// API docs. Defaults to `https://docs.rs/<crate>`.
    #[serde(default)]
    docs: Option<String>,
}

/// One item in the API map: a signature, what it does, and when to use it.
pub struct ApiEntry {
    /// The heading text, e.g. `` `Error::msg` `` — inline code is honoured.
    pub signature: String,
    pub body_html: String,
    /// Rendered `**When to use it:** ...` callout; `None` if the author
    /// didn't write one.
    pub when_html: Option<String>,
}

/// A named group of API entries, e.g. "Creating errors".
pub struct ApiGroup {
    pub title: String,
    pub intro_html: String,
    pub entries: Vec<ApiEntry>,
}

/// One "the crate is a good fit here" example.
pub struct UseCase {
    pub title: String,
    pub body_html: String,
    /// Rendered `**Why it fits:** ...` callout; `None` if absent.
    pub fit_html: Option<String>,
}

pub struct Crate {
    pub title: String,
    pub crate_name: String,
    pub version: Option<String>,
    pub no_std: Option<String>,
    pub author: String,
    pub github: String,
    pub date: String,
    pub summary: String,
    pub categories: Vec<String>,
    pub repository: Option<String>,
    pub docs: String,
    /// File stem, e.g. `anyhow`.
    pub slug: String,
    /// Site-root-relative output path, e.g. `crates/anyhow.html`.
    pub href: String,

    pub overview_html: String,
    pub use_cases_intro_html: String,
    pub use_cases: Vec<UseCase>,
    pub api_intro_html: String,
    pub api_groups: Vec<ApiGroup>,
}

impl Crate {
    /// The GitHub profile URL for the byline link.
    pub fn github_url(&self) -> String {
        let handle = self.github.trim_start_matches('@');
        format!("https://github.com/{handle}")
    }

    pub fn crates_io_url(&self) -> String {
        format!("https://crates.io/crates/{}", self.crate_name)
    }

    /// The `data-vote-key` / vote-issue title for this page's like chip,
    /// mirroring the article scheme (`crate::<slug>`).
    pub fn vote_key(&self) -> String {
        format!("crate::{}", self.slug)
    }

    /// Total API entries across every group, shown on the index card so a
    /// reader can see how thorough a page's API map is at a glance.
    pub fn api_count(&self) -> usize {
        self.api_groups.iter().map(|g| g.entries.len()).sum()
    }
}

/// Load every `pages/crates/*.md` (flat — no subgroup level), A-Z by name.
/// Files whose name starts with `_` (e.g. `_CRATE_TEMPLATE.md`) are treated
/// as templates/drafts and skipped.
///
/// A file with missing/invalid required frontmatter prints a clear `error:`
/// line naming the file and the offending field and is skipped, so the rest of
/// the site still builds — the maintainer's PR-review safety net, not a hard
/// build stopper. A missing *section* is only a warning: the page still
/// publishes, minus that section.
pub fn load(pages_root: &Path) -> Vec<Crate> {
    let dir = pages_root.join("crates");
    let mut crates = Vec::new();

    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return crates, // no crate pages yet — fine.
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("crate")
            .to_string();
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
                eprintln!("error: {} — invalid frontmatter: {e}", path.display());
                continue;
            }
        };

        let crate_name = front.crate_name.unwrap_or_else(|| slug.clone());
        let docs = front
            .docs
            .unwrap_or_else(|| format!("https://docs.rs/{crate_name}"));

        let h2 = markdown::split_h2(body);
        let overview_md = section(&h2, "Overview");
        let when_md = section(&h2, "When to use it");
        let api_md = section(&h2, "API map");
        for (name, md) in [
            ("Overview", overview_md),
            ("When to use it", when_md),
            ("API map", api_md),
        ] {
            if md.trim().is_empty() {
                eprintln!(
                    "  warning: {} — crate page is missing a '## {name}' section.",
                    path.display()
                );
            }
        }

        let (use_cases_intro_md, use_case_blocks) = markdown::split_use_cases(when_md);
        let use_cases: Vec<UseCase> = use_case_blocks
            .into_iter()
            .map(|(title, md)| {
                let (body_md, fit_md) = markdown::split_why_it_fits(&md);
                UseCase {
                    title,
                    body_html: markdown::to_html(&body_md),
                    fit_html: fit_md.map(|f| markdown::to_html(&f)),
                }
            })
            .collect();

        let api_groups = build_api_groups(&path, api_md);

        let href = format!("crates/{slug}.html");
        crates.push(Crate {
            title: front.title,
            crate_name,
            version: front.version,
            no_std: front.no_std,
            author: front.author,
            github: front.github,
            date: front.date,
            summary: front.summary,
            categories: front.categories,
            repository: front.repository,
            docs,
            slug,
            href,
            overview_html: markdown::to_html(overview_md),
            use_cases_intro_html: markdown::to_html(&use_cases_intro_md),
            use_cases,
            api_intro_html: markdown::to_html(intro_before(api_md, "### ").trim()),
            api_groups,
        });
    }

    // A directory reads best alphabetically; ties can't happen (slugs are
    // unique file names) but compare them anyway for a total order.
    crates.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then_with(|| a.slug.cmp(&b.slug))
    });
    crates
}

/// Build the API map: `###` headings are groups, each `####` under one is an
/// entry (signature + explanation + optional "when to use it" callout).
/// A group with no entries is kept — its prose still renders — but warned
/// about, since an API map with no call examples misses the point.
fn build_api_groups(path: &Path, api_md: &str) -> Vec<ApiGroup> {
    markdown::split_h3(api_md)
        .into_iter()
        .map(|(title, group_md)| {
            let entry_blocks = markdown::split_h4(&group_md);
            let intro_md = intro_before(&group_md, "#### ").trim().to_string();
            if entry_blocks.is_empty() {
                eprintln!(
                    "  warning: {} — API group \"{title}\" has no '#### <api>' entries.",
                    path.display()
                );
            }
            let entries = entry_blocks
                .into_iter()
                .map(|(signature, entry_md)| {
                    let (body_md, when_md) = markdown::split_when_to_use(&entry_md);
                    ApiEntry {
                        signature,
                        body_html: markdown::to_html(&body_md),
                        when_html: when_md.map(|w| markdown::to_html(&w)),
                    }
                })
                .collect();
            ApiGroup {
                title,
                intro_html: markdown::to_html(&intro_md),
                entries,
            }
        })
        .collect()
}

/// The prose in `md` before the first line starting with `marker` — the part
/// `markdown::split_h3`/`split_h4` drop, since they only collect what follows
/// a heading.
fn intro_before<'a>(md: &'a str, marker: &str) -> &'a str {
    if md.starts_with(marker) {
        return "";
    }
    match md.find(&format!("\n{marker}")) {
        Some(i) => &md[..i],
        None => md,
    }
}

fn section<'a>(h2: &'a [(String, String)], title: &str) -> &'a str {
    h2.iter()
        .find(|(t, _)| t.eq_ignore_ascii_case(title))
        .map(|(_, body)| body.as_str())
        .unwrap_or_default()
}

/// Rewrite relative `.md` links in each crate page to the correct `.html` URL,
/// so crate pages can link liberally into the wiki (mirrors `bodylinks` for
/// pages). Sources live at `pages/crates/<slug>.md`, so their link base
/// directory is `crates/`.
pub fn rewrite_body_links(crates: &mut [Crate], pages: &[Page]) {
    let known = bodylinks::known_hrefs(pages);
    let fix = |html: &str| bodylinks::rewrite_links_in(html, "crates", DEPTH, &known);
    for c in crates.iter_mut() {
        c.overview_html = fix(&c.overview_html);
        c.use_cases_intro_html = fix(&c.use_cases_intro_html);
        for u in c.use_cases.iter_mut() {
            u.body_html = fix(&u.body_html);
            if let Some(f) = u.fit_html.take() {
                u.fit_html = Some(fix(&f));
            }
        }
        c.api_intro_html = fix(&c.api_intro_html);
        for g in c.api_groups.iter_mut() {
            g.intro_html = fix(&g.intro_html);
            for e in g.entries.iter_mut() {
                e.body_html = fix(&e.body_html);
                if let Some(w) = e.when_html.take() {
                    e.when_html = Some(fix(&w));
                }
            }
        }
    }
}

/// Write the crates index + one page per crate.
pub fn build(docs_root: &Path, crates: &[Crate], pages: &[Page]) {
    if let Err(e) = render::write_pages(docs_root, crates, pages) {
        eprintln!("crates: could not write pages: {e}");
    } else {
        println!("crates: rendered index + {} crate page(s)", crates.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Crate {
        Crate {
            title: "anyhow".into(),
            crate_name: "anyhow".into(),
            version: None,
            no_std: None,
            author: "A".into(),
            github: "@handle".into(),
            date: "2026-01-01".into(),
            summary: "s".into(),
            categories: vec![],
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
    fn links_and_vote_key() {
        let c = sample();
        assert_eq!(c.github_url(), "https://github.com/handle");
        assert_eq!(c.crates_io_url(), "https://crates.io/crates/anyhow");
        assert_eq!(c.vote_key(), "crate::anyhow");
    }

    #[test]
    fn api_groups_split_intro_from_entries() {
        let md = "### Creating errors\nSome group prose.\n\n#### `anyhow!`\nBuilds an error.\n\n**When to use it:** ad-hoc errors.\n\n#### `Error::msg`\nWraps a message.\n";
        let groups = build_api_groups(Path::new("test.md"), md);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].title, "Creating errors");
        assert!(groups[0].intro_html.contains("Some group prose."));
        assert_eq!(groups[0].entries.len(), 2);
        assert_eq!(groups[0].entries[0].signature, "`anyhow!`");
        assert!(groups[0].entries[0].when_html.is_some());
        assert!(groups[0].entries[1].when_html.is_none());
    }

    #[test]
    fn api_group_with_no_intro_has_empty_intro() {
        let md = "### Errors\n#### `anyhow!`\nBuilds an error.\n";
        let groups = build_api_groups(Path::new("test.md"), md);
        assert!(groups[0].intro_html.is_empty());
        assert_eq!(groups[0].entries.len(), 1);
    }
}
