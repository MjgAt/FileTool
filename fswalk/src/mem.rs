//! In-memory [`WalkProvider`](crate::WalkProvider) for unit testing.
//!
//! # Usage
//!
//! ```rust
//! use std::path::PathBuf;
//! use fswalk::{EntryInfo, WalkProvider, mem::MemWalkProvider};
//!
//! let entries = vec![
//!     EntryInfo {
//!         path: PathBuf::from("/root/a.rs"),
//!         rel_path: PathBuf::from("a.rs"),
//!         is_dir: false,
//!         size: Some(1024),
//!         file_id: Some(1),
//!         parent_file_id: Some(0),
//!         hard_link_count: Some(1),
//!         volume_serial: Some(42),
//!         created_at: None,
//!         modified_at: None,
//!         #[cfg(windows)]
//!         win32_attributes: None,
//!     },
//! ];
//!
//! let provider = MemWalkProvider::new(entries);
//! let root = std::path::Path::new("/root");
//! let results: Vec<_> = provider.walk(root, false).collect();
//! assert_eq!(results.len(), 1);
//! ```
//!
//! # Glob filtering
//!
//! [`MemWalkProvider::walk_glob`] filters the stored entries using
//! [`globset`], matching each entry's `rel_path` against the pattern.
//! This mirrors the behaviour of the production glob provider without
//! touching the disk.

use crate::{EntryInfo, WalkProvider};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

/// An in-memory walk provider that returns a fixed set of [`EntryInfo`]
/// values.  Intended exclusively for unit tests.
#[derive(Clone)]
pub struct MemWalkProvider {
    entries: Vec<EntryInfo>,
}

impl MemWalkProvider {
    /// Create a provider from an explicit list of entries.
    pub fn new(entries: Vec<EntryInfo>) -> Self {
        Self { entries }
    }

    /// Build a provider from a simple list of `(path, is_dir, size)` tuples,
    /// with synthetic sequential file IDs starting from 1.
    ///
    /// `root` is used to compute `rel_path`; entries whose path does not start
    /// with `root` get `rel_path == path`.
    pub fn from_tuples(root: &Path, entries: &[(&str, bool, Option<u64>)]) -> Self {
        let infos = entries
            .iter()
            .enumerate()
            .map(|(i, (p, is_dir, size))| {
                let path = std::path::PathBuf::from(p);
                let rel_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
                let parent_file_id = if i == 0 { None } else { Some(0u64) };
                EntryInfo {
                    path,
                    rel_path,
                    is_dir: *is_dir,
                    size: *size,
                    file_id: Some(i as u64 + 1),
                    parent_file_id,
                    hard_link_count: Some(1),
                    volume_serial: Some(1),
                    created_at: None,
                    modified_at: None,
                    #[cfg(windows)]
                    win32_attributes: None,
                }
            })
            .collect();
        Self { entries: infos }
    }
}

impl WalkProvider for MemWalkProvider {
    /// Return all stored entries whose `path` starts with `root`.
    fn walk(&self, root: &Path, _follow_links: bool) -> Box<dyn Iterator<Item = EntryInfo>> {
        let root = root.to_path_buf();
        let results: Vec<EntryInfo> = self
            .entries
            .iter()
            .filter(move |e| e.path.starts_with(&root))
            .cloned()
            .collect();
        Box::new(results.into_iter())
    }

    /// Return stored entries whose `rel_path` (relative to `base`) matches
    /// `pattern`, using [`globset`] for matching.
    fn walk_glob(
        &self,
        base: &Path,
        pattern: &str,
        _follow_links: bool,
    ) -> Box<dyn Iterator<Item = EntryInfo>> {
        let glob_set = match build_glob_set(pattern) {
            Some(g) => g,
            None => return Box::new(std::iter::empty()),
        };
        let base = base.to_path_buf();
        let results: Vec<EntryInfo> = self
            .entries
            .iter()
            .filter(|e| {
                if !e.path.starts_with(&base) {
                    return false;
                }
                let rel = e.path.strip_prefix(&base).unwrap_or(&e.path);
                glob_set.is_match(rel)
            })
            .cloned()
            .collect();
        Box::new(results.into_iter())
    }
}

fn build_glob_set(pattern: &str) -> Option<GlobSet> {
    let glob = Glob::new(pattern).ok()?;
    let mut builder = GlobSetBuilder::new();
    builder.add(glob);
    builder.build().ok()
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_provider() -> MemWalkProvider {
        MemWalkProvider::from_tuples(
            Path::new("/repo"),
            &[
                ("/repo", true, None),
                ("/repo/src", true, None),
                ("/repo/src/main.rs", false, Some(512)),
                ("/repo/src/lib.rs", false, Some(256)),
                ("/repo/Cargo.toml", false, Some(128)),
                ("/repo/tests/cli.rs", false, Some(1024)),
            ],
        )
    }

    #[test]
    fn walk_returns_all_under_root() {
        let p = make_provider();
        let count = p.walk(Path::new("/repo"), false).count();
        assert_eq!(count, 6);
    }

    #[test]
    fn walk_filters_by_root() {
        let p = make_provider();
        let count = p.walk(Path::new("/repo/src"), false).count();
        assert_eq!(count, 3); // /repo/src, main.rs, lib.rs
    }

    #[test]
    fn walk_glob_rs_files() {
        let p = make_provider();
        let entries: Vec<EntryInfo> = p
            .walk_glob(Path::new("/repo"), "**/*.rs", false)
            .collect();
        assert_eq!(entries.len(), 3); // main.rs, lib.rs, cli.rs
        for e in &entries {
            assert!(e.path.extension().map(|x| x == "rs").unwrap_or(false));
        }
    }

    #[test]
    fn walk_glob_toml_files() {
        let p = make_provider();
        let entries: Vec<EntryInfo> = p
            .walk_glob(Path::new("/repo"), "**/*.toml", false)
            .collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, PathBuf::from("/repo/Cargo.toml"));
    }

    #[test]
    fn file_ids_are_set() {
        let p = make_provider();
        let entries: Vec<EntryInfo> = p.walk(Path::new("/repo"), false).collect();
        assert!(entries.iter().all(|e| e.file_id.is_some()));
    }
}
