//! Generate a VS Code theme extension from the palette.
//!
//! The point of generating rather than hand-writing it is that the palette
//! already drives the site and the LangColorMap page; a hand-maintained theme
//! would be a third copy of twenty-one colours and would go stale the first
//! time one of them moved.
//!
//! The role colours are carried by `semanticTokenColors`, not `tokenColors`.
//! TextMate scopes are regex-based and cannot tell `Type::new()` from
//! `x.method()` — the same limitation the site's old client-side highlighter
//! had. rust-analyzer's semantic tokens can, because they come from a real
//! analysis. `tokenColors` is kept deliberately coarse: comments, literals,
//! keywords and punctuation only, which is what TextMate gets right and what
//! still has to look sane in a file rust-analyzer has not loaded yet.

use std::io;
use std::path::Path;

use serde_json::{json, Map, Value};

use crate::palette::{Slot, SLOTS};

/// Slot -> the rust-analyzer semantic token selectors it should paint.
///
/// Several slots list more than one selector. VS Code silently ignores a
/// selector that never matches, so covering the plausible spellings costs
/// nothing and saves the theme from depending on one exact modifier name.
/// Confirm what your rust-analyzer actually emits with the command
/// "Developer: Inspect Editor Tokens and Scopes".
const SEMANTIC: &[(&str, &[&str])] = &[
    ("type-def", &[
        "struct.declaration", "enum.declaration", "union.declaration",
        "interface.declaration", "typeAlias.declaration",
    ]),
    ("fn-def", &["function.declaration", "method.declaration"]),
    ("type", &["struct", "enum", "union", "typeAlias", "builtinType", "type"]),
    ("trait", &["interface"]),
    ("generic-param", &["typeParameter"]),
    ("variable", &["variable", "parameter"]),
    ("field", &["property", "enumMember"]),
    ("constant", &["variable.constant", "variable.static", "constant"]),
    // Associated calls are the one mapping worth verifying first: which of
    // these rust-analyzer emits has varied between releases.
    ("call-assoc", &["method.static", "function.associated", "function.static"]),
    ("call-method", &["method"]),
    ("call-free", &["function"]),
    ("macro", &["macro"]),
    ("module", &["namespace"]),
    ("lifetime", &["lifetime"]),
    ("attribute", &["attribute", "decorator"]),
    ("keyword", &["keyword", "selfKeyword"]),
    ("string", &["string"]),
    ("number", &["number", "boolean"]),
    ("comment", &["comment"]),
    ("punct", &["operator"]),
];

/// Slots VS Code cannot express, and why. Kept explicit so the coverage test
/// can tell "deliberately unmapped" from "forgotten".
const UNMAPPABLE: &[(&str, &str)] = &[(
    "type-return",
    "VS Code has no return-position token — rust-analyzer reports a return type \
     as an ordinary type, so it falls back to the `type` colour.",
)];

/// The coarse TextMate fallback, for files rust-analyzer has not analysed.
/// Only the roles TextMate can actually get right appear here; anything that
/// needs to know how a name was reached is left to the semantic layer.
const TEXTMATE: &[(&str, &[&str])] = &[
    ("comment", &["comment", "punctuation.definition.comment"]),
    ("string", &["string", "punctuation.definition.string"]),
    ("number", &["constant.numeric", "constant.language.boolean"]),
    ("keyword", &["keyword", "storage.modifier", "keyword.control"]),
    ("macro", &["entity.name.function.macro", "support.function.macro"]),
    ("attribute", &["meta.attribute", "punctuation.definition.attribute"]),
    ("lifetime", &["storage.modifier.lifetime", "entity.name.type.lifetime"]),
    ("punct", &["keyword.operator", "punctuation"]),
];

/// The workbench around the editor, taken from the site's own chrome so the
/// two read as one thing.
struct Chrome {
    /// The code panel: `--code-bg` on the site.
    editor: &'static str,
    /// The content panel behind the code: `--content-bg`.
    content: &'static str,
    /// The site's top bar.
    bar: &'static str,
    bar_fg: &'static str,
    border: &'static str,
    muted: &'static str,
    accent: &'static str,
    /// Translucent accent, for selections and highlights.
    wash: &'static str,
}

