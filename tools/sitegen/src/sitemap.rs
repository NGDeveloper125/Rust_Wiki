use crate::articles::Article;
use crate::model::Page;
use crate::render::abs_url;
use crate::util::html_escape;

/// Build `sitemap.xml` listing every generated URL as an absolute location.
///
/// `pages` and `articles` cover the content and article pages; `extra` holds
/// site-root-relative paths the two lists don't cover (the conversations index
/// and any thread pages). The landing page and the articles index are added
/// here.
pub fn build(pages: &[Page], articles: &[Article], extra: &[String]) -> String {
    let mut locs: Vec<String> = Vec::new();

    locs.push(abs_url("")); // landing page
    for p in pages {
        locs.push(abs_url(&p.href));
    }
    locs.push(abs_url("articles/index.html"));
    for a in articles {
        locs.push(abs_url(&a.href));
    }
    for path in extra {
        locs.push(abs_url(path));
    }

    let mut body = String::new();
    for loc in &locs {
        body.push_str(&format!("  <url><loc>{}</loc></url>\n", html_escape(loc)));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{body}</urlset>
"#
    )
}
