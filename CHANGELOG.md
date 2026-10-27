# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- A session that started before the update to 0.7.4 shows its branch and context usage again; its hooks keep writing the old format until Claude restarts (#62).

## [0.7.4] - 2026-10-27

### Fixed

- The context usage and branch are read by the sidebar instead of the hooks, so a finished turn's usage shows right away instead of one event late, and the row shows no percentage right after `/compact` instead of the pre-compaction value until the next prompt. A sidebar that was open before the update shows neither until you reopen it.

## [0.7.3] - 2026-10-27

### Fixed

- After `/compact`, a session's context usage is no longer shown at its pre-compaction value until Claude answers again; the row shows no percentage until then.

## [0.7.2] - 2026-10-23

### Fixed

- A subagent waiting for approval keeps the session "waiting" when a background task finishes and its notification reaches the main thread, instead of turning it "running" while the approval dialog is still open.

## [0.7.1] - 2026-10-20

### Fixed

- A session whose turn has ended no longer stays "running" because of a background monitor. Monitors wait for events and some never end, such as the watch Claude Code keeps on a published artifact, which left the session spinning for hours.
- A session waiting for approval or an answer stays "waiting" while other agents keep working: tool events from another subagent or the main thread, and a `Stop` with background tasks still running, no longer turn it "running". Only the agent that asked ends the wait, or a `SubagentStop` when that subagent ends. A subagent's directory, such as its worktree, no longer replaces the session's.
- A turn that ends on an API error turns the session "done" (the `StopFailure` hook) instead of leaving it "running". Run `install.sh --hooks-only` again if you installed the hooks with it.
- A permission prompt answered around six seconds after it appeared no longer turns the running session back to "waiting": the `permission_prompt` notification that Claude Code sends late is ignored once the tool has reported back.
- An MCP server asking for input (the `Elicitation` hook) turns the session "waiting" right away, including a request to open a URL, and the answer (`ElicitationResult`) turns it back to "running". Run `install.sh --hooks-only` again if you installed the hooks with it.
- A session whose permission prompt was denied no longer stays "waiting" until the next prompt: like an Esc interrupt, it turns idle 60 seconds later. A tool that fails after approval turns the session back to "running".

## [0.7.0] - 2026-10-17

### Changed

- A session stays "running" while background subagents, shells or monitors it started are still working, instead of showing "done" as soon as the turn ends. It turns done at the next `Stop` with nothing left running (Claude stops again after a background task reports back). Needs a Claude Code version whose `Stop` hook reports `background_tasks`; older versions behave as before.

## [0.6.0] - 2026-10-17

### Changed

- The cursor wraps around: moving up from the first row goes to the last, and down from the last goes to the first.
- The cursor skips session headings. It stops on Claude sessions, and on the `none` line of a tmux session without one (Enter jumps to that session).

## [0.5.0] - 2026-10-16

### Added

- Each session has a second line with the last prompt you sent and how full the context window is (yellow from 50%, red from 80%). Models with a 1M context window (`[1m]`, Fable, Mythos) are counted against 1M tokens.
- The git branch is shown after the directory.

### Changed

- The window index (`1:api`) is shown only when the tmux session has more than one window.

## [0.4.0] - 2026-10-15

### Added

- `@cts_help` option (`on` / `off`) to hide the key help at the bottom of the sidebar.

### Changed

- The session in the current window is marked with a bar on the left edge, separate from the cursor. The cursor is a background highlight that keeps the row's colors instead of reverse video.
- Rows no longer repeat the status word (`running`, `waiting`, …) that the icon already shows; the right side shows the tool and the elapsed time.
- The "none" placeholder and the key help are easier to read: keys are bold, labels are a lighter gray.

## [0.3.0] - 2026-10-15

### Changed

- The toggle key now focuses the sidebar when it is open but not focused, and closes it only when it already has the focus.
- The README no longer suggests `C-s` as a toggle key, since it is tmux-resurrect's default save key.

## [0.2.1] - 2026-10-15

### Security

- Release binaries carry a build provenance attestation (`gh attestation verify`), and are built without restoring a build cache.
- CI checks dependencies with `cargo deny` (advisories, licenses, sources); Dependabot also updates Cargo dependencies.
- The README recommends pinning a release tag.
- GitHub Actions updated (checkout v7, upload-artifact v7, download-artifact v8, setup-rust-toolchain v2), still pinned by commit SHA.

## [0.2.0] - 2026-10-14

### Changed

- Rewritten in Rust as a single binary. Python and jq are no longer needed at runtime; the binary is downloaded from the GitHub release on first use (or built with `cargo`).
- Development: `cargo xtask ci` runs rustfmt, clippy, tests, shellcheck, shfmt and module size limits.

### Fixed

- Closing the sidebar with `q` now restores the window layout too (the pane used to close before the layout could be restored).

## [0.1.2] - 2026-10-14

### Fixed

- Opening and closing the sidebar no longer shrinks panes over time: closing it restores the layout from before it opened, unless panes were added or removed meanwhile.

## [0.1.1] - 2026-10-14

### Changed

- Opening the sidebar focuses it; closing it (toggle or `q`) returns the focus to the pane you were in.

### Fixed

- The sidebar selection could disagree with the focused pane after jumping to another window and coming back.

## [0.1.0] - 2026-10-14

### Added

- Sidebar listing every tmux session and the Claude Code sessions running in it, with running / waiting / done / idle status.
- Popup list (`prefix + B`) that jumps to the selected pane and closes.
- Jump to the next / previous done or waiting Claude (`prefix + C-n` / `prefix + C-p`, `n` / `p` in the sidebar).
- `AskUserQuestion` shows as waiting.
- TPM support (`set -g @plugin 'torikatsupg/claude-tmux-sidebar'`).
- Claude Code plugin `tmux-sidebar` that registers the status hooks.
- `install.sh` for manual setups, with `--hooks-only` and `--uninstall`.
- Options: `@cts_width`, `@cts_toggle_key` (several keys allowed), `@cts_popup_key`, `@cts_next_key`, `@cts_prev_key`, `@cts_default`, `@cts_lang` (`en` / `ja`), `@cts_hook_index`.

[Unreleased]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.7.4...HEAD
[0.7.4]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.7.3...v0.7.4
[0.7.3]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.7.2...v0.7.3
[0.7.2]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/torikatsupg/claude-tmux-sidebar/releases/tag/v0.1.0
