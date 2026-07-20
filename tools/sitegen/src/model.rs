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
    pub basic_usage_html: String,
    pub best_practices_intro_html: String,
    pub scenarios: Vec<Scenario>,
    pub embedded_notes_html: String,
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
            ("literals", "Literals"),
            ("punctuation", "Punctuation"),
            ("comments", "Comments"),
        ],
        Section::Concepts => &[
            ("ownership-borrowing", "Ownership & Borrowing"),
            ("types-data-modeling", "Types & Data Modeling"),
            ("traits-polymorphism", "Traits & Polymorphism"),
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
