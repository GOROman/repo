"""Exercise real Git repositories and the executable with isolated fake ghq/fzf."""
import os, pathlib, subprocess, tempfile
BIN = pathlib.Path(__file__).resolve().parents[1] / 'target/release/repo'
with tempfile.TemporaryDirectory(prefix='repo-test-') as tmp:
    root = pathlib.Path(tmp)
    tools = root / 'bin'; tools.mkdir()
    cache = root / 'cache'
    project = root / 'owner' / 'space project'; project.mkdir(parents=True)
    def git(*args):
        return subprocess.check_output(['git', '-C', str(project), *args], stderr=subprocess.DEVNULL, text=True).strip()
    git('init', '-b', 'main'); git('config', 'user.name', 'Test'); git('config', 'user.email', 'test@example.com')
    git('commit', '--allow-empty', '-m', 'initial')
    git('update-ref', 'refs/remotes/origin/main', 'HEAD')
    git('config', 'branch.main.remote', 'origin'); git('config', 'branch.main.merge', 'refs/heads/main')
    git('config', 'remote.origin.url', str(project)); git('config', 'remote.origin.fetch', '+refs/heads/*:refs/remotes/origin/*')
    child = project.parent / 'space project=worktree-feature'
    git('worktree', 'add', '-b', 'feature', str(child))
    for name, text in {
        'ghq': '#!/bin/sh\nprintf "%s\\n" "$TEST_PROJECT" "$TEST_CHILD"\n',
        'fzf': '#!/usr/bin/env python3\nimport sys,os\ns=sys.stdin.read();open(os.environ["FZF_CAPTURE"],"w").write(s);print(s.splitlines()[0])\n',
    }.items():
        p=tools/name; p.write_text(text); p.chmod(0o755)
    env=dict(os.environ, PATH=str(tools)+':'+os.environ['PATH'], XDG_CACHE_HOME=str(cache), TEST_PROJECT=str(project), TEST_CHILD=str(child), FZF_CAPTURE=str(root/'capture'))
    def run(*args): return subprocess.check_output([str(BIN), *args], env=env, text=True)
    assert run('--list').splitlines() == [str(project), str(child)]
    before=run(); assert before.strip() == str(project)
    assert '(main)' in (root/'capture').read_text()
    assert run() == before  # warm cache, paths containing spaces
    git('commit', '--allow-empty', '-m', 'ahead')
    run(); assert '⬆1' in (root/'capture').read_text()  # ref invalidates cache
    git('update-ref', 'refs/remotes/origin/main', 'HEAD')
    run(); assert '⬆1' not in (root/'capture').read_text()  # upstream ref invalidation
    git('checkout', '-b', 'new-branch')
    run(); assert '(new-branch)' in (root/'capture').read_text()
    assert run('--refresh', '--list').splitlines() == [str(project), str(child)]
    # Corrupt cache must recover automatically.
    (cache/'repo-rs/info-v1').write_text('invalid\tcache\n')
    assert run('--list').splitlines() == [str(project), str(child)]
print('PASS: warm cache, local/upstream invalidation, branch switch, worktree, spaces, refresh, corrupt cache')
