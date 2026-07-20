mod bodylinks;
mod links;
mod markdown;
mod model;
mod nav;
mod parse;
mod render;
mod search;
mod util;

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

    let index = links::LinkIndex::build(&pages);

    let assets_dir = docs_root.join("assets");
    std::fs::create_dir_all(&assets_dir).expect("create docs/assets");

    std::fs::copy(templates_root.join("site.css"), assets_dir.join("site.css"))
        .expect("copy site.css");
    std::fs::copy(templates_root.join("site.js"), assets_dir.join("site.js"))
        .expect("copy site.js");

    let search_index_js = search::build_search_index(&pages);
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

    println!(
        "generated {} pages + 1 landing page into {}",
        pages.len(),
        docs_root.display()
    );
}
