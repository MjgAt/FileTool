# Checklist: csvdb Schema Management (`--update-schema`)

## Goal
Add `--update-schema` and `--schema-path` flags to the base `fileutil` command.
When `--update-schema` is supplied, a `schema.csvdb.json` file is written to the
schema path (default: `./schema.csvdb.json`). The file records the canonical CSV
column layout for every built-in subtool. Stale or missing table entries are
replaced with the current definition; new tables are added; unrecognised tables
are preserved (so external plugins can register their own schemas).

## Tasks
- [x] Create `fileutil/src/schema.rs` — schema types (`ColumnDef`, `TableSchema`,
      `CsvdbSchema`) + `update_schema()` entry point
- [x] Add `pub fn csv_columns() -> Vec<ColumnDef>` to `commands/ls/mod.rs`
- [x] Add `pub fn csv_columns() -> Vec<ColumnDef>` to `commands/enum/mod.rs`
- [x] Expose `schema` module from `lib.rs`
- [x] Add `--update-schema` and `--schema-path` to `Cli` in `main.rs`; make
      subcommand optional; call `schema::update_schema()` when flag is set
- [x] Add integration tests for `--update-schema`
- [x] Build (debug + release) passes cleanly

---

## CSV datetime: omit timezone offset

**Rationale:** By CSV convention, datetime values are assumed to be in the
machine-local timezone.  Embedding the RFC 3339 offset on every cell is
redundant noise and breaks Excel/LibreOffice auto-parsing.
JSON and table output retain the full RFC 3339 string.

- [x] Add `strip_datetime_tz()` helper to `main.rs::render_csv()`; call it for
      every CSV string cell so any datetime-shaped value has its trailing `±HH:MM`
      stripped before writing
- [x] Document the decision in `fileutil/src/commands/DESIGN-NOTES.md`
- [x] Build passes cleanly, CSV output verified (offset absent, JSON/table unaffected)
