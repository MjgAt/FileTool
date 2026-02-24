# Copilot Instructions

# Terminal / Git rules — hang prevention

**These rules prevent terminal hangs that freeze the session.**

- Every `git` command that can produce paged output **must** be run with
  `git --no-pager <subcommand>`. This includes (but is not limited to)
  `diff`, `show`, `log`, `blame`, `reflog`, `stash list`, `branch -v`.
- Never run `git commit` without `-m "…"`.
- Never run `git pull` or `git merge` without `--no-edit`.
- Never run interactive commands: `git rebase -i`, `git add -p`, etc.
- Do not use `less`, `more`, or any other interactive pager.
- Never use PowerShell multi-line string operators (`@"…"@`) in terminal commands.

## Interaction Guidelines
- Prefer concise responses: minimize verbosity, reduce repetition, and avoid excessive formatting/emojis. Get straight to the point in all interactions.

## Validation and green-before-done
- After any substantive change, run the relevant build/tests/linters automatically. For runnable code that you created or edited, immediately run a test to validate the code works (fast, minimal input) yourself. Prefer automated code-based tests where possible. Then provide optional fenced code blocks with commands for larger or platform-specific runs. Don't end a turn with a broken build if you can fix it. If failures occur, iterate up to three targeted fixes; if still failing, summarize the root cause, options, and exact failing output.
- Run all tests, including integration tests that require binaries: After building the project, run `cargo test` to execute unit tests, and separately ensure integration tests pass by building any required binaries first (e.g., `cargo build --bin <name>` followed by `cargo test`). For projects with CLI binaries, validate integration tests that exercise the full application.

## Design Autonomy — Behavior is owned, never inherited from dependencies

We **define** our behavior. We **choose** dependencies that can satisfy our definition.

It is never acceptable to describe our behavior as "whatever crate X does" or "we delegate to
library Y." That framing surrenders our autonomy to decide what is correct for our users and makes
it impossible to reason about correctness, versioning risk, or future migration.

The correct framing is always:
1. State **what our specified behavior is** (inputs we accept, outputs we produce, errors we raise).
2. Note **which dependency is used to achieve it** and that the dependency was chosen because its
   behavior matches our specification.
3. If a dependency's actual behavior diverges from our specification, the dependency is wrong,
   not our specification. We either constrain the dependency, wrap it, or replace it.

We may align our specification with a dependency's behavior when that behavior is sensible for our
users — but the specification must still be written down explicitly and owned by us. When a
dependency is upgraded or replaced, our specification does not change; only the implementation does.

This applies everywhere: file formats, parse rules, error messages, wire protocols, encoding choices.

# Source-Components

- Source-Components are directory hierarchies in the repository rooted at some directory.
- Source-Components are identified by the presence of either a Cargo.toml file or a COMPONENT.md file in the directory.
- The root of the repository contains a Cargo.toml file, so the entire repository is a source-component, but there are also smaller source-components within the repository which may have their own Cargo.toml or COMPONENT.md files.

Examples:
- `src/tools/csv/` (has COMPONENT.md)
- `src/tools/csv/csv/` (has Cargo.toml)

# Always plan
- Always form a plan in the form of a CHECKLIST.md, at the lowest common source-component for the change
- Keep the plan up to date as you execute on the plan
- Keep a file at the root of the repository, called PLANS.md, which tracks all the CHECKLIST.md files in the repository and their status (not started, in progress, completed). If it does not exist, create it. If it does exist, update it with the new CHECKLIST.md file and its status.
- When a CHECKLIST.md file is completed, move it to a table in a different file called COMPLETED-PLANS.md in the same directory, with a brief summary of the work completed, and remove it from PLANS.md.

PLANS.md format (markdown table):
| Path to CHECKLIST.md | Status | Brief description | Design Notes |
|---|---|---|---|

COMPLETED-PLANS.md format (markdown table):
| Path to CHECKLIST.md | Completion Date | Brief description | Design Notes |
|---|---|---|---|

Status values: "not started", "in progress", "completed"

Design Notes column: Path(s) to DESIGN-NOTES.md file(s) that document the work, or "N/A" if none exist

# Plan sizing

When a plan starts to take over 2 minutes to form be sure to have a checkpoint of the plan into a
CHECKLIST.md file that is available in the repo and is not lost if the copilot session is lost.



# Design note files

Any directory in the repository may have a DESIGN-NOTES.md file.

The DESIGN-NOTES.md file should record design decisions about the code in that directory and its children.

If a decision should be recorded, it should be recorded in a DESIGN-NOTES.md file. The DESIGN-NOTES.md
file to use is either the DESIGN-NOTES.md file in the source-component directory which should be created
if it does not already exist, or if there is an already existing DESIGN-NOTES.md file in any ancestor
directory between the file being changed and the source-component root, use that one instead.

