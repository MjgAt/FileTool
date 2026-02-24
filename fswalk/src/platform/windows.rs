//! Windows (`Win32`) walk provider.
//!
//! Every path visited during a walk is opened with `CreateFileW` using
//! `FILE_FLAG_BACKUP_SEMANTICS` (required to open directories) and
//! `FILE_READ_ATTRIBUTES` (requires no special privilege).  A single call to
//! `GetFileInformationByHandle` then yields:
//!
//! * `nFileIndexHigh` / `nFileIndexLow` → [`EntryInfo::file_id`]
//! * `nNumberOfLinks` → [`EntryInfo::hard_link_count`]
//! * `dwVolumeSerialNumber` → [`EntryInfo::volume_serial`]
//! * `dwFileAttributes` → [`EntryInfo::win32_attributes`]
//!
//! The parent directory is opened with the same approach to populate
//! [`EntryInfo::parent_file_id`].
//!
//! If `CreateFileW` fails (e.g. access denied, path too long) every affected
//! field is set to `None` rather than aborting the walk.

use crate::{EntryInfo, WalkProvider};
use globwalk::GlobWalkerBuilder;
use std::path::Path;
use walkdir::WalkDir;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_READ_ATTRIBUTES,
    FILE_SHARE_READ, FILE_SHARE_WRITE, GetFileInformationByHandle, OPEN_EXISTING,
};
use windows::core::PCWSTR;

pub struct Win32WalkProvider;

impl WalkProvider for Win32WalkProvider {
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
    let rel_path = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    let meta = std::fs::metadata(path).ok();
    let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or_else(|| path.is_dir());
    let size = meta.as_ref().and_then(|m| if m.is_file() { Some(m.len()) } else { None });
    let created_at = meta.as_ref().and_then(|m| m.created().ok());
    let modified_at = meta.as_ref().and_then(|m| m.modified().ok());

    let (file_id, hard_link_count, volume_serial, win32_attributes) =
        query_handle_info(path).unwrap_or((None, None, None, None));

    let parent_file_id = path
        .parent()
        .and_then(|p| query_handle_info(p).ok())
        .and_then(|t| t.0);

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
        win32_attributes,
    }
}

/// Open `path` with `CreateFileW` and read `BY_HANDLE_FILE_INFORMATION`.
/// Returns `(file_id, hard_link_count, volume_serial, win32_attributes)`.
fn query_handle_info(
    path: &Path,
) -> windows::core::Result<(Option<u64>, Option<u32>, Option<u64>, Option<u32>)> {
    use std::os::windows::ffi::OsStrExt;

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            FILE_READ_ATTRIBUTES.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            Some(std::ptr::null()),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            HANDLE::default(),
        )?
    };

    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    let ok = unsafe { GetFileInformationByHandle(handle, &mut info).is_ok() };
    unsafe { windows::Win32::Foundation::CloseHandle(handle).ok() };

    if ok {
        let file_id =
            Some(((info.nFileIndexHigh as u64) << 32) | (info.nFileIndexLow as u64));
        let hard_link_count = Some(info.nNumberOfLinks);
        let volume_serial = Some(info.dwVolumeSerialNumber as u64);
        let win32_attributes = Some(info.dwFileAttributes);
        Ok((file_id, hard_link_count, volume_serial, win32_attributes))
    } else {
        Ok((None, None, None, None))
    }
}

