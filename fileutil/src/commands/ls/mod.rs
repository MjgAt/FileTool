use clap::Args;
use fswalk::{default_provider, WalkProvider};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

use crate::schema::ColumnDef;

/// Canonical CSV column definitions for the `ls` subcommand.
/// Columns are listed in the order they appear in CSV output (alphabetical
/// by field name, because serde_json serialises via BTreeMap).
pub fn csv_columns() -> Vec<ColumnDef> {
    vec![
        ColumnDef::boolean("is_dir"),
        ColumnDef::string("path"),
        ColumnDef::nullable_integer("size"),
    ]
}

#[derive(Args, Serialize, Deserialize, Debug)]
pub struct LsArgs {
    #[arg(short, long, default_value = ".")]
    pub path: String,
    #[arg(short = 'r', long)]
    pub recursive: bool,
}

#[derive(Serialize)]
struct FileEntry {
    path: String,
    is_dir: bool,
    size: Option<u64>,
}

pub fn run(args: LsArgs) -> Value {
    run_with_provider(args, &default_provider())
}

/// Testable core: accepts any [`WalkProvider`] implementation.
pub fn run_with_provider(args: LsArgs, provider: &dyn WalkProvider) -> Value {
    let root = Path::new(&args.path);
    // For non-recursive ls we still use the provider but filter to depth 1.
    // WalkProvider::walk always recurses; we skip sub-entries when not recursive.
    let mut entries = Vec::new();
    for entry in provider.walk(root, false) {
        // Depth 0 = the root itself. Depth 1 = immediate children.
        let depth = entry.rel_path.components().count();
        if !args.recursive && depth > 1 {
            continue;
        }
        entries.push(FileEntry {
            path: entry.path.to_string_lossy().to_string(),
            is_dir: entry.is_dir,
            size: entry.size,
        });
    }
    json!(entries)
}
