//! Unix walk provider.
//!
//! Uses [`walkdir`] for traversal and `std::os::unix::fs::MetadataExt` for
//! the platform-level metadata fields:
//!
//! * `ino()` → [`EntryInfo::file_id`]
//! * `nlink()` → [`EntryInfo::hard_link_count`]
//! * `dev()` → [`EntryInfo::volume_serial`]
//!
//! Separately, the parent directory's `ino()` is used for
//! [`EntryInfo::parent_file_id`].  Because `walkdir` already calls `stat(2)`
//! internally, the parent open is the only extra syscall per entry.

use crate::{EntryInfo, WalkProvider};
use globwalk::GlobWalkerBuilder;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct UnixWalkProvider;

impl WalkProvider for UnixWalkProvider {
    fn walk(&self, root: &Path, follow_links: bool) -> Box<dyn Iterator<Item = EntryInfo>> {
        let root = root.to_path_buf();
        let entries: Vec<EntryInfo> = WalkDir::new(&root)
            .follow_links(follow_links)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| build_entry(e.path(), &root))
            .collect();
        Box::new(entries.into_iter())
    }

    fn walk_glob(
        &self,
        base: &Path,
        pattern: &str,
        follow_links: bool,
    ) -> Box<dyn Iterator<Item = EntryInfo>> {
        let base = base.to_path_buf();
        let entries: Vec<EntryInfo> = match GlobWalkerBuilder::from_patterns(&base, &[pattern])
            .follow_links(follow_links)
            .build()
        {
            Ok(w) => w
                .filter_map(|e| e.ok())
                .map(|e| build_entry(e.path(), &base))
                .collect(),
            Err(_) => vec![],
        };
        Box::new(entries.into_iter())
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn build_entry(path: &Path, root: &Path) -> EntryInfo {
    use std::os::unix::fs::MetadataExt;

    let rel_path = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    let meta = std::fs::metadata(path).ok();
    let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
    let size = meta
        .as_ref()
        .and_then(|m| if m.is_file() { Some(m.len()) } else { None });

    let (file_id, hard_link_count, volume_serial) =
        meta.as_ref().map(|m| {
            (
                Some(m.ino()),
                Some(m.nlink() as u32),
                Some(m.dev()),
            )
        }).unwrap_or((None, None, None));

    let parent_file_id = path
        .parent()
        .and_then(|p| std::fs::metadata(p).ok())
        .map(|m| m.ino());

    let created_at = meta.as_ref().and_then(|m| m.created().ok());
    let modified_at = meta.as_ref().and_then(|m| m.modified().ok());

    EntryInfo {
        path: path.to_path_buf(),
        rel_path,
        is_dir,
        size,
        file_id,
        parent_file_id,
        hard_link_count,
        volume_serial,
        created_at,
        modified_at,
    }
}
