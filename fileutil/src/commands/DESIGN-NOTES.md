# Design Notes for commands/

## Submodule Naming Convention

Submodules under `commands/` are named to match their corresponding subcommand names for consistency and clarity. For example, the `ls` subcommand uses `commands/ls/`, and the `enum` subcommand uses `commands/enum/`.

If a subcommand name conflicts with a Rust keyword (e.g., `enum`), use a raw identifier (`r#enum`) in module declarations and imports to avoid syntax errors, while keeping the directory and file names keyword-free.

This convention ensures that the codebase structure reflects the CLI structure, making it easier to navigate and maintain.