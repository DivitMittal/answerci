# Security Policy

Thank you for helping keep AnswerCI and its users safe.

## Supported Versions

AnswerCI is currently a research/MVP artifact. Security fixes are accepted for the `main` branch and the latest tagged release, if one exists.

## Reporting a Vulnerability

Please do not report security vulnerabilities through public GitHub issues.

To report a vulnerability, contact the maintainer directly by email or open a private GitHub security advisory if the repository has advisories enabled. Include:

- A concise description of the issue and its impact.
- Steps to reproduce, proof-of-concept input, or affected files.
- Whether the issue can expose provider secrets, private prompts, captured responses, local files, or CI credentials.
- Any suggested mitigation or patch, if available.

The maintainer will acknowledge reports as soon as possible and coordinate a fix or disclosure timeline based on severity.

## Security Expectations

- Never commit API keys, provider tokens, private prompts, captured private responses, or `.env` files.
- Keep live provider integrations opt-in; tests and CI should remain deterministic and offline by default.
- Redact secrets from reports, logs, screenshots, prompt suites, and manual-import JSONL fixtures.
- Treat captured answer-engine responses as potentially sensitive if they include private prompts, private repository contents, or customer data.
- Prefer least-privilege GitHub tokens and CI permissions when adding workflows or integrations.

## Scope

In scope:

- Secret exposure in examples, reports, fixtures, logs, or CI output.
- Unsafe handling of provider credentials or captured responses.
- Vulnerabilities in CLI behavior that can overwrite unexpected files, read secrets, or mis-handle untrusted project input.
- Supply-chain concerns in build, Nix, Cargo, or APM metadata.

Out of scope:

- Publicly known dependency advisories without an AnswerCI-specific exploit path.
- Findings that require malicious local shell access beyond the repository's documented command surface.
- Availability-only reports against public services not controlled by this project.
