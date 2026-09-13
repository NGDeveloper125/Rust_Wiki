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

/// The named HTML entities the site's own renderers emit, mapped back to the
/// characters they stand for. Numeric references are handled separately.
const ENTITIES: &[(&str, &str)] = &[
    ("&amp;", "&"),
    ("&lt;", "<"),
    ("&gt;", ">"),
    ("&quot;", "\""),
    ("&apos;", "'"),
    ("&nbsp;", " "),
    ("&mdash;", "\u{2014}"),
    ("&ndash;", "\u{2013}"),
    ("&hellip;", "\u{2026}"),
    ("&middot;", "\u{b7}"),
    ("&rsquo;", "\u{2019}"),
    ("&lsquo;", "\u{2018}"),
    ("&ldquo;", "\u{201c}"),
    ("&rdquo;", "\u{201d}"),
    ("&rsaquo;", "\u{203a}"),
    ("&lsaquo;", "\u{2039}"),
    ("&rarr;", "\u{2192}"),
];

/// Decode the entities above, plus `&#NN;` / `&#xNN;` numeric references.
///
/// Everything is decoded in one left-to-right pass, so an escaped escape such
/// as `&amp;lt;` yields the literal text `&lt;` rather than being decoded a
/// second time into `<`.
fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    'outer: while i < s.len() {
        if s.as_bytes()[i] == b'&' {
            for (ent, ch) in ENTITIES {
                if s[i..].starts_with(ent) {
                    out.push_str(ch);
                    i += ent.len();
                    continue 'outer;
                }
            }
            // Bounded, so a stray `&#` cannot scan the rest of the document
            // looking for a terminator that never comes.
            if s[i..].starts_with("&#") {
                if let Some(end) = s[i..].find(';').filter(|e| *e <= 12) {
                    let body = &s[i + 2..i + end];
                    let parsed = match body.strip_prefix(['x', 'X']) {
                        Some(hex) => u32::from_str_radix(hex, 16).ok(),
                        None => body.parse::<u32>().ok(),
                    };
                    if let Some(ch) = parsed.and_then(char::from_u32) {
                        out.push(ch);
                        i += end + 1;
                        continue 'outer;
                    }
                }
            }
        }
        // Step by character, not byte, so multi-byte UTF-8 survives intact.
        let ch = s[i..].chars().next().expect("i is on a char boundary");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Plain text of `html`: tags dropped, entities decoded, runs of whitespace
/// collapsed to single spaces.
///
/// Relies on the generated markup being well formed — every literal `<` and
/// `>` in text content has already been escaped by [`html_escape`], so a plain
/// in-tag/out-of-tag toggle cannot be fooled by prose.
pub fn html_to_text(html: &str) -> String {
    let mut stripped = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => stripped.push(c),
            _ => {}
        }
    }
    decode_entities(&stripped)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Text of the leading `<p>` elements of `html`, taking as many as it needs to
/// reach `min_len` characters so that a one-line opener still yields a usable
/// summary. Falls back to the whole block when there is no paragraph markup.
fn leading_paragraphs(html: &str, min_len: usize) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(open) = rest.find("<p") {
        let after = &rest[open..];
        // `<p` also prefixes `<pre`; a code block is not prose worth quoting.
        if after.starts_with("<pre") {
            match after.find("</pre>") {
                Some(e) => {
                    rest = &after[e + "</pre>".len()..];
                    continue;
                }
                None => break,
            }
        }
        let (Some(body_start), Some(close)) = (after.find('>'), after.find("</p>")) else {
            break;
        };
        if close < body_start {
            break;
        }
        let text = html_to_text(&after[body_start + 1..close]);
        if !text.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&text);
        }
        if out.chars().count() >= min_len {
            return out;
        }
        rest = &after[close + "</p>".len()..];
    }
    if out.is_empty() {
        html_to_text(html)
    } else {
        out
    }
}

/// Shorten `text` to at most `max` characters for a `<meta name="description">`.
///
/// Prefers to end on a sentence boundary so the summary reads as a finished
/// thought. A `.` only ends a sentence when whitespace follows it, which keeps
/// paths and versions (`std::mem::take`, `1.0`) from being mistaken for one.
/// Failing that it cuts on a word boundary, marking the cut with an ellipsis.
fn truncate_summary(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut = text
        .char_indices()
        .nth(max)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    let window = &text[..cut];

    let sentence_end = window
        .char_indices()
        .filter(|(i, c)| {
            matches!(c, '.' | '!' | '?')
                && window[i + c.len_utf8()..]
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace)
        })
        .map(|(i, c)| i + c.len_utf8())
        .next_back();
    // Only worth taking if enough of the text survives to still say something.
    if let Some(end) = sentence_end.filter(|e| text[..*e].chars().count() >= max / 3) {
        return window[..end].trim_end().to_string();
    }

    match window.rfind(' ') {
        Some(sp) => format!("{}\u{2026}", window[..sp].trim_end()),
        None => format!("{}\u{2026}", window.trim_end()),
    }
}

