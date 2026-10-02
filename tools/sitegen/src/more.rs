//! The "More" section: a small hub plus the pages that sit under it.
//!
//! Right now that is LangColorMap, the reference for the site's syntax colour
//! system. The colour table lives here rather than in markdown because it is
//! structured data, and because the same table will later drive the highlighter
//! itself — there should only ever be one copy of it.

use std::io;
use std::path::Path;

use crate::highlight::rust_to_html;
use crate::markdown;
use crate::model::Page;
use crate::palette::{Slot, SLOTS};
use crate::crates::domain::Domain;
use crate::nav::{render_sidebar, TopNav};
use crate::render::{abs_url, href_from, shell, shell_with_page_class, Head, REPO_URL};

/// Site-root-relative URLs of the pages this module writes, in the order
/// [`write_pages`] writes them. The canonical links and the sitemap are both
/// built from this list, so a page added here cannot end up in one and not
/// the other.
const URLS: [&str; 2] = ["more/", "more/langcolormap.html"];

/// Where the templates themselves live. A repository of their own, because
/// `cargo generate --git` clones the whole thing: delivering a couple of
/// kilobytes of template out of the site repository would mean cloning tens of
/// megabytes of generated HTML to do it.
const TEMPLATES_REPO: &str = "https://github.com/NGDeveloper125/rusty-yellow-pages-templates";

/// The Templates directory and the pages under it. They sit one level deeper
/// than the rest of More, so a template's own page can be reached at
/// `more/templates/<name>.html` while the directory keeps a clean URL.
const TEMPLATES_URL: &str = "more/templates/";
const TEMPLATES_DEPTH: usize = 2;

/// `docs/more/*.html` — one directory below the site root.
const DEPTH: usize = 1;

/// The snippet under the list, as Rust source. It is painted by the same
/// highlighter that paints every other code block on the site, so the map
/// cannot show colours the rest of the site does not actually use.
const SPECIMEN_SRC: &str = include_str!("../templates/langcolormap-specimen.rs");

/// The panel a preview is painted on, per theme. The colours only mean anything
/// against the background they were chosen for, so the preview carries its own
/// panel rather than borrowing whichever one the site happens to be showing.
struct Panel {
    bg: &'static str,
    border: &'static str,
    muted: &'static str,
}

const DARK_PANEL: Panel = Panel { bg: "#0a2c2e", border: "#17494b", muted: "#8FB3B0" };
const LIGHT_PANEL: Panel = Panel { bg: "#e9f1ef", border: "#cfe0dc", muted: "#4a6b69" };

/// The `--cm-*` custom properties for both themes.
///
/// Scoped to `.cmap-preview[data-cm-theme=…]` rather than to `:root`, so the
/// in-page toggle can show either palette without touching the site's own
/// theme — a reader can compare the two without leaving the page they are on.
fn palette_style() -> String {
    let block = |pick: fn(&Slot) -> &'static str, panel: &Panel| {
        let slots = SLOTS
            .iter()
            .map(|s| format!("--t-{}:{};", s.class, pick(s)))
            .collect::<Vec<_>>()
            .join("");
        format!(
            "{slots}--cm-panel-bg:{bg};--cm-panel-border:{border};--cm-muted:{muted};",
            bg = panel.bg,
            border = panel.border,
            muted = panel.muted,
        )
    };
    format!(
        "<style>\n.cmap-preview[data-cm-theme=\"dark\"]{{{dark}}}\n.cmap-preview[data-cm-theme=\"light\"]{{{light}}}\n</style>",
        dark = block(|s| s.dark, &DARK_PANEL),
        light = block(|s| s.light, &LIGHT_PANEL),
    )
}

