# AnswerCI project instructions

AnswerCI is a Rust workspace for CLI-first, CI-native visibility regression testing of AI answer engines.

## Working rules

- Preserve the offline-first prototype behavior: manual imports and mock providers must keep CI deterministic.
- Do not add or require live answer-engine calls unless the user explicitly asks for provider integration.
- Keep CLI behavior, docs, examples, schemas, and reports aligned when user-facing formats change.
- Prefer small, focused Rust functions with explicit error handling and helpful diagnostics.
- Never log provider secrets, API keys, prompts containing secrets, or raw environment variables.

## Validation

Run these before submitting code changes:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
nix flake check
```

If Nix is unavailable, still run the Cargo checks and note that `nix flake check` was skipped.
