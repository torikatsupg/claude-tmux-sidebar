# claude-tmux-sidebar

tmux の全ウィンドウ左側に、全 tmux セッションとその中で起動中の Claude Code セッションを一覧表示するサイドバー。

```
 Claude ●1 ◆1 ✓1
──────────────────────
 ▾ work
 ⠹ 0:api     Bash 稼働中 2s
 ◆ 1:web   Edit 承認待ち 1m
 ▾ personal
 ✓ 0:blog         完了 5m
 ▾ scratch
   なし
```

Claude が動いていない tmux セッションも「なし」として表示します。選択行は今フォーカスしているペイン（Claude が無ければそのセッション）に追従します。

| 表示 | 状態 | 元になる Claude Code hook |
|---|---|---|
| ⠹ (緑) | 稼働中 | `UserPromptSubmit` / `PreToolUse` / `PostToolUse` |
| ◆ (黄) | 承認待ち | `PermissionRequest` / `Notification(permission_prompt)` |
| ✓ (水色) | 完了 | `Stop` |
| ○ (灰) | 待機 | `SessionStart`、60 秒放置 (`Notification(idle_prompt)`) |

## 必要なもの

tmux 3.2+ / python3 / jq

## インストール

```sh
ghq get https://github.com/torikatsupg/claude-tmux-sidebar
~/ghq/github.com/torikatsupg/claude-tmux-sidebar/install.sh
```

- `~/.claude/settings.json` に hook を追加します（実行前にバックアップ `settings.json.bak.<日時>` を作成）。
- `~/.tmux.conf` に `run-shell '.../sidebar.tmux'` を追記し、tmux の中で実行した場合はその場で読み込みます。
- 起動済みの Claude Code は再起動すると hook が有効になります。

アンインストール: `./install.sh --uninstall`

## 操作

| キー | 動作 |
|---|---|
| `prefix + b` | サイドバー開閉（全ウィンドウ） |
| `prefix + B` | ポップアップで一覧（選択するとジャンプして閉じる） |
| `j` / `k` / `g` / `G` | 選択移動（サイドバー内） |
| `Enter` / クリック | そのペイン（セッション行ならその tmux セッション）へ移動 |
| `q` | 閉じる |

## 設定 (`~/.tmux.conf`、`run-shell` 行より前に書く)

```tmux
set -g @cts_width 34          # サイドバー幅
set -g @cts_toggle_key b
set -g @cts_popup_key B
set -g @cts_default on        # tmux 起動時に開いておくか
set -g @cts_hook_index 77     # tmux hook 配列のインデックス（衝突時のみ変更）
```

## 仕組み

- `bin/cts-hook`: Claude Code の各 hook から呼ばれ、`~/.cache/claude-tmux-sidebar/<session_id>.json` に状態・`$TMUX_PANE`・Claude の PID を書きます。`SessionEnd` で削除します。
- `bin/cts`: tmux 側の制御。`after-new-window` などの tmux hook でウィンドウ左端に全高ペインを足し、レイアウト変更時は幅を戻します。
- `bin/cts-sidebar`: curses の描画。ペインが消えた・プロセスが死んだセッションの状態ファイルは掃除します。サイドバーしか残っていないウィンドウでは自分を閉じます。

デバッグ: `CTS_DEBUG=1` を付けて Claude を起動すると、hook の生 payload が `~/.cache/claude-tmux-sidebar/debug.log` に残ります。

## 制約

- Esc で中断したターンは `Stop` が飛ばないため、60 秒後の `idle_prompt` で「待機」に戻るまで「稼働中」のままです。
- zoom 中のウィンドウにはサイドバーを足しません（zoom が解除されてしまうため）。
