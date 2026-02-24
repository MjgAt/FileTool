//! Platform-specific [`WalkProvider`](crate::WalkProvider) implementation.
//!
//! On all targets the public type is [`PlatformProvider`], a zero-size unit
//! struct.  The backing implementation differs:
//!
//! * **Windows** — `Win32WalkProvider`: opens every path with `CreateFileW`
//!   and reads `BY_HANDLE_FILE_INFORMATION` in a single syscall, providing
//!   `file_id`, `parent_file_id`, `hard_link_count`, `volume_serial`, and
//!   `win32_attributes` with no additional overhead beyond what `walkdir`
//!   already touches.
//!
//! * **Unix** — `UnixWalkProvider`: uses `std::os::unix::fs::MetadataExt` to
//!   read `ino`, `nlink`, and `dev`; no unsafe code required.
//!
//! Both delegate directory traversal to [`walkdir`] and glob walking to
//! [`globwalk`].

#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod unix;

// Re-export the right provider as the single public name.
#[cfg(windows)]
pub use windows::Win32WalkProvider as PlatformProvider;

#[cfg(unix)]
pub use unix::UnixWalkProvider as PlatformProvider;

// Fallback for non-windows, non-unix targets (e.g. wasm) — minimal impl.
#[cfg(not(any(windows, unix)))]
pub use fallback::FallbackWalkProvider as PlatformProvider;

#[cfg(not(any(windows, unix)))]
mod fallback {
    use crate::{EntryInfo, WalkProvider};
    use std::path::Path;

    pub struct FallbackWalkProvider;

    impl WalkProvider for FallbackWalkProvider {
        fn walk(&self, root: &Path, follow_links: bool) -> Box<dyn Iterator<Item = EntryInfo>> {
            use walkdir::WalkDir;
            let root = root.to_path_buf();
            let entries: Vec<EntryInfo> = WalkDir::new(&root)
                .follow_links(follow_links)
                .into_iter()
                .filter_map(|e| e.ok())
                .map(|e| {
                    let path = e.path().to_path_buf();
                    let rel_path = path.strip_prefix(&root).unwrap_or(&path).to_path_buf();
                    let is_dir = e.file_type().is_dir();
                    let size = e.metadata().ok().and_then(|m| if m.is_file() { Some(m.len()) } else { None });
                    EntryInfo {
                        path,
                        rel_path,
                        is_dir,
                        size,
                        file_id: None,
                        parent_file_id: None,
                        hard_link_count: None,
                        volume_serial: None,
                    }
                })
                .collect();
            Box::new(entries.into_iter())
        }

        fn walk_glob(&self, base: &Path, pattern: &str, follow_links: bool) -> Box<dyn Iterator<Item = EntryInfo>> {
            use globwalk::GlobWalkerBuilder;
            let base = base.to_path_buf();
            let entries: Vec<EntryInfo> = match GlobWalkerBuilder::from_patterns(&base, &[pattern])
                .follow_links(follow_links)
                .build()
            {
                Ok(w) => w.filter_map(|e| e.ok()).map(|e| {
                    let path = e.path().to_path_buf();
                    let rel_path = path.strip_prefix(&base).unwrap_or(&path).to_path_buf();
                    let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    let size = e.metadata().ok().and_then(|m| if m.is_file() { Some(m.len()) } else { None });
                    EntryInfo {
                        path,
                        rel_path,
                        is_dir,
                        size,
                        file_id: None,
                        parent_file_id: None,
                        hard_link_count: None,
                        volume_serial: None,
                    }
                }).collect(),
                Err(_) => vec![],
            };
            Box::new(entries.into_iter())
        }
    }
}