/// Starts on whatever the site is showing, then the two run independently —
/// flipping the preview to compare palettes should not repaint the whole site.
const PREVIEW_TOGGLE_JS: &str = r#"<script>
(function () {
  var wrap = document.querySelector('.cmap-preview');
  var seg = document.getElementById('cmap-theme');
  if (!wrap || !seg) return;
  function set(mode) {
    wrap.dataset.cmTheme = mode;
    seg.querySelectorAll('button').forEach(function (b) {
      var on = b.dataset.cm === mode;
      b.classList.toggle('on', on);
      b.setAttribute('aria-pressed', on ? 'true' : 'false');
    });
  }
  set(document.documentElement.getAttribute('data-theme') === 'light' ? 'light' : 'dark');
  seg.addEventListener('click', function (e) {
    var b = e.target.closest('button');
    if (b) set(b.dataset.cm);
  });
})();
</script>"#;

fn render_slot_list() -> String {
    let rows: String = SLOTS
        .iter()
        .map(|s| {
            format!(
                r#"<div class="cmap-row" title="{covers}">
            <span class="cmap-chip" style="background:var(--t-{class})"></span>
            <span class="cmap-name tok-{class}">{class}</span>
            <span class="cmap-hex"><span class="cmap-hex-dark">{dark}</span><span class="cmap-hex-light">{light}</span></span>
          </div>"#,
                class = s.class,
                covers = s.covers,
                dark = s.dark,
                light = s.light,
            )
        })
        .collect::<Vec<_>>()
        .join("\n          ");

    // Column-major fill needs an explicit row count to know where to wrap.
    // Rounding up leaves the odd gap at the foot of the second column rather
    // than splitting the list unevenly.
    let rows_per_column = SLOTS.len().div_ceil(2);
    format!(
        "<div class=\"cmap-list\" style=\"grid-template-rows: repeat({rows_per_column}, auto)\">\n          {rows}\n        </div>"
    )
}

