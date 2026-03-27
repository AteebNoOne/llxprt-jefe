# Building and development

This page is for contributors working on `jefe` itself.

## Requirements

- Rust toolchain (edition 2024 crate)
- `tmux` installed and available on PATH
- `llxprt` CLI installed and available on PATH

## Build and run locally

```bash
cargo run
```

Version:

```bash
cargo run -- --version
```

## Development verification

Run the CI-equivalent local gate before opening a PR:

```bash
make build
```

(`make ci-check` is kept as an alias.)

This command runs:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --all-features -- \
  -A clippy::all \
  -A clippy::pedantic \
  -A clippy::nursery \
  -D clippy::cognitive_complexity \
  -D clippy::too_many_lines \
  -D clippy::too_many_arguments \
  -D clippy::type_complexity \
  -D clippy::struct_excessive_bools
cargo llvm-cov \
  --workspace \
  --all-features \
  --summary-only \
  --ignore-filename-regex '(/vendor/|/tmp/|/rustc-)' \
  --fail-under-lines 30
cargo build --workspace --all-features --locked
cargo test --workspace --all-features --locked
```

Optional local-only speed pass while iterating:

```bash
cargo fmt
cargo check -q
cargo test -q
```

## Project structure

- `src/main.rs` — app entry + event/render loop wiring
- `src/state/` — app state machine and events
- `src/runtime/` — tmux/PTTY attach, input, snapshots, liveness
- `src/ui/` — screens/components/modals
- `src/theme/` — themes and color resolution
- `src/persistence/` — load/save settings and state
- `docs/` — technical and product docs
