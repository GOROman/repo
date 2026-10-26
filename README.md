# repo

Rust製の高速な `ghq` + `fzf` リポジトリ選択コマンド。
外部Rustクレートへの依存なし。macOS / Linux向け。

## インストール

Rust、Git、ghq、fzfが必要です。

```sh
cargo install --path .
```

`.zshrc` または `.bashrc` に追加します（以前の `repo()` は削除）：

```sh
eval "$(command repo --shell-init)"
```

```sh
repo                 # 選択したリポジトリへ移動
repo chromatic       # 初期検索文字列
repo --refresh       # 一覧・Git情報の強制再取得
repo --list          # 選択画面を出さずパス一覧を出力
```

更新日時順に表示し、最近のリポジトリをfzfの下側に配置。
`=worktree` を含む同じ親ディレクトリのworktreeを親の直後にまとめます。
ブランチ名と未push数を表示。fzfで複数選択した場合、コマンドは全パスを出力し、
シェル関数は最初のパスへ移動します。

## 高速化

- `ghq list` は10秒キャッシュ。新規clone直後などは `--refresh`。
- ブランチ名は `.git/HEAD` から直接取得。worktreeのgitdir/commondirにも対応。
- HEAD、refs、packed-refs、configなどの内容からキャッシュキーを計算。
  ローカルコミット・ブランチ切替・fetch後に対応する情報を再計算。
- 未push数の `git rev-list` は初回と変更時のみ、最大8並列で実行。
- キャッシュは `$XDG_CACHE_HOME/repo-rs`（既定 `~/.cache/repo-rs`）。
  削除しても再生成されます。ネットワークへのfetchは行いません。
- fzfの表示文字列からパスを復元せず、行IDで選択結果を対応付け。

146リポジトリを使ったローカル計測（選択画面が開く前の一覧準備）：

| 実装 | 時間 |
| --- | ---: |
| 既存Python版・逐次実行 | 6.50秒 |
| Python版・8並列 | 2.90秒 |
| Rust版・強制再取得 | 1.17秒 |
| Rust版・キャッシュあり | 0.033–0.042秒 |

環境・リポジトリ数・参照数により変わります。fzf操作やシェル起動時間は含みません。

## 検証

```sh
cargo build --release --offline
cargo clippy --offline -- -D warnings
python3 tests/smoke.py
```

実Gitリポジトリで、キャッシュ・ローカル/追跡参照の更新・ブランチ切替・worktree・
空白入りパス・強制更新・壊れた情報キャッシュからの復旧を検証します。
