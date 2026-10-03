//! The WebAssembly section: what wasm is, which Rust targets reach it, and
//! how the tooling around it got to its current state.
//!
//! One page for now. It is laid out as a directory (`wasm/`) rather than a
//! single file so pages can be added under it later without changing the URL
//! of this one, the way `more/templates/` did.
//!
//! The prose lives in `pages/wasm/_index.md` rather than here. It is prose,
//! it dates — version numbers, who maintains what — and it should be
//! editable without touching the generator.

use std::io;
use std::path::Path;

use crate::crates::domain::Domain;
use crate::markdown;
use crate::model::Page;
use crate::nav::{render_sidebar, TopNav};
use crate::render::{abs_url, href_from, shell, Head};

/// Site-root-relative URLs this module writes. The canonical link and the
/// sitemap are both built from it, so a page added here cannot reach one and
/// not the other.
const URLS: [&str; 1] = ["wasm/"];

/// `docs/wasm/*.html` — one directory below the site root.
const DEPTH: usize = 1;

/// The section's opening prose, from `pages/wasm/_index.md`.
fn load_intro(pages_root: &Path) -> String {
    let path = pages_root.join("wasm").join("_index.md");
    match std::fs::read_to_string(&path) {
        Ok(md) => markdown::to_html(markdown::strip_leading_comment(&md).trim()),
        Err(_) => String::new(),
    }
}

fn render_index(pages_root: &Path, pages: &[Page], domains: &[&'static Domain]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::Wasm, domains);
    let home = href_from(DEPTH, "");

    let main = format!(
        r##"      <nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">WebAssembly</span>
      </nav>

      <div class="page-head">
        <div class="title-block">
          <h1 class="page-title">WebAssembly</h1>
        </div>
      </div>

      <p class="lead">A compilation target with a host that grants it everything, the Rust targets that reach it, and where the tooling stands.</p>

      <hr class="divider">

      <div class="wasm-prose">
{intro}      </div>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
      </div>
"##,
        intro = load_intro(pages_root),
    );

    let head = Head {
        title: "Rust - WebAssembly - Rusty Yellow Pages".to_string(),
        description:
            "What WebAssembly is, how its targets differ in Rust, and the state of the tooling after the rustwasm organisation was archived."
                .to_string(),
        canonical: abs_url(URLS[0]),
        og_type: "website",
        image: None,
    };
    shell(&head, DEPTH, &sidebar, &main)
}

fn write_pages(
    docs_root: &Path,
    pages_root: &Path,
    pages: &[Page],
    domains: &[&'static Domain],
) -> io::Result<()> {
    let dir = docs_root.join("wasm");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(
        dir.join("index.html"),
        render_index(pages_root, pages, domains),
    )?;
    Ok(())
}

/// Write the WebAssembly section.
///
/// Returns the site-root-relative paths it wrote, for the sitemap; empty if
/// writing failed, matching how the other section modules report the same
/// thing.
pub fn build(
    docs_root: &Path,
    pages_root: &Path,
    pages: &[Page],
    domains: &[&'static Domain],
) -> Vec<String> {
    if let Err(e) = write_pages(docs_root, pages_root, pages, domains) {
        eprintln!("wasm: could not write pages: {e}");
        return Vec::new();
    }
    println!("wasm: rendered section index");
    URLS.iter().map(|u| u.to_string()).collect()
}
