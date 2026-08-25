# act-lib-core agent instructions

Lowercase `agents.md` is canonical. Also follow the Anticaptrad organization
policy in `anticaptrad/.github/agents.md` when it is available locally.

- Preserve concurrent and unrelated work. Inspect status, remotes, history,
  interfaces, consumers, and open review work before editing.
- Work on the primary branch unless an explicit release or review policy
  requires otherwise. Never rebase, stash, force-push, reset, clean, rewrite
  history, create a worktree, or bypass required review and CI.
- Stage explicit paths only. Fetch before editing and before pushing; merge
  upstream changes conceptually when required.
- Keep the `act-interfaces` and `ores-lib-core` revisions immutable and keep
  Cargo and Zed dependency declarations semantically aligned.
- Preserve the read-only database capability. Do not add generic SQL, write
  methods, startup migrations, or a path that accepts a write-capable database.
- Treat Shared Auth as authentication, never product authorization. Product,
  membership, and resource checks remain mandatory in server consumers.
- Never log or persist tokens, service credentials, cookies, database URLs,
  private command payloads, raw upstream bodies, or customer data.
- Run Rust format, Clippy, locked tests, Zed validation, secret/conflict-marker
  scans, and staged-diff checks before publishing changes.
