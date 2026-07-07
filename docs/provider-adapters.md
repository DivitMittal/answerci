# Provider Adapters

Provider adapters implement:

```rust
trait Provider {
    fn name(&self) -> &str;
    fn capture(&mut self, prompt: &Prompt) -> anyhow::Result<RawResponse>;
}
```

## Manual imports

Manual imports read JSONL fixtures and are the recommended MVP path for CI because they are deterministic and require no network calls:

```bash
answerci run \
  --suite examples/prompt-suites/oss-project.yaml \
  --providers manual:examples/manual-imports/sample-run.jsonl \
  --run-id sample \
  --export-jsonl .answerci/sample-observations.jsonl
```

Each line contains `prompt_id`, `provider`, and `content` fields. Optional `id` and `metadata` fields are preserved.

## Mock provider

Use `--providers mock` for deterministic offline captures generated from the prompt suite. `MockProvider` maps prompt IDs to response strings and never performs network calls.

## OpenAI stub

`OpenAiProvider` is intentionally a stub. It returns an actionable error explaining that live API capture is not implemented yet. This keeps tests offline and prevents accidental provider traffic.