fn render_colormap(pages: &[Page], domains: &[&'static Domain]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::More, domains);
    let home = href_from(DEPTH, "");
    let more = href_from(DEPTH, "more/");

    let main = format!(
        r##"      <nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <a href="{more}">More</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">LangColorMap</span>
      </nav>

      <div class="page-head">
        <div class="title-block">
          <h1 class="page-title">LangColorMap</h1>
        </div>
      </div>

      <div class="related cmap-nav">
        <div class="related-row">
          <span class="related-label">On this page</span>
          <a class="chip" href="#palette">The palette</a>
          <a class="chip" href="#snippet">Every role, one snippet</a>
          <a class="chip" href="#vscode">Use it in VS Code</a>
        </div>
      </div>

      <p class="lead">This is an index I went looking for and could not find. The goal is a different colour for every kind of syntax, so that code can be understood much more quickly without reading all of it &mdash; you see a certain colour and you already know what something is, before you have actually read the word. It looks bad right now, I know. I am still tuning it, and I want to open it up so other people can have a go at setting the colours better too.</p>

      <p class="cmap-note">Ordered by hue, so neighbouring entries are neighbouring colours. Hover a row for what it covers.</p>

      <hr class="divider">

      {palette}

      <div class="cmap-bar" id="palette">
        <span class="cmap-bar-label">Preview on</span>
        <div class="segmented" id="cmap-theme" role="group" aria-label="Preview background">
          <button type="button" data-cm="dark" aria-pressed="true">Dark</button>
          <button type="button" data-cm="light" aria-pressed="false">Light</button>
        </div>
      </div>

      <div class="cmap-preview" data-cm-theme="dark">
        {list}

        <h2 class="cmap-specimen-title" id="snippet">Every role, one snippet</h2>
        <p class="cmap-note">Each colour above appears at least once below.</p>

        <pre class="cmap-code"><code>{specimen}</code></pre>
      </div>

      {toggle_js}

      <h2 class="cmap-specimen-title" id="vscode">Use it in VS Code</h2>
      <p class="cmap-note">The same table generates a VS Code theme, so an editor can paint code the way this page does. It needs <a href="https://rust-analyzer.github.io/" target="_blank" rel="noopener">rust-analyzer</a>: the role colours ride on its semantic tokens, which is how the editor tells <code>Type::new()</code> from <code>value.method()</code>. Without it you get comments, literals and keywords and nothing more.</p>

      <ol class="cmap-install">
        <li>Download the <a href="{repo}/tree/main/vscode-theme" target="_blank" rel="noopener"><code>vscode-theme</code></a> folder from the repository.</li>
        <li>Copy it into your extensions directory: <code>%USERPROFILE%\.vscode\extensions\</code> on Windows, <code>~/.vscode/extensions/</code> on macOS and Linux.</li>
        <li>Reload the window &mdash; <code>Ctrl+Shift+P</code>, then <em>Developer: Reload Window</em>. VS Code only reads that folder at startup, so it will not appear before you do.</li>
        <li>Choose it with <code>Ctrl+K Ctrl+T</code>: <strong>Rusty Yellow Pages Dark</strong> or <strong>Light</strong>.</li>
      </ol>

      <p class="cmap-note">For an installable package instead, run <code>npx @vscode/vsce package</code> inside that folder and install the <code>.vsix</code> it produces through <em>Extensions: Install from VSIX&hellip;</em>. Give rust-analyzer a moment to finish indexing after you switch &mdash; the role colours only arrive once it has.</p>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>One colour per role, and that role only</span>
      </div>
"##,
        palette = palette_style(),
        list = render_slot_list(),
        specimen = rust_to_html(SPECIMEN_SRC),
        toggle_js = PREVIEW_TOGGLE_JS,
        repo = REPO_URL,
    );

    let head = Head {
        title: "Rust - LangColorMap - Rusty Yellow Pages".to_string(),
        description:
            "A colour for every distinguishable role in Rust syntax — declarations, types, values, calls, literals and structure — each used for that role and nothing else."
                .to_string(),
        canonical: abs_url(URLS[1]),
        og_type: "website",
        image: None,
    };
    // Wider than the site's prose measure: two columns of swatches inside 84ch
    // squeeze both halves until the longer slot names truncate. The prose on
    // the page keeps its own measure — see `.page-colormap .lead`.
    shell_with_page_class(&head, DEPTH, &sidebar, &main, "page-colormap")
}

/// One project template: the card on the Templates directory, and the page
/// that card links to.
///
/// This is structured data rather than prose so that adding a template means
/// adding an entry — the card, the heading, the list of crates it produces and
/// the command form are generated from one place and cannot drift apart.
/// Anything that needs explaining in sentences goes in that template's own
/// markdown body instead.
struct TemplateEntry {
    /// Directory under `templates/` in the repository, the name the template
    /// is referred to by on the command line, and its page's file name.
    name: &'static str,
    /// One line. Used on the card, and again as the lead of its own page.
    summary: &'static str,
    /// The crates it generates: (crate name, what that crate is for). The name
    /// is written as the reader sees it, with `<name>` standing in for
    /// whatever they pass to `--name`.
    produces: &'static [(&'static str, &'static str)],
    /// Worked invocations: (what this one shows, the arguments following the
    /// `--git ... <subfolder>` prefix that every template shares).
    examples: &'static [(&'static str, &'static str)],
}

const TEMPLATES: [TemplateEntry; 1] = [TemplateEntry {
    name: "cli_lib",
    summary: "A workspace with a console crate and a library crate, the path dependency and the call between them already written.",
    produces: &[
        (
            "&lt;name&gt;_cli",
            "a binary crate that depends on the library by path and calls its example method",
        ),
        (
            "&lt;name&gt;_lib",
            "a library crate holding one example method and the unit test for it",
        ),
    ],
    examples: &[
        (
            "Crate names default to the project name",
            "--name inventory_tool",
        ),
        (
            "Or name either crate yourself",
            "--name shop -d cli_name=shop_console -d lib_name=shop_engine",
        ),
    ],
}];

/// A template's page URL, site-root-relative.
fn template_url(name: &str) -> String {
    format!("{TEMPLATES_URL}{name}.html")
}

/// Render one of the hand-edited markdown bodies under `pages/more/`.
///
/// The prose is markdown rather than string literals because it is prose: the
/// maintainer should be able to reword the case for templates, or what a
/// particular one is for, without touching the generator. Everything
/// structured about a template stays in [`TemplateEntry`].
fn load_body(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(md) => markdown::to_html(markdown::strip_leading_comment(&md).trim()),
        Err(_) => String::new(),
    }
}

