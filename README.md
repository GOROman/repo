# repo

[![CI](https://github.com/GOROman/repo/actions/workflows/ci.yml/badge.svg)](https://github.com/GOROman/repo/actions/workflows/ci.yml)
[Releases](https://github.com/GOROman/repo/releases)

[English](README.md) | [日本語](README.ja.md)


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
