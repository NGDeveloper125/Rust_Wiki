use crate::model::{group_order, Page, Section};
use crate::render::href_from;
use crate::util::html_escape;

const CHEVRON_SVG: &str = r#"<svg class="chev" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="m9 6 6 6-6 6"/></svg>"#;

pub fn render_sidebar(pages: &[Page], current: Option<&Page>, from_depth: usize) -> String {
    let mut out = String::new();
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

            let contains_current = match current {
                Some(cur) => group_pages.iter().any(|p| std::ptr::eq(*p, cur)),
                None => false,
            };
            let open_class = if contains_current { " open" } else { "" };
            let expanded = if contains_current { "true" } else { "false" };

            out.push_str(&format!(
                "\n    <div class=\"nav-group{open_class}\" data-group=\"{folder}\">\n      <button class=\"nav-toggle\" aria-expanded=\"{expanded}\">\n        {CHEVRON_SVG}\n        <span class=\"g-label\">{label}</span><span class=\"g-count\">{count}</span>\n      </button>\n      <div class=\"nav-children\"><div>\n",
                count = group_pages.len(),
            ));

            for p in &group_pages {
                let href = href_from(from_depth, &p.href);
                let is_active = match current {
                    Some(cur) => std::ptr::eq(*p, cur),
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

            out.push_str("      </div></div>\n    </div>\n");
        }
    }
    out
}