/// The `cargo generate` line for a template, followed by one worked example's
/// arguments.
fn generate_command(name: &str, tail: &str) -> String {
    format!("cargo generate --git {TEMPLATES_REPO} {name} {tail}")
}

/// Where to read a template before generating from it. Derived rather than
/// stored on [`TemplateEntry`], so it cannot name a directory that the command
/// above does not.
fn template_source_url(name: &str) -> String {
    format!("{TEMPLATES_REPO}/tree/main/{name}")
}

/// The shared "install it first" block. Both the directory and every template
/// page carry it: a reader who lands on a template page from a search has not
/// passed through the directory.
const INSTALL_BLOCK: &str = r#"      <h2 class="tmpl-h2" id="install">Installing cargo-generate</h2>
      <p class="tmpl-note">cargo-generate is a separate subcommand, installed once.</p>
      <pre class="tmpl-cmd"><code>cargo install cargo-generate</code></pre>
"#;

fn render_template_cards() -> String {
    let cards = TEMPLATES
        .iter()
        .map(|t| {
            format!(
                "        <a class=\"card more-card\" href=\"{href}\">\n          <h3>{name}</h3>\n          <p>{summary}</p>\n        </a>\n",
                href = href_from(TEMPLATES_DEPTH, &template_url(t.name)),
                name = t.name,
                summary = t.summary,
            )
        })
        .collect::<String>();
    format!("      <div class=\"more-grid tmpl-grid\">\n{cards}      </div>\n")
}

