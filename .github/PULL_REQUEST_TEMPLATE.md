<!-- Please fill out the sections below when opening a PR. This template encodes the project's "must pass before push" checklist. -->

## Summary

Describe the change at a high level. Link to relevant issues or design notes.

---

## Checklist (must pass before pushing / merging)

- [ ] Code is formatted: `cargo fmt --all --check`
- [ ] Linting: `rustup run stable cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Clippy allow policy: `bash scripts/check-clippy-allows.sh` (no tracked `#[allow(clippy::...)]`)
- [ ] Source file length check: run the repository's source-file-size script or ensure no files exceed recommended limits (see CI: [.github/workflows/ci.yml](.github/workflows/ci.yml#L1))
- [ ] Complexity gates: run CI complexity checks (cognitive complexity, `too_many_lines`, etc.)
- [ ] Coverage: `cargo llvm-cov --workspace --all-features --summary-only --fail-under-lines 30` (or verify coverage report meets project threshold)
- [ ] Build: `cargo build --workspace --all-features --locked`
- [ ] Tests: `cargo test --workspace --all-features --locked` (unit + integration)
- [ ] Optional: TUI smoke scenario (if the change affects runtime/UI): run `target/debug/jefe-tmux-harness` scenario as configured in CI

---

## Testing notes

Describe how this change was tested locally (commands run, important logs, smoke scenarios).

## Reviewers / Assignees

Tag reviewers or teams to review.

--
Generated PR template: please keep this checklist in sync with [.github/workflows/ci.yml](.github/workflows/ci.yml#L1).
