#!/usr/bin/env python3
"""Install a managed shell block without replacing unrelated shell settings."""
from datetime import datetime
from pathlib import Path
import os
import shutil

path = Path(os.environ.get('REPO_SHELL_RC', str(Path.home() / '.zshrc')))
begin, end = '# >>> repo-rs >>>', '# <<< repo-rs <<<'
block = '\n'.join([begin, 'export PATH="$HOME/.local/bin:$PATH"', 'eval "$("$HOME/.local/bin/repo" --shell-init)"', end])
text = path.read_text() if path.exists() else ''
if begin in text:
    start = text.index(begin)
    stop = text.index(end, start) + len(end)
    updated = text[:start] + block + text[stop:]
else:
    updated = text.rstrip() + '\n\n' + block + '\n'
if updated != text:
    if path.exists():
        backup = path.with_name(path.name + '.repo-backup-' + datetime.now().strftime('%Y%m%d-%H%M%S-%f'))
        shutil.copy2(path, backup)
        print(f'Backup: {backup}')
    path.write_text(updated)  # Follow a dotfiles symlink, preserving its target.
print(f'Shell configured: {path}')
