use anyhow::{Context, Result};
use clap::Parser;
use jdoc::{parse_doc_comments, DocEntry};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "jdoc",
    version = "0.1.0",
    about = "Extract Go-style doc comments from .go files"
)]
struct Args {
    /// Directory or .go file to scan
    path: PathBuf,

    /// Emit JSON array of {name, kind, doc} entries
    #[arg(long)]
    json: bool,

    /// Only include exported (capitalized) names
    #[arg(long)]
    public_only: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut entries = collect_entries(&args.path)?;
    if args.public_only {
        entries.retain(|e| e.name.chars().next().is_some_and(|c| c.is_uppercase()));
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for entry in &entries {
            println!("{} {} {}", entry.kind, entry.name, entry.doc);
        }
    }

    Ok(())
}

fn collect_entries(path: &Path) -> Result<Vec<DocEntry>> {
    let mut entries = Vec::new();
    if path.is_file() {
        if is_go_file(path) {
            entries.extend(parse_go_file(path)?);
        }
        return Ok(entries);
    }

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_file() && is_go_file(p) {
            entries.extend(parse_go_file(p)?);
        }
    }

    Ok(entries)
}

fn is_go_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("go")
}

fn parse_go_file(path: &Path) -> Result<Vec<DocEntry>> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    Ok(parse_doc_comments(&source))
}
