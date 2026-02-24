# Completed Plans

| Path to CHECKLIST.md | Completion Date | Brief description | Design Notes |
|---|---|---|---|
| fileutil/src/commands/enum/CHECKLIST.md | 2026/02/23 | Added glob pattern support (`**/*.rs`, `src/**`, etc.) to `fileutil enum` via dual-dispatch: plain paths use WalkDir, glob patterns use globwalk | fileutil/src/commands/DESIGN-NOTES.md |
| fileutil/CHECKLIST.md | 2026/02/23 | Added `--update-schema` / `--schema-path` flags; schema.rs module; csv_columns() per command; integration tests | fileutil/src/commands/DESIGN-NOTES.md |
| fswalk/CHECKLIST.md | 2026/02/23 | New `fswalk` crate: WalkProvider trait, EntryInfo, Win32/Unix/Fallback/Mem providers; fileutil enum+ls ported to use fswalk | fswalk/DESIGN-NOTES.md |
| fileutil/CHECKLIST.md | 2026/02/23 | CSV datetime: strip timezone offset in render_csv; JSON/table unaffected. Design note added. | fileutil/src/commands/DESIGN-NOTES.md |