/// `more/templates/` — the idea, then a card per template.
fn render_templates_index(pages_root: &Path, pages: &[Page], domains: &[&'static Domain]) -> String {
    let sidebar = render_sidebar(pages, None, TEMPLATES_DEPTH, TopNav::More, domains);
    let home = href_from(TEMPLATES_DEPTH, "");
    let more = href_from(TEMPLATES_DEPTH, URLS[0]);
    let intro = load_body(&pages_root.join("more").join("_templates.md"));

    let main = format!(
        r##"      <nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <a href="{more}">More</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">Templates</span>
      </nav>

      <div class="page-head">
        <div class="title-block">
          <h1 class="page-title">Templates</h1>
        </div>
      </div>

      <p class="lead">Generated Rust project layouts: the crates, modules and configuration <code>cargo new</code> does not create.</p>

      <hr class="divider">

      <div class="tmpl-intro">
{intro}      </div>

      <hr class="divider">

{install}
      <h2 class="tmpl-h2" id="list">The templates</h2>
      <p class="tmpl-note">One page each: what it generates, and the command.</p>

{cards}
      <h2 class="tmpl-h2" id="shorthand">A shorter command</h2>
      <p class="tmpl-note">A favourite in <code>$CARGO_HOME/cargo-generate.toml</code> maps a name to the repository and subfolder, replacing the URL on the command line.</p>
      <pre class="tmpl-cmd"><code>[favorites.{first}]
git = "{repo}"
subfolder = "{first}"</code></pre>
      <p class="tmpl-caption">After that</p>
      <pre class="tmpl-cmd"><code>cargo generate {first} --name inventory_tool</code></pre>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
      </div>
"##,
        install = INSTALL_BLOCK,
        cards = render_template_cards(),
        first = TEMPLATES[0].name,
        repo = TEMPLATES_REPO,
    );

    let head = Head {
        title: "Rust - Templates - Rusty Yellow Pages".to_string(),
        description:
            "Generated Rust project layouts: the crates, modules and configuration cargo new does not create, produced with cargo-generate."
                .to_string(),
        canonical: abs_url(TEMPLATES_URL),
        og_type: "website",
        image: None,
    };
    shell(&head, TEMPLATES_DEPTH, &sidebar, &main)
}

/// `more/templates/<name>.html` — one template.
fn render_template_page(
    t: &TemplateEntry,
    pages_root: &Path,
    pages: &[Page],
    domains: &[&'static Domain],
) -> String {
    let sidebar = render_sidebar(pages, None, TEMPLATES_DEPTH, TopNav::More, domains);
    let home = href_from(TEMPLATES_DEPTH, "");
    let more = href_from(TEMPLATES_DEPTH, URLS[0]);
    let index = href_from(TEMPLATES_DEPTH, TEMPLATES_URL);
    let body = load_body(
        &pages_root
            .join("more")
            .join("templates")
            .join(format!("{}.md", t.name)),
    );

    let produces = t
        .produces
        .iter()
        .map(|(name, what)| format!("        <li><code>{name}</code> &mdash; {what}</li>\n"))
        .collect::<String>();
    let examples = t
        .examples
        .iter()
        .map(|(caption, tail)| {
            format!(
                "      <p class=\"tmpl-caption\">{caption}</p>\n      <pre class=\"tmpl-cmd\"><code>{cmd}</code></pre>\n",
                cmd = generate_command(t.name, tail),
            )
        })
        .collect::<String>();

    let main = format!(
        r##"      <nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <a href="{more}">More</a><span class="sep">&rsaquo;</span>
        <a href="{index}">Templates</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">{name}</span>
      </nav>

      <div class="page-head">
        <div class="title-block">
          <h1 class="page-title tmpl-page-title">{name}</h1>
        </div>
      </div>

      <p class="lead">{summary}</p>

      <hr class="divider">

      <h2 class="tmpl-h2" id="generates">What it generates</h2>
      <ul class="tmpl-produces">
{produces}      </ul>
      <p class="tmpl-note">Source: <a href="{source}"><code>{name}</code></a> in the templates repository.</p>

      <div class="tmpl-intro">
{body}      </div>

      <hr class="divider">

      <h2 class="tmpl-h2" id="generate">Generating it</h2>
{examples}
{install}
      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span><a href="{index}">All templates</a></span>
      </div>
"##,
        name = t.name,
        summary = t.summary,
        source = template_source_url(t.name),
        install = INSTALL_BLOCK,
    );

    let head = Head {
        title: format!("Rust - {} template - Rusty Yellow Pages", t.name),
        description: t.summary.to_string(),
        canonical: abs_url(&template_url(t.name)),
        og_type: "website",
        image: None,
    };
    shell(&head, TEMPLATES_DEPTH, &sidebar, &main)
}

fn render_hub(pages: &[Page], domains: &[&'static Domain]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::More, domains);
    let home = href_from(DEPTH, "");
    let colormap = href_from(DEPTH, URLS[1]);
    let templates = href_from(DEPTH, TEMPLATES_URL);

    let main = format!(
        r#"      <nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">More</span>
      </nav>

      <div class="page-head">
        <div class="title-block">
          <h1 class="page-title">More</h1>
        </div>
      </div>

      <p class="lead">Reference material about the site itself, and about how Rust is presented here.</p>

      <hr class="divider">

      <div class="more-grid">
        <a class="card more-card" href="{colormap}">
          <h3>LangColorMap</h3>
          <p>The colour assigned to each role in Rust syntax, and what each one covers.</p>
        </a>
        <a class="card more-card" href="{templates}">
          <h3>Templates</h3>
          <p>Generated Rust project layouts, beyond the single package cargo new creates.</p>
        </a>
      </div>

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
      </div>
"#
    );

    let head = Head {
        title: "Rust - More - Rusty Yellow Pages".to_string(),
        description: "Reference material about Rusty Yellow Pages and how Rust is presented here."
            .to_string(),
        canonical: abs_url(URLS[0]),
        og_type: "website",
        image: None,
    };
    shell(&head, DEPTH, &sidebar, &main)
}

fn write_pages(
    docs_root: &Path,
    pages_root: &Path,
    pages: &[Page],
    domains: &[&'static Domain],
) -> io::Result<()> {
    let dir = docs_root.join("more");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("index.html"), render_hub(pages, domains))?;
    std::fs::write(dir.join("langcolormap.html"), render_colormap(pages, domains))?;
    let tdir = dir.join("templates");
    std::fs::create_dir_all(&tdir)?;
    std::fs::write(
        tdir.join("index.html"),
        render_templates_index(pages_root, pages, domains),
    )?;
    for t in &TEMPLATES {
        std::fs::write(
            tdir.join(format!("{}.html", t.name)),
            render_template_page(t, pages_root, pages, domains),
        )?;
    }
    Ok(())
}

/// Write the More hub and the pages under it.
///
/// Returns the site-root-relative paths it wrote, for inclusion in the
/// sitemap; empty if writing failed, matching how `conversations::build`
/// reports the same thing.
pub fn build(
    docs_root: &Path,
    pages_root: &Path,
    pages: &[Page],
    domains: &[&'static Domain],
) -> Vec<String> {
    if let Err(e) = write_pages(docs_root, pages_root, pages, domains) {
        eprintln!("more: could not write pages: {e}");
        return Vec::new();
    }
    println!(
        "more: rendered hub + LangColorMap + Templates ({} template page(s))",
        TEMPLATES.len()
    );
    let mut urls: Vec<String> = URLS.iter().map(|u| u.to_string()).collect();
    urls.push(TEMPLATES_URL.to_string());
    urls.extend(TEMPLATES.iter().map(|t| template_url(t.name)));
    urls
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_slot_has_a_unique_class() {
        let mut seen = HashSet::new();
        for s in SLOTS {
            assert!(seen.insert(s.class), "duplicate slot class: {}", s.class);
        }
    }

    #[test]
    fn the_specimen_exercises_every_slot() {
        // The page claims every colour appears at least once below the list.
        // Because the snippet is painted by the real highlighter, this also
        // proves the highlighter can actually reach every role it has a colour
        // for — a slot no rule ever assigns would fail here.
        let painted = rust_to_html(SPECIMEN_SRC);
        for s in SLOTS {
            let marker = format!("class=\"tok-{}\"", s.class);
            let with_extra = format!("class=\"tok-{} ", s.class);
            assert!(
                painted.contains(&marker) || painted.contains(&with_extra),
                "slot `{}` never appears in the painted specimen",
                s.class
            );
        }
    }

    #[test]
    fn the_preview_overrides_every_slot() {
        // The preview repaints its subtree by redefining the site's slot
        // variables. One left out would silently fall through to the site
        // theme, so the toggle would lie about that colour.
        let css = palette_style();
        for s in SLOTS {
            let decl = format!("--t-{}:", s.class);
            assert_eq!(
                css.matches(&decl).count(),
                2,
                "slot `{}` is not overridden in both preview themes",
                s.class
            );
        }
    }

    #[test]
    fn every_colour_is_a_full_hex() {
        for s in SLOTS {
            for hex in [s.dark, s.light] {
                assert!(
                    hex.len() == 7
                        && hex.starts_with('#')
                        && hex[1..].chars().all(|c| c.is_ascii_hexdigit()),
                    "slot `{}` has a malformed colour: {hex}",
                    s.class
                );
            }
        }
    }
}
