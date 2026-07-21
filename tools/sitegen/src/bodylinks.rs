use std::collections::HashSet;

use crate::model::{Page, Section};
use crate::render::href_from;

fn section_dir_name(section: Section) -> &'static str {
    match section {
        Section::Syntax => "syntax",
        Section::Concepts => "concepts",
    }
}

/// Resolve a relative markdown link target against the directory the
/// linking page's *source* file lives in (mirrors `pages/<section>/<subgroup>/`
/// 1:1 with `docs/<section>/<subgroup>/`, so path math is the same in both trees).
fn resolve_relative(current_dir: &str, target: &str) -> String {
    let mut parts: Vec<&str> = current_dir.split('/').filter(|s| !s.is_empty()).collect();
    for seg in target.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(seg),
        }
    }
    parts.join("/")
}

/// Rewrite every `href="....md"` in rendered section HTML to the
/// corresponding generated page's relative `.html` href. Non-`.md`,
/// absolute, and external hrefs are left untouched. Unresolvable targets
/// (page not yet written) still get a best-effort `.html` link plus a
/// build warning, matching the front-matter cross-reference policy.
fn rewrite_links_in(html: &str, current_dir: &str, depth: usize, known: &HashSet<String>) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    loop {
        match rest.find("href=\"") {
            None => {
                out.push_str(rest);
                break;
            }
            Some(pos) => {
                out.push_str(&rest[..pos]);
                let after = &rest[pos + 6..];
                let end = match after.find('"') {
                    Some(e) => e,
                    None => {
                        out.push_str("href=\"");
                        rest = after;
                        continue;
                    }
                };
                let target = &after[..end];
                let is_external = target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with('#')
                    || target.starts_with("mailto:");
                if target.ends_with(".md") && !is_external {
                    let resolved_md = resolve_relative(current_dir, target);
                    let resolved_html = format!("{}.html", resolved_md.trim_end_matches(".md"));
                    if !known.contains(&resolved_html) {
                        eprintln!(
                            "  warning: body link \"{target}\" (resolved {resolved_html}) has no matching page yet"
                        );
                    }
                    out.push_str("href=\"");
                    out.push_str(&href_from(depth, &resolved_html));
                } else {
                    out.push_str("href=\"");
                    out.push_str(target);
                }
                out.push('"');
                rest = &after[end + 1..];
            }
        }
    }
    out
}

pub fn rewrite_all(pages: &mut [Page]) {
    let known: HashSet<String> = pages.iter().map(|p| p.href.clone()).collect();
    for page in pages.iter_mut() {
        let current_dir = format!("{}/{}", section_dir_name(page.section), page.subgroup);
        let depth = page.href.matches('/').count();
        page.explanation_html = rewrite_links_in(&page.explanation_html, &current_dir, depth, &known);
        page.basic_usage_html = rewrite_links_in(&page.basic_usage_html, &current_dir, depth, &known);
        page.best_practices_intro_html =
            rewrite_links_in(&page.best_practices_intro_html, &current_dir, depth, &known);
        page.embedded_notes_html =
            rewrite_links_in(&page.embedded_notes_html, &current_dir, depth, &known);
        for s in page.scenarios.iter_mut() {
            s.body_html = rewrite_links_in(&s.body_html, &current_dir, depth, &known);
            if let Some(r) = s.rationale_html.take() {
                s.rationale_html = Some(rewrite_links_in(&r, &current_dir, depth, &known));
            }
        }
        for ex in page.usage_examples.iter_mut() {
            ex.body_html = rewrite_links_in(&ex.body_html, &current_dir, depth, &known);
        }
    }
}
