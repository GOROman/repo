# repo

[![CI](https://github.com/GOROman/repo/actions/workflows/ci.yml/badge.svg)](https://github.com/GOROman/repo/actions/workflows/ci.yml)
[Releases](https://github.com/GOROman/repo/releases)

[日本語](#日本語) | [English](#english)

## 日本語

Rust製の高速な `ghq` + `fzf` リポジトリ選択コマンド。
外部Rustクレートへの依存なし。macOS / Linux向け。

### 必要なもの

- 実行時：Git、[ghq](https://github.com/x-motemen/ghq)、[fzf](https://github.com/junegunn/fzf)。各コマンドを `PATH` に追加してください。
- インストール時：Python 3、Bash。ソースからビルドする場合は Rust stable と Cargo も必要です。
- Rust の外部クレートや Python の追加モジュールは不要です。アイコン表示には Nerd Font 対応フォントを推奨します。
- macOS で Homebrew を使う場合：`brew install git ghq fzf`。ソースからビルドする場合は別途 Rust をインストールしてください。

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

## English

A fast `ghq` + `fzf` repository picker written in Rust for macOS and Linux. It has no external Rust crate dependencies.

### Requirements

- Runtime: Git, [ghq](https://github.com/x-motemen/ghq), and [fzf](https://github.com/junegunn/fzf), available on `PATH`.
- Installer: Python 3 and Bash. Building from source also requires stable Rust and Cargo.
- No additional Python modules or Rust crates are required. A Nerd Font is recommended for the icons.
- On macOS with Homebrew: `brew install git ghq fzf`. Install Rust separately to build from source.

### Installation

Build from source:

```sh
git clone https://github.com/GOROman/repo.git
cd repo
./install.sh
source ~/.zshrc
```

The installer places the binary in `~/.local/bin/repo` and adds a managed block to `.zshrc`. It preserves existing settings, backs up the file before changing it, and replaces an earlier `repo()` function at shell startup. For Bash, use `REPO_SHELL_RC="$HOME/.bashrc" ./install.sh` and source `.bashrc`.

To install a prebuilt binary without Rust, download the appropriate archive and its checksum from [Releases](https://github.com/GOROman/repo/releases), verify and extract it, then run `./install.sh`. The example above uses `gh release download`; GitHub CLI is only needed for that download method. Choose `macos-arm64` for Apple Silicon, `macos-x86_64` for Intel Macs, or `linux-x86_64` for Linux x86-64.

For manual integration, run `cargo install --path .`, ensure `~/.cargo/bin` is on `PATH`, and add `eval "$(command repo --shell-init)"` to your shell configuration.

### Usage

```sh
repo                 # Select and enter a repository
repo chromatic       # Start with a search query
repo --refresh       # Refresh the list and Git information
repo --list          # Print paths without opening fzf
```

Repositories are ordered by directory modification time, with recent entries at the bottom of fzf. Worktrees named with `=worktree` are grouped after their parent. Branches and unpushed commit counts are shown. When multiple entries are selected, the binary outputs all paths and the shell function enters the first one.

### Performance and cache

The `ghq list` result is cached for 10 seconds. Git information is refreshed when relevant refs or configuration change; `--refresh` forces a refresh. Up to eight Git queries run concurrently. The cache lives in `$XDG_CACHE_HOME/repo-rs` or `~/.cache/repo-rs` and can be deleted safely. The program does not fetch from the network.

In a local test with 146 repositories, list preparation took 6.50 seconds for the original sequential Python version, 2.90 seconds for an eight-worker Python version, 1.17 seconds for a forced Rust refresh, and 0.033–0.042 seconds with the Rust cache. Results depend on the environment and exclude fzf interaction and shell startup.

### Verification and releases

```sh
cargo build --release --offline
cargo clippy --offline -- -D warnings
python3 tests/smoke.py
```

The smoke test uses real Git repositories to check caching, ref changes, branch switches, worktrees, paths with spaces, forced refresh, and recovery from a corrupt cache. CI runs formatting, Clippy, tests, builds, and the smoke test on Linux and macOS. Version tags matching `Cargo.toml` build and publish release archives and SHA-256 checksums; manual release workflow runs produce artifacts only.
