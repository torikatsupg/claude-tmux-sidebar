# claude-tmux-sidebar

[English](README.md) | [日本語](README.ja.md)

tmux の全ウィンドウ左側に、全 tmux セッションとその中で起動中の [Claude Code](https://claude.com/claude-code) セッションを状態つきで一覧表示するサイドバー。

![screenshot](docs/screenshot-ja.svg)

- どの Claude が作業中か、承認待ちか、終わったかが一目で分かります。
- `Enter` かクリックでそのセッションへ移動できます。
- Claude が動いていない tmux セッションも表示するので、セッション切り替えにも使えます。

| 表示 | 状態 | 元になる Claude Code hook |
|---|---|---|
| ⠹ (緑) | 稼働中 | `UserPromptSubmit` / `PreToolUse` / `PostToolUse` |
| ◆ (黄) | 承認待ち・質問待ち | `PermissionRequest` / `Notification(permission_prompt)` / `PreToolUse(AskUserQuestion)` |
| ✓ (水色) | 完了 | `Stop` |
| ○ (灰) | 待機 | `SessionStart`、60 秒放置 (`Notification(idle_prompt)`) |

## 必要なもの

- tmux 3.2+
- python3 / jq / bash
- Claude Code

## インストール

サイドバーを描く **tmux プラグイン** と、各セッションの状態を書き出す **Claude Code の hook** の 2 つを入れます。

### 1. tmux プラグイン

[TPM](https://github.com/tmux-plugins/tpm) を使う場合、`~/.tmux.conf` に追記して `prefix + I`:

```tmux
set -g @plugin 'torikatsupg/claude-tmux-sidebar'
```

リリースを固定するなら `set -g @plugin 'torikatsupg/claude-tmux-sidebar#v0.1.1'`。

### 2. Claude Code の hook

Claude Code の中で `tmux-sidebar` プラグインを入れます:

```
/plugin install tmux-sidebar --marketplace torikatsupg/claude-tmux-sidebar
```

古い Claude Code の場合:

```
/plugin marketplace add torikatsupg/claude-tmux-sidebar
/plugin install tmux-sidebar@torikatsupg
```

起動済みの Claude Code は再起動すると hook が有効になります。

Claude Code プラグインを使わない場合は、`~/.claude/settings.json` に直接 hook を登録します（実行前にバックアップ `settings.json.bak.<日時>` を作成）:

```sh
~/.tmux/plugins/claude-tmux-sidebar/install.sh --hooks-only
```

両方入れると hook が 2 回ずつ走るので、どちらか一方にしてください。

### TPM を使わない場合

```sh
git clone https://github.com/torikatsupg/claude-tmux-sidebar ~/.tmux/plugins/claude-tmux-sidebar
~/.tmux/plugins/claude-tmux-sidebar/install.sh
```

hook を登録し、`~/.tmux.conf` に `run-shell '.../sidebar.tmux'` を追記します（tmux の中で実行した場合はその場で読み込みます）。

### アンインストール

- TPM: `@plugin` 行を消して `prefix + alt + u`
- Claude Code プラグイン: `/plugin uninstall tmux-sidebar@torikatsupg`
- `install.sh`: `./install.sh --uninstall`

## 操作

| キー | 動作 |
|---|---|
| `prefix + b` | サイドバー開閉（全ウィンドウ） |
| `prefix + B` | ポップアップで一覧（選択するとジャンプして閉じる） |
| `prefix + C-n` / `prefix + C-p` | 次 / 前の完了・承認待ちの Claude へ移動 |
| `j` / `k` / `g` / `G` | 選択移動 |
| `n` / `p` | 次 / 前の完了・承認待ちの Claude を選択 |
| `Enter` / クリック | そのペイン（セッション行ならその tmux セッション）へ移動 |
| `q` | 閉じる |

セッション・ウィンドウ・ペインは tmux と同じ順に並びます。選択行は今フォーカスしているペインに追従します。

## 設定

`~/.tmux.conf` の、プラグインを読み込む行（`run '~/.tmux/plugins/tpm/tpm'`）より**前**に書きます。

```tmux
set -g @cts_width 34          # サイドバー幅
set -g @cts_toggle_key b      # 複数指定可（例: 'b C-s'）
set -g @cts_popup_key B
set -g @cts_next_key C-n
set -g @cts_prev_key C-p
set -g @cts_default off       # tmux 起動時に開いておくか
set -g @cts_lang ja           # en / ja
set -g @cts_hook_index 77     # tmux hook 配列のインデックス（衝突時のみ変更）
```

tmux が既に割り当てているキー（例: `C-b` は `send-prefix`）を指定すると、プラグイン側の割り当てで上書きされます。

## 仕組み

- `bin/cts-hook`: Claude Code の各 hook から呼ばれ、`~/.cache/claude-tmux-sidebar/<session_id>.json` に状態・`$TMUX_PANE`・Claude の PID を書きます。`SessionEnd` で削除します。
- `bin/cts`: tmux 側の制御。`after-new-window` などの tmux hook でウィンドウ左端に全高ペインを足し、レイアウト変更時は幅を戻します。
- `bin/cts-sidebar`: curses の描画。ペインが消えた・プロセスが死んだセッションの状態ファイルは掃除します。サイドバーしか残っていないウィンドウでは自分を閉じます。

## プライバシー

データは手元のマシンから出ません。状態ファイルにはセッション ID・作業ディレクトリ・ツール名・tmux ペインが入り、本人だけが読める権限で作られます。

デバッグ用に `CTS_DEBUG=1` を付けて Claude を起動すると、hook の生 payload が `~/.cache/claude-tmux-sidebar/debug.log` に残ります。**このログにはプロンプトやツールの入力がそのまま入る**ので、使い終わったら消し、共有する前に中身を確認してください。

## 制約

- Esc で中断したターンは `Stop` が飛ばないため、60 秒後の `idle_prompt` で「待機」に戻るまで「稼働中」のままです。
- zoom 中のウィンドウにはサイドバーを足しません（zoom が解除されてしまうため）。次に訪れたときに足します。

## 開発

```sh
shellcheck bin/cts bin/cts-hook sidebar.tmux install.sh scripts/*.sh
scripts/screenshot.sh        # docs/screenshot.svg を再生成（日本語版は scripts/screenshot.sh ja）
```

リリースは、`.claude-plugin/plugin.json` と `.claude-plugin/marketplace.json` の `version` を上げ、`CHANGELOG.md` に節を足して `main` にマージしたあと、Actions タブから **Release** ワークフローを実行します。`v<version>` のタグと GitHub Release を作ります。実行できるのはリポジトリのオーナーだけです。

## ライセンス

[MIT](LICENSE)
