//! csvdb schema management.
//!
//! The schema file (`schema.csvdb.json` by default) records the canonical CSV
//! column layout produced by each `fileutil` subcommand. It is intended to be
//! consumed by downstream tools (e.g. csvdb) that need to know column names and
//! types without actually running the tool.
//!
//! Layout of `schema.csvdb.json`:
//! ```json
//! {
//!   "version": 1,
//!   "tables": {
//!     "enum": {
//!       "columns": [
//!         { "name": "filename",   "type": "string"  },
//!         { "name": "id",         "type": "integer", "nullable": true },
//!         ...
//!       ]
//!     },
//!     "ls": { "columns": [ ... ] }
//!   }
//! }
//! ```
//!
//! When `--update-schema` is passed, every known subtool's table entry is
//! compared against the current definition.  If the entry is absent or differs,
//! it is replaced.  Unknown tables (registered by external plugins) are left
//! untouched.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

// ── Schema types ─────────────────────────────────────────────────────────────

/// The type of a column's values as they appear in CSV output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnType {
    String,
    Integer,
    Boolean,
    Float,
    DateTime,
}

/// Description of a single CSV output column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: ColumnType,
    #[serde(default, skip_serializing_if = "is_false")]
    pub nullable: bool,
}

fn is_false(b: &bool) -> bool {
    !b
}

impl ColumnDef {
    pub fn string(name: &str) -> Self {
        Self {
            name: name.into(),
            col_type: ColumnType::String,
            nullable: false,
        }
    }
    pub fn integer(name: &str) -> Self {
        Self {
            name: name.into(),
            col_type: ColumnType::Integer,
            nullable: false,
        }
    }
    pub fn boolean(name: &str) -> Self {
        Self {
            name: name.into(),
            col_type: ColumnType::Boolean,
            nullable: false,
        }
    }
    pub fn nullable_integer(name: &str) -> Self {
        Self {
            name: name.into(),
            col_type: ColumnType::Integer,
            nullable: true,
        }
    }
    pub fn nullable_datetime(name: &str) -> Self {
        Self {
            name: name.into(),
            col_type: ColumnType::DateTime,
            nullable: true,
        }
    }
}

/// The schema for one subtool's CSV output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSchema {
    pub columns: Vec<ColumnDef>,
}

/// Root schema document (`schema.csvdb.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvdbSchema {
    pub version: u32,
    /// Keyed by subtool name (e.g. `"enum"`, `"ls"`).
    pub tables: BTreeMap<String, TableSchema>,
}

impl Default for CsvdbSchema {
    fn default() -> Self {
        Self {
            version: 1,
            tables: BTreeMap::new(),
        }
    }
}

// ── Canonical definitions ─────────────────────────────────────────────────────

/// All built-in table definitions, keyed by subtool name.
/// This is the single source of truth — command modules delegate here by
/// calling `crate::schema::builtin_tables()` or expose their slice via their
/// own `csv_columns()` helpers (which are re-exported here).
pub fn builtin_tables() -> BTreeMap<String, TableSchema> {
    use crate::commands;
    let mut m = BTreeMap::new();
    m.insert(
        "enum".into(),
        TableSchema {
            columns: commands::r#enum::csv_columns(),
        },
    );
    m.insert(
        "ls".into(),
        TableSchema {
            columns: commands::ls::csv_columns(),
        },
    );
    m
}

// ── Update logic ─────────────────────────────────────────────────────────────

/// Load (or create) the schema file at `path`, update all built-in table
/// entries, and write the result back.
///
/// Returns the number of tables that were added or replaced.
pub fn update_schema(path: &Path) -> std::io::Result<usize> {
    // Load existing schema, or start fresh.
    let mut schema: CsvdbSchema = if path.exists() {
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str(&text).unwrap_or_default()
    } else {
        CsvdbSchema::default()
    };

    let builtins = builtin_tables();
    let mut changed = 0usize;

    for (name, canonical) in &builtins {
        let entry = schema.tables.get(name);
        if entry != Some(canonical) {
            schema.tables.insert(name.clone(), canonical.clone());
            changed += 1;
        }
    }

    // Always write when the file doesn't exist yet, even if nothing "changed".
    if changed > 0 || !path.exists() {
        let text = serde_json::to_string_pretty(&schema)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, text)?;
    }

    Ok(changed)
}

/// Return `true` when the schema file at `path` exists but contains at least
/// one built-in table definition that differs from the current canonical
/// definition (or is absent from the file entirely).
///
/// Returns `false` when the file does not exist (nothing to warn about) or
/// cannot be parsed (we treat unreadable schemas conservatively as non-stale
/// to avoid spurious warnings).
pub fn is_schema_stale(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let schema: CsvdbSchema = match serde_json::from_str(&text) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let builtins = builtin_tables();
    builtins
        .iter()
        .any(|(name, canonical)| schema.tables.get(name) != Some(canonical))
}
