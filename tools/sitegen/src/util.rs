/// HTML-escape `s`, but render inline `` `code` `` spans (backtick-delimited)
/// as a distinct monospace token. Lets a summary or an API signature mark an
/// operator or ident (e.g. `` `?` ``) so it reads as code rather than stray
/// punctuation. If the backticks are unbalanced, they're left as literal
/// characters.
pub fn render_inline(s: &str) -> String {
    if s.matches('`').count() < 2 {
        return html_escape(s);
    }
    let balanced = s.matches('`').count() % 2 == 0;
    let mut out = String::new();
    let mut in_code = false;
    for (i, seg) in s.split('`').enumerate() {
        if i > 0 {
            if balanced {
                in_code = !in_code;
            } else {
                out.push('`'); // unbalanced — keep backticks literal
            }
        }
        if in_code {
            out.push_str(&format!(
                "<code class=\"tok-inline\">{}</code>",
                html_escape(seg)
            ));
        } else {
            out.push_str(&html_escape(seg));
        }
    }
    out
}

/// `YYYY-MM-DD` -> `Mon D, YYYY`; anything else is shown verbatim.
pub fn fmt_date(iso: &str) -> String {
    let d = iso.get(..10).unwrap_or(iso);
    let parts: Vec<&str> = d.split('-').collect();
    if parts.len() == 3 {
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        if let (Ok(m), Ok(day)) = (parts[1].parse::<usize>(), parts[2].parse::<u32>()) {
            if (1..=12).contains(&m) {
                return format!("{} {}, {}", MONTHS[m - 1], day, parts[0]);
            }
        }
    }
    d.to_string()
}

pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_date_iso() {
        assert_eq!(fmt_date("2026-07-25"), "Jul 25, 2026");
        assert_eq!(fmt_date("whenever"), "whenever");
    }

    #[test]
    fn inline_code_spans() {
        assert_eq!(
            render_inline("the `?` operator"),
            "the <code class=\"tok-inline\">?</code> operator"
        );
        assert_eq!(render_inline("no backticks"), "no backticks");
        assert_eq!(render_inline("one ` backtick"), "one ` backtick");
    }
}