## What to include

The design note files should include anything that a future developer should or may want to know about the
code to help them "get up to speed" or diagnose interesting or bad behaviors.

## What not to include

Like with code comments, don't include super obvious things.

Example: A query processor design note must describe its intent and unique approach in a paragraph, not provide a comprehensive tutorial on the underlying technology or theory. It may include links to external resources for further reading, but should not attempt to teach the reader about query processing in general.

## Historical Record

As features age out of a source-component, at the very least, move notes which are no longer relevant to a
different file, DESIGN-NOTES-AGED-OUT.md.

When moving the section to DESIGN-NOTES-AGED-OUT.md, include the date of the move, in YYYY/MM/DD format.

# When it a work item done

A work item is done when all the text is complete, the build passes with clean builds,
release and debug, and the tests pass, for unit tests, benchmarks, and integration tests.

# csvdb JSON Schema Format

A `.csvdb.json` file maps CSV file name stems to typed column layouts so that
`csvdb` can parse, query, and validate them.  Place the file in the same
directory as your CSV files (or any ancestor directory).

---

## Top-level structure

```json
{
  "version": "1.0",
  "sources": { ... },
  "layouts": { ... }
}
```

| Field | Required | Description |
|---|---|---|
| `version` | yes | Always `"1.0"` |
| `layouts` | yes | Named layout definitions (see below) |
| `sources` | no | Maps filename stems to layouts; inferred from layout names when absent |

---

## `sources`

Each key is a filename stem (no extension) or a glob pattern; each value is a
layout name string (shorthand) or a full object:

```json
"sources": {
  "q1_sales":  "sales",
  "q2_sales":  "sales",
  "archive_*": "sales",
  "special": { "layout": "sales" }
}
```

- Keys are matched in insertion order; **first match wins**.
- Glob matching is case-insensitive on Windows, case-sensitive on Unix.
- When `sources` is absent, a CSV file whose stem exactly equals a layout name
  is bound to that layout automatically.

---

## `layouts`

Each entry describes the structure of one class of CSV file.

### Simple layout

```json
"sales": {
  "format": "simple",
  "header": "present",
  "columns": {
    "id":         "Integer",
    "amount":     "Decimal",
    "sale_date":  { "type": "DateTime", "format": "iso8601" },
    "notes":      { "type": "String", "description": "Optional" }
  }
}
```

| Field | Default | Values |
|---|---|---|
| `format` | — | `"simple"` or `"tagged"` (required) |
| `header` | `"present"` | `"present"` · `"absent"` · `"ignore"` |
| `delimiter` | `","` | Any single character |
| `null_markers` | `[""]` | Array of strings treated as NULL |
| `encoding` | `"utf8"` | `"utf8"` · `"utf16"` · `"ascii"` · `"latin1"` |

### Column types

Columns are declared as a shorthand type string or a full object:

```json
"id":        "Integer",
"amount":    { "type": "Decimal", "description": "Sale total" }
```

| Type | Description |
|---|---|
| `"Integer"` | 64-bit signed integer |
| `"Float"` | 64-bit IEEE 754 double |
| `"Decimal"` | Arbitrary-precision decimal |
| `"String"` | Unicode text (whitespace-trimmed by default) |
| `"DateTime"` | Date/time value |
| `"Boolean"` | `true`/`false` |
| `"Guid"` | RFC 4122 UUID |
| `"General"` | Untyped; stored as-is |

**Column properties (full form)**

| Property | Default | Description |
|---|---|---|
| `type` | — | Type name from the table above (required) |
| `description` | — | Documentation string |
| `trim_csv` | `true` | Strip leading/trailing whitespace from raw cell |
| `format` | automatic | Type-specific parse/format hint (see below) |
| `default` | — | Value emitted for NULL/missing cells |

**`format` hints by type**

- `DateTime`: `"iso8601"` · `"rfc3339"` · `"custom:%Y-%m-%d %H:%M:%S"` · `"automatic"` (default — tries common ISO variants)
- `Boolean`: `"true_false"` · `"1_0"` · `"yes_no"` · `"custom:Y|N"` · `"automatic"` (default)
- `Float`: `"automatic"` (default, Excel General rules) · `"ryu"` · `"hex"` · `"scientific"`
- `Decimal`: `{ "radix_point": "," }` for locales that use `,` as the decimal separator

---

### Tagged layout

A tagged layout is a single CSV file where the **first column** selects the
schema for that row.  Each distinct tag value corresponds to a separate simple
child layout.

