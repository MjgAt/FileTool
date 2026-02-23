use std::process::Command;
use serde_json::Value;

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

    assert!(output.status.success(), "Command failed: {:?}", String::from_utf8_lossy(&output.stderr));

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
        assert!(obj.contains_key("parent_id"), "Should have 'parent_id' field");
        assert!(obj.contains_key("id"), "Should have 'id' field");
        assert!(obj.contains_key("size"), "Should have 'size' field");
    }

    // Check that some files have IDs
    let has_file_with_id = arr.iter().any(|item| {
        item.get("id").and_then(|v| v.as_u64()).is_some()
    });
    assert!(has_file_with_id, "At least one file should have an ID");
}

#[test]
fn test_enum_command_csv_output() {
    let bin_path = get_binary_path();
    let output = Command::new(&bin_path)
        .args(&["--format", "csv", "enum"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success(), "Command failed: {:?}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() > 1, "Should have header and at least one data line");

    // Check header
    assert_eq!(lines[0], "filename,id,parent_id,rel_path,root,size", "Header should match");

    // Check that some lines have IDs (not all empty)
    let has_non_empty_id = lines.iter().skip(1).any(|line| {
        let parts: Vec<&str> = line.split(',').collect();
        parts.len() > 1 && !parts[1].is_empty()
    });
    assert!(has_non_empty_id, "At least one entry should have a non-empty ID");
}

#[test]
fn test_ls_command() {
    let bin_path = get_binary_path();
    let output = Command::new(&bin_path)
        .arg("ls")
        .output()
        .expect("Failed to run command");

    assert!(output.status.success(), "Command failed: {:?}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    // ls outputs table by default, check it contains expected content
    assert!(stdout.contains("Cargo.toml"), "Should list Cargo.toml");
}