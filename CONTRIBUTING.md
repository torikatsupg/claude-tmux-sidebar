# Contributing

## How it works

One binary, `claude-tmux-sidebar`, does everything; `bin/cts` and `bin/cts-hook` are small shell entry points for it. `scripts/fetch-binary.sh` puts the binary in `bin/` on first run: it downloads it from the GitHub release matching the version (verifying its SHA-256), or builds it with `cargo` when no release binary fits. Claude Code skips the hook events that arrive before the binary is ready.

- `claude-tmux-sidebar hook` runs on each Claude Code hook and writes the status, `$TMUX_PANE` and Claude's PID to `~/.cache/claude-tmux-sidebar/<session_id>.json`. `SessionEnd` removes the file.
- `claude-tmux-sidebar toggle`, `ensure`, `fix-width`, … control tmux: tmux hooks such as `after-new-window` add a full-height pane on the left of each window and restore its width after layout changes. Closing the sidebar restores the window's layout from before it opened.
- `claude-tmux-sidebar sidebar` draws the list and cleans up state files whose pane or process is gone. It closes itself in a window where only the sidebar is left.

Statuses come from these Claude Code hooks:

| Status | Hook |
|---|---|
| running | `UserPromptSubmit` / `PreToolUse` / `PostToolUse` / `PostToolUseFailure`, or `Stop` whose `background_tasks` still has a running task |
| waiting for you | `PermissionRequest` / `Notification(permission_prompt)` / `PreToolUse(AskUserQuestion)` |
| done | `Stop` with no running background task |
| idle | `SessionStart`, `Notification(idle_prompt)` (unless background tasks are running) |

A turn interrupted with Esc does not fire `Stop`; `idle_prompt` turns it idle 60 seconds later.

Hooks from subagents carry the parent's `session_id` and an `agent_id`. While a prompt is pending, only the agent that raised it (its tool events, or `SubagentStop` for a subagent) ends the wait; other agents' events and a `Stop` with background tasks still running keep it waiting. Only main-thread events update the session's `cwd`.

## Development

Requires a Rust toolchain (the version is pinned in `rust-toolchain.toml`), [shellcheck](https://www.shellcheck.net) and [shfmt](https://github.com/mvdan/sh).

```sh
cargo xtask ci                   # rustfmt, clippy, tests, shellcheck, shfmt and module size limits (what CI runs)
scripts/fetch-binary.sh --build  # build bin/claude-tmux-sidebar from this checkout for a live tmux
scripts/screenshot.sh            # regenerate docs/screenshot.svg (scripts/screenshot.sh ja for the Japanese one)
scripts/demo.sh                  # re-record docs/demo.gif (needs vhs)
```

Code layout (`src/`): pure modules turn values into values, and side effects stay at the edges.

| Module | Role |
|---|---|
| `cli`, `lib`, `main` | argument parsing and dispatch |
| `tmux`, `session::store`, `process` | side effects: the tmux server, state files, other processes |
| `hook` | hook payload → status transition (`transition` is a pure state machine) |
| `control` | opening / closing sidebars and restoring layouts |
| `sidebar` | the list UI: `update` is a pure `(model, msg) → (model, effects)` function, `view` renders, `effect` runs effects |
| `listing`, `jump` | grouping in tmux order, next / previous done or waiting session |

Style rules, enforced by clippy (`Cargo.toml` `[workspace.lints]`, `clippy.toml`) and `cargo xtask ci`: no `unwrap` / `expect` / `panic` / indexing / `as` casts, functions of at most 30 lines, modules of at most 200 lines, and iterator adapters (`map`, `filter`, `fold`) over mutable loops.

## Releasing

1. Open a PR that bumps the version in `Cargo.toml`, `Cargo.lock`, `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json`, adds the section and compare links to `CHANGELOG.md`, and updates the `#vX.Y.Z` pins in `README.md` and `README.ja.md`.
2. Once it is merged, run the **Release** workflow on `main` (Actions tab, or `gh workflow run release.yml --ref main`). It builds the binaries for macOS and Linux (arm64 / x86_64) with build provenance attestations, tags `v<version>` and publishes an immutable GitHub release with them. Only the repository owner can run it.