/// A `<meta name="description">` distilled from a page's own rendered prose.
///
/// Returns `None` when the prose is too thin to describe the page, leaving the
/// caller to fall back to whatever it would otherwise have written — a summary
/// that is merely short is worse than a templated one.
pub fn meta_description_from_html(html: &str) -> Option<String> {
    const MIN_USEFUL: usize = 60;
    const TARGET: usize = 200;
    let text = leading_paragraphs(html, 120);
    if text.chars().count() < MIN_USEFUL {
        return None;
    }
    Some(truncate_summary(&text, TARGET))
}

/// Escape `s` for use as a JSON string body (without the surrounding quotes).
///
/// `<` is escaped alongside the characters JSON itself requires, so a value
/// containing `</script>` cannot close the `<script type="application/ld+json">`
/// block it is embedded in.
pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003c"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
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

    #[test]
    fn text_drops_tags_and_decodes_entities() {
        assert_eq!(
            html_to_text("<p>a <code>Vec&lt;T&gt;</code>   grows</p>"),
            "a Vec<T> grows"
        );
        assert_eq!(html_to_text("<p>a &mdash; b</p>"), "a \u{2014} b");
    }

    #[test]
    fn entities_are_decoded_once_only() {
        // `&amp;lt;` is how a page shows a reader the literal text `&lt;`.
        // Decoding twice would turn it into `<` and misquote the page.
        assert_eq!(html_to_text("&amp;lt; stays text"), "&lt; stays text");
    }

    #[test]
    fn numeric_references_decode_and_a_stray_ampersand_hash_survives() {
        assert_eq!(html_to_text("&#65;&#x42;"), "AB");
        assert_eq!(html_to_text("a &# b"), "a &# b");
    }

    #[test]
    fn description_ends_on_a_sentence_boundary() {
        let html = "<p>The opening sentence stands on its own and is easily                     long enough to serve as a summary by itself. The second                     one then runs on well past any reasonable limit, which                     means the cut has to land somewhere inside it rather                     than at its end.</p>";
        let d = meta_description_from_html(html).expect("prose is long enough");
        assert!(d.ends_with("by itself."), "did not cut at the sentence: {d}");
    }

    #[test]
    fn a_dotted_path_is_not_a_sentence_end() {
        // None of these dots is followed by whitespace, so none of them ends a
        // sentence. With no boundary to find, the summary falls back to a word
        // cut rather than stopping inside `std::mem::take` or `1.0`.
        let text = "Call std::mem::take on it, which as of 1.0 leaves the                     field holding Default::default() rather than anything                     uninitialised, and hands you back whatever the field was                     holding before the call, without cloning it anywhere                     along the way";
        let d = meta_description_from_html(&format!("<p>{text}</p>")).unwrap();
        assert!(d.ends_with("\u{2026}"), "expected a word cut: {d}");
        assert!(d.contains("std::mem::take"), "{d}");
    }

    #[test]
    fn code_blocks_are_not_quoted_as_prose() {
        let html = "<pre><code>fn main() {}</code></pre>                    <p>This paragraph is the real explanation, and it is long                     enough to be worth using as the page description.</p>";
        let d = meta_description_from_html(html).unwrap();
        assert!(d.starts_with("This paragraph"), "{d}");
    }

    #[test]
    fn short_openers_pull_in_the_next_paragraph() {
        let html = "<p>Short.</p><p>The paragraph that follows carries the                     weight of the explanation and is long enough to matter.</p>";
        let d = meta_description_from_html(html).unwrap();
        assert!(d.starts_with("Short. The paragraph"), "{d}");
    }

    #[test]
    fn prose_too_thin_to_describe_a_page_gets_no_description() {
        assert_eq!(meta_description_from_html("<p>Too short.</p>"), None);
        assert_eq!(meta_description_from_html(""), None);
    }

    #[test]
    fn json_escape_cannot_close_the_script_block() {
        assert_eq!(json_escape("a</script>b"), "a\\u003c/script>b");
    }

    #[test]
    fn json_escape_handles_quotes_backslashes_and_controls() {
        assert_eq!(json_escape("q\"w"), "q\\\"w");
        assert_eq!(json_escape("a\nb"), "a\\nb");
        assert_eq!(json_escape("p\\q"), "p\\\\q");
        assert_eq!(json_escape("x\u{7}y"), "x\\u0007y");
    }
}
