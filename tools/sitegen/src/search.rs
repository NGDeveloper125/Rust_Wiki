use crate::articles::Article;
use crate::crates::Crate;
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

pub fn build_search_index(pages: &[Page], articles: &[Article], crates: &[Crate]) -> String {
    let mut entries = Vec::with_capacity(pages.len() + articles.len() + crates.len());
    for p in pages {
        let kind_label = match p.section {
            Section::Syntax => "syntax",
            Section::Concepts => "concept",
        };
        let is_token = p.section == Section::Syntax
            && p.front.kind.as_deref().map(is_token_kind).unwrap_or(false);
        let group_lbl = group_label(p.section, &p.subgroup);
        let crumb = format!("{} \u{203a} {}", p.section.label(), group_lbl);
        let aliases: Vec<String> = p
            .front
            .search_aliases
            .iter()
            .map(|a| a.trim().to_lowercase())
            .filter(|a| !a.is_empty())
            .collect();
        let kw = format!(
            "{} {} {}",
            p.front.title,
            p.slug.replace('-', " "),
            aliases.join(" ")
        )
        .trim_end()
        .to_lowercase();

        // Emitted only for the handful of pages that declare aliases, so the
        // shipped index stays as small as it was for everything else. The
        // client scores an exact alias hit as highly as an exact title hit.
        let alias_field = if aliases.is_empty() {
            String::new()
        } else {
            let list = aliases
                .iter()
                .map(|a| format!("\"{}\"", js_string_escape(a)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("alias: [{list}], ")
        };

        entries.push(format!(
            "  {{ title: \"{title}\", crumb: \"{crumb}\", kind: \"{kind}\", isToken: {tok}, {alias}kw: \"{kw}\", href: \"{href}\" }}",
            title = js_string_escape(&p.front.title),
            crumb = js_string_escape(&crumb),
            kind = kind_label,
            tok = is_token,
            alias = alias_field,
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

    // Crate pages are indexed on name + summary + categories, matching the
    // articles policy of keeping the shipped index small and stable.
    for c in crates {
        let kw = format!(
            "{} {} {} {} {}",
            c.title,
            c.crate_name,
            c.summary,
            c.categories.join(" "),
            c.publisher.as_deref().unwrap_or_default(),
        )
        .replace('`', "")
        .to_lowercase();
        entries.push(format!(
            "  {{ title: \"{title}\", crumb: \"Crates\", kind: \"crate\", isToken: false, kw: \"{kw}\", href: \"{href}\" }}",
            title = js_string_escape(&c.title),
            kw = js_string_escape(&kw),
            href = js_string_escape(&c.href),
        ));
    }

    format!(
        "window.SEARCH_INDEX = [\n{}\n];\n",
        entries.join(",\n")
    )
}
