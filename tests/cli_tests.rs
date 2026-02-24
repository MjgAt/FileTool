use serde_json::Value;
use std::fs;
use std::process::Command;

fn get_binary_path() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove the test binary name
    path.pop(); // remove deps
    path.push("fileutil.exe");
    path
}

#[test]
fn test_enum_command_json_output() {
    let bin_path = get_binary_path();
    let output = Command::new(&bin_path)
        .args(&["--format", "json", "enum"])
        .output()
        .expect("Failed to run command");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let data: Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(data.is_array(), "Output should be an array");

    let arr = data.as_array().unwrap();
    assert!(!arr.is_empty(), "Array should not be empty");

    // Check the first item has the expected structure
    if let Some(first) = arr.first() {
        assert!(first.is_object(), "Each item should be an object");
        let obj = first.as_object().unwrap();
        assert!(obj.contains_key("root"), "Should have 'root' field");
        assert!(obj.contains_key("rel_path"), "Should have 'rel_path' field");
        assert!(obj.contains_key("filename"), "Should have 'filename' field");
        assert!(
            obj.contains_key("parent_id"),
            "Should have 'parent_id' field"
        );
        assert!(obj.contains_key("id"), "Should have 'id' field");
        assert!(obj.contains_key("size"), "Should have 'size' field");
    }

    // Check that some files have IDs
    let has_file_with_id = arr
        .iter()
        .any(|item| item.get("id").and_then(|v| v.as_u64()).is_some());
    assert!(has_file_with_id, "At least one file should have an ID");
}

#[test]
fn test_enum_command_csv_output() {
    let bin_path = get_binary_path();
    let output = Command::new(&bin_path)
        .args(&["--format", "csv", "enum"])
        .output()
        .expect("Failed to run command");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines.len() > 1,
        "Should have header and at least one data line"
    );

    // Check header
    assert_eq!(
        lines[0], "filename,id,parent_id,rel_path,root,size",
        "Header should match"
    );

    // Check that some lines have IDs (not all empty)
    let has_non_empty_id = lines.iter().skip(1).any(|line| {
        let parts: Vec<&str> = line.split(',').collect();
        parts.len() > 1 && !parts[1].is_empty()
    });
    assert!(
        has_non_empty_id,
        "At least one entry should have a non-empty ID"
    );
}

#[test]
fn test_ls_command() {
    let bin_path = get_binary_path();
    let output = Command::new(&bin_path)
        .arg("ls")
        .output()
        .expect("Failed to run command");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    // ls outputs table by default, check it contains expected content
    assert!(stdout.contains("Cargo.toml"), "Should list Cargo.toml");
}

#[test]
fn test_enum_glob_pattern() {
    let bin_path = get_binary_path();
    // Use a glob pattern that should match all Rust source files in the repo.
    let output = Command::new(&bin_path)
        .args(&["--format", "json", "enum", "**/*.rs"])
        .output()
        .expect("Failed to run command");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let data: Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(data.is_array(), "Output should be an array");
    let arr = data.as_array().unwrap();
    assert!(
        !arr.is_empty(),
        "Glob **/*.rs should match at least one file"
    );

    // Every entry should have a filename ending in ".rs"
    for item in arr {
        let filename = item.get("filename").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            filename.ends_with(".rs"),
            "Expected .rs file, got: {filename}"
        );
    }

    // The root field should reflect the original glob pattern.
    let root = arr[0].get("root").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(root, "**/*.rs", "Root should be the glob pattern");
}

#[test]
fn test_enum_plain_subdir() {
    let bin_path = get_binary_path();
    let output = Command::new(&bin_path)
        .args(&["--format", "json", "enum", "fileutil/src"])
        .output()
        .expect("Failed to run command");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let data: Value = serde_json::from_str(&stdout).expect("Invalid JSON");

    assert!(data.is_array(), "Output should be an array");
    let arr = data.as_array().unwrap();
    assert!(!arr.is_empty(), "fileutil/src should contain files");

    // All entries should be under the fileutil/src subtree.
    let root = arr[0].get("root").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(
        root, "fileutil/src",
        "Root should be the plain directory path"
    );
}

// ── --update-schema tests ────────────────────────────────────────────────────

fn schema_test_path(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("fileutil_test_schema_{name}.json"));
    p
}

