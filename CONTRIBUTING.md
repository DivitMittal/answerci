# Contributing

Contributions are welcome — bug reports, fixes, docs improvements, and new deterministic fixtures for AnswerCI.

## Setup

```sh
nix develop   # preferred: enter the pinned development shell
```

If Nix is unavailable, install the stable Rust toolchain and run the Cargo commands directly.

## Guidelines

- Format Rust code with `cargo fmt --check`.
- Run `cargo clippy --all-targets -- -D warnings` before submitting.
- Run `cargo test`; CI must not require live answer-engine or provider API calls.
- Run `nix flake check` when Nix is available — CI runs the same flake check.
- Keep fixtures offline-first and reproducible. Manual imports, mock providers, examples, and report snapshots should be deterministic.
- Update docs, schemas, examples, and reports when changing user-facing behavior or output formats.
- Keep changes focused; avoid mixing provider integrations, scoring changes, and documentation rewrites in one PR.

## Submitting Changes

1. Fork the repo and create a branch: `feat/description` or `fix/description`.
2. Keep commits atomic; use [Conventional Commits](https://www.conventionalcommits.org/) format.
3. Open a PR against `main` with a clear description of what changed and why.
4. Include reproduction steps, sample prompt suites, or before/after report snippets when the change affects scoring or CI behavior.

## Reporting Issues

Open a GitHub issue with:

- Your OS, Rust version, and whether you used Nix.
- The command you ran and the full error output.
- Minimal prompt suite, manual-import JSONL, or fixture data needed to reproduce.
- Expected vs actual behavior.
