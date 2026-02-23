# FileTool
Tool to examine the file system

## Development

This is a Rust workspace. To build and run the tool:

```bash
cargo build
cargo run --bin fileutil -- [options] <subcommand> [args]
```

## Options

- `--format <FORMAT>`: Output format (table, csv, json). Default: table

## Subcommands

- `ls [path]`: List directory contents (default path: current directory)
  - `-r, --recursive`: Recurse into subdirectories

## Output Formats

- **table**: Pretty-printed table for console viewing
- **csv**: Comma-separated values for data export
- **json**: Structured JSON for shell integration (e.g., Nushell)

## Plugin Protocol (for External Subcommands)

For extensibility, `fileutil` supports external plugins as separate executables. Plugins should be named `fileutil-<subcommand>` (e.g., `fileutil-custom.exe`).

- **Input**: JSON string via stdin containing the subcommand arguments.
- **Output**: Plain text or JSON to stdout on success; error messages to stderr on failure.
- **Example**: For a subcommand with args `{"path": "/tmp"}`, the plugin receives `{"path":"/tmp"}` and outputs directory listing.

To add a built-in subcommand, modify `src/commands/` and `src/main.rs`. For external, ensure the binary is in PATH.
