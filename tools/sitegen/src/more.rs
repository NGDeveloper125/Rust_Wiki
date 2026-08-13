//! The "More" section: a small hub plus the pages that sit under it.
//!
//! Right now that is LangColorMap, the reference for the site's syntax colour
//! system. The colour data lives here rather than in markdown because it is
//! structured — every slot has a name, a description, a sample and a colour per
//! theme — and because the same table will later drive the highlighter itself,
//! so there should only ever be one copy of it.

use std::io;
use std::path::Path;

use crate::model::Page;
use crate::nav::{render_sidebar, TopNav};
use crate::render::{abs_url, href_from, shell, Head};

/// `docs/more/*.html` — one directory below the site root.
const DEPTH: usize = 1;

/// One syntax role and the colour reserved for it.
///
/// `class` is the CSS slug. It is deliberately prefixed `cm-` on the page
/// rather than reusing the live highlighter's `tok-` classes: the palette is
/// still being tuned, and the map should be able to show a proposed colour
/// without repainting every code block on the site.
pub struct Slot {
    pub class: &'static str,
    pub label: &'static str,
    pub covers: &'static str,
    /// Pre-built HTML: the sample with its own token wrapped in `cm-<class>`.
    pub sample: &'static str,
    pub dark: &'static str,
    pub light: &'static str,
}

pub struct Group {
    pub title: &'static str,
    pub blurb: &'static str,
    pub slots: &'static [Slot],
}

