use pulldown_cmark::{html, Options, Parser};

/// Render a markdown chunk to HTML. All code fences in this repo are plain
/// (untagged) and always Rust, so every `<pre><code>` pulldown-cmark emits
/// gets tagged `class="rust"` for the shared client-side highlighter.
pub fn to_html(md: &str) -> String {
    let parser = Parser::new_ext(md, Options::empty());
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out.replace("<pre><code>", "<pre><code class=\"rust\">")
}

/// Split a body on top-level `## Heading` lines, preserving heading order.
/// Returns (heading text, raw markdown body under that heading).
pub fn split_h2(body: &str) -> Vec<(String, String)> {
    split_on_prefix(body, "## ")
}

/// Split a "Best practices" section body into a leading intro (markdown
/// before the first `### Scenario: ...`) and the scenario blocks themselves.
pub fn split_scenarios(body: &str) -> (String, Vec<(String, String)>) {
    let marker = "### Scenario: ";
    match body.find(&format!("\n{marker}")).or_else(|| {
        if body.starts_with(marker) {
            Some(0)
        } else {
            None
        }
    }) {
        None => (body.trim().to_string(), Vec::new()),
        Some(idx) => {
            let split_at = if body.starts_with(marker) { 0 } else { idx + 1 };
            let intro = body[..split_at].trim().to_string();
            let rest = &body[split_at..];
            (intro, split_on_prefix(rest, marker))
        }
    }
}

/// Split a syntax page's "Usage examples" section body on top-level
/// `### <title>` lines. Returns (title, raw markdown body under that title).
pub fn split_examples(body: &str) -> Vec<(String, String)> {
    split_on_prefix(body, "### ")
}

fn split_on_prefix(body: &str, prefix: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_body = String::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix(prefix) {
            if let Some(title) = current_title.take() {
                out.push((title, current_body.trim().to_string()));
            }
            current_title = Some(rest.trim().to_string());
            current_body = String::new();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    if let Some(title) = current_title {
        out.push((title, current_body.trim().to_string()));
    }
    out
}

/// Split markdown into blank-line-separated blocks, without splitting
/// inside a fenced code block (lines delimited by a bare ``` line).
fn split_blocks(md: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut in_fence = false;
    for line in md.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        if line.trim().is_empty() && !in_fence {
            if !current.trim().is_empty() {
                blocks.push(current.trim_end().to_string());
            }
            current.clear();
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    if !current.trim().is_empty() {
        blocks.push(current.trim_end().to_string());
    }
    blocks
}

/// Split a scenario's markdown body into (main body markdown, optional
/// "**Why this way:** ..." rationale markdown), per SECTION3_GUIDE.md's
/// fixed format (the rationale is always the scenario's last block).
pub fn split_rationale(scenario_md: &str) -> (String, Option<String>) {
    let mut blocks = split_blocks(scenario_md);
    if let Some(last) = blocks.last() {
        if last.trim_start().starts_with("**Why this way:**") {
            let rationale = blocks.pop().unwrap();
            return (blocks.join("\n\n"), Some(rationale));
        }
    }
    (scenario_md.to_string(), None)
}
