# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `cts status` prints how many Claude sessions are waiting, running and done (`◆2 ⠹1 ✓3`) for tmux's `status-right`; `--color` colors each count with the `@cts_color_*` options (#116).
- `@cts_notify` (`off` by default, `tmux`, `os` or `all`) notifies you when a session starts waiting for you or is done, unless the client you use shows its window: a tmux message and a bell in its window, or a desktop notification with `osascript` / `notify-send`. The hooks read the value from a file the plugin writes when tmux loads it, so reload the config after a change (#116).
- While the sidebar has the focus, a preview below the list shows what the selected row last said: the end of Claude's last reply from the transcript, what Claude asks for while it waits (the tool and its command, file or question), or the end of the screen for a window without Claude. Lines wrap to the sidebar width, and the rule above names the row in the color of tmux's pane borders. The preview follows the cursor, refreshes with the list and takes at most a third of the sidebar; without the focus the list keeps the full height. This is on by default, so the focused sidebar looks different after the update; `set -g @cts_preview off` brings back the list alone. `@cts_preview popup` shows the pane's screen in a tmux popup on a key instead (`Space`, set with `@cts_preview_key`) (#117).

## [0.12.1] - 2026-11-11

### Changed

- Updated the `unicode-width` (0.2.3) and `rustix` (1.1.6) dependencies (#112, #113).

## [0.12.0] - 2026-11-02

### Changed

- The window you are in is marked by the `@cts_color_active` background (default `colour235`) on its rows, instead of the bar in the left column. Rows now start at the left edge, one column wider (#108).
- When the cursor is on a row of the window you are in, the background is the cursor's and the window index (`2:`) takes the accent color, so the two marks do not hide each other (#108).
- The tree is indented one space per level instead of two. `@cts_indent` sets 0 to 4 spaces per level (#108).
- In the window tree, a window without Claude is a faint `#I:#W` at the window level, like the window headings, instead of a `·` row whose name sat one level deeper (#104).
- Every Claude session takes two lines: the detail line shows even before the first prompt, and a session that has not replied yet shows `0%` context usage. Right after `/compact` the usage is still left blank until the next reply (#105).

## [0.11.0] - 2026-11-02

### Changed

- The window tree of `@cts_label auto` is the default. Set `@cts_label dir` for the flat list of earlier versions (#97).

- `@cts_label auto` shows a tree of sessions, windows and Claude rows, and keeps the directory and branch on each row. In a session with several windows, each window is a heading: `#I:#W` when named by hand, otherwise `#I`. A session with one window has no window heading and reads `session/name` when the window is named; the new `@cts_single_window lines` option gives a named window its own heading instead (#94).
- Session headings lose their `▾`, which looked like a control that folds the list (#94).

## [0.10.0] - 2026-11-02

### Added

- `@cts_label auto` shows a window named by hand as `#I:#W` and keeps the directory and branch for any other, since tmux's automatic name for a Claude Code window is its version number (#89).

### Fixed

- Context usage is right without any setup for models that run with the 1M window by default: Opus 4.7 and later, Sonnet 5 and later, and Haiku 5.5 and later no longer show five times their real usage. The size follows Claude Code's documented rules (#88).

## [0.9.0] - 2026-11-02

### Added

- `claude-tmux-sidebar statusline` records each session's real context window size from Claude Code's status line input. Add one line to your status line script, as the README shows, and usage needs no `@cts_context_windows`; the recorded size goes before that option and the guess (#84).

## [0.8.0] - 2026-11-02

### Added

- `@cts_color_accent`, `@cts_color_cursor`, `@cts_color_muted`, `@cts_color_dim`, `@cts_color_waiting`, `@cts_color_running`, `@cts_color_done`, `@cts_color_warning` and `@cts_color_critical` options set the sidebar's colors as a basic tmux color name, `colourN` or `#rrggbb` (#71). Without them the sidebar looks as before.
- The `@cts_context_windows` option sets the context window size of each model, as space-separated `model=tokens` pairs. A model that runs with the 1M window but has no `[1m]` in its id, such as `claude-opus-5-5` on Claude Code 2.1.316, showed five times its real context usage; list it with its size to fix that. Models the option does not list keep the size guessed so far (#67).
- The `@cts_label` option chooses what each row names: `dir` (the default) shows the directory and branch as before, and `window` shows the tmux window as `#I:#W` (index and name), following a renamed window on the next refresh (#69).
- The `@cts_all_windows` option lists every tmux window when set to `on`: a window without Claude shows as a dim `#I:#W` row that `Enter` and a click go to, and that `n` / `p` skip (#68). A window linked into several sessions is listed once. Unset or `off` keeps the list as before.

### Fixed

- The list scrolls to keep the cursor on screen when its rows do not fit the pane; before, the cursor could move off the bottom and rows there could not be seen or clicked (#78).

## [0.7.5] - 2026-10-28

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

[Unreleased]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.12.1...HEAD
[0.12.1]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.7.5...v0.8.0
[0.7.5]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.7.4...v0.7.5
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
