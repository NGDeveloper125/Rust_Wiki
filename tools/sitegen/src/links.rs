use std::collections::HashMap;

use crate::model::Page;
use crate::util::html_escape;

pub struct LinkIndex {
    by_title: HashMap<String, usize>,
    by_slug: HashMap<String, usize>,
    /// Title with a trailing parenthetical stripped, e.g.
    /// "Operator overloading (std::ops traits)" -> "Operator overloading".
    by_stripped_title: HashMap<String, usize>,
    by_title_lower: HashMap<String, usize>,
}

fn strip_trailing_parenthetical(title: &str) -> Option<&str> {
    let trimmed = title.trim_end();
    if !trimmed.ends_with(')') {
        return None;
    }
    let open = trimmed.rfind(" (")?;
    Some(trimmed[..open].trim_end())
}

impl LinkIndex {
    pub fn build(pages: &[Page]) -> Self {
        let mut by_title = HashMap::new();
        let mut by_slug = HashMap::new();
        let mut by_stripped_title = HashMap::new();
        let mut by_title_lower = HashMap::new();
        for (i, p) in pages.iter().enumerate() {
            by_title.insert(p.front.title.clone(), i);
            by_slug.insert(p.slug.clone(), i);
            by_title_lower
                .entry(p.front.title.to_lowercase())
                .or_insert(i);
            if let Some(stripped) = strip_trailing_parenthetical(&p.front.title) {
                by_stripped_title.entry(stripped.to_string()).or_insert(i);
            }
        }
        LinkIndex {
            by_title,
            by_slug,
            by_stripped_title,
            by_title_lower,
        }
    }

    pub fn resolve<'a>(&self, pages: &'a [Page], reference: &str) -> Option<&'a Page> {
        self.by_title
            .get(reference)
            .or_else(|| self.by_slug.get(reference))
            .or_else(|| self.by_stripped_title.get(reference))
            .or_else(|| self.by_title_lower.get(&reference.to_lowercase()))
            .map(|&i| &pages[i])
    }
}

/// Render one "related" chip. `as_token` wraps the label in the monospace
/// `.tok` span (used for `related_syntax`, never for `related_concepts` —
/// matches the mockup exactly).
pub fn render_chip(
    index: &LinkIndex,
    pages: &[Page],
    from_depth: usize,
    label: &str,
    as_token: bool,
) -> String {
    let inner = if as_token {
        format!("<span class=\"tok\">{}</span>", html_escape(label))
    } else {
        html_escape(label)
    };
    match index.resolve(pages, label) {
        Some(target) => {
            let href = crate::render::href_from(from_depth, &target.href);
            format!("<a class=\"chip\" href=\"{href}\">{inner}</a>")
        }
        None => {
            eprintln!("  warning: unresolved cross-reference \"{label}\"");
            format!("<span class=\"chip chip-unresolved\">{inner}</span>")
        }
    }
}

pub fn render_chip_row(
    index: &LinkIndex,
    pages: &[Page],
    from_depth: usize,
    label: &str,
    items: &[String],
    as_token: bool,
) -> String {
    if items.is_empty() {
        return String::new();
    }
    let chips: String = items
        .iter()
        .map(|item| render_chip(index, pages, from_depth, item, as_token))
        .collect::<Vec<_>>()
        .join("\n          ");
    format!(
        "<div class=\"related-row\">\n          <span class=\"related-label\">{label}</span>\n          {chips}\n        </div>"
    )
}
