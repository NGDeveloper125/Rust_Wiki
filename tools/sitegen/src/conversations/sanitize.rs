//! Safe markdown rendering for **untrusted** discussion content.
//!
//! Unlike `crate::markdown::to_html` (used for our own trusted authored
//! pages, which passes raw HTML through verbatim), this renderer is
//! safe-by-construction: it emits only HTML that pulldown-cmark generates
//! from known markdown events, and it never forwards a user's raw HTML.
//!
//! Two defenses:
//!   1. Every raw-HTML event (`Html` / `InlineHtml`) is dropped, so a
//!      `<script>`, `<img onerror=...>`, `<iframe>`, inline event handler,
//!      etc. can never reach the output.
//!   2. Link/image destinations are scheme-checked; anything that isn't
//!      `http`/`https`/`mailto` (e.g. `javascript:`, `data:`) is neutralized
//!      to `#`.
//!
//! Text and attribute values are HTML-escaped by pulldown's own
//! `push_html`, so ordinary special characters can't break out either.

use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};

/// Render untrusted markdown to sanitized HTML.
pub fn to_html(md: &str) -> String {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut events: Vec<Event> = Vec::new();

    for ev in Parser::new_ext(md, opts) {
        match ev {
            // Drop any raw HTML the user wrote. This is the primary XSS guard.
            Event::Html(_) | Event::InlineHtml(_) => {}
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => events.push(Event::Start(Tag::Link {
                link_type,
                dest_url: CowStr::from(sanitize_url(&dest_url)),
                title,
                id,
            })),
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => events.push(Event::Start(Tag::Image {
                link_type,
                dest_url: CowStr::from(sanitize_url(&dest_url)),
                title,
                id,
            })),
            other => events.push(other),
        }
    }

    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    out
}

/// Return `url` unchanged if it uses a safe scheme (or is relative),
/// otherwise `#`. Relative URLs (no scheme) and fragments are allowed.
fn sanitize_url(url: &str) -> String {
    let trimmed = url.trim();
    if let Some(colon) = trimmed.find(':') {
        let before = &trimmed[..colon];
        // A ':' after a '/', '?' or '#' belongs to the path/query/fragment of
        // a relative URL (e.g. "a/b:c"), not a scheme — leave those alone.
        let looks_like_scheme =
            !before.contains('/') && !before.contains('?') && !before.contains('#');
        if looks_like_scheme {
            // Strip control/whitespace chars an attacker might inject to hide
            // the scheme (e.g. "java\tscript:") before comparing.
            let scheme: String = before
                .chars()
                .filter(|c| !c.is_control() && !c.is_whitespace())
                .collect::<String>()
                .to_ascii_lowercase();
            if !matches!(scheme.as_str(), "http" | "https" | "mailto") {
                return "#".to_string();
            }
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_tags() {
        let out = to_html("hello\n\n<script>alert('xss')</script>\n\nworld");
        assert!(!out.contains("<script"), "script tag survived: {out}");
        assert!(!out.contains("alert("), "script body survived: {out}");
        assert!(out.contains("hello"));
        assert!(out.contains("world"));
    }

    #[test]
    fn strips_inline_event_handlers_and_raw_img() {
        let out = to_html("text <img src=x onerror=alert(1)> more");
        assert!(!out.contains("onerror"), "event handler survived: {out}");
        assert!(!out.contains("<img"), "raw img survived: {out}");
    }

    #[test]
    fn neutralizes_javascript_links() {
        let out = to_html("[click me](javascript:alert(1))");
        assert!(!out.contains("javascript:"), "js url survived: {out}");
        assert!(out.contains("href=\"#\""), "url not neutralized: {out}");
        // The visible link text is still rendered.
        assert!(out.contains("click me"));
    }

    #[test]
    fn neutralizes_data_url_images() {
        let out = to_html("![x](data:text/html;base64,PHNjcmlwdD4=)");
        assert!(!out.contains("data:"), "data url survived: {out}");
    }

    #[test]
    fn keeps_safe_links_and_formatting() {
        let out = to_html("A [link](https://example.com) and **bold** and `code`.");
        assert!(out.contains("href=\"https://example.com\""));
        assert!(out.contains("<strong>bold</strong>"));
        assert!(out.contains("<code>code</code>"));
    }

    #[test]
    fn keeps_relative_and_mailto_links() {
        let out = to_html("[rel](./page.html#frag) [mail](mailto:a@b.com)");
        assert!(out.contains("href=\"./page.html#frag\""));
        assert!(out.contains("href=\"mailto:a@b.com\""));
    }

    #[test]
    fn escapes_special_text() {
        let out = to_html("1 < 2 && 3 > 2");
        assert!(!out.contains("< 2 &&"), "unescaped special chars: {out}");
        assert!(out.contains("&lt;"));
        assert!(out.contains("&amp;"));
    }
}
