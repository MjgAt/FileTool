use chrono::{DateTime, Local};
use clap::Args;
use fswalk::{default_provider, WalkProvider};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::time::SystemTime;

use crate::schema::ColumnDef;

/// Canonical CSV column definitions for the `enum` subcommand.
/// Columns are listed in output order.
pub fn csv_columns() -> Vec<ColumnDef> {
    vec![
        ColumnDef::string("root"),
        ColumnDef::string("rel_path"),
        ColumnDef::string("filename"),
        ColumnDef::nullable_integer("parent_id"),
        ColumnDef::nullable_integer("id"),
        ColumnDef::nullable_integer("size"),
        ColumnDef::nullable_datetime("created_at"),
        ColumnDef::nullable_datetime("modified_at"),
    ]
}

/// Format a `SystemTime` as an RFC 3339 string in the local timezone, or `None` on failure.
fn fmt_time(t: Option<SystemTime>) -> Option<String> {
    t.map(|st| DateTime::<Local>::from(st).to_rfc3339())
}

#[derive(Args, Serialize, Deserialize, Debug)]
pub struct EnumArgs {
    /// Path or glob pattern to enumerate, relative to the current working directory.
    /// Plain paths (e.g. "src/") enumerate that directory recursively.
    /// Glob patterns (e.g. "**/*.rs", "src/**/*.toml") filter by pattern.
    /// Defaults to "." (the current working directory).
    #[arg(default_value = ".")]
    pub paths: Vec<String>,
    /// Follow symbolic links and junctions
    #[arg(long)]
    pub follow_links: bool,
}

#[derive(Serialize)]
struct FileEntry {
    root: String,
    rel_path: String,
    filename: String,
    parent_id: Option<u64>,
    id: Option<u64>,
    size: Option<u64>,
    created_at: Option<String>,
    modified_at: Option<String>,
}

/// Returns true when `s` contains any glob metacharacter.
fn has_glob_chars(s: &str) -> bool {
    s.contains(['*', '?', '[', '{'])
}

pub fn run(args: EnumArgs) -> Value {
    run_with_provider(args, &default_provider())
}

/// Testable core: accepts any [`WalkProvider`] implementation.
pub fn run_with_provider(args: EnumArgs, provider: &dyn WalkProvider) -> Value {
    let mut entries = Vec::new();
    for path_or_pattern in &args.paths {
        if has_glob_chars(path_or_pattern) {
            let base_str = glob_base(path_or_pattern);
            let base = Path::new(&base_str);
            let rel_pattern = glob_relative_pattern(path_or_pattern, &base_str);
            for entry in provider.walk_glob(base, &rel_pattern, args.follow_links) {
                entries.push(FileEntry {
                    root: path_or_pattern.clone(),
                    rel_path: entry.rel_path.to_string_lossy().to_string(),
                    filename: entry.file_name().to_string_lossy().to_string(),
                    parent_id: entry.parent_file_id,
                    id: entry.file_id,
                    size: entry.size,
                    created_at: fmt_time(entry.created_at),
                    modified_at: fmt_time(entry.modified_at),
                });
            }
        } else {
            for entry in provider.walk(Path::new(path_or_pattern), args.follow_links) {
                entries.push(FileEntry {
                    root: path_or_pattern.clone(),
                    rel_path: entry.rel_path.to_string_lossy().to_string(),
                    filename: entry.file_name().to_string_lossy().to_string(),
                    parent_id: entry.parent_file_id,
                    id: entry.file_id,
                    size: entry.size,
                    created_at: fmt_time(entry.created_at),
                    modified_at: fmt_time(entry.modified_at),
                });
            }
        }
    }
    json!(entries)
}

/// Return the longest leading path component that contains no glob metacharacters.
/// E.g. "src/foo/**/*.rs" → "src/foo", "**/*.rs" → ".".
fn glob_base(pattern: &str) -> String {
    let parts: Vec<&str> = pattern.split(['/', '\\']).collect();
    let mut base_parts: Vec<&str> = Vec::new();
    for part in &parts {
        if has_glob_chars(part) {
            break;
        }
        base_parts.push(part);
    }
    let base = base_parts.join("/");
    if base.is_empty() || base == "." {
        ".".to_string()
    } else {
        base
    }
}

/// Strip the `base/` prefix from `pattern` to get a pattern relative to the base.
/// E.g. pattern "src/**/*.rs", base "src" → "**/*.rs".
fn glob_relative_pattern(pattern: &str, base: &str) -> String {
    if base == "." {
        return pattern.to_string();
    }
    let prefix = format!("{}/", base.replace('\\', "/"));
    let normalized = pattern.replace('\\', "/");
    if normalized.starts_with(&prefix) {
        normalized[prefix.len()..].to_string()
    } else {
        pattern.to_string()
    }
}
