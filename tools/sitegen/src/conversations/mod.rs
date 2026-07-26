//! Build-time mirror of the repo's GitHub Discussions.
//!
//! Discussions live entirely on GitHub — the site never accepts writes. At
//! build time we fetch them via the GitHub GraphQL API and render read-only,
//! styled HTML (an index + one page per thread), each carrying links out to
//! GitHub for posting/commenting.
//!
//! This whole subsystem is best-effort: any failure (no token, network error,
//! API change) must NOT fail the build. We fall back to the last committed
//! snapshot (`data/conversations.json`), then to an honest empty state, and
//! the rest of the site always renders.

mod fetch;
mod render;
mod sanitize;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::model::Page;

/// Repo whose Discussions we mirror. Matches `VOTES_REPO` in site.js.
pub const REPO_OWNER: &str = "NGDeveloper125";
pub const REPO_NAME: &str = "Rust_Wiki";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub login: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// `None` for a deleted/ghost account.
    #[serde(default)]
    pub author: Option<Author>,
    /// ISO-8601 timestamp as returned by the API.
    pub created_at: String,
    /// Raw markdown; sanitized to HTML at render time.
    pub body_md: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub number: u64,
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub author: Option<Author>,
    pub created_at: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub category_emoji: String,
    /// Raw markdown; sanitized to HTML at render time.
    pub body_md: String,
    #[serde(default)]
    pub comments: Vec<Comment>,
    /// Total comments on GitHub. May exceed `comments.len()` if a thread has
    /// more than the per-thread fetch cap — see `fetch::COMMENTS_PER_THREAD`.
    #[serde(default)]
    pub total_comment_count: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub conversations: Vec<Conversation>,
}

/// Render the conversations index + thread pages into `docs/conversations/`.
///
/// Best-effort and infallible from the caller's perspective: it logs and
/// degrades rather than returning an error, so `main` never has to guard it.
/// Returns the site-root-relative paths of the pages it wrote (index + each
/// thread) for inclusion in the sitemap; empty if writing failed.
pub fn build(repo_root: &Path, docs_root: &Path, pages: &[Page]) -> Vec<String> {
    let snapshot_path = repo_root.join("data").join("conversations.json");

    let token = std::env::var("GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.trim().is_empty());

    let snapshot = match token {
        Some(tok) => match fetch::fetch_all(REPO_OWNER, REPO_NAME, &tok) {
            Ok(conversations) => {
                let snap = Snapshot {
                    repo: format!("{REPO_OWNER}/{REPO_NAME}"),
                    conversations,
                };
                if let Err(e) = save_snapshot(&snapshot_path, &snap) {
                    eprintln!("conversations: could not write snapshot: {e}");
                }
                println!(
                    "conversations: fetched {} thread(s) from GitHub",
                    snap.conversations.len()
                );
                snap
            }
            Err(e) => {
                eprintln!(
                    "conversations: fetch failed ({e}); falling back to committed snapshot"
                );
                load_snapshot(&snapshot_path)
            }
        },
        None => {
            eprintln!(
                "conversations: no GITHUB_TOKEN set; rendering from committed snapshot if present"
            );
            load_snapshot(&snapshot_path)
        }
    };

    if let Err(e) = render::write_pages(docs_root, &snapshot, pages) {
        eprintln!("conversations: could not write pages: {e}");
        return Vec::new();
    }
    println!(
        "conversations: rendered index + {} thread page(s)",
        snapshot.conversations.len()
    );

    let mut urls = vec!["conversations/index.html".to_string()];
    for c in &snapshot.conversations {
        urls.push(format!("conversations/{}", render::thread_filename(c)));
    }
    urls
}

fn load_snapshot(path: &Path) -> Snapshot {
    match std::fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|e| {
            eprintln!("conversations: snapshot at {} is unreadable ({e}); using empty state", path.display());
            Snapshot::default()
        }),
        Err(_) => {
            eprintln!(
                "conversations: no snapshot at {}; using empty state",
                path.display()
            );
            Snapshot::default()
        }
    }
}

fn save_snapshot(path: &Path, snap: &Snapshot) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(snap).map_err(|e| e.to_string())?;
    std::fs::write(path, json + "\n").map_err(|e| e.to_string())
}
