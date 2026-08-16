//! The syntax colour palette: one colour per role in Rust, used for that
//! role and nothing else.
//!
//! Single source of truth. The LangColorMap page presents this table, the
//! highlighter paints code with it, and the site stylesheet gets its custom
//! properties generated from it — so a colour can only ever be changed here.

/// The stylesheet block that paints code across the whole site: the custom
/// properties for both themes, plus one `.tok-*` rule per slot.
///
/// Generated rather than hand-written in site.css so that the palette and the
/// styling cannot drift — adding a slot cannot leave it uncoloured.
pub fn stylesheet() -> String {
    let vars = |pick: fn(&Slot) -> &'static str| {
        SLOTS
            .iter()
            .map(|s| format!("    --t-{}: {};\n", s.class, pick(s)))
            .collect::<String>()
    };
    let rules = SLOTS
        .iter()
        .map(|s| {
            let extra = match s.class {
                "type-def" | "fn-def" | "trait" | "constant" | "call-assoc" | "call-method"
                | "call-free" => " font-weight: 700;",
                "generic-param" | "lifetime" => " font-style: italic;",
                "comment" => " font-style: italic;",
                _ => "",
            };
            format!("  .tok-{c} {{ color: var(--t-{c});{extra} }}\n", c = s.class)
        })
        .collect::<String>();

    format!(
        "\n  /* ---------- SYNTAX PALETTE (generated from palette.rs) ---------- */\n\n\
         \x20 :root {{\n{dark}  }}\n\
         \x20 :root[data-theme=\"light\"] {{\n{light}  }}\n\n\
         \x20 /* Code panels take their body colour from the variable slot, which is\n\
         \x20    what unpainted text in a snippet almost always is. */\n\
         \x20 :root, :root[data-theme=\"light\"] {{ --t-default: var(--t-variable); }}\n\n\
         {rules}",
        dark = vars(|s| s.dark),
        light = vars(|s| s.light),
    )
}

/// One syntax role and the colour reserved for it.
///
/// `class` is both the CSS slug and the name shown in the list. It is prefixed
/// `cm-` on the page rather than reusing the live highlighter's `tok-` classes:
/// the palette is still being tuned, and the map must be able to show it
/// without repainting every code block on the site.
pub struct Slot {
    pub class: &'static str,
    /// Shown as the row's tooltip.
    pub covers: &'static str,
    pub dark: &'static str,
    pub light: &'static str,
}

/// Ordered by hue rather than alphabetically or by role, so neighbouring rows
/// are neighbouring colours and the palette can be read as a whole.
pub const SLOTS: &[Slot] = &[
    Slot { class: "call-free",     covers: "bare foo()",                              dark: "#D9A98A", light: "#D9A98A" },
    Slot { class: "string",        covers: "string, char, byte, raw",                 dark: "#FFAE00", light: "#C23B24" },
    Slot { class: "constant",      covers: "const / static names",                    dark: "#E5D9A8", light: "#C29B00" },
    Slot { class: "lifetime",      covers: "'a, 'static, labels",                     dark: "#F2DB69", light: "#FD4EDD" },
    Slot { class: "call-method",   covers: "x.foo()",                                 dark: "#FFEA00", light: "#7004C8" },
    Slot { class: "call-assoc",    covers: "Type::new()",                             dark: "#FFF952", light: "#F5A55A" },
    Slot { class: "punct",         covers: "operators, :: -> ?",                      dark: "#FBF42D", light: "#180070" },
    Slot { class: "module",        covers: "std, collections",                        dark: "#C8D373", light: "#AA261D" },
    Slot { class: "type-def",      covers: "the name at struct/enum/trait/type X",    dark: "#1FFF9E", light: "#1189DF" },
    Slot { class: "variable",      covers: "every binding, param, closure param",     dark: "#6BFFEE", light: "#06C3C6" },
    Slot { class: "trait",         covers: "trait names",                             dark: "#02AEC5", light: "#019D59" },
    Slot { class: "type",          covers: "struct, enum, primitive, alias — at use", dark: "#00E1FF", light: "#2B6B73" },
    Slot { class: "field",         covers: "field def, x.field, .0, enum variants",   dark: "#A8E2FF", light: "#8280FF" },
    Slot { class: "generic-param", covers: "T, E, const generics",                    dark: "#A9D6F5", light: "#C89797" },
    Slot { class: "fn-def",        covers: "the name at fn X(&hellip;), all kinds",   dark: "#B899FF", light: "#9966A9" },
    Slot { class: "macro",         covers: "println!, vec!",                          dark: "#D972EE", light: "#D972EE" },
    Slot { class: "attribute",     covers: "#[derive(&hellip;)]",                     dark: "#FED6FF", light: "#752F00" },
    Slot { class: "number",        covers: "int, float, bool",                        dark: "#A17297", light: "#0939C8" },
    Slot { class: "comment",       covers: "all comments",                            dark: "#FFFFFF", light: "#156C04" },
    Slot { class: "keyword",       covers: "fn let const mut self impl pub &hellip;", dark: "#BABABA", light: "#FA0000" },
];
