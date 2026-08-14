//! A Rust highlighter that paints the palette's roles, not just token classes.
//!
//! It runs at build time rather than in the browser, which buys three things:
//! the colours survive with JavaScript off, there is no flash of unpainted
//! code on load, and the role rules can be unit-tested instead of eyeballed.
//!
//! Roles are decided from the token stream and its shape alone — no name
//! resolution, no type inference. That is possible because the distinctions
//! the palette draws are all positional: `::` before a call means associated,
//! `.` means method, an identifier after `fn` is a declaration, a type after
//! `->` is a return type. Where the text genuinely cannot say (a lone
//! identifier that might be a local or a unit struct), the rules fall back to
//! the quieter role rather than guessing.

use crate::util::html_escape;

/// Keywords that get the single `keyword` colour. `true`/`false` are absent on
/// purpose: they are literals, and the palette paints them as numbers.
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "type", "union", "unsafe",
    "use", "where", "while", "yield",
];

/// Prelude variants. A bare capitalised name is ambiguous from the text alone
/// — `None` could as easily be a unit struct — but these four are so much more
/// common than anything they could be confused with that naming them is worth
/// more than the rule it costs. Everything else falls back to the general rules.
const PRELUDE_VARIANTS: &[&str] = &["Some", "None", "Ok", "Err"];

const PRIMITIVES: &[&str] = &[
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64", "()",
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Space,
    Comment,
    /// `// <-` — the site's "read this line" convention, lifted out of the
    /// comment colour so it reads as an annotation rather than as dead text.
    Anno,
    Str,
    Num,
    Lifetime,
    Attr,
    /// An identifier with its `!` already attached.
    MacroCall,
    Ident,
    Punct,
}