/// Grouped by what the thing *is*, not by what colour it ended up.
///
/// Ordering by hue puts `field` beside `generic-param` because both happen to
/// be blue, which tells a reader nothing. Grouping by role puts the things a
/// reader is trying to tell apart next to each other, which is the whole point
/// of the system.
pub const GROUPS: &[Group] = &[
    Group {
        title: "Declarations",
        blurb: "Introducing a name is a different act from using it, so the declaration sites get their own colours. These are the anchors of a snippet — the places where something comes into existence.",
        slots: &[
            Slot {
                class: "type-def",
                label: "Type declaration",
                covers: "The name at <code>struct X</code>, <code>enum X</code>, <code>union X</code>, <code>trait X</code> or <code>type X =</code>.",
                sample: r#"<span class="cm-keyword">struct</span> <span class="cm-type-def">Counter</span>"#,
                dark: "#7BFF57",
                light: "#7BFF57",
            },
            Slot {
                class: "fn-def",
                label: "Function declaration",
                covers: "The name at <code>fn X(&hellip;)</code>, whether it is free, associated or a method.",
                sample: r#"<span class="cm-keyword">fn</span> <span class="cm-fn-def">record</span><span class="cm-punct">(&hellip;)</span>"#,
                dark: "#FF5C7A",
                light: "#FF5C7A",
            },
        ],
    },
    Group {
        title: "Types",
        blurb: "Everything that names a type rather than a value. Return position gets its own colour because what a function hands back is the thing most often looked up in a hurry.",
        slots: &[
            Slot {
                class: "type",
                label: "Type",
                covers: "A struct, enum, union, primitive or alias used as a type.",
                sample: r#"<span class="cm-type">HashMap</span><span class="cm-punct">&lt;</span><span class="cm-type">String</span><span class="cm-punct">,</span> <span class="cm-type">u32</span><span class="cm-punct">&gt;</span>"#,
                dark: "#1FFF9E",
                light: "#1FFF9E",
            },
            Slot {
                class: "trait",
                label: "Trait",
                covers: "A trait named in a bound, an <code>impl</code> block or behind <code>dyn</code>.",
                sample: r#"<span class="cm-keyword">impl</span> <span class="cm-trait">Summarize</span> <span class="cm-keyword">for</span> <span class="cm-type">Counter</span>"#,
                dark: "#019D59",
                light: "#019D59",
            },
            Slot {
                class: "generic-param",
                label: "Generic parameter",
                covers: "A type or const parameter declared in <code>&lt;&hellip;&gt;</code>.",
                sample: r#"<span class="cm-punct">&lt;</span><span class="cm-generic-param">T</span><span class="cm-punct">,</span> <span class="cm-keyword">const</span> <span class="cm-generic-param">N</span><span class="cm-punct">:</span> <span class="cm-type">usize</span><span class="cm-punct">&gt;</span>"#,
                dark: "#A9D6F5",
                light: "#A9D6F5",
            },
            Slot {
                class: "type-return",
                label: "Return type",
                covers: "Any type in return position, after <code>-&gt;</code>. It overrides the colour the same name would carry anywhere else.",
                sample: r#"<span class="cm-punct">-&gt;</span> <span class="cm-type-return">Option</span><span class="cm-punct">&lt;</span><span class="cm-type-return">usize</span><span class="cm-punct">&gt;</span>"#,
                dark: "#70FFE7",
                light: "#70FFE7",
            },
        ],
    },
    Group {
        title: "Values",
        blurb: "The names that hold data. Rust draws no line between a variable and an instance &mdash; both are bindings holding a value &mdash; so neither does this map.",
        slots: &[
            Slot {
                class: "variable",
                label: "Variable",
                covers: "Every binding, function parameter and closure parameter.",
                sample: r#"<span class="cm-keyword">let</span> <span class="cm-keyword">mut</span> <span class="cm-variable">counter</span>"#,
                dark: "#34E7EA",
                light: "#34E7EA",
            },
            Slot {
                class: "field",
                label: "Field",
                covers: "A struct field, a tuple index, and an enum variant &mdash; all of them name a member of a type.",
                sample: r#"<span class="cm-keyword">self</span><span class="cm-punct">.</span><span class="cm-field">counts</span><span class="cm-punct">,</span> <span class="cm-field">Some</span><span class="cm-punct">,</span> <span class="cm-field">None</span>"#,
                dark: "#7ED1FB",
                light: "#7ED1FB",
            },
            Slot {
                class: "constant",
                label: "Constant",
                covers: "A <code>const</code> or <code>static</code> name.",
                sample: r#"<span class="cm-keyword">const</span> <span class="cm-constant">MAX_ENTRIES</span>"#,
                dark: "#E5D9A8",
                light: "#E5D9A8",
            },
        ],
    },
    Group {
        title: "Calls",
        blurb: "Where the system earns its keep. A function reached through a type, through a value, or through nothing at all are three different things, and most highlighting paints them identically.",
        slots: &[
            Slot {
                class: "call-assoc",
                label: "Associated call",
                covers: "A function called on the type itself, through <code>::</code>.",
                sample: r#"<span class="cm-type">Counter</span><span class="cm-punct">::</span><span class="cm-call-assoc">new</span><span class="cm-punct">()</span>"#,
                dark: "#F5A55A",
                light: "#F5A55A",
            },
            Slot {
                class: "call-method",
                label: "Method call",
                covers: "A function called on a value, through <code>.</code>.",
                sample: r#"<span class="cm-variable">counter</span><span class="cm-punct">.</span><span class="cm-call-method">record</span><span class="cm-punct">()</span>"#,
                dark: "#FFEA00",
                light: "#FFEA00",
            },
            Slot {
                class: "call-free",
                label: "Free call",
                covers: "A function called with no receiver at all.",
                sample: r#"<span class="cm-call-free">tally</span><span class="cm-punct">(&amp;</span><span class="cm-variable">values</span><span class="cm-punct">)</span>"#,
                dark: "#D9A98A",
                light: "#D9A98A",
            },
            Slot {
                class: "macro",
                label: "Macro",
                covers: "A macro invocation. The trailing <code>!</code> already marks it, so the colour is free to be quiet.",
                sample: r#"<span class="cm-macro">println!</span><span class="cm-punct">(</span><span class="cm-string">"{}"</span><span class="cm-punct">)</span>"#,
                dark: "#D972EE",
                light: "#D972EE",
            },
        ],
    },
    Group {
        title: "Literals",
        blurb: "Values written directly into the source rather than named.",
        slots: &[
            Slot {
                class: "string",
                label: "String",
                covers: "String, char, byte and raw literals.",
                sample: r#"<span class="cm-string">"words"</span><span class="cm-punct">,</span> <span class="cm-string">'&middot;'</span>"#,
                dark: "#86D7BF",
                light: "#86D7BF",
            },
            Slot {
                class: "number",
                label: "Number",
                covers: "Integer, float and bool literals.",
                sample: r#"<span class="cm-number">128</span><span class="cm-punct">,</span> <span class="cm-number">0.5</span><span class="cm-punct">,</span> <span class="cm-number">true</span>"#,
                dark: "#A17297",
                light: "#A17297",
            },
            Slot {
                class: "lifetime",
                label: "Lifetime",
                covers: "A lifetime, and a loop label, which is written the same way.",
                sample: r#"<span class="cm-punct">&amp;</span><span class="cm-lifetime">'a</span> <span class="cm-type">str</span><span class="cm-punct">,</span> <span class="cm-lifetime">'outer</span><span class="cm-punct">:</span>"#,
                dark: "#F2DB69",
                light: "#F2DB69",
            },
        ],
    },
    Group {
        title: "Structure",
        blurb: "The scaffolding a snippet is written on. These carry one colour each and are meant to recede, so the names above them stand out.",
        slots: &[
            Slot {
                class: "keyword",
                label: "Keyword",
                covers: "Every language keyword, from <code>let</code> to <code>unsafe</code>. Splitting them apart adds colours without adding meaning.",
                sample: r#"<span class="cm-keyword">let</span> <span class="cm-keyword">mut</span><span class="cm-punct">,</span> <span class="cm-keyword">impl</span><span class="cm-punct">,</span> <span class="cm-keyword">unsafe</span>"#,
                dark: "#FFB700",
                light: "#FFB700",
            },
            Slot {
                class: "module",
                label: "Module",
                covers: "A crate or module segment inside a path.",
                sample: r#"<span class="cm-module">std</span><span class="cm-punct">::</span><span class="cm-module">collections</span><span class="cm-punct">::</span><span class="cm-type">HashMap</span>"#,
                dark: "#8FA8D8",
                light: "#8FA8D8",
            },
            Slot {
                class: "attribute",
                label: "Attribute",
                covers: "An outer or inner attribute, in full.",
                sample: r#"<span class="cm-attribute">#[derive(Debug)]</span>"#,
                dark: "#944695",
                light: "#944695",
            },
            Slot {
                class: "comment",
                label: "Comment",
                covers: "Every comment form, including doc comments.",
                sample: r#"<span class="cm-comment">/// Counts words, keyed by label.</span>"#,
                dark: "#FFFFFF",
                light: "#FFFFFF",
            },
            Slot {
                class: "punct",
                label: "Punctuation",
                covers: "Operators, delimiters and separators.",
                sample: r#"<span class="cm-punct">? &nbsp; -&gt; &nbsp; :: &nbsp; &amp;&amp;</span>"#,
                dark: "#A9C4C2",
                light: "#A9C4C2",
            },
        ],
    },
];

