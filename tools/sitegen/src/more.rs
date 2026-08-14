//! The "More" section: a small hub plus the pages that sit under it.
//!
//! Right now that is LangColorMap, the reference for the site's syntax colour
//! system. The colour table lives here rather than in markdown because it is
//! structured data, and because the same table will later drive the highlighter
//! itself — there should only ever be one copy of it.

use std::io;
use std::path::Path;

use crate::highlight::rust_to_html;
use crate::model::Page;
use crate::palette::{Slot, SLOTS};
use crate::nav::{render_sidebar, TopNav};
use crate::render::{abs_url, href_from, shell, shell_with_page_class, Head};

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

fn render_colormap(pages: &[Page]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::More);
    let home = href_from(DEPTH, "");
    let more = href_from(DEPTH, "more/");

    let main = format!(
        r#"      <nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="{home}">Home</a><span class="sep">&rsaquo;</span>
        <a href="{more}">More</a><span class="sep">&rsaquo;</span>
        <span style="color:var(--content-fg);font-weight:600">LangColorMap</span>
      </nav>

      <div class="page-head">
        <div class="title-block">
          <h1 class="page-title">LangColorMap</h1>
        </div>
      </div>

      <p class="lead">This is an index I went looking for and could not find. The goal is a different colour for every kind of syntax, so that code can be understood much more quickly without reading all of it &mdash; you see a certain colour and you already know what something is, before you have actually read the word. It looks bad right now, I know. I am still tuning it, and I want to open it up so other people can have a go at setting the colours better too.</p>

      <p class="cmap-note">Ordered by hue, so neighbouring entries are neighbouring colours. Hover a row for what it covers.</p>

      <hr class="divider">

      {palette}

      <div class="cmap-bar">
        <span class="cmap-bar-label">Preview on</span>
        <div class="segmented" id="cmap-theme" role="group" aria-label="Preview background">
          <button type="button" data-cm="dark" aria-pressed="true">Dark</button>
          <button type="button" data-cm="light" aria-pressed="false">Light</button>
        </div>
      </div>

      <div class="cmap-preview" data-cm-theme="dark">
        {list}

        <h2 class="cmap-specimen-title">Every role, one snippet</h2>
        <p class="cmap-note">Each colour above appears at least once below.</p>

        <pre class="cmap-code"><code>{specimen}</code></pre>
      </div>

      {toggle_js}

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>One colour per role, and that role only</span>
      </div>
"#,
        palette = palette_style(),
        list = render_slot_list(),
        specimen = rust_to_html(SPECIMEN_SRC),
        toggle_js = PREVIEW_TOGGLE_JS,
    );

    let head = Head {
        title: "Rust - LangColorMap - Rusty Yellow Pages".to_string(),
        description:
            "A colour for every distinguishable role in Rust syntax — declarations, types, values, calls, literals and structure — each used for that role and nothing else."
                .to_string(),
        canonical: abs_url("more/langcolormap.html"),
        og_type: "website",
        image: None,
    };
    // Wider than the site's prose measure: two columns of swatches inside 84ch
    // squeeze both halves until the longer slot names truncate. The prose on
    // the page keeps its own measure — see `.page-colormap .lead`.
    shell_with_page_class(&head, DEPTH, &sidebar, &main, "page-colormap")
}

fn render_hub(pages: &[Page]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::More);
    let home = href_from(DEPTH, "");
    let colormap = href_from(DEPTH, "more/langcolormap.html");

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
        canonical: abs_url("more/"),
        og_type: "website",
        image: None,
    };
    shell(&head, DEPTH, &sidebar, &main)
}

fn write_pages(docs_root: &Path, pages: &[Page]) -> io::Result<()> {
    let dir = docs_root.join("more");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("index.html"), render_hub(pages))?;
    std::fs::write(dir.join("langcolormap.html"), render_colormap(pages))?;
    Ok(())
}

/// Write the More hub and the pages under it.
pub fn build(docs_root: &Path, pages: &[Page]) {
    if let Err(e) = write_pages(docs_root, pages) {
        eprintln!("more: could not write pages: {e}");
    } else {
        println!("more: rendered hub + LangColorMap");
    }
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
