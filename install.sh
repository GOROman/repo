#!/usr/bin/env bash
set -euo pipefail
root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
if [[ -x "$root/repo" && -f "$root/repo" ]]; then
    binary="$root/repo"
else
    cargo build --release --locked --offline --manifest-path "$root/Cargo.toml"
    binary="$root/target/release/repo"
fi
mkdir -p "$HOME/.local/bin"
install -m 755 "$binary" "$HOME/.local/bin/repo"
python3 "$root/scripts/configure-shell.py"
printf '\nInstalled: %s\nOpen a new shell, or run: source ~/.zshrc\n' "$HOME/.local/bin/repo"