const DARK_CHROME: Chrome = Chrome {
    editor: "#0a2c2e",
    content: "#0d3b3d",
    bar: "#2b2f36",
    bar_fg: "#F5C518",
    border: "#1c5c5d",
    muted: "#6f9c98",
    accent: "#F5C518",
    wash: "#f5c51826",
};

const LIGHT_CHROME: Chrome = Chrome {
    editor: "#e9f1ef",
    content: "#eef3f2",
    bar: "#f5c518",
    bar_fg: "#23262b",
    border: "#cdddda",
    muted: "#6a807c",
    accent: "#d9ac0a",
    wash: "#d9ac0a2e",
};

struct Theme {
    label: &'static str,
    slug: &'static str,
    kind: &'static str,
    ui: &'static str,
    pick: fn(&Slot) -> &'static str,
    chrome: &'static Chrome,
}

const THEMES: &[Theme] = &[
    Theme {
        label: "Rusty Yellow Pages Dark",
        slug: "rusty-yellow-pages-dark",
        kind: "dark",
        ui: "vs-dark",
        pick: |s| s.dark,
        chrome: &DARK_CHROME,
    },
    Theme {
        label: "Rusty Yellow Pages Light",
        slug: "rusty-yellow-pages-light",
        kind: "light",
        ui: "vs",
        pick: |s| s.light,
        chrome: &LIGHT_CHROME,
    },
];

fn colour_of(t: &Theme, slot: &str) -> Option<&'static str> {
    SLOTS.iter().find(|s| s.class == slot).map(t.pick)
}

fn workbench(t: &Theme) -> Value {
    let c = t.chrome;
    // Untokenised text is almost always a binding, so the editor's base colour
    // is the variable slot — the same choice the site's code panels make.
    let fg = colour_of(t, "variable").unwrap_or(c.muted);
    // A flat table rather than a `json!` literal: the object is big enough to
    // blow the macro's recursion limit, and this reads better besides.
    let entries: &[(&str, &str)] = &[        ("editor.background", c.editor),
        ("editor.foreground", fg),
        ("editorLineNumber.foreground", c.muted),
        ("editorLineNumber.activeForeground", c.accent),
        ("editorCursor.foreground", c.accent),
        ("editor.selectionBackground", c.wash),
        ("editor.lineHighlightBackground", c.content),
        ("editorIndentGuide.background1", c.border),
        ("editorWhitespace.foreground", c.border),
        ("editorGutter.background", c.editor),
        ("sideBar.background", c.content),
        ("sideBar.foreground", fg),
        ("sideBar.border", c.border),
        ("sideBarSectionHeader.background", c.content),
        ("sideBarTitle.foreground", c.muted),
        ("activityBar.background", c.bar),
        ("activityBar.foreground", c.bar_fg),
        ("activityBar.inactiveForeground", c.muted),
        ("activityBarBadge.background", c.accent),
        ("activityBarBadge.foreground", c.bar),
        ("titleBar.activeBackground", c.bar),
        ("titleBar.activeForeground", c.bar_fg),
        ("titleBar.inactiveBackground", c.bar),
        ("titleBar.inactiveForeground", c.muted),
        ("statusBar.background", c.bar),
        ("statusBar.foreground", c.bar_fg),
        ("statusBar.border", c.border),
        ("tab.activeBackground", c.editor),
        ("tab.activeForeground", fg),
        ("tab.activeBorderTop", c.accent),
        ("tab.inactiveBackground", c.content),
        ("tab.inactiveForeground", c.muted),
        ("tab.border", c.border),
        ("editorGroupHeader.tabsBackground", c.content),
        ("editorGroupHeader.tabsBorder", c.border),
        ("panel.background", c.editor),
        ("panel.border", c.border),
        ("terminal.background", c.editor),
        ("terminal.foreground", fg),
        ("editorWidget.background", c.content),
        ("editorWidget.border", c.border),
        ("input.background", c.editor),
        ("input.foreground", fg),
        ("input.border", c.border),
        ("dropdown.background", c.content),
        ("dropdown.foreground", fg),
        ("list.activeSelectionBackground", c.wash),
        ("list.hoverBackground", c.wash),
        ("focusBorder", c.accent),
        ("contrastBorder", c.border),
    ];
    let mut map = Map::new();
    for (k, v) in entries {
        map.insert((*k).to_string(), Value::String((*v).to_string()));
    }
    Value::Object(map)
}

