# GitHub Mode

`answerci scan-github <owner/repo>` builds a starter prompt suite from local README and docs files. It does not call the GitHub API; the command assumes you are running from a checkout or documentation mirror.

```bash
answerci scan-github DivitMittal/OS-nixCfg --root . --output prompt-suites/os-nixcfg.yaml
```

The scanner:

1. Parses `owner/repo` or `https://github.com/owner/repo.git`.
2. Reads local `README.md` and Markdown files under `docs/`.
3. Creates target aliases from the repository name, full slug, and first documentation heading.
4. Generates starter overview, alternatives, and install prompts.

No network access is required in tests or CI.
