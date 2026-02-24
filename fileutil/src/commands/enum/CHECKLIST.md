# Checklist: Glob Pattern Support for `enum` Command

## Goal
Extend `fileutil enum [pathname]` so that `pathname` can be an optional glob pattern
(e.g. `**/*.rs`, `src/**`, `"."`) relative to the current working directory.

## Tasks
- [x] Add `globwalk` dependency to `fileutil/Cargo.toml`
- [x] Refactor `enum/mod.rs` to detect glob metacharacters and dispatch to glob-based walker
- [x] Plain paths continue to use `WalkDir` as before (no behavior change)
- [x] Add integration test for glob enumeration in `tests/cli_tests.rs`
- [x] Update `DESIGN-NOTES.md` to document the dual-dispatch strategy
- [x] Complete CHECKLIST and move to `COMPLETED-PLANS.md`
