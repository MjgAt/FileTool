use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use walkdir::WalkDir;
use windows::Win32::Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, FILE_READ_ATTRIBUTES, FILE_FLAG_BACKUP_SEMANTICS};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HANDLE;


#[derive(Args, Serialize, Deserialize, Debug)]
pub struct EnumArgs {
    /// Root directories to enumerate
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
}

pub fn run(args: EnumArgs) -> Value {
    let mut entries = Vec::new();
    for root in &args.paths {
        let root_path = Path::new(root);
        let walker = WalkDir::new(root).follow_links(args.follow_links);
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let rel_path = path.strip_prefix(root_path).unwrap_or(path).to_string_lossy().to_string();
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let parent_id = if let Some(parent) = path.parent() {
                get_file_id(parent)
            } else {
                None
            };
            let id = get_file_id(path);
            let size = entry.metadata().ok().and_then(|m| if m.is_file() { Some(m.len()) } else { None });
            entries.push(FileEntry {
                root: root.clone(),
                rel_path,
                filename,
                parent_id,
                id,
                size,
            });
        }
    }
    json!(entries)
}

fn get_file_id(path: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let handle = match unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_READ_ATTRIBUTES.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            Some(std::ptr::null()),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            HANDLE::default(),
        )
    } {
        Ok(h) => h,
        Err(_) => return None,
    };
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    let success = unsafe { GetFileInformationByHandle(handle, &mut info).is_ok() };
    unsafe { windows::Win32::Foundation::CloseHandle(handle).ok() };
    if success {
        Some(((info.nFileIndexHigh as u64) << 32) | (info.nFileIndexLow as u64))
    } else {
        None
    }
}