#[test]
fn test_update_schema_creates_file() {
    let bin_path = get_binary_path();
    let schema_path = schema_test_path("create");
    // Ensure clean state.
    let _ = fs::remove_file(&schema_path);

    let output = Command::new(&bin_path)
        .args([
            "--update-schema",
            "--schema-path",
            schema_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    assert!(
        output.status.success(),
        "Command failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(schema_path.exists(), "Schema file should have been created");

    let text = fs::read_to_string(&schema_path).expect("Could not read schema file");
    let schema: Value = serde_json::from_str(&text).expect("Schema is not valid JSON");

    assert_eq!(schema["version"].as_u64(), Some(1));
    let tables = schema["tables"]
        .as_object()
        .expect("tables should be an object");
    assert!(
        tables.contains_key("enum"),
        "Schema should contain 'enum' table"
    );
    assert!(
        tables.contains_key("ls"),
        "Schema should contain 'ls' table"
    );

    let _ = fs::remove_file(&schema_path);
}

#[test]
fn test_update_schema_enum_columns() {
    let bin_path = get_binary_path();
    let schema_path = schema_test_path("enum_cols");
    let _ = fs::remove_file(&schema_path);

    Command::new(&bin_path)
        .args([
            "--update-schema",
            "--schema-path",
            schema_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    let text = fs::read_to_string(&schema_path).unwrap();
    let schema: Value = serde_json::from_str(&text).unwrap();
    let cols = schema["tables"]["enum"]["columns"]
        .as_array()
        .expect("columns should be array");

    let names: Vec<&str> = cols
        .iter()
        .map(|c| c["name"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        names,
        ["filename", "id", "parent_id", "rel_path", "root", "size"]
    );

    // Nullable columns
    let id_col = cols.iter().find(|c| c["name"] == "id").unwrap();
    assert_eq!(
        id_col["nullable"].as_bool(),
        Some(true),
        "id should be nullable"
    );

    let _ = fs::remove_file(&schema_path);
}

#[test]
fn test_update_schema_ls_columns() {
    let bin_path = get_binary_path();
    let schema_path = schema_test_path("ls_cols");
    let _ = fs::remove_file(&schema_path);

    Command::new(&bin_path)
        .args([
            "--update-schema",
            "--schema-path",
            schema_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    let text = fs::read_to_string(&schema_path).unwrap();
    let schema: Value = serde_json::from_str(&text).unwrap();
    let cols = schema["tables"]["ls"]["columns"]
        .as_array()
        .expect("columns should be array");

    let names: Vec<&str> = cols
        .iter()
        .map(|c| c["name"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(names, ["is_dir", "path", "size"]);

    let is_dir_col = cols.iter().find(|c| c["name"] == "is_dir").unwrap();
    assert_eq!(is_dir_col["type"].as_str(), Some("boolean"));

    let _ = fs::remove_file(&schema_path);
}

#[test]
fn test_update_schema_idempotent() {
    let bin_path = get_binary_path();
    let schema_path = schema_test_path("idempotent");
    let _ = fs::remove_file(&schema_path);

    // First run: creates the file.
    let out1 = Command::new(&bin_path)
        .args([
            "--update-schema",
            "--schema-path",
            schema_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");
    assert!(out1.status.success());

    let content_after_first = fs::read_to_string(&schema_path).unwrap();

    // Second run: should report "already up to date" and not modify the file.
    let out2 = Command::new(&bin_path)
        .args([
            "--update-schema",
            "--schema-path",
            schema_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");
    assert!(out2.status.success());

    let stderr2 = String::from_utf8(out2.stderr).unwrap();
    assert!(
        stderr2.contains("up to date"),
        "Second run should report 'up to date'"
    );

    let content_after_second = fs::read_to_string(&schema_path).unwrap();
    assert_eq!(
        content_after_first, content_after_second,
        "File should not change on second run"
    );

    let _ = fs::remove_file(&schema_path);
}

#[test]
fn test_update_schema_replaces_stale_entry() {
    let bin_path = get_binary_path();
    let schema_path = schema_test_path("stale");
    let _ = fs::remove_file(&schema_path);

    // Write a schema with a wrong definition for 'ls'.
    let stale = serde_json::json!({
        "version": 1,
        "tables": {
            "ls": {
                "columns": [
                    {"name": "old_column", "type": "string"}
                ]
            }
        }
    });
    fs::write(&schema_path, serde_json::to_string_pretty(&stale).unwrap()).unwrap();

    let output = Command::new(&bin_path)
        .args([
            "--update-schema",
            "--schema-path",
            schema_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");
    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("changed"), "Should report tables changed");

    let text = fs::read_to_string(&schema_path).unwrap();
    let schema: Value = serde_json::from_str(&text).unwrap();
    let cols = schema["tables"]["ls"]["columns"].as_array().unwrap();
    assert!(
        !cols.iter().any(|c| c["name"] == "old_column"),
        "Stale column should be gone"
    );
    assert!(
        cols.iter().any(|c| c["name"] == "path"),
        "Current column 'path' should be present"
    );

    let _ = fs::remove_file(&schema_path);
}

#[test]
fn test_update_schema_preserves_unknown_tables() {
    let bin_path = get_binary_path();
    let schema_path = schema_test_path("preserve");
    let _ = fs::remove_file(&schema_path);

    // Write a schema that includes a table registered by a hypothetical plugin.
    let existing = serde_json::json!({
        "version": 1,
        "tables": {
            "my_plugin": {
                "columns": [
                    {"name": "custom_col", "type": "string"}
                ]
            }
        }
    });
    fs::write(
        &schema_path,
        serde_json::to_string_pretty(&existing).unwrap(),
    )
    .unwrap();

    Command::new(&bin_path)
        .args([
            "--update-schema",
            "--schema-path",
            schema_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    let text = fs::read_to_string(&schema_path).unwrap();
    let schema: Value = serde_json::from_str(&text).unwrap();
    let tables = schema["tables"].as_object().unwrap();

    assert!(
        tables.contains_key("my_plugin"),
        "Plugin table should be preserved"
    );
    assert!(
        tables.contains_key("enum"),
        "Built-in 'enum' should be added"
    );

    let _ = fs::remove_file(&schema_path);
}
