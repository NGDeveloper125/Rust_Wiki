use std::path::Path;

use crate::markdown;
use crate::model::{FrontMatter, Page, Scenario, Section};

fn split_frontmatter(raw: &str) -> Result<(&str, &str), String> {
    let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    let rest = raw.strip_prefix("---").ok_or("missing opening `---`")?;
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
        .ok_or("expected newline after opening `---`")?;

    let mut close_at = None;
    for (pos, _) in rest.match_indices("\n---") {
        let after = &rest[pos + 4..];
        if after.is_empty() || after.starts_with('\n') || after.starts_with("\r\n") {
            close_at = Some(pos);
            break;
        }
    }
    let close_at = close_at.ok_or("missing closing `---`")?;
    let yaml = &rest[..close_at];
    let body = &rest[close_at + 4..];
    let body = body
        .strip_prefix("\r\n")
        .or_else(|| body.strip_prefix('\n'))
        .unwrap_or(body);
    Ok((yaml, body))
}

fn find_section<'a>(sections: &'a [(String, String)], prefix: &str) -> Option<&'a str> {
    sections
        .iter()
        .find(|(title, _)| title.eq_ignore_ascii_case(prefix))
        .map(|(_, body)| body.as_str())
}

pub fn build_page(
    path: &Path,
    section: Section,
    subgroup: &str,
    slug: &str,
    href: &str,
) -> Result<Page, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (yaml, body) = split_frontmatter(&raw)?;
    let front: FrontMatter =
        serde_yaml::from_str(yaml).map_err(|e| format!("frontmatter parse error: {e}"))?;

    let h2 = markdown::split_h2(body);
    let explanation_md = find_section(&h2, "Explanation").unwrap_or_default();
    if explanation_md.contains("```") {
        eprintln!(
            "  warning: {} — Explanation contains a fenced code block; it should be prose only \
             (inline `code` spans are fine). Move the runnable example into Basic usage example.",
            path.display()
        );
    }
    let basic_usage_md = find_section(&h2, "Basic usage example").unwrap_or_default();
    let best_practices_md =
        find_section(&h2, "Best practices & deeper information").unwrap_or_default();
    let embedded_md = find_section(&h2, "Embedded Rust Notes").unwrap_or_default();

    let (intro_md, scenario_blocks) = markdown::split_scenarios(best_practices_md);
    let scenarios = scenario_blocks
        .into_iter()
        .map(|(title, scenario_md)| {
            let (body_md, rationale_md) = markdown::split_rationale(&scenario_md);
            Scenario {
                title,
                body_html: markdown::to_html(&body_md),
                rationale_html: rationale_md.map(|r| markdown::to_html(&r)),
            }
        })
        .collect();

    Ok(Page {
        front,
        section,
        subgroup: subgroup.to_string(),
        slug: slug.to_string(),
        href: href.to_string(),
        explanation_html: markdown::to_html(explanation_md),
        basic_usage_html: markdown::to_html(basic_usage_md),
        best_practices_intro_html: markdown::to_html(&intro_md),
        scenarios,
        embedded_notes_html: markdown::to_html(embedded_md),
    })
}
