#![forbid(unsafe_code)]

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocEntry {
    pub name: String,
    pub kind: String,
    pub doc: String,
}

/// Extract top-level doc comments immediately preceding `func`, `type`, `var`, or `const`.
pub fn parse_doc_comments(source: &str) -> Vec<DocEntry> {
    let re = Regex::new(
        r"(?ms)^(?P<doc>(?:(?:[ \t]*//[^\n]*(?:\n|$))+|(?:[ \t]*/\*.*?\*/[ \t]*(?:\n|$))+))\s*(?P<kind>func|type|var|const)\s+(?:(?:\([^)]*\)\s+)?)(?P<name>[A-Za-z_][A-Za-z0-9_]*)",
    )
    .expect("valid doc-comment regex");

    re.captures_iter(source)
        .filter_map(|cap| {
            let doc_raw = cap.name("doc")?.as_str();
            let doc = clean_doc(doc_raw);
            if doc.is_empty() {
                return None;
            }
            Some(DocEntry {
                name: cap.name("name")?.as_str().to_string(),
                kind: cap.name("kind")?.as_str().to_string(),
                doc,
            })
        })
        .collect()
}

fn clean_doc(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with("/*") {
        let inner = trimmed
            .trim_start_matches("/*")
            .trim_end_matches("*/");
        return inner
            .lines()
            .map(|line| line.trim().trim_start_matches('*').trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
    }

    trimmed
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("//")
                .map(|rest| rest.trim_start().to_string())
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_line_comments_before_func() {
        let src = "// Hello world.\nfunc Foo() {}\n";
        let entries = parse_doc_comments(src);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Foo");
        assert_eq!(entries[0].kind, "func");
        assert_eq!(entries[0].doc, "Hello world.");
    }

    #[test]
    fn ignores_undocumented_declarations() {
        let src = "func Bar() {}\n";
        assert!(parse_doc_comments(src).is_empty());
    }
}
