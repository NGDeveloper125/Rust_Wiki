use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FrontMatter {
    pub title: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub area: Option<String>,
    #[serde(default)]
    pub embedded_support: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub related_concepts: Vec<String>,
    #[serde(default)]
    pub related_syntax: Vec<String>,
    #[serde(default)]
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Syntax,
    Concepts,
}

impl Section {
    pub fn label(self) -> &'static str {
        match self {
            Section::Syntax => "Syntax",
            Section::Concepts => "Concepts",
        }
    }
}

pub struct Scenario {
    pub title: String,
    pub body_html: String,
    pub rationale_html: Option<String>,
}

/// A single titled entry under a syntax page's "Usage examples" section.
/// Unlike `Scenario` (concept pages only), the intro/code/closing blurb all
/// flow together as one blob — no separate rationale callout.
pub struct Example {
    pub title: String,
    pub body_html: String,
}

pub struct Page {
    pub front: FrontMatter,
    pub section: Section,
    /// Parent folder name under pages/<section>/<subgroup>/, e.g. "operators".
    pub subgroup: String,
    /// File stem, e.g. "ampersand".
    pub slug: String,
    /// Site-root-relative output path, e.g. "syntax/operators/ampersand.html".
    pub href: String,

    pub explanation_html: String,
    /// Concept pages only; empty for syntax pages.
    pub basic_usage_html: String,
    /// Concept pages only; empty for syntax pages.
    pub best_practices_intro_html: String,
    /// Concept pages only; empty for syntax pages.
    pub scenarios: Vec<Scenario>,
    /// Syntax pages only; empty for concept pages.
    pub usage_examples: Vec<Example>,

    /// `embedded_support: none` pages only: the short "not supported" note.
    /// Empty for `full`/`partial` pages, which use the fields below instead.
    pub embedded_notes_html: String,
    /// `full`/`partial` pages only: the embedded counterpart of `explanation_html`.
    pub embedded_explanation_html: String,
    /// `full`/`partial` concept pages only.
    pub embedded_basic_usage_html: String,
    /// `full`/`partial` concept pages only.
    pub embedded_best_practices_intro_html: String,
    /// `full`/`partial` concept pages only.
    pub embedded_scenarios: Vec<Scenario>,
    /// `full`/`partial` syntax pages only.
    pub embedded_usage_examples: Vec<Example>,
}

impl Page {
    pub fn kind_badge(&self) -> String {
        match self.section {
            Section::Syntax => title_case(self.front.kind.as_deref().unwrap_or("syntax")),
            Section::Concepts => "Concept".to_string(),
        }
    }

    pub fn embedded_support(&self) -> &str {
        self.front.embedded_support.as_deref().unwrap_or("full")
    }
}

pub fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Group folder-name -> display label, in display order, per section.
pub fn group_order(section: Section) -> &'static [(&'static str, &'static str)] {
    match section {
        Section::Syntax => &[
            ("keywords", "Keywords"),
            ("operators", "Operators & Sigils"),
            ("lifetimes", "Lifetimes"),
            ("literals", "Literals"),
            ("punctuation", "Punctuation"),
            ("comments", "Comments"),
            ("attributes", "Attributes"),
            ("macros", "Macros"),
        ],
        Section::Concepts => &[
            ("ownership-borrowing", "Ownership & Borrowing"),
            ("types-data-modeling", "Types & Data Modeling"),
            ("traits-polymorphism", "Traits & Polymorphism"),
            ("functions-closures", "Functions & Closures"),
            ("iterators", "Iterators"),
            ("error-handling", "Error Handling"),
            ("pattern-matching", "Pattern Matching"),
            ("modules-crates-visibility", "Modules, Crates & Visibility"),
            ("concurrency-async", "Concurrency & Async"),
            ("memory-unsafe", "Memory & Unsafe"),
            ("macros-metaprogramming", "Macros & Metaprogramming"),
            ("collections-strings", "Collections & Strings"),
            ("testing-tooling", "Testing & Tooling"),
            ("design-patterns-idioms", "Design Patterns & Idioms"),
            ("philosophy-principles", "Rust Philosophy & Design Principles"),
        ],
    }
}

pub fn group_label(section: Section, subgroup: &str) -> String {
    group_order(section)
        .iter()
        .find(|(k, _)| *k == subgroup)
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| title_case(subgroup))
}

/// For folders large/mixed enough to benefit from a second nesting level in
/// the sidebar, the ordered list of sub-group names to bucket pages into.
/// A page's bucket is its first `groups` frontmatter entry; buckets not
/// listed here (or pages with no `groups` at all) fall into a trailing
/// "Other" bucket instead of being dropped. Folders not covered here render
/// as a single flat list, same as before this existed.
pub fn subgroup_order(section: Section, folder: &str) -> Option<&'static [&'static str]> {
    match (section, folder) {
        (Section::Syntax, "keywords") => Some(&[
            "Basics",
            "Control Flow",
            "Ownership & Borrowing",
            "Types & Data Structures",
            "Traits & Polymorphism",
            "Modules & Visibility",
            "Concurrency & Async",
            "Memory & Unsafe",
            "Macros",
            "Reserved Keywords",
        ]),
        (Section::Syntax, "operators") => Some(&[
            "Arithmetic",
            "Comparison",
            "Logical",
            "Bitwise",
            "Assignment",
            "Ownership & Borrowing",
            "Types & Data Structures",
            "Modules & Visibility",
            "Error Handling",
        ]),
        (Section::Syntax, "attributes") => Some(&[
            "Core Syntax",
            "Conditional Compilation",
            "Testing",
            "Lints & Diagnostics",
            "Documentation",
            "Traits & Derives",
            "Macros",
            "Types & Layout",
            "Modules & Visibility",
            "FFI & Linkage",
            "No-std & Embedded Runtime",
            "Compiler Hints & Limits",
        ]),
        (Section::Syntax, "macros") => Some(&[
            "Macro Definition Syntax",
            "Output & Formatting",
            "Collections",
            "Errors & Assertions",
            "Compile-time Introspection",
        ]),
        (Section::Concepts, "design-patterns-idioms") => Some(&[
            "Design Patterns",
            "Idioms",
            "Anti-patterns",
        ]),
        _ => None,
    }
}

/// A page's sidebar sub-bucket within its folder: its first `groups` entry,
/// or "Other" if it has none. See `subgroup_order`.
pub fn nav_bucket(front: &FrontMatter) -> &str {
    front.groups.first().map(|s| s.as_str()).unwrap_or("Other")
}
