# claude-tmux-sidebar

[English](README.md) | [日本語](README.ja.md)

A tmux sidebar that lists every tmux session and the [Claude Code](https://claude.com/claude-code) sessions running in it, with live status.

![screenshot](docs/screenshot.svg)

- See at a glance which Claude is working, which one is waiting for your approval, and which one has finished.
- Jump to any session with `Enter` or a click.
- tmux sessions without Claude are listed too, so the sidebar doubles as a session switcher.

| Icon | Status | Claude Code hook |
|---|---|---|
| ⠹ (green) | running | `UserPromptSubmit` / `PreToolUse` / `PostToolUse` |
| ◆ (yellow) | waiting for you | `PermissionRequest` / `Notification(permission_prompt)` / `PreToolUse(AskUserQuestion)` |
| ✓ (cyan) | done | `Stop` |
| ○ (gray) | idle | `SessionStart`, 60 s without input (`Notification(idle_prompt)`) |

Each session shows its directory and git branch, with the tool in use and how long it has been in that status on the right. The line below shows the last prompt you sent and how full the context window is (yellow from 50%, red from 80%). The window index (`1:api`) appears only when the tmux session has more than one window. A magenta bar on the left edge marks the window you are in.

## Requirements

- tmux 3.2+
- Claude Code
- macOS or Linux (arm64 / x86_64), with `curl` to download the prebuilt binary — or a [Rust toolchain](https://rustup.rs) to build it
- `jq`, only for `install.sh`

The plugin is one Rust binary. The first time the tmux plugin or a Claude Code hook runs, `scripts/fetch-binary.sh` downloads the binary for your platform from the matching GitHub release (verifying its SHA-256) into `bin/`, or builds it with `cargo` when no release binary fits. Claude Code skips the hook events that arrive before the binary is ready.

Release binaries after v0.2.0 carry a [build provenance attestation](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations). To check that a downloaded binary was built by this repository's release workflow:

```sh
gh attestation verify ~/.tmux/plugins/claude-tmux-sidebar/bin/claude-tmux-sidebar --repo torikatsupg/claude-tmux-sidebar
```

## Install

There are two parts: the **tmux plugin** (draws the sidebar) and the **Claude Code hooks** (report each session's status).

### 1. tmux plugin

With [TPM](https://github.com/tmux-plugins/tpm), add to `~/.tmux.conf`, then press `prefix + I`:

```tmux
set -g @plugin 'torikatsupg/claude-tmux-sidebar'
```

Pinning a release is recommended: `set -g @plugin 'torikatsupg/claude-tmux-sidebar#v0.4.0'`. You then run a tagged release rather than whatever is on `main`, and nothing changes until you change the tag. Release tags cannot be moved or deleted.

### 2. Claude Code hooks

Install the `tmux-sidebar` plugin from inside Claude Code:

```
/plugin install tmux-sidebar --marketplace torikatsupg/claude-tmux-sidebar
```

On older Claude Code versions:

```
/plugin marketplace add torikatsupg/claude-tmux-sidebar
/plugin install tmux-sidebar@torikatsupg
```

To pin a release, add the marketplace with the tag: `/plugin marketplace add torikatsupg/claude-tmux-sidebar#v0.4.0`.

Restart running Claude Code sessions to pick up the hooks.

If you prefer not to use the Claude Code plugin, register the hooks in `~/.claude/settings.json` instead (a backup `settings.json.bak.<timestamp>` is made first):

```sh
~/.tmux/plugins/claude-tmux-sidebar/install.sh --hooks-only
```

Use only one of the two, or every hook runs twice.

### Without TPM

```sh
git clone --branch v0.4.0 https://github.com/torikatsupg/claude-tmux-sidebar ~/.tmux/plugins/claude-tmux-sidebar
~/.tmux/plugins/claude-tmux-sidebar/install.sh
```

This registers the hooks and appends `run-shell '.../sidebar.tmux'` to `~/.tmux.conf` (and loads it right away when run inside tmux).

### Uninstall

- TPM: remove the `@plugin` line and press `prefix + alt + u`.
- Claude Code plugin: `/plugin uninstall tmux-sidebar@torikatsupg`.
- `install.sh`: run `./install.sh --uninstall`.

## Usage

| Key | Action |
|---|---|
| `prefix + b` | Open the sidebar (in every window) and focus it; when it is already open, focus it; when it has the focus, close it |
| `prefix + B` | Show the list in a popup (jumps and closes on select) |
| `prefix + C-n` / `prefix + C-p` | Go to the next / previous Claude that is done or waiting for you |
| `j` / `k` / `g` / `G` | Move the selection |
| `n` / `p` | Select the next / previous done or waiting Claude |
| `Enter` / click | Go to that pane (or that tmux session, on a session row) |
| `q` | Close |

Sessions, windows and panes are listed in tmux's own order. The selection follows the pane you are focused on.

## Options

Set them in `~/.tmux.conf` **before** the plugin is loaded (before `run '~/.tmux/plugins/tpm/tpm'`).

```tmux
set -g @cts_width 34          # sidebar width
set -g @cts_toggle_key b      # several keys allowed, space-separated
set -g @cts_popup_key B
set -g @cts_next_key C-n
set -g @cts_prev_key C-p
set -g @cts_default off       # open the sidebar when tmux starts
set -g @cts_lang en           # en / ja
set -g @cts_help on           # show the key help at the bottom
set -g @cts_hook_index 77     # tmux hook array index (change only on conflicts)
```

Avoid keys tmux already binds (e.g. `C-b` is `send-prefix`): the plugin's binding replaces them.

## How it works

One binary, `claude-tmux-sidebar`, does everything; `bin/cts` and `bin/cts-hook` are small shell entry points for it.

- `claude-tmux-sidebar hook` runs on each Claude Code hook and writes the status, `$TMUX_PANE` and Claude's PID to `~/.cache/claude-tmux-sidebar/<session_id>.json`. `SessionEnd` removes the file.
- `claude-tmux-sidebar toggle`, `ensure`, `fix-width`, … control tmux: tmux hooks such as `after-new-window` add a full-height pane on the left of each window and restore its width after layout changes. Closing the sidebar restores the window's layout from before it opened.
- `claude-tmux-sidebar sidebar` draws the list and cleans up state files whose pane or process is gone. It closes itself in a window where only the sidebar is left.

## Privacy

Everything stays on your machine. State files hold the session ID, working directory, tool name, tmux pane, git branch, context usage and the first 200 characters of your last prompt, and are readable only by you. The git branch and context usage are read from the end of the session's transcript.

For debugging, start Claude with `CTS_DEBUG=1` to append raw hook payloads to `~/.cache/claude-tmux-sidebar/debug.log`. **The log contains your prompts and tool inputs** — delete it when done and check it before sharing.

## Limitations

- A turn interrupted with Esc does not fire `Stop`, so it stays "running" until `idle_prompt` turns it idle 60 seconds later.
- The sidebar is not added to a zoomed window (it would unzoom it); it is added on the next visit.

## Development

Requires a Rust toolchain (the version is pinned in `rust-toolchain.toml`), [shellcheck](https://www.shellcheck.net) and [shfmt](https://github.com/mvdan/sh).

```sh
cargo xtask ci                 # rustfmt, clippy, tests, shellcheck, shfmt and module size limits (what CI runs)
scripts/fetch-binary.sh --build  # build bin/claude-tmux-sidebar from this checkout for a live tmux
scripts/screenshot.sh          # regenerate docs/screenshot.svg (scripts/screenshot.sh ja for the Japanese one)
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

To release, bump `version` in `Cargo.toml`, `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json`, add the section to `CHANGELOG.md`, merge to `main`, and run the **Release** workflow from the Actions tab. It builds the binaries for macOS and Linux (arm64 / x86_64), tags `v<version>` and publishes a GitHub release with them; only the repository owner can run it.

## License

[MIT](LICENSE)