fn semantic_token_colors(t: &Theme) -> Value {
    let mut map = Map::new();
    for (slot, selectors) in SEMANTIC {
        let Some(hex) = colour_of(t, slot) else { continue };
        for sel in *selectors {
            let mut entry = Map::new();
            entry.insert("foreground".into(), Value::String(hex.to_string()));
            if matches!(
                *slot,
                "type-def" | "fn-def" | "trait" | "constant" | "call-assoc" | "call-method"
                    | "call-free"
            ) {
                entry.insert("bold".into(), Value::Bool(true));
            }
            if matches!(*slot, "generic-param" | "lifetime" | "comment") {
                entry.insert("italic".into(), Value::Bool(true));
            }
            map.insert((*sel).to_string(), Value::Object(entry));
        }
    }
    Value::Object(map)
}

fn token_colors(t: &Theme) -> Value {
    let mut out = Vec::new();
    for (slot, scopes) in TEXTMATE {
        let Some(hex) = colour_of(t, slot) else { continue };
        let mut settings = Map::new();
        settings.insert("foreground".into(), Value::String(hex.to_string()));
        if *slot == "comment" {
            settings.insert("fontStyle".into(), Value::String("italic".into()));
        }
        out.push(json!({
            "name": *slot,
            "scope": scopes.iter().map(|s| Value::String((*s).into())).collect::<Vec<_>>(),
            "settings": Value::Object(settings),
        }));
    }
    Value::Array(out)
}