struct Tok<'a> {
    kind: Kind,
    text: &'a str,
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn lex(src: &str) -> Vec<Tok<'_>> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;

    while i < src.len() {
        let start = i;
        let c = src[i..].chars().next().unwrap();

        // whitespace
        if c.is_whitespace() {
            while i < src.len() && src[i..].chars().next().unwrap().is_whitespace() {
                i += src[i..].chars().next().unwrap().len_utf8();
            }
            out.push(Tok { kind: Kind::Space, text: &src[start..i] });
            continue;
        }

        // comments
        if c == '/' && i + 1 < src.len() {
            if b[i + 1] == b'/' {
                while i < src.len() && b[i] != b'\n' {
                    i += 1;
                }
                let text = &src[start..i];
                let kind = if text.trim_start_matches('/').trim_start().starts_with("<-") {
                    Kind::Anno
                } else {
                    Kind::Comment
                };
                out.push(Tok { kind, text });
                continue;
            }
            if b[i + 1] == b'*' {
                i += 2;
                let mut depth = 1;
                while i < src.len() && depth > 0 {
                    if src[i..].starts_with("*/") {
                        depth -= 1;
                        i += 2;
                    } else if src[i..].starts_with("/*") {
                        depth += 1;
                        i += 2;
                    } else {
                        // Whole chars, not bytes: the next iteration slices
                        // `src[i..]`, and prose comments contain em dashes.
                        i += src[i..].chars().next().map_or(1, char::len_utf8);
                    }
                }
                out.push(Tok { kind: Kind::Comment, text: &src[start..i] });
                continue;
            }
        }

        // raw / byte strings: r"..", r#".."#, b"..", br#".."#
        if (c == 'r' || c == 'b') && i + 1 < src.len() {
            let mut j = i;
            if b[j] == b'b' && j + 1 < src.len() && b[j + 1] == b'r' {
                j += 1;
            }
            if b[j] == b'r' {
                let mut k = j + 1;
                let hashes = {
                    let h = k;
                    while k < src.len() && b[k] == b'#' {
                        k += 1;
                    }
                    k - h
                };
                if k < src.len() && b[k] == b'"' {
                    let close = format!("\"{}", "#".repeat(hashes));
                    k += 1;
                    match src[k..].find(&close) {
                        Some(rel) => i = k + rel + close.len(),
                        None => i = src.len(),
                    }
                    out.push(Tok { kind: Kind::Str, text: &src[start..i] });
                    continue;
                }
            }
            if b[j] == b'"' {
                i = j;
                // falls through to the plain-string arm below
            }
        }

        // strings
        if c == '"' {
            i += 1;
            while i < src.len() {
                if b[i] == b'\\' {
                    i += 1;
                    if i < src.len() {
                        i += src[i..].chars().next().map_or(1, char::len_utf8);
                    }
                    continue;
                }
                if b[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.push(Tok { kind: Kind::Str, text: &src[start..i] });
            continue;
        }

        // `'` opens either a char literal or a lifetime/label. A lifetime is an
        // identifier that is not closed by a second quote.
        if c == '\'' {
            let rest = &src[i + 1..];
            let is_lifetime = rest
                .chars()
                .next()
                .is_some_and(is_ident_start)
                && {
                    let mut k = i + 1;
                    while k < src.len() && is_ident_char(src[k..].chars().next().unwrap()) {
                        k += src[k..].chars().next().unwrap().len_utf8();
                    }
                    k >= src.len() || b[k] != b'\''
                };
            if is_lifetime {
                i += 1;
                while i < src.len() && is_ident_char(src[i..].chars().next().unwrap()) {
                    i += src[i..].chars().next().unwrap().len_utf8();
                }
                out.push(Tok { kind: Kind::Lifetime, text: &src[start..i] });
            } else {
                i += 1;
                while i < src.len() {
                    if b[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b[i] == b'\'' {
                        i += 1;
                        break;
                    }
                    i += src[i..].chars().next().unwrap().len_utf8();
                }
                out.push(Tok { kind: Kind::Str, text: &src[start..i] });
            }
            continue;
        }

        // attributes: `#[..]` / `#![..]`, taken whole so the brackets and the
        // path inside share one colour.
        if c == '#' && src[i..].starts_with('#') {
            let mut k = i + 1;
            if k < src.len() && b[k] == b'!' {
                k += 1;
            }
            if k < src.len() && b[k] == b'[' {
                let mut depth = 0;
                while k < src.len() {
                    match b[k] {
                        b'[' => depth += 1,
                        b']' => {
                            depth -= 1;
                            if depth == 0 {
                                k += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    k += 1;
                }
                i = k;
                out.push(Tok { kind: Kind::Attr, text: &src[start..i] });
                continue;
            }
        }

        // numbers
        if c.is_ascii_digit() {
            while i < src.len() {
                let ch = src[i..].chars().next().unwrap();
                if is_ident_char(ch) || ch == '.' && src[i + 1..].chars().next().is_some_and(|n| n.is_ascii_digit())
                {
                    i += ch.len_utf8();
                } else {
                    break;
                }
            }
            out.push(Tok { kind: Kind::Num, text: &src[start..i] });
            continue;
        }

        // identifiers, and macro names with their `!`
        if is_ident_start(c) {
            while i < src.len() && is_ident_char(src[i..].chars().next().unwrap()) {
                i += src[i..].chars().next().unwrap().len_utf8();
            }
            // `!=` is an operator, not a macro
            if i < src.len() && b[i] == b'!' && !(i + 1 < src.len() && b[i + 1] == b'=') {
                i += 1;
                out.push(Tok { kind: Kind::MacroCall, text: &src[start..i] });
            } else {
                out.push(Tok { kind: Kind::Ident, text: &src[start..i] });
            }
            continue;
        }

        // multi-character punctuation that the role rules look for by name
        // Longest first. `>>` and `<<` are deliberately absent: `Vec<Vec<T>>`
        // would lex its closing brackets as one shift and lose the nesting
        // depth the trait-bound rule relies on.
        for op in [
            "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=",
            "%=", "..",
        ] {
            if src[i..].starts_with(op) {
                i += op.len();
                out.push(Tok { kind: Kind::Punct, text: &src[start..i] });
                break;
            }
        }
        if i > start {
            continue;
        }

        i += c.len_utf8();
        out.push(Tok { kind: Kind::Punct, text: &src[start..i] });
    }
    out
}

fn is_screaming(s: &str) -> bool {
    s.len() > 1
        && s.chars().any(|c| c.is_ascii_uppercase())
        && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

fn starts_upper(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_uppercase())
}

/// Index of the previous/next token that isn't whitespace or a comment.
fn prev_sig(toks: &[Tok], i: usize) -> Option<usize> {
    (0..i).rev().find(|&k| !matches!(toks[k].kind, Kind::Space | Kind::Comment | Kind::Anno))
}
fn next_sig(toks: &[Tok], i: usize) -> Option<usize> {
    (i + 1..toks.len()).find(|&k| !matches!(toks[k].kind, Kind::Space | Kind::Comment | Kind::Anno))
}

/// Highlight a block of Rust into HTML spans carrying palette classes.
pub fn rust_to_html(src: &str) -> String {
    let toks = lex(src);
    let mut out = String::with_capacity(src.len() * 3);

    // Set while walking a return type (after `->`, until the body or the end of
    // the signature) and while inside `<…>`, where a bare identifier after `:`
    // is a trait bound rather than a field.
    let mut in_return = false;
    let mut angle_depth: i32 = 0;

    for i in 0..toks.len() {
        let t = &toks[i];
        let text = t.text;

        let class: Option<&str> = match t.kind {
            Kind::Space => None,
            Kind::Comment => Some("comment"),
            Kind::Anno => Some("comment tok-anno"),
            Kind::Str => Some("string"),
            Kind::Num => Some("number"),
            Kind::Lifetime => Some("lifetime"),
            Kind::Attr => Some("attribute"),
            Kind::MacroCall => Some("macro"),
            Kind::Punct => {
                match text {
                    "->" => in_return = true,
                    // The signature is over once the body, the end of a trait
                    // method, or a where-clause begins.
                    "{" | ";" => in_return = false,
                    "<" => angle_depth += 1,
                    ">" => angle_depth = (angle_depth - 1).max(0),
                    _ => {}
                }
                Some("punct")
            }
            Kind::Ident => {
                let prev_idx = prev_sig(&toks, i);
                let prev = prev_idx.map(|k| toks[k].text);
                let prev2 = prev_idx.and_then(|k| prev_sig(&toks, k)).map(|k| toks[k].text);
                let next = next_sig(&toks, i).map(|k| toks[k].text);

                if text == "true" || text == "false" {
                    Some("number")
                } else if KEYWORDS.contains(&text) {
                    if text == "where" {
                        in_return = false;
                    }
                    Some("keyword")
                } else if prev == Some("fn") {
                    Some("fn-def")
                } else if matches!(prev, Some("struct") | Some("enum") | Some("union") | Some("trait") | Some("type")) {
                    Some("type-def")
                } else if PRELUDE_VARIANTS.contains(&text) {
                    Some("field")
                } else if next == Some("(") && starts_upper(text) {
                    // `Wrapper(x)` is a tuple struct or a variant being built,
                    // not a function being called — capitalisation is what
                    // separates the two, and the palette calls variants members.
                    Some("field")
                } else if next == Some("(") {
                    // A call. Which kind is decided entirely by what reaches it.
                    match prev {
                        Some(".") => Some("call-method"),
                        Some("::") => Some("call-assoc"),
                        _ => Some("call-free"),
                    }
                } else if prev == Some(".") {
                    Some("field")
                } else if prev == Some("dyn") {
                    Some("trait")
                } else if next == Some("for") && starts_upper(text) {
                    // `impl Trait for Type` — the name before `for` is the
                    // trait; the one after it is the type, and falls through.
                    Some("trait")
                } else if prev == Some(":") && angle_depth > 0 && starts_upper(text) {
                    Some("trait")
                } else if is_screaming(text) {
                    Some("constant")
                } else if next == Some("::") {
                    // `std::collections::HashMap` — the lowercase segments are
                    // modules, the capitalised one is the type it resolves to.
                    if starts_upper(text) {
                        Some(if in_return { "type-return" } else { "type" })
                    } else {
                        Some("module")
                    }
                } else if prev == Some("::") && starts_upper(text) {
                    // Reached through `::`. If the segment before was itself a
                    // type, this is a variant, which the palette treats as a
                    // member; if it was a module, this is the type the path
                    // resolves to — `Mode::Fast` against `collections::HashMap`.
                    if prev2.is_some_and(starts_upper) {
                        Some("field")
                    } else {
                        Some(if in_return { "type-return" } else { "type" })
                    }
                } else if starts_upper(text) {
                    if text.len() == 1 {
                        Some("generic-param")
                    } else if in_return {
                        Some("type-return")
                    } else {
                        Some("type")
                    }
                } else if PRIMITIVES.contains(&text) {
                    Some(if in_return { "type-return" } else { "type" })
                } else {
                    Some("variable")
                }
            }
        };

        match class {
            None => out.push_str(&html_escape(text)),
            Some(c) => {
                let cls = if let Some(extra) = c.strip_prefix("comment ") {
                    format!("tok-comment {extra}")
                } else {
                    format!("tok-{c}")
                };
                out.push_str(&format!("<span class=\"{cls}\">{}</span>", html_escape(text)));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::SLOTS;

    /// Every class the highlighter can emit, for cross-checking against the
    /// palette. `tok-anno` is a treatment on top of `comment`, not a slot.
    fn emitted_classes() -> Vec<&'static str> {
        vec![
            "comment", "string", "number", "lifetime", "attribute", "macro", "punct", "keyword",
            "fn-def", "type-def", "call-method", "call-assoc", "call-free", "field", "trait",
            "constant", "module", "type-return", "type", "generic-param", "variable",
        ]
    }

    #[test]
    fn every_emitted_class_is_a_palette_slot() {
        for c in emitted_classes() {
            assert!(
                SLOTS.iter().any(|s| s.class == c),
                "highlighter emits `{c}`, which has no colour in the palette"
            );
        }
    }

    #[test]
    fn every_palette_slot_can_be_emitted() {
        for s in SLOTS {
            assert!(
                emitted_classes().contains(&s.class),
                "palette defines `{}`, which the highlighter never emits",
                s.class
            );
        }
    }

    fn classes_of(src: &str, needle: &str) -> Vec<String> {
        let html = rust_to_html(src);
        let mut out = Vec::new();
        let pat = format!(">{needle}<");
        let mut from = 0;
        while let Some(rel) = html[from..].find(&pat) {
            let at = from + rel;
            let open = html[..at].rfind("class=\"").unwrap() + 7;
            let close = html[open..].find('"').unwrap() + open;
            out.push(html[open..close].to_string());
            from = at + pat.len();
        }
        out
    }

    #[test]
    fn the_three_call_kinds_are_told_apart() {
        let src = "Counter::new(); counter.record(); tally();";
        assert_eq!(classes_of(src, "new"), ["tok-call-assoc"]);
        assert_eq!(classes_of(src, "record"), ["tok-call-method"]);
        assert_eq!(classes_of(src, "tally"), ["tok-call-free"]);
    }

    #[test]
    fn declaring_differs_from_using() {
        let src = "struct Counter; fn record() {} let c = Counter::new(); record();";
        assert_eq!(classes_of(src, "Counter"), ["tok-type-def", "tok-type"]);
        assert_eq!(classes_of(src, "record"), ["tok-fn-def", "tok-call-free"]);
    }

    #[test]
    fn return_position_overrides_type() {
        let src = "fn f(x: String) -> String { }";
        assert_eq!(classes_of(src, "String"), ["tok-type", "tok-type-return"]);
    }

    #[test]
    fn paths_split_into_modules_and_types() {
        let src = "use std::collections::HashMap;";
        assert_eq!(classes_of(src, "std"), ["tok-module"]);
        assert_eq!(classes_of(src, "collections"), ["tok-module"]);
        assert_eq!(classes_of(src, "HashMap"), ["tok-type"]);
    }

    #[test]
    fn variants_and_fields_share_a_colour() {
        let src = "self.counts; Mode::Fast;";
        assert_eq!(classes_of(src, "counts"), ["tok-field"]);
        assert_eq!(classes_of(src, "Fast"), ["tok-field"]);
    }

    #[test]
    fn literals_lifetimes_and_macros() {
        let html = rust_to_html("println!(\"hi {}\", 1u32); let x: &'a str = 'c'; true");
        assert!(html.contains("<span class=\"tok-macro\">println!</span>"));
        assert!(html.contains("<span class=\"tok-string\">&quot;hi {}&quot;</span>"));
        assert!(html.contains("<span class=\"tok-number\">1u32</span>"));
        assert!(html.contains("<span class=\"tok-lifetime\">'a</span>"));
        assert!(html.contains("<span class=\"tok-string\">'c'</span>"));
        assert!(html.contains("<span class=\"tok-number\">true</span>"));
    }

    #[test]
    fn attributes_and_comments() {
        let html = rust_to_html("#[derive(Debug)]\n// plain\n// <- look here\nconst MAX: u8 = 1;");
        assert!(html.contains("<span class=\"tok-attribute\">#[derive(Debug)]</span>"));
        assert!(html.contains("<span class=\"tok-comment\">// plain</span>"));
        assert!(html.contains("tok-anno"));
        assert!(html.contains("<span class=\"tok-constant\">MAX</span>"));
    }

    #[test]
    fn prelude_variants_are_members_not_calls() {
        // `Some(x)` looks exactly like a function call and must not be painted
        // as one; `None` looks exactly like a type name and must not be either.
        let src = "if x { Some(String::new()) } else { None }";
        assert_eq!(classes_of(src, "Some"), ["tok-field"]);
        assert_eq!(classes_of(src, "None"), ["tok-field"]);
        assert_eq!(classes_of(src, "String"), ["tok-type"]);
        assert_eq!(classes_of(src, "new"), ["tok-call-assoc"]);
    }

    #[test]
    fn tuple_constructors_are_members() {
        assert_eq!(classes_of("let w = Wrapper(1);", "Wrapper"), ["tok-field"]);
    }

    #[test]
    fn nested_generics_keep_their_depth() {
        // `>>` must not lex as a shift, or the closing brackets of a nested
        // generic stop unwinding the depth the trait-bound rule reads.
        let src = "fn f<T: Clone>(v: Vec<Vec<T>>) {}";
        assert_eq!(classes_of(src, "Clone"), ["tok-trait"]);
        assert_eq!(classes_of(src, "Vec"), ["tok-type", "tok-type"]);
    }

    #[test]
    fn comparison_operators_are_one_token() {
        let html = rust_to_html("a == b");
        assert!(html.contains("<span class=\"tok-punct\">==</span>"));
    }

    #[test]
    fn multibyte_text_in_comments_survives() {
        // Prose comments carry em dashes; a byte-wise scanner splits them.
        let src = "/* a — b */\n// c — d\nlet s = \"e — f\";\n";
        assert_eq!(rust_to_html(src).matches('—').count(), 3);
    }

    #[test]
    fn traits_are_told_from_types() {
        let src = "impl Summarize for Counter { }";
        assert_eq!(classes_of(src, "Summarize"), ["tok-trait"]);
        assert_eq!(classes_of(src, "Counter"), ["tok-type"]);
    }

    #[test]
    fn html_in_code_is_escaped() {
        let html = rust_to_html("let v: Vec<&str> = vec![];");
        assert!(!html.contains("<&str>"));
        assert!(html.contains("&lt;"));
        assert!(html.contains("&amp;"));
    }

    #[test]
    fn output_preserves_the_source_text() {
        // Painting must never change what the code says. Stripping the tags
        // and unescaping has to give back the input byte for byte.
        let src = "fn main() {\n    let x = foo(&b, \"s\") ?; // note\n}\n";
        let html = rust_to_html(src);
        let mut plain = String::new();
        let mut in_tag = false;
        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => plain.push(c),
                _ => {}
            }
        }
        let plain = plain.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"")
            .replace("&#39;", "'").replace("&amp;", "&");
        assert_eq!(plain, src);
    }
}
