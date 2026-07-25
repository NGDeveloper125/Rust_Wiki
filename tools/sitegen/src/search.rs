use crate::articles::Article;
use crate::model::{group_label, Page, Section};

fn js_string_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(c),
        }
    }
    out
}

fn is_token_kind(kind: &str) -> bool {
    matches!(kind, "operator" | "punctuation" | "keyword" | "comment")
}

pub fn build_search_index(pages: &[Page], articles: &[Article]) -> String {
    let mut entries = Vec::with_capacity(pages.len() + articles.len());
    for p in pages {
        let kind_label = match p.section {
            Section::Syntax => "syntax",
            Section::Concepts => "concept",
        };
        let is_token = p.section == Section::Syntax
            && p.front.kind.as_deref().map(is_token_kind).unwrap_or(false);
        let group_lbl = group_label(p.section, &p.subgroup);
        let crumb = format!("{} \u{203a} {}", p.section.label(), group_lbl);
        let kw = format!("{} {}", p.front.title, p.slug.replace('-', " ")).to_lowercase();

        entries.push(format!(
            "  {{ title: \"{title}\", crumb: \"{crumb}\", kind: \"{kind}\", isToken: {tok}, kw: \"{kw}\", href: \"{href}\" }}",
            title = js_string_escape(&p.front.title),
            crumb = js_string_escape(&crumb),
            kind = kind_label,
            tok = is_token,
            kw = js_string_escape(&kw),
            href = js_string_escape(&p.href),
        ));
    }

    // Articles are indexed on title + summary + tags (not full body) to keep
    // the shipped index small and stable.
    for a in articles {
        // Strip backticks so inline-code markers in the summary don't leak
        // into the (plain-text) search keywords.
        let kw = format!("{} {} {}", a.title, a.summary, a.tags.join(" "))
            .replace('`', "")
            .to_lowercase();
        entries.push(format!(
            "  {{ title: \"{title}\", crumb: \"Articles\", kind: \"article\", isToken: false, kw: \"{kw}\", href: \"{href}\" }}",
            title = js_string_escape(&a.title),
            kw = js_string_escape(&kw),
            href = js_string_escape(&a.href),
        ));
    }

    format!(
        "window.SEARCH_INDEX = [\n{}\n];\n",
        entries.join(",\n")
    )
}
