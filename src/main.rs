use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, HashMap},
    env, fs,
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::UNIX_EPOCH,
};

#[derive(Clone, Debug, PartialEq)]
struct Info {
    path: String,
    branch: String,
    ahead: u64,
    stamp: u64,
    mtime: u128,
}
fn run(args: &[&str]) -> Option<String> {
    let out = Command::new(args[0]).args(&args[1..]).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim_end().to_owned())
}
fn hex(s: &str) -> String {
    s.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(s: &str) -> Option<String> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    String::from_utf8(
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
            .collect::<Option<Vec<_>>>()?,
    )
    .ok()
}
fn cache_dir() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache"))
        .join("repo-rs")
}
fn atomic_write(path: &Path, text: &str) {
    let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
    if fs::write(&tmp, text).is_ok() {
        let _ = fs::rename(&tmp, path);
    }
}
fn repositories(cache: &Path, refresh: bool) -> Result<Vec<String>, String> {
    let list = cache.join("repositories");
    let fresh = fs::metadata(&list)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|d| d.as_secs() < 10);
    if !refresh && fresh {
        if let Ok(s) = fs::read_to_string(&list) {
            return Ok(s
                .lines()
                .filter_map(unhex)
                .filter(|p| Path::new(p).is_dir())
                .collect());
        }
    }
    let text = run(&["ghq", "list", "--full-path"])
        .ok_or("ghq list failed; install ghq and check its configuration")?;
    let repos: Vec<_> = text
        .lines()
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect();
    atomic_write(
        &list,
        &repos.iter().map(|p| hex(p)).collect::<Vec<_>>().join("\n"),
    );
    Ok(repos)
}
fn git_dirs(path: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(path);
    let dot = root.join(".git");
    let git = if dot.is_file() {
        fs::read_to_string(&dot)
            .ok()
            .and_then(|s| s.trim().strip_prefix("gitdir: ").map(|s| root.join(s)))
            .unwrap_or(dot)
    } else {
        dot
    };
    let common = fs::read_to_string(git.join("commondir"))
        .map(|s| git.join(s.trim()))
        .unwrap_or_else(|_| git.clone());
    (git, common)
}
fn hash_file(path: &Path, h: &mut DefaultHasher) {
    path.hash(h);
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();
            for p in paths {
                if !p.is_symlink() {
                    hash_file(&p, h);
                }
            }
        }
    } else {
        fs::read(path).unwrap_or_default().hash(h);
    }
}
fn inspect(path: &str, old: Option<&Info>, refresh: bool) -> Info {
    let (git, common) = git_dirs(path);
    let mut h = DefaultHasher::new();
    for p in [
        git.join("HEAD"),
        common.join("refs"),
        common.join("packed-refs"),
        common.join("config"),
        git.join("config.worktree"),
        git.join("shallow"),
    ] {
        hash_file(&p, &mut h);
    }
    let stamp = h.finish();
    let mtime = fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    if !refresh {
        if let Some(info) = old {
            if stamp == info.stamp {
                return Info {
                    mtime,
                    ..info.clone()
                };
            }
        }
    }
    let head = fs::read_to_string(git.join("HEAD")).unwrap_or_default();
    let branch = head
        .trim()
        .strip_prefix("ref: refs/heads/")
        .unwrap_or("detached")
        .to_owned();
    let ahead = run(&["git", "-C", path, "rev-list", "--count", "@{u}..HEAD"])
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    Info {
        path: path.into(),
        branch,
        ahead,
        stamp,
        mtime,
    }
}
fn load_cache(path: &Path) -> HashMap<String, Info> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let v: Vec<_> = line.split('\t').collect();
            if v.len() != 4 {
                return None;
            }
            let info = Info {
                path: unhex(v[0])?,
                branch: unhex(v[1])?,
                ahead: v[2].parse().ok()?,
                stamp: v[3].parse().ok()?,
                mtime: 0,
            };
            Some((info.path.clone(), info))
        })
        .collect()
}
fn base(path: &str) -> (String, bool) {
    let name = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let (name, child) = name
        .split_once("=worktree")
        .map_or((name.as_ref(), false), |(base, _)| (base, true));
    // Include the parent so identical names in different owners remain distinct.
    (
        format!(
            "{}/{}",
            Path::new(path).parent().unwrap_or(Path::new("")).display(),
            name
        ),
        child,
    )
}
fn ordered(mut infos: Vec<Info>) -> Vec<(Info, bool)> {
    infos.sort_by_key(|i| i.mtime);
    let mut children: BTreeMap<String, Vec<Info>> = BTreeMap::new();
    for i in &infos {
        let (key, child) = base(&i.path);
        if child {
            children.entry(key).or_default().push(i.clone());
        }
    }
    let mut out = Vec::new();
    for i in infos {
        let (key, child) = base(&i.path);
        if child {
            continue;
        }
        out.push((i, false));
        if let Some(mut group) = children.remove(&key) {
            group.sort_by(|a, b| a.path.cmp(&b.path));
            out.extend(group.into_iter().map(|i| (i, true)));
        }
    }
    for (_, mut group) in children {
        group.sort_by(|a, b| a.path.cmp(&b.path));
        out.extend(group.into_iter().map(|i| (i, true)));
    }
    out
}
fn main() {
    if let Err(e) = main_result() {
        eprintln!("repo: {e}");
        std::process::exit(1);
    }
}
fn main_result() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!("repo [--refresh] [--list] [QUERY...]\n\nCached ghq + fzf repository picker. Selection paths go to stdout.\n  --refresh    Refresh repository list and all Git information\n  --list       Print the ordered repository paths without fzf\n  --shell-init Print a zsh/bash wrapper for changing directories");
        return Ok(());
    }
    if args.iter().any(|s| s == "--shell-init") {
        print!("{}", include_str!("../shell-init.sh"));
        return Ok(());
    }
    let refresh = args.iter().any(|s| s == "--refresh");
    let cache = cache_dir();
    let _ = fs::create_dir_all(&cache);
    let repos = repositories(&cache, refresh)?;
    let old = load_cache(&cache.join("info-v1"));
    let next = AtomicUsize::new(0);
    let results = Arc::new(Mutex::new(Vec::new()));
    std::thread::scope(|scope| {
        for _ in 0..8.min(repos.len()) {
            let repos = &repos;
            let old = &old;
            let next = &next;
            let results = Arc::clone(&results);
            scope.spawn(move || loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= repos.len() {
                    break;
                }
                let path = &repos[index];
                let info = inspect(path, old.get(path), refresh);
                results.lock().unwrap().push(info);
            });
        }
    });
    let infos = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    atomic_write(
        &cache.join("info-v1"),
        &infos
            .iter()
            .map(|i| {
                format!(
                    "{}\t{}\t{}\t{}",
                    hex(&i.path),
                    hex(&i.branch),
                    i.ahead,
                    i.stamp
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let rows = ordered(infos);
    if args.iter().any(|s| s == "--list") {
        for (i, _) in rows {
            println!("{}", i.path);
        }
        return Ok(());
    }
    if rows.is_empty() {
        return Ok(());
    }
    let home = env::var("HOME").unwrap_or_default();
    let mut input = String::new();
    for (n, (i, child)) in rows.iter().enumerate() {
        let display = if !home.is_empty() {
            i.path
                .strip_prefix(&home)
                .map(|s| format!("...{s}"))
                .unwrap_or_else(|| i.path.clone())
        } else {
            i.path.clone()
        };
        let icon = if *child {
            "⎿ \u{e725}"
        } else if i.path.contains("github.com") {
            "\u{f09b}"
        } else {
            "\u{f07b}"
        };
        let ahead = if i.ahead > 0 {
            format!(" \x1b[31m⬆{}\x1b[0m", i.ahead)
        } else {
            String::new()
        };
        input.push_str(&format!(
            "{n}\t{icon} {display:<55} \x1b[33m({})\x1b[0m{ahead}\n",
            i.branch
        ));
    }
    let mut cmd = Command::new("fzf");
    cmd.args([
        "-m",
        "--no-sort",
        "--tac",
        "--ansi",
        "--delimiter=\t",
        "--with-nth=2..",
        "--color=pointer:blue,hl+:blue,fg+:blue,bg+:black",
    ]);
    let query = args
        .iter()
        .filter(|s| !matches!(s.as_str(), "--refresh" | "--list"))
        .cloned()
        .collect::<String>();
    if !query.is_empty() {
        cmd.arg("--query").arg(format!("'{query}"));
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("start fzf: {e}"))?;
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(input.as_bytes());
    });
    let result = child.wait_with_output().map_err(|e| e.to_string())?;
    let _ = writer.join();
    if result.status.success() {
        for line in String::from_utf8_lossy(&result.stdout).lines() {
            if let Some(index) = line
                .split('\t')
                .next()
                .and_then(|n| n.parse::<usize>().ok())
            {
                if let Some((info, _)) = rows.get(index) {
                    println!("{}", info.path);
                }
            }
        }
    } else if !matches!(result.status.code(), Some(1 | 130)) {
        return Err(format!("fzf exited with {}", result.status));
    }
    Ok(())
}