/// Every slot, flattened, in group order.
pub fn all_slots() -> impl Iterator<Item = &'static Slot> {
    GROUPS.iter().flat_map(|g| g.slots.iter())
}

/// The `--cm-*` custom properties for both themes, emitted into the page so the
/// palette can move without a stylesheet edit while it is still being tuned.
fn palette_style() -> String {
    let vars = |pick: fn(&Slot) -> &'static str| {
        all_slots()
            .map(|s| format!("--cm-{}:{};", s.class, pick(s)))
            .collect::<Vec<_>>()
            .join("")
    };
    format!(
        "<style>\n:root{{{dark}}}\n:root[data-theme=\"light\"]{{{light}}}\n</style>",
        dark = vars(|s| s.dark),
        light = vars(|s| s.light),
    )
}

fn render_slot(s: &Slot) -> String {
    format!(
        r#"<div class="cmap-row">
            <div class="cmap-id">
              <span class="cmap-chip" style="background:var(--cm-{class})"></span>
              <span class="cmap-label cm-{class}">{label}</span>
            </div>
            <code class="cmap-sample">{sample}</code>
            <p class="cmap-covers">{covers}</p>
            <code class="cmap-hex"><span class="cmap-hex-dark">{dark}</span><span class="cmap-hex-light">{light}</span></code>
          </div>"#,
        class = s.class,
        label = s.label,
        sample = s.sample,
        covers = s.covers,
        dark = s.dark,
        light = s.light,
    )
}

fn render_group(g: &Group) -> String {
    let rows: String = g
        .slots
        .iter()
        .map(render_slot)
        .collect::<Vec<_>>()
        .join("\n          ");
    format!(
        r#"<section class="cmap-group">
          <h2 class="cmap-group-title">{title}</h2>
          <p class="cmap-group-blurb">{blurb}</p>
          <div class="cmap-rows">
          {rows}
          </div>
        </section>"#,
        title = g.title,
        blurb = g.blurb,
    )
}

fn render_colormap(pages: &[Page]) -> String {
    let sidebar = render_sidebar(pages, None, DEPTH, TopNav::More);
    let home = href_from(DEPTH, "");
    let more = href_from(DEPTH, "more/");

    let groups: String = GROUPS
        .iter()
        .map(render_group)
        .collect::<Vec<_>>()
        .join("\n\n        ");

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

      <p class="lead">Most syntax highlighting colours by token class &mdash; keywords one colour, identifiers another, and everything that looks like a name painted the same. Rust carries more than that. Reaching a function through a type is a different act from reaching one through a value, and declaring a struct is not the same as naming it as a type. This map gives each of those roles a colour of its own, used for that role and nothing else, so that a snippet tells you what every name <em>is</em> before you have read what it says.</p>

      <p class="cmap-note">Grouped by what the thing is rather than by how the colours sort, so the roles you are trying to tell apart sit next to each other. The palette is still being tuned and this page moves with it.</p>

      <hr class="divider">

      {palette}

        {groups}

      <div class="footer-note">
        <span>Rusty Yellow Pages &middot; a free, open-source Rust reference</span>
        <span>One colour per role, and that role only</span>
      </div>
"#,
        palette = palette_style(),
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
    shell(&head, DEPTH, &sidebar, &main)
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
        for s in all_slots() {
            assert!(seen.insert(s.class), "duplicate slot class: {}", s.class);
        }
    }

    #[test]
    fn every_slot_samples_its_own_colour() {
        // A row whose sample never uses its own class would show the reader a
        // colour swatch next to code painted in someone else's colour.
        for s in all_slots() {
            let marker = format!("cm-{}\"", s.class);
            assert!(
                s.sample.contains(&marker),
                "slot `{}` has no token of its own in its sample",
                s.class
            );
        }
    }

    #[test]
    fn every_colour_is_a_full_hex() {
        for s in all_slots() {
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