```json
"xperf": {
  "format": "tagged",
  "header": "absent",
  "tag_column": 0,
  "tag_column_name": "record_type",
  "tag_match": "case_insensitive",
  "tags": [
    { "name": "DPC",       "value": "DPC",       "layout": "xperf_dpc" },
    { "name": "Interrupt", "value": "Interrupt",  "layout": "xperf_interrupt" },
    { "name": "T_Start",   "value": "T-Start",    "layout": "xperf_thread" },
    { "name": "T_End",     "value": "T-End",      "layout": "xperf_thread" }
  ]
}
```

| Field | Default | Description |
|---|---|---|
| `tag_column` | `0` | Zero-based index of the tag column (currently must be 0) |
| `tag_column_name` | `"record_type"` | Name of the tag column in query results |
| `tag_match` | `"case_sensitive"` | `"case_sensitive"` or `"case_insensitive"` |
| `tags` | — | Array of tag entries (see below) |

**Tag entry fields**

| Field | Description |
|---|---|
| `name` | Logical name used in queries (`source.DPC`) |
| `value` | Raw string that appears in the CSV file's tag column (whitespace is trimmed before comparison) |
| `layout` | Name of the simple child layout to apply; must exist in `layouts` |

Multiple tag entries may share the same `layout` (e.g. `T-Start` and `T-End`
both map to `xperf_thread`).

Child layouts for tagged files use `"header": "absent"` and define only the
**data columns** — the tag column is not listed and is never included in the
child layout's `columns`.

The query-visible schema is the union of all child layouts' columns with the
tag column prepended.  Columns absent from a given record type emit
`Value::DbNull` for that record type's rows.

**Querying tagged slices**  
To read only one record type, use dot notation:

```
source.DPC | take 10
```

This emits only DPC rows and drops the tag column from output, exposing the
exact columns defined in the `DPC` child layout.

---

## Complete example — simple layouts

```json
{
  "version": "1.0",
  "sources": {
    "sales_2024_*": "sales",
    "customers":    "customers"
  },
  "layouts": {
    "sales": {
      "format": "simple",
      "header": "present",
      "columns": {
        "order_id":   "Integer",
        "customer_id":"Integer",
        "product":    "String",
        "quantity":   { "type": "Integer", "constraints": { "min": 1 } },
        "total":      "Decimal",
        "ordered_at": { "type": "DateTime", "format": "iso8601" },
        "shipped":    { "type": "Boolean",  "format": "1_0", "default": false }
      }
    },
    "customers": {
      "format": "simple",
      "header": "present",
      "columns": {
        "customer_id": "Integer",
        "email":       "String",
        "country":     { "type": "String", "default": "US" }
      }
    }
  }
}
```

## Complete example — tagged layout (xperf style)

```json
{
  "version": "1.0",
  "sources": {
    "my_trace": "xperf"
  },
  "layouts": {
    "xperf": {
      "format": "tagged",
      "header": "absent",
      "tag_column": 0,
      "tag_column_name": "record_type",
      "tag_match": "case_insensitive",
      "tags": [
        { "name": "DPC",       "value": "DPC",       "layout": "xperf_dpc" },
        { "name": "Interrupt", "value": "Interrupt",  "layout": "xperf_interrupt" }
      ]
    },
    "xperf_dpc": {
      "format": "simple",
      "header": "absent",
      "columns": {
        "timestamp":     "Integer",
        "elapsed_time":  "Integer",
        "cpu":           "Integer",
        "service_addr":  "General",
        "image_function":"String"
      }
    },
    "xperf_interrupt": {
      "format": "simple",
      "header": "absent",
      "columns": {
        "timestamp":     "Integer",
        "elapsed_time":  "Integer",
        "cpu":           "Integer",
        "vector":        "Integer",
        "service_addr":  "General",
        "image_function":"String"
      }
    }
  }
}
```

---

## Rules for Copilot when writing schemas

- Always set `"version": "1.0"`.
- Use `"General"` for hex addresses, opaque identifiers, and fields that should
  pass through unparsed.
- Use `"Integer"` for timestamps, counts, and IDs — not `"Float"`.
- Use `"Decimal"` for money/financial values, never `"Float"`.
- For tagged layouts: child layouts must use `"header": "absent"` and must **not**
  include the tag column in their `columns`.
- Tag `"value"` is the literal string in the CSV; the executor trims surrounding
  whitespace before matching, so do not include padding in the value.
- Prefer `"tag_match": "case_insensitive"` for tool-generated CSV where tag
  casing may vary across versions.
- Layout names must match `[a-zA-Z][a-zA-Z0-9_]*`.
- Source names (keys in `sources`) may be plain stems or glob patterns; use
  `*` to cover multiple files sharing a layout.