fn theme_json(t: &Theme) -> String {
    let doc = json!({
        "name": t.label,
        "type": t.kind,
        "semanticHighlighting": true,
        "colors": workbench(t),
        "semanticTokenColors": semantic_token_colors(t),
        "tokenColors": token_colors(t),
    });
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

fn package_json() -> String {
    let themes: Vec<Value> = THEMES
        .iter()
        .map(|t| {
            json!({
                "label": t.label,
                "uiTheme": t.ui,
                "path": format!("./themes/{}.json", t.slug),
            })
        })
        .collect();
    let doc = json!({
        "name": "rusty-yellow-pages-theme",
        "displayName": "Rusty Yellow Pages",
        "description": "One colour per role in Rust syntax, used for that role and nothing else.",
        "version": "0.1.0",
        "publisher": "rustyyellowpages",
        "license": "MIT",
        "engines": { "vscode": "^1.70.0" },
        "categories": ["Themes"],
        "contributes": { "themes": themes },
    });
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

fn readme() -> String {
    let unmapped: String = UNMAPPABLE
        .iter()
        .map(|(slot, why)| format!("- `{slot}` — {why}\n"))
        .collect();
    format!(
        "# Rusty Yellow Pages — VS Code theme\n\
         \n\
         One colour per role in Rust syntax, used for that role and nothing else.\n\
         Reaching a function through a type is painted differently from reaching one\n\
         through a value; declaring a struct is painted differently from naming it as\n\
         a type.\n\
         \n\
         Generated from the same palette that paints the site, so the two cannot drift.\n\
         Edit `tools/sitegen/src/palette.rs` and re-run the generator — never edit the\n\
         JSON in `themes/` by hand.\n\
         \n\
         ## Install\n\
         \n\
         Copy this folder into your extensions directory and reload the window:\n\
         \n\
         - Windows: `%USERPROFILE%\\.vscode\\extensions\\`\n\
         - macOS / Linux: `~/.vscode/extensions/`\n\
         \n\
         Then pick it from **Preferences: Color Theme**.\n\
         \n\
         ## Requires rust-analyzer\n\
         \n\
         The role colours ride on semantic tokens, which rust-analyzer provides. Without\n\
         it you get the coarse fallback — comments, literals, keywords and punctuation —\n\
         because TextMate scopes cannot tell an associated call from a method call.\n\
         \n\
         To see what any identifier actually resolves to, run **Developer: Inspect\n\
         Editor Tokens and Scopes** from the command palette and click it.\n\
         \n\
         ## What does not survive the trip\n\
         \n\
         {unmapped}"
    )
}

fn write_all(root: &Path) -> io::Result<()> {
    let dir = root.join("vscode-theme");
    let themes = dir.join("themes");
    std::fs::create_dir_all(&themes)?;
    std::fs::write(dir.join("package.json"), package_json())?;
    std::fs::write(dir.join("README.md"), readme())?;
    for t in THEMES {
        std::fs::write(themes.join(format!("{}.json", t.slug)), theme_json(t))?;
    }
    Ok(())
}

/// Write the VS Code theme extension into `vscode-theme/` at the repo root.
pub fn build(root: &Path) {
    if let Err(e) = write_all(root) {
        eprintln!("vscode: could not write the theme: {e}");
    } else {
        println!("vscode: rendered {} theme(s) + manifest", THEMES.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slot_is_mapped_or_explained() {
        // A new slot must either reach a semantic token or be recorded as
        // unmappable with a reason. Silently going unthemed is the failure
        // this catches.
        for s in SLOTS {
            let mapped = SEMANTIC.iter().any(|(slot, _)| *slot == s.class);
            let explained = UNMAPPABLE.iter().any(|(slot, _)| *slot == s.class);
            assert!(
                mapped || explained,
                "slot `{}` has no semantic token and no reason for not having one",
                s.class
            );
        }
    }

    #[test]
    fn no_slot_is_both_mapped_and_excused() {
        for (slot, _) in UNMAPPABLE {
            assert!(
                !SEMANTIC.iter().any(|(s, _)| s == slot),
                "slot `{slot}` is listed as unmappable but also mapped"
            );
        }
    }

    #[test]
    fn mappings_name_real_slots() {
        for (slot, _) in SEMANTIC {
            assert!(
                SLOTS.iter().any(|s| s.class == *slot),
                "semantic mapping names `{slot}`, which is not a palette slot"
            );
        }
        for (slot, _) in TEXTMATE {
            assert!(
                SLOTS.iter().any(|s| s.class == *slot),
                "textmate mapping names `{slot}`, which is not a palette slot"
            );
        }
        for (slot, _) in UNMAPPABLE {
            assert!(
                SLOTS.iter().any(|s| s.class == *slot),
                "unmappable list names `{slot}`, which is not a palette slot"
            );
        }
    }

    #[test]
    fn no_semantic_selector_is_claimed_twice() {
        // Two slots pointing at one selector would make the winner depend on
        // map ordering, and one of the two colours would never appear.
        let mut seen = std::collections::HashMap::new();
        for (slot, selectors) in SEMANTIC {
            for sel in *selectors {
                if let Some(other) = seen.insert(*sel, *slot) {
                    panic!("selector `{sel}` is claimed by both `{other}` and `{slot}`");
                }
            }
        }
    }

    #[test]
    fn both_themes_produce_valid_json_carrying_the_palette() {
        for t in THEMES {
            let parsed: Value = serde_json::from_str(&theme_json(t)).expect("theme is valid JSON");
            assert_eq!(parsed["semanticHighlighting"], Value::Bool(true));
            assert_eq!(parsed["type"], Value::String(t.kind.into()));

            let tokens = parsed["semanticTokenColors"].as_object().unwrap();
            for (slot, selectors) in SEMANTIC {
                let expected = colour_of(t, slot).unwrap();
                for sel in *selectors {
                    assert_eq!(
                        tokens[*sel]["foreground"],
                        Value::String(expected.into()),
                        "{} / {sel} does not carry the palette colour",
                        t.slug
                    );
                }
            }
            assert!(!parsed["colors"]["editor.background"].is_null());
        }
    }

    #[test]
    fn the_manifest_points_at_files_that_get_written() {
        let parsed: Value = serde_json::from_str(&package_json()).expect("manifest is valid JSON");
        let listed = parsed["contributes"]["themes"].as_array().unwrap();
        assert_eq!(listed.len(), THEMES.len());
        for (entry, t) in listed.iter().zip(THEMES) {
            assert_eq!(
                entry["path"],
                Value::String(format!("./themes/{}.json", t.slug))
            );
        }
    }
}
