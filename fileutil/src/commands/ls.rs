use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use walkdir::WalkDir;

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
    let mut entries = Vec::new();
    if args.recursive {
        for entry in WalkDir::new(&args.path).into_iter().filter_map(|e| e.ok()) {
            let metadata = entry.metadata().ok();
            entries.push(FileEntry {
                path: entry.path().to_string_lossy().to_string(),
                is_dir: entry.file_type().is_dir(),
                size: metadata.and_then(|m| if m.is_file() { Some(m.len()) } else { None }),
            });
        }
    } else {
        if let Ok(dir_entries) = fs::read_dir(&args.path) {
            for entry in dir_entries.filter_map(|e| e.ok()) {
                let metadata = entry.metadata().ok();
                entries.push(FileEntry {
                    path: entry.path().to_string_lossy().to_string(),
                    is_dir: entry.file_type().ok().map(|ft| ft.is_dir()).unwrap_or(false),
                    size: metadata.and_then(|m| if m.is_file() { Some(m.len()) } else { None }),
                });
            }
        }
    }
    json!(entries)
}