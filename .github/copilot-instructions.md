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
