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

## Repository-local Git worktrees

- Create or use a Git worktree only when the human operator explicitly authorizes it for the current task. Concurrency or a dirty checkout is not permission by itself.
- Put every authorized worktree at `<repository-root>/tmp/worktrees/<name>`; from the repository root, use `./tmp/worktrees/<name>`. Never place worktrees beside repositories or organization directories.
- Keep `tmp`, `temp`, `tmp/worktrees`, and `temp/worktrees` ignored in the repository-root `.gitignore`. Do not commit files from those directories.
- Relocate or remove a worktree only when the operator explicitly requests it. Before removal, preserve and publish intended changes, verify its commit is represented on the target branch, and confirm there are no tracked, untracked, ignored-sensitive, or in-use files that must survive. Remove it with `git worktree remove <path>` without `--force`; never delete a worktree directory with `rm`.

<!-- BEGIN ores-agents-pointer: managed by ORESoftware/my-ai; edit there, not here -->

## Canonical agent instructions

Before doing anything else in this repository, also read:

    .ores/agents/AGENTS.md

That path is a symlink to `~/codes/oresoftware/my-ai/AGENTS.md`, whose canonical copy is
<https://github.com/ORESoftware/my-ai/blob/main/AGENTS.md>.

It exists at a fixed path *inside* the repository because some agents cannot walk up past
the repository root, so machine-wide instructions one or more directories above are
invisible to them. This pointer plus that path make the same file reachable from a working
directory anywhere in the tree.

The symlink is deliberately **not committed**: it names an absolute path that is only valid
on a machine with `~/codes/oresoftware/my-ai` checked out, so committing it would produce a
broken link for everyone else and for CI. `.ores/` is git-ignored for that reason. If
`.ores/agents/AGENTS.md` is missing on your machine, create it with:

    mkdir -p .ores/agents
    ln -sfn "$HOME/codes/oresoftware/my-ai/AGENTS.md" .ores/agents/AGENTS.md

or run `~/codes/oresoftware/my-ai/scripts/link-repo-agents.sh` once to do it for every git
repository under `~/codes`, and `--check` to verify them.

A missing `.ores/agents/AGENTS.md` is a setup gap on the reader's machine, never a reason to
skip the canonical instructions: fetch them from the URL above instead.

<!-- END ores-agents-pointer -->
