//! `fswalk` — cross-platform filesystem-walking abstraction.
//!
//! # Overview
//!
//! Provides a [`WalkProvider`] trait that decouples callers from the concrete
//! I/O mechanism used to traverse a directory tree.  Two production
//! implementations are shipped:
//!
//! * [`platform::PlatformProvider`] — the default.  On Windows it uses
//!   `CreateFileW` / `GetFileInformationByHandle` to gather Win32-level
//!   metadata (file index, hard-link count, volume serial, attributes).  On
//!   Unix it uses `MetadataExt` for inode, nlink, and device number.  Both
//!   delegate directory traversal to [`walkdir`].
//!
//! * [`mem::MemWalkProvider`] — an in-memory fake.  Callers build a
//!   `Vec<EntryInfo>` and inject it; no disk I/O at all.  Intended for unit
//!   tests.
//!
//! Glob pattern walking (`**/*.rs`, `src/**`) is supported via an additional
//! [`WalkProvider::walk_glob`] method.  The production implementation
//! delegates to [`globwalk`]; the in-memory implementation filters the entry
//! list with [`globset`].

pub mod mem;
pub mod platform;

use std::path::{Path, PathBuf};

// ── Core data type ────────────────────────────────────────────────────────────

/// All metadata collected for a single filesystem entry during a walk.
///
/// Fields that cannot be populated (either because the OS returned an error or
/// because the provider is the in-memory fake) are `None`.
#[derive(Debug, Clone)]
pub struct EntryInfo {
    /// Absolute path to the entry, as returned by the OS.
    pub path: PathBuf,
    /// Path relative to the walk root (empty for the root itself).
    pub rel_path: PathBuf,
    /// `true` when the entry is a directory.
    pub is_dir: bool,
    /// File size in bytes, `None` for directories.
    pub size: Option<u64>,
    /// Platform-unique file identifier (inode on Unix, `nFileIndex` on
    /// Windows).
    pub file_id: Option<u64>,
    /// File identifier of the *parent* directory.
    pub parent_file_id: Option<u64>,
    /// Number of hard links.
    pub hard_link_count: Option<u32>,
    /// Volume / device identifier on which the file resides.
    /// On Windows: `dwVolumeSerialNumber`.  On Unix: `dev_t`.
    pub volume_serial: Option<u64>,

    /// Creation time of the entry, as reported by the OS.
    /// `None` when unavailable (e.g. some Linux filesystems do not expose
    /// birth time; the in-memory provider always returns `None`).
    pub created_at: Option<std::time::SystemTime>,
    /// Last-modification time of the entry's data.
    pub modified_at: Option<std::time::SystemTime>,

    // ── Windows-only fields ───────────────────────────────────────────────
    /// Raw `dwFileAttributes` bitmask (e.g. hidden, system, sparse,
    /// compressed, reparse-point …).
    #[cfg(windows)]
    pub win32_attributes: Option<u32>,
}

impl EntryInfo {
    /// Convenience accessor: the file name component of [`EntryInfo::path`].
    pub fn file_name(&self) -> &std::ffi::OsStr {
        self.path.file_name().unwrap_or_default()
    }

    /// `true` when the entry has no file-name component (i.e. is a root).
    pub fn is_root_entry(&self) -> bool {
        self.rel_path.as_os_str().is_empty()
    }
}

// ── Trait ─────────────────────────────────────────────────────────────────────

/// Abstraction over filesystem directory-walking operations.
///
/// Implementations are expected to be cheap to clone / store (e.g. a
/// zero-size unit struct for the platform provider, a `Vec` wrapper for the
/// in-memory provider).
pub trait WalkProvider: Send + Sync {
    /// Walk `root` recursively, yielding one [`EntryInfo`] per filesystem
    /// entry (including the root itself).
    ///
    /// The order of entries is unspecified.
    fn walk(&self, root: &Path, follow_links: bool) -> Box<dyn Iterator<Item = EntryInfo>>;

    /// Walk entries whose paths (relative to `base`) match `pattern`.
    ///
    /// `pattern` uses the same glob syntax as [`globwalk`]:
    /// `**/*.rs`, `src/**`, `foo/{a,b}.txt`, etc.
    ///
    /// The `root` stored in each returned entry is set to `pattern`
    /// (the original glob string).
    fn walk_glob(
        &self,
        base: &Path,
        pattern: &str,
        follow_links: bool,
    ) -> Box<dyn Iterator<Item = EntryInfo>>;
}

// ── Convenience constructor ───────────────────────────────────────────────────

/// Return the default platform provider for the current target.
///
/// On Windows this is [`platform::PlatformProvider`] (Win32-backed).
/// On Unix this is also [`platform::PlatformProvider`] (MetadataExt-backed).
pub fn default_provider() -> platform::PlatformProvider {
    platform::PlatformProvider
}
