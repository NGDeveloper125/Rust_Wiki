use crate::model::{group_order, nav_bucket, subgroup_order, Page, Section};
use crate::render::href_from;
use crate::util::html_escape;

pub const CHEVRON_SVG: &str = r#"<svg class="chev" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="m9 6 6 6-6 6"/></svg>"#;

fn render_page_link(p: &Page, current: Option<&Page>, from_depth: usize, out: &mut String) {
    let href = href_from(from_depth, &p.href);
    let is_active = match current {
        Some(cur) => std::ptr::eq(p, cur),
        None => false,
    };
    let active_class = if is_active { " active" } else { "" };
    let label_html = if p.section == Section::Syntax {
        format!("<span class=\"tok\">{}</span>", html_escape(&p.front.title))
    } else {
        html_escape(&p.front.title)
    };
    out.push_str(&format!(
        "        <a class=\"nav-link{active_class}\" href=\"{href}\">{label_html}</a>\n"
    ));
}

fn contains_current(pages: &[&Page], current: Option<&Page>) -> bool {
    match current {
        Some(cur) => pages.iter().any(|p| std::ptr::eq(*p, cur)),
        None => false,
    }
}

/// Bucket `pages` by `nav_bucket()`, in `order` first, then any unlisted
/// buckets alphabetically, then render each as its own nested, collapsible
/// `.nav-group` inside the caller's `.nav-children`.
fn render_nested_groups(
    pages: &[&Page],
    order: &[&str],
    current: Option<&Page>,
    from_depth: usize,
    out: &mut String,
) {
    let mut buckets: Vec<(String, Vec<&Page>)> = Vec::new();
    for &p in pages {
        let key = nav_bucket(&p.front).to_string();
        if let Some(entry) = buckets.iter_mut().find(|(k, _)| *k == key) {
            entry.1.push(p);
        } else {
            buckets.push((key, vec![p]));
        }
    }
    // Listed buckets in `order`'s sequence; unlisted buckets after, alphabetical.
    let listed_len = order.len();
    buckets.sort_by(|(a_key, _), (b_key, _)| {
        let a_rank = order.iter().position(|o| o == a_key).unwrap_or(listed_len);
        let b_rank = order.iter().position(|o| o == b_key).unwrap_or(listed_len);
        a_rank.cmp(&b_rank).then_with(|| a_key.cmp(b_key))
    });

    for (bucket_label, mut bucket_pages) in buckets {
        bucket_pages.sort_by(|a, b| a.front.title.cmp(&b.front.title));
        let is_current = contains_current(&bucket_pages, current);
        let open_class = if is_current { " open" } else { "" };
        let expanded = if is_current { "true" } else { "false" };
        out.push_str(&format!(
            "\n        <div class=\"nav-group nav-subgroup{open_class}\">\n          <button class=\"nav-toggle\" aria-expanded=\"{expanded}\">\n            {CHEVRON_SVG}\n            <span class=\"g-label\">{label}</span><span class=\"g-count\">{count}</span>\n          </button>\n          <div class=\"nav-children\"><div>\n",
            label = html_escape(&bucket_label),
            count = bucket_pages.len(),
        ));
        for p in &bucket_pages {
            render_page_link(p, current, from_depth, out);
        }
        out.push_str("          </div></div>\n        </div>\n");
    }
}

const CHAT_SVG: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>"#;
const ARTICLE_SVG: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h11a2 2 0 0 1 2 2v13a1 1 0 0 0 1 1H6a2 2 0 0 1-2-2z"/><path d="M8 8h6M8 12h6M8 16h4"/></svg>"#;
const CRATE_SVG: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 8.5 12 4l9 4.5v7L12 20l-9-4.5z"/><path d="M3 8.5 12 13l9-4.5M12 13v7"/></svg>"#;

/// Which top-level ("not a wiki page") nav link is active, if any. The
/// conversations/articles/crates pages aren't in `pages`, so they can't be
/// matched via `current` the way syntax/concept pages are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopNav {
    None,
    Conversations,
    Articles,
    Crates,
}

/// The site's full sidebar. `active` marks whichever top-level link
/// (Conversations / Articles / Crates) the current page belongs to.
pub fn render_sidebar(
    pages: &[Page],
    current: Option<&Page>,
    from_depth: usize,
    active: TopNav,
) -> String {
    let mut out = String::new();

    let sel = |t: TopNav| if active == t { " active" } else { "" };
    out.push_str(&format!(
        "\n    <div class=\"nav-toplinks\">\n      <a class=\"nav-toplink{ca}\" href=\"{ch}\">{cicon}<span>Conversations</span></a>\n      <a class=\"nav-toplink{aa}\" href=\"{ah}\">{aicon}<span>Articles</span></a>\n      <a class=\"nav-toplink{ka}\" href=\"{kh}\">{kicon}<span>Crates</span></a>\n    </div>\n",
        ca = sel(TopNav::Conversations),
        ch = href_from(from_depth, "conversations/"),
        cicon = CHAT_SVG,
        aa = sel(TopNav::Articles),
        ah = href_from(from_depth, "articles/"),
        aicon = ARTICLE_SVG,
        ka = sel(TopNav::Crates),
        kh = href_from(from_depth, "crates/"),
        kicon = CRATE_SVG,
    ));

    for section in [Section::Syntax, Section::Concepts] {
        out.push_str(&format!(
            "\n    <div class=\"nav-section-label\">{}</div>\n",
            section.label()
        ));
        for (folder, label) in group_order(section) {
            let mut group_pages: Vec<&Page> = pages
                .iter()
                .filter(|p| p.section == section && p.subgroup == *folder)
                .collect();
            if group_pages.is_empty() {
                continue;
            }
            group_pages.sort_by(|a, b| a.front.title.cmp(&b.front.title));

            let group_has_current = contains_current(&group_pages, current);
            let open_class = if group_has_current { " open" } else { "" };
            let expanded = if group_has_current { "true" } else { "false" };

            out.push_str(&format!(
                "\n    <div class=\"nav-group{open_class}\" data-group=\"{folder}\">\n      <button class=\"nav-toggle\" aria-expanded=\"{expanded}\">\n        {CHEVRON_SVG}\n        <span class=\"g-label\">{label}</span><span class=\"g-count\">{count}</span>\n      </button>\n      <div class=\"nav-children\"><div>\n",
                count = group_pages.len(),
            ));

            match subgroup_order(section, folder) {
                Some(order) => render_nested_groups(&group_pages, order, current, from_depth, &mut out),
                None => {
                    for p in &group_pages {
                        render_page_link(p, current, from_depth, &mut out);
                    }
                }
            }

            out.push_str("      </div></div>\n    </div>\n");
        }
    }
    out
}
