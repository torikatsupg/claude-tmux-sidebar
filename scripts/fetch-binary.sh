#!/usr/bin/env bash
# Installs bin/claude-tmux-sidebar for this checkout's version: the release binary when there is one
# (checked against its published SHA-256), otherwise a local `cargo build`. Does nothing when the right
# version is already installed.
#   scripts/fetch-binary.sh          install if missing or outdated
#   scripts/fetch-binary.sh --build  build from source (development)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROOT
readonly REPO_URL="https://github.com/torikatsupg/claude-tmux-sidebar"
readonly NAME="claude-tmux-sidebar"
readonly BIN="$ROOT/bin/$NAME"
readonly LOCK="$ROOT/bin/.fetch.lock"
readonly LOCK_WAIT_SECONDS=300

version() {
    sed -n 's/^version = "\(.*\)"$/\1/p' "$ROOT/Cargo.toml" | head -n 1
}

installed() { # version
    [ -x "$BIN" ] && [ "$("$BIN" --version 2>/dev/null)" = "$NAME $1" ]
}

target() {
    case "$(uname -s)/$(uname -m)" in
    Darwin/arm64) echo aarch64-apple-darwin ;;
    Darwin/x86_64) echo x86_64-apple-darwin ;;
    Linux/x86_64) echo x86_64-unknown-linux-musl ;;
    Linux/aarch64 | Linux/arm64) echo aarch64-unknown-linux-musl ;;
    *) return 1 ;;
    esac
}

sha256() { # file
    if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d ' ' -f 1
}

# Moves a finished binary into place atomically, so running sidebars and hooks never see half a file.
place() { # file
    chmod 755 "$1" && mv -f "$1" "$BIN"
}

download() { # version
    local target url tmp expected
    target=$(target) || return 1
    command -v curl >/dev/null || return 1
    url="$REPO_URL/releases/download/v$1/$NAME-$target"
    tmp=$(mktemp "$BIN.XXXXXX")
    if curl -fsSL --retry 2 -o "$tmp" "$url" &&
        expected=$(curl -fsSL --retry 2 "$url.sha256") &&
        [ "$(sha256 "$tmp")" = "${expected%% *}" ]; then
        place "$tmp"
    else
        rm -f "$tmp"
        return 1
    fi
}

build() {
    local tmp
    command -v cargo >/dev/null || return 1
    cargo build --release --locked --quiet --manifest-path "$ROOT/Cargo.toml" || return 1
    tmp=$(mktemp "$BIN.XXXXXX")
    cp "$ROOT/target/release/$NAME" "$tmp" && place "$tmp"
}

# tmux and Claude Code start several installs at once; one runs, the others wait for it.
lock() {
    local waited=0 holder
    until mkdir "$LOCK" 2>/dev/null; do
        holder=$(cat "$LOCK/pid" 2>/dev/null || true)
        if [ -n "$holder" ] && ! kill -0 "$holder" 2>/dev/null; then
            rm -rf "$LOCK" # left behind by an install that was killed
            continue
        fi
        [ "$waited" -ge "$LOCK_WAIT_SECONDS" ] && return 1
        sleep 1
        waited=$((waited + 1))
    done
    echo $$ >"$LOCK/pid"
    trap 'rm -rf "$LOCK"' EXIT
}

fail() { # version
    local -r message="$NAME: no prebuilt binary for v$1 on this platform, and no cargo to build one"
    echo "$message" >&2
    [ -n "${TMUX:-}" ] && tmux display-message "$message"
    return 1
}

main() {
    local -r mode="${1:-}"
    local version
    version=$(version)
    [ "$mode" != --build ] && installed "$version" && return 0
    lock
    if [ "$mode" = --build ]; then
        build
        return
    fi
    installed "$version" && return 0 # installed by another process while this one waited
    download "$version" || build || fail "$version"
}

main "$@"
