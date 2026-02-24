# Design Notes for commands/

## Submodule Naming Convention

Submodules under `commands/` are named to match their corresponding subcommand names for consistency and clarity. For example, the `ls` subcommand uses `commands/ls/`, and the `enum` subcommand uses `commands/enum/`.

If a subcommand name conflicts with a Rust keyword (e.g., `enum`), use a raw identifier (`r#enum`) in module declarations and imports to avoid syntax errors, while keeping the directory and file names keyword-free.

This convention ensures that the codebase structure reflects the CLI structure, making it easier to navigate and maintain.

## Schema Management (`--update-schema` / `--schema-path`)

The `schema` module (`src/schema.rs`) owns all csvdb schema types and update logic.

- `ColumnDef` / `TableSchema` / `CsvdbSchema` are the serialisable schema types (round-trippable JSON).
- Each command module exposes `pub fn csv_columns() -> Vec<ColumnDef>` — the single source of truth for that tool's CSV output columns. Column order matches the alphabetical BTreeMap serialisation order used by serde_json.
- `schema::builtin_tables()` collects all command definitions into one place. Adding a new command = add it here and add `csv_columns()` to its module.
- `schema::update_schema(path)` loads an existing file (if any), compares built-in tables by equality, replaces stale/absent entries, leaves unknown tables untouched, and writes only if something changed.
- `--update-schema` may be used standalone (no subcommand required) or alongside a normal subcommand run. Progress is reported on stderr so it never pollutes CSV/JSON stdout.

## DateTime Formatting — CSV vs JSON/Table

Datetime fields (e.g. `created_at`, `modified_at`) are stored internally as
`std::time::SystemTime` in `EntryInfo` and converted to RFC 3339 strings with
a local-timezone offset (e.g. `2026-02-23T22:45:31.961327-05:00`) for JSON
and table output.

For **CSV output**, the timezone offset is stripped by `strip_datetime_tz()` in
`main.rs::render_csv()`.  CSV datetime values are, by convention, assumed to be
in the local locale: spreadsheet applications (Excel, LibreOffice Calc) do not
parse the RFC 3339 offset and will either reject the field or treat it as a
plain string.  Because every row in a given export implicitly shares the same
timezone (the machine-local one at time of export), embedding the offset is
redundant noise that hurts interoperability.

The stripping is a pure rendering concern and lives in the output layer, not in
the command modules.  JSON and table renderers are unaffected.

The `enum` command accepts one or more positional arguments (`paths`). Each argument is handled by a dual-dispatch strategy:

- **Plain path** (no glob metacharacters `* ? [ {`): Treated as a directory root and walked recursively using `walkdir::WalkDir`. This preserves the original Windows-level file-ID enumeration via `GetFileInformationByHandle`, which does not exist in the globwalk stack.
- **Glob pattern** (contains at least one metacharacter): Walked using the `globwalk` crate (`GlobWalkerBuilder`). The non-glob leading directory segments of the pattern are extracted as the *base directory*; the remainder is passed as the glob pattern relative to that base. For example, `fileutil/**/*.rs` → base `fileutil`, relative pattern `**/*.rs`.

The `root` field in each output record is set to the original argument as typed by the user (not normalized), so callers can correlate output rows with their input patterns.
