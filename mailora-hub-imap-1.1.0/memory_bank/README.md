# Memory Bank for mailora-hub-imap

Purpose:
- Persist small pieces of contextual state and heuristics that improve agent behavior during development and debugging.
- Keep it repo-local, self-describing, and easy to review in PRs.

Conventions:
- Plaintext or JSON items; each item is immutable once committed (append-only logs or new files per topic).
- No secrets, access tokens, or PII. Store only non-sensitive hints.
- Reference code paths with repo-relative paths.
- Keep entries short; link to issues/PRs when useful.

Structure:
- memory_bank/
  - README.md (this file)
  - rules.md (core rules the agent should follow in this repo)
  - facts/*.md (atomic facts about this codebase)
  - playbooks/*.md (step-by-step procedures)
  - decisions/*.md (record of important decisions and why)

Usage:
- Contributors can add/update facts and playbooks as understanding improves.
- CI or local tools can surface these notes in review.
