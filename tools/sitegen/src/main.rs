mod articles;
mod bodylinks;
mod conversations;
mod crates;
mod highlight;
mod links;
mod markdown;
mod model;
mod more;
mod nav;
mod palette;
mod parse;
mod render;
mod search;
mod sitemap;
mod util;
mod vscode;

use std::path::{Path, PathBuf};

use model::{Page, Section};

fn collect_md_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(subgroups) = std::fs::read_dir(root) else {
        return out;
    };
    for entry in subgroups.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(files) = std::fs::read_dir(&path) {
                for f in files.flatten() {
                    let fp = f.path();
                    if fp.extension().and_then(|e| e.to_str()) == Some("md") {
                        out.push(fp);
                    }
                }
            }
        }
    }
    out
}

fn load_pages(pages_root: &Path, section: Section) -> Vec<Page> {
    let section_dir = pages_root.join(match section {
        Section::Syntax => "syntax",
        Section::Concepts => "concepts",
    });
    let mut pages = Vec::new();
    for file in collect_md_files(&section_dir) {
        let subgroup = file
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("misc")
            .to_string();
        let slug = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("page")
            .to_string();
        let section_name = match section {
            Section::Syntax => "syntax",
            Section::Concepts => "concepts",
        };
        let href = format!("{section_name}/{subgroup}/{slug}.html");

        match parse::build_page(&file, section, &subgroup, &slug, &href) {
            Ok(page) => pages.push(page),
            Err(e) => eprintln!("error parsing {}: {e}", file.display()),
        }
    }
    pages
}

fn main() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let pages_root = repo_root.join("pages");
    let docs_root = repo_root.join("docs");
    let templates_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");

    let mut pages = load_pages(&pages_root, Section::Syntax);
    pages.extend(load_pages(&pages_root, Section::Concepts));

    if pages.is_empty() {
        eprintln!("no pages found under {}; aborting", pages_root.display());
        std::process::exit(1);
    }

    bodylinks::rewrite_all(&mut pages);

    let mut articles = articles::load(&pages_root);
    articles::rewrite_body_links(&mut articles, &pages);

    let mut crate_pages = crates::load(&pages_root);
    crates::rewrite_body_links(&mut crate_pages, &pages);

    let index = links::LinkIndex::build(&pages);

    let assets_dir = docs_root.join("assets");
    std::fs::create_dir_all(&assets_dir).expect("create docs/assets");

    // The stylesheet is the template plus the generated palette, so a colour
    // change in palette.rs reaches every code block on the site.
    let base_css = std::fs::read_to_string(templates_root.join("site.css")).expect("read site.css");
    std::fs::write(
        assets_dir.join("site.css"),
        format!("{base_css}\n{}", palette::stylesheet()),
    )
    .expect("write site.css");
    std::fs::copy(templates_root.join("site.js"), assets_dir.join("site.js"))
        .expect("copy site.js");

    let search_index_js = search::build_search_index(&pages, &articles, &crate_pages);
    std::fs::write(assets_dir.join("search-index.js"), search_index_js)
        .expect("write search-index.js");

    for page in &pages {
        let html = render::render_page_document(page, &pages, &index);
        let out_path = docs_root.join(&page.href);
        std::fs::create_dir_all(out_path.parent().unwrap()).expect("create page dir");
        std::fs::write(&out_path, html).expect("write page html");
    }

    let landing_html = render::render_landing_page(&pages);
    std::fs::write(docs_root.join("index.html"), landing_html).expect("write index.html");

    let not_found_html = render::render_not_found_page(&pages);
    std::fs::write(docs_root.join("404.html"), not_found_html).expect("write 404.html");

    articles::build(&pages_root, &docs_root, &articles, &pages);

    crates::build(&docs_root, &crate_pages, &pages);

    more::build(&docs_root, &pages);

    // Not part of the site: the editor theme is generated from the same palette
    // so the two cannot drift.
    vscode::build(&repo_root);

    // Best-effort GitHub Discussions mirror. Never fails the build.
    let conversation_urls = conversations::build(&repo_root, &docs_root, &pages);

    let sitemap_xml = sitemap::build(&pages, &articles, &crate_pages, &conversation_urls);
    std::fs::write(docs_root.join("sitemap.xml"), sitemap_xml).expect("write sitemap.xml");

    let robots = format!(
        "User-agent: *\nAllow: /\nSitemap: {}sitemap.xml\n",
        render::SITE_BASE
    );
    std::fs::write(docs_root.join("robots.txt"), robots).expect("write robots.txt");

    let stale_md_links = report_unrewritten_md_links(&docs_root);

    println!(
        "generated {} pages + 1 landing page into {}",
        pages.len(),
        docs_root.display()
    );
    if stale_md_links > 0 {
        eprintln!(
            "warning: {stale_md_links} markdown link(s) reached the generated site and will 404 \
             for readers — a rendered-markdown field is most likely missing from \
             bodylinks::rewrite_all"
        );
    }
}

/// Count `href="....md"` targets that survived into the generated site, naming
/// each one.
///
/// Body-link rewriting turns markdown cross-references into `.html` hrefs, so a
/// surviving `.md` target is always a dead link. Nothing else catches it: the
/// page around it renders correctly, and the link only fails when a reader
/// clicks it. Links out to markdown on another host (the repo's own
/// CONTRIBUTING.md, say) are left alone.
fn report_unrewritten_md_links(docs_root: &Path) -> usize {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("html") {
                out.push(path);
            }
        }
    }

    let mut html_files = Vec::new();
    walk(docs_root, &mut html_files);
    html_files.sort();

    let mut found = 0;
    for file in &html_files {
        let Ok(html) = std::fs::read_to_string(file) else {
            continue;
        };
        for (idx, _) in html.match_indices("href=\"") {
            let after = &html[idx + 6..];
            let Some(end) = after.find('"') else { continue };
            let target = &after[..end];
            let is_external = target.starts_with("http://") || target.starts_with("https://");
            if target.ends_with(".md") && !is_external {
                found += 1;
                eprintln!("  warning: {} links to \"{target}\"", file.display());
            }
        }
    }
    found
}
