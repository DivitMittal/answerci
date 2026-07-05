# CI Mode

AnswerCI is designed to run after docs or README changes produce a new captured run.

```bash
answerci diff \
  --suite examples/prompt-suites/oss-project.yaml \
  --baseline baseline \
  --candidate candidate

answerci ci \
  --suite examples/prompt-suites/oss-project.yaml \
  --baseline baseline \
  --candidate candidate \
  --fail-on citation-regression
```

`answerci ci --fail-on citation-regression` exits with code `1` when the candidate citation rate is lower than the baseline citation rate. Other fail-on policies are planned for Strong v1.

Recommended workflow:

1. Keep a baseline run artifact for the default branch.
2. Capture or import a candidate run for the pull request.
3. Generate a Markdown report and upload it as a PR artifact.
4. Gate only on high-confidence regressions at first, such as citation drops.
