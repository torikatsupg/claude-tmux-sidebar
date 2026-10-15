# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Security

- Release binaries carry a build provenance attestation (`gh attestation verify`), and are built without restoring a build cache.
- CI checks dependencies with `cargo deny` (advisories, licenses, sources); Dependabot also updates Cargo dependencies.
- The README recommends pinning a release tag.

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

[Unreleased]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/torikatsupg/claude-tmux-sidebar/releases/tag/v0.1.0
