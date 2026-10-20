# claude-tmux-sidebar

[English](README.md) | [日本語](README.ja.md)

tmux で Claude Code を何本も並行して動かしていると、どれが承認待ちで、どれがもう終わったのか、すぐに見失います。claude-tmux-sidebar は tmux の全ウィンドウの左側にサイドバーを出し、全 tmux セッションとその中で動いている [Claude Code](https://claude.com/claude-code) セッションを、状態つきでリアルタイムに一覧表示します。

![デモ: サイドバーに並ぶ Claude Code セッションの 1 つが黄色（承認待ち）に変わり、サイドバーで Enter を押してそこへ移動する様子](docs/demo.gif)

- どの Claude が作業中か、承認待ちか、終わったかが一目で分かります。
- `Enter` かクリックでそのセッションへ移動できます。`prefix + C-n` なら、完了・承認待ちの次の Claude へ直接飛べます。
- Claude が動いていない tmux セッションも表示するので、セッション切り替えにも使えます。

## クイックスタート

1. [TPM](https://github.com/tmux-plugins/tpm) を使う場合、`~/.tmux.conf` に次の行を追記して `prefix + I` を押します:

   ```tmux
   set -g @plugin 'torikatsupg/claude-tmux-sidebar#v0.7.1'
   ```

2. Claude Code の中で hook を入れ、起動済みの Claude Code を再起動します:

   ```
   /plugin install tmux-sidebar --marketplace torikatsupg/claude-tmux-sidebar
   ```

`prefix + b` でサイドバーが開きます。ほかの入れ方は [インストール](#インストール) を参照してください。

## 状態表示

![各状態のセッションを表示したサイドバー](docs/screenshot-ja.svg)

| 表示 | 状態 | どんなとき |
|---|---|---|
| ⠹ (緑) | 稼働中 | Claude が作業中、またはバックグラウンドで起動したサブエージェント・shell がまだ動いている |
| ◆ (黄) | 承認待ち・質問待ち | Claude が承認を求めている、または質問している |
| ✓ (水色) | 完了 | ターンが終わり、動いているものが残っていない |
| ○ (灰) | 待機 | 起動直後、または 60 秒入力がない |

各セッションにはディレクトリと git ブランチを表示し、右側に使用中のツールとその状態になってからの経過時間を表示します。下の行には最後に送ったプロンプトと context window の使用率を表示します（50% から黄色、80% から赤）。ウィンドウ番号（`1:api`）は、その tmux セッションにウィンドウが 2 つ以上あるときだけ付きます。左端のマゼンタのバーは、今いるウィンドウの印です。

## 操作

| キー | 動作 |
|---|---|
| `prefix + b` | サイドバーを開いてフォーカス（全ウィンドウ）。開いていればフォーカス、フォーカス中なら閉じる |
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
set -g @cts_toggle_key b      # 複数指定可（空白区切り）
set -g @cts_popup_key B
set -g @cts_next_key C-n
set -g @cts_prev_key C-p
set -g @cts_default off       # tmux 起動時に開いておくか
set -g @cts_lang ja           # en / ja
set -g @cts_help on           # 下端のキー操作ヘルプを表示
set -g @cts_hook_index 77     # tmux hook 配列のインデックス（衝突時のみ変更）
```

tmux が既に割り当てているキー（例: `C-b` は `send-prefix`）を指定すると、プラグイン側の割り当てで上書きされます。

## 必要なもの

- tmux 3.2+
- Claude Code
- macOS または Linux（arm64 / x86_64）。ビルド済みバイナリのダウンロードに `curl`、または自前でビルドするなら [Rust ツールチェーン](https://rustup.rs)
- `jq`（`install.sh` を使う場合のみ）

初回の起動時に、対応する GitHub Release から自分の環境向けのバイナリをダウンロードし、SHA-256 を検証します。合うバイナリが無ければ `cargo` でビルドします。リリースのバイナリには [build provenance attestation](https://docs.github.com/ja/actions/security-for-github-actions/using-artifact-attestations) が付いていて、次で確認できます:

```sh
gh attestation verify ~/.tmux/plugins/claude-tmux-sidebar/bin/claude-tmux-sidebar --repo torikatsupg/claude-tmux-sidebar
```

## インストール

サイドバーを描く **tmux プラグイン** と、各セッションの状態を書き出す **Claude Code の hook** の 2 つを入れます。[クイックスタート](#クイックスタート) で両方入ります。それ以外の入れ方:

- **リリースではなく `main` を追う**: `set -g @plugin 'torikatsupg/claude-tmux-sidebar'`。タグでの固定をおすすめします。タグを書き換えるまで何も変わらず、リリースのタグは移動も削除もできません。
- **Claude Code プラグインをリリースに固定する**: `/plugin marketplace add torikatsupg/claude-tmux-sidebar#v0.7.1` のあと `/plugin install tmux-sidebar@torikatsupg`。タグを付けずに同じ 2 行を実行すれば、`--marketplace` の無い古い Claude Code でも入ります。
- **Claude Code プラグインを使わずに hook を入れる**: `~/.tmux/plugins/claude-tmux-sidebar/install.sh --hooks-only` で `~/.claude/settings.json` に登録します（実行前にバックアップ `settings.json.bak.<日時>` を作成）。プラグインと両方入れると hook が 2 回ずつ走るので、どちらか一方にしてください。
- **TPM を使わない**:

  ```sh
  git clone --branch v0.7.1 https://github.com/torikatsupg/claude-tmux-sidebar ~/.tmux/plugins/claude-tmux-sidebar
  ~/.tmux/plugins/claude-tmux-sidebar/install.sh
  ```

  hook を登録し、`~/.tmux.conf` に `run-shell '.../sidebar.tmux'` を追記します（tmux の中で実行した場合はその場で読み込みます）。

起動済みの Claude Code は再起動すると hook が有効になります。

### アンインストール

- TPM: `@plugin` 行を消して `prefix + alt + u`
- Claude Code プラグイン: `/plugin uninstall tmux-sidebar@torikatsupg`
- `install.sh`: `./install.sh --uninstall`

## プライバシー

データは手元のマシンから出ません。各 Claude セッションの状態は `~/.cache/claude-tmux-sidebar/<session_id>.json` に本人だけが読める権限で保存され、セッションが終わると消えます。中身はセッション ID・作業ディレクトリ・ツール名・tmux ペイン・Claude のプロセス ID・git ブランチ・context の使用率・最後のプロンプトの先頭 200 文字です。git ブランチと context の使用率は、セッションの transcript の末尾から読み取ります。

デバッグ用に `CTS_DEBUG=1` を付けて Claude を起動すると、hook の生 payload が `~/.cache/claude-tmux-sidebar/debug.log` に残ります。**このログにはプロンプトやツールの入力がそのまま入る**ので、使い終わったら消し、共有する前に中身を確認してください。

## 制約

- Esc で中断したターンや、承認を拒否して止まったターンは、60 秒後に「待機」に戻るまで「稼働中」か「承認待ち」のままです。
- zoom 中のウィンドウにはサイドバーを足しません（zoom が解除されてしまうため）。次に訪れたときに足します。

## 開発に参加する

仕組み・開発環境・リリース手順は [CONTRIBUTING.md](CONTRIBUTING.md)（英語）にまとめています。

## ライセンス

[MIT](LICENSE)
