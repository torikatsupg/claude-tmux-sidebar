# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/torikatsupg/claude-tmux-sidebar/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/torikatsupg/claude-tmux-sidebar/releases/tag/v0.1.0
