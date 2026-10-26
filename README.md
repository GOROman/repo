# repo

[![CI](https://github.com/GOROman/repo/actions/workflows/ci.yml/badge.svg)](https://github.com/GOROman/repo/actions/workflows/ci.yml)
[Releases](https://github.com/GOROman/repo/releases)

Rust製の高速な `ghq` + `fzf` リポジトリ選択コマンド。
外部Rustクレートへの依存なし。macOS / Linux向け。

## インストール

Git、ghq、fzfを使用します。インストーラーにはPython 3が必要です。

### ソースから（macOS / Linux）

Rust stableも必要です。

```sh
git clone https://github.com/GOROman/repo.git
cd repo
./install.sh
source ~/.zshrc
```

`~/.local/bin/repo` にインストールし、`.zshrc` の末尾へ管理ブロックを追加します。
既存の設定は保持し、変更前にバックアップを作成。以前のPython版 `repo()` は
新しい関数で上書きされます。Mac Studioでも同じ手順で利用できます。
Bashでは `REPO_SHELL_RC="$HOME/.bashrc" ./install.sh` を使ってください。

### Releaseバイナリから（Rust不要）

[Releases](https://github.com/GOROman/repo/releases)から環境に合うアーカイブを取得します。
非公開リポジトリの場合は、認証済みの `gh` を使えます。

```sh
mkdir repo-release && cd repo-release
gh release download v0.1.0 --repo GOROman/repo --pattern 'repo-macos-arm64.tar.gz*'
shasum -a 256 -c repo-macos-arm64.tar.gz.sha256
tar -xzf repo-macos-arm64.tar.gz
./install.sh
source ~/.zshrc
```

Apple Siliconは `macos-arm64`、Intel Macは `macos-x86_64`、
Linux x86_64は `linux-x86_64` を選びます。

### 手動でシェルへ組み込む

```sh
cargo install --path .
# ~/.cargo/bin がPATHに含まれていることを確認して .zshrc / .bashrc に追加
eval "$(command repo --shell-init)"
```

## 使い方

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

## CI / CD

- `main` へのpush・PR：Linux / macOSでfmt、Clippy、テスト、releaseビルド、実Gitを使うスモークテスト。
- `v*` タグ：macOS arm64 / x86_64とLinux x86_64で同じ検証を行い、バイナリを作成。
- 全ビルド成功後、GitHub ReleaseへアーカイブとSHA-256を自動アップロード。
- タグは `Cargo.toml` のバージョンと一致する必要があります。
- Releaseワークフローの手動実行ではartifactだけを生成します。

```sh
# Cargo.toml の version を更新・commitしてから
git tag -a v0.1.0 -m "repo v0.1.0"
git push origin main v0.1.0
```
