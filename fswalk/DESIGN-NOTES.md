# Design Notes — `fswalk`

## Purpose

`fswalk` decouples filesystem traversal from the command modules (`enum`, `ls`)
so that:

1. Unit tests can inject an in-memory provider (`MemWalkProvider`) without
   touching the disk.
2. The production path gets richer metadata "for free" from the same handle
   already opened for the file ID.
3. Future commands can share the traversal layer without duplicating platform
   code.

## Core type: `EntryInfo`

A plain struct (not a trait) returned by every walk iteration.  All fields
beyond `path` / `rel_path` / `is_dir` are `Option<_>` because:

* The in-memory provider sets them to synthetic values or `None`.
* The real provider may fail to open a path (access denied, path too long) and
  must degrade gracefully rather than aborting the walk.

Windows-only fields (`win32_attributes`) are gated with `#[cfg(windows)]` on
the struct field itself, which is stable Rust since 1.54.

## Trait: `WalkProvider`

Two methods:
- `walk(root, follow_links)` — full recursive descent.
- `walk_glob(base, pattern, follow_links)` — filter by glob pattern relative
  to `base`.

Both return `Box<dyn Iterator<Item = EntryInfo>>`.  Using a `Box<dyn Iterator>`
rather than an associated type avoids propagating a generic parameter through
the entire command call stack while keeping the API object-safe.  The perf cost
is negligible for a CLI tool.

## Platform dispatch

`platform::PlatformProvider` is a unit struct type-aliased from the right
platform module:

| Target | Concrete type | Metadata source |
|---|---|---|
| Windows | `Win32WalkProvider` | `CreateFileW` + `GetFileInformationByHandle` |
| Unix | `UnixWalkProvider` | `std::os::unix::fs::MetadataExt` |
| Other | `FallbackWalkProvider` | walkdir metadata only |

The Windows provider opens two handles per entry (one for the file, one for
its parent directory) to populate `file_id` and `parent_file_id`.  This
matches the per-entry overhead of the previous inline implementation in
`enum/mod.rs`.

## `MemWalkProvider`

Holds a `Vec<EntryInfo>`.  `walk(root)` filters by `path.starts_with(root)`;
`walk_glob(base, pattern)` additionally filters by `globset` on `rel_path`.
Both return a cloned subset — intentionally simple.

`from_tuples` is a convenience constructor that assigns sequential synthetic
file IDs starting from 1, making assertions in tests straightforward.

## Adding a new command

1. Write the command's `run_with_provider(args, &dyn WalkProvider)` function.
2. `run(args)` just calls `run_with_provider(args, &fswalk::default_provider())`.
3. Unit tests pass a `MemWalkProvider::from_tuples(...)`.

No changes to `fswalk` itself are required for new commands.
