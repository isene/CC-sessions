//! cc-sessions — shared plumbing for the cc / cl / cc-bookmark binaries.
//!
//! Bookmarks live in ~/.cc-sessions/bookmarks.json as
//! `{"version": 2, "sessions": {"<session-id>": {"path": "...", "tags": [...]}}}`.
//! Session ids are Claude Code transcript ids (the *.jsonl basename under
//! ~/.claude/projects/<encoded-path>/); entries whose transcript could
//! never be found are keyed `path:<dir>` and resumed with `claude -c`.
//!
//! Rust port of the original Ruby scripts (v1.x). Design notes:
//! * JSON maps keep insertion order (serde_json preserve_order) — the
//!   file order IS the picker order; J/K reorder persists by rewriting.
//! * No subprocess spawns for process discovery: /proc is read directly.
//! * Terminal input uses termios VMIN/VTIME — never O_NONBLOCK, which
//!   (leaked by the old Ruby read_nonblock) once killed the parent
//!   shell and its terminal window.

use crust::seq;
use crust::style;
use crust::Crust;
use serde_json::{json, Map, Value};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/root".into()))
}
pub fn config_dir() -> PathBuf { home().join(".cc-sessions") }
pub fn bookmarks_file() -> PathBuf { config_dir().join("bookmarks.json") }
pub fn resume_dir() -> PathBuf { config_dir().join("resumed") }
pub fn claude_projects() -> PathBuf { home().join(".claude/projects") }

/// ~/.claude/projects encodes a directory path with every non-alnum
/// byte replaced by '-'.
pub fn encode_path(p: &str) -> String {
    p.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect()
}

/// Walk up from `cwd` until a directory whose encoding exists under
/// ~/.claude/projects is found; that's the session's launch dir.
pub fn resolve_session_dir(cwd: &str) -> String {
    let mut path = PathBuf::from(cwd);
    loop {
        let enc = encode_path(&path.to_string_lossy());
        if claude_projects().join(&enc).is_dir() {
            return path.to_string_lossy().into_owned();
        }
        match path.parent() {
            Some(p) if p != path => path = p.to_path_buf(),
            _ => break,
        }
    }
    cwd.to_string()
}

/// Newest transcript id in the project dir for `session_dir`, optionally
/// excluding one id (used to find "the NEW session" after continuation).
pub fn detect_session_id(session_dir: &str, exclude: Option<&str>) -> Option<String> {
    let dir = claude_projects().join(encode_path(session_dir));
    let mut newest: Option<(std::time::SystemTime, String)> = None;
    for e in fs::read_dir(&dir).ok()? {
        let e = e.ok()?;
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".jsonl") else { continue };
        if exclude == Some(stem) { continue; }
        let Ok(md) = e.metadata() else { continue };
        if !md.is_file() { continue; }
        let mt = md.modified().unwrap_or(std::time::UNIX_EPOCH);
        if newest.as_ref().map(|(t, _)| mt > *t).unwrap_or(true) {
            newest = Some((mt, stem.to_string()));
        }
    }
    newest.map(|(_, id)| id)
}

/// Locate a session's transcript anywhere under ~/.claude/projects.
pub fn find_transcript(session_id: &str) -> Option<PathBuf> {
    for e in fs::read_dir(claude_projects()).ok()? {
        let p = e.ok()?.path().join(format!("{}.jsonl", session_id));
        if p.is_file() { return Some(p); }
    }
    None
}

/// The directory a session was launched from — the first `cwd` value in
/// its transcript (authoritative; shell cwd drifts during a session).
pub fn session_launch_dir(session_id: &str) -> Option<String> {
    let path = find_transcript(session_id)?;
    let f = fs::File::open(path).ok()?;
    let mut r = io::BufReader::new(f);
    let mut line = String::new();
    for _ in 0..50 {
        line.clear();
        if io::BufRead::read_line(&mut r, &mut line).ok()? == 0 { break; }
        if let Ok(v) = serde_json::from_str::<Value>(&line) {
            if let Some(cwd) = v.get("cwd").and_then(|c| c.as_str()) {
                return Some(cwd.to_string());
            }
        }
    }
    None
}

pub fn session_file_exists(session_id: &str, session_dir: &str) -> bool {
    claude_projects()
        .join(encode_path(session_dir))
        .join(format!("{}.jsonl", session_id))
        .is_file()
}

// ------------------------------------------------------------------ //
// Bookmarks

pub fn load_bookmarks() -> Map<String, Value> {
    let raw = match fs::read_to_string(bookmarks_file()) {
        Ok(s) => s,
        Err(_) => return fresh(),
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else { return fresh() };
    let Some(obj) = v.as_object() else { return fresh() };
    if obj.contains_key("version") {
        return obj.clone();
    }
    // v1 migration: {path: [tags]} keyed by directory.
    let mut sessions = Map::new();
    for (path, tags) in obj {
        let key = detect_session_id(path, None)
            .unwrap_or_else(|| format!("path:{}", path));
        sessions.insert(key, json!({ "path": path, "tags": tags }));
    }
    let mut new = Map::new();
    new.insert("version".into(), json!(2));
    new.insert("sessions".into(), Value::Object(sessions));
    save_bookmarks(&new);
    new
}

fn fresh() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("version".into(), json!(2));
    m.insert("sessions".into(), Value::Object(Map::new()));
    m
}

pub fn sessions(b: &Map<String, Value>) -> &Map<String, Value> {
    // load_bookmarks() always installs the key; a hand-built map that
    // lacks it is a programming error.
    b.get("sessions").and_then(|s| s.as_object())
        .expect("bookmarks map missing 'sessions'")
}
pub fn sessions_mut(b: &mut Map<String, Value>) -> &mut Map<String, Value> {
    if !b.get("sessions").map(|s| s.is_object()).unwrap_or(false) {
        b.insert("sessions".into(), Value::Object(Map::new()));
    }
    b.get_mut("sessions").unwrap().as_object_mut().unwrap()
}

pub fn save_bookmarks(b: &Map<String, Value>) {
    let _ = fs::create_dir_all(config_dir());
    let path = bookmarks_file();
    if let Ok(s) = serde_json::to_string_pretty(&Value::Object(b.clone())) {
        let _ = fs::write(&path, s);
        fix_ownership(&path);
    }
}

/// When running under sudo, keep files owned by the real user.
pub fn fix_ownership(path: &Path) {
    let sudo_user = std::env::var("SUDO_USER").ok();
    if unsafe { libc::geteuid() } != 0 && sudo_user.is_none() { return; }
    let user = sudo_user.unwrap_or_else(|| "geir".into());
    let Ok(cname) = std::ffi::CString::new(user) else { return };
    unsafe {
        let pw = libc::getpwnam(cname.as_ptr());
        if pw.is_null() { return; }
        let Ok(cpath) = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()) else { return };
        libc::chown(cpath.as_ptr(), (*pw).pw_uid, (*pw).pw_gid);
    }
}

pub struct TagMatch { pub id: String, pub path: String }

pub fn find_session_by_tag(tag: &str) -> Option<TagMatch> {
    let b = load_bookmarks();
    for (id, entry) in sessions(&b) {
        let tags = entry.get("tags").and_then(|t| t.as_array());
        if tags.map(|a| a.iter().any(|t| t.as_str() == Some(tag))).unwrap_or(false) {
            return Some(TagMatch {
                id: id.clone(),
                path: entry.get("path").and_then(|p| p.as_str()).unwrap_or("").to_string(),
            });
        }
    }
    None
}

pub fn entry_tags(entry: &Value) -> Vec<String> {
    entry.get("tags").and_then(|t| t.as_array())
        .map(|a| a.iter().filter_map(|t| t.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

// ------------------------------------------------------------------ //
// Process discovery — /proc, no forks.

/// Session ids of currently running `claude` processes.
pub fn running_session_ids() -> Vec<String> {
    let mut ids = Vec::new();
    for (_, cwd) in running_claude_procs() {
        let dir = resolve_session_dir(&cwd);
        if let Some(id) = detect_session_id(&dir, None) {
            if !ids.contains(&id) { ids.push(id); }
        }
    }
    ids
}

/// (pid, cwd) for every running process whose comm is `claude`.
pub fn running_claude_procs() -> Vec<(i32, String)> {
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir("/proc") else { return out };
    for e in rd.flatten() {
        let name = e.file_name();
        let Some(pid) = name.to_str().and_then(|s| s.parse::<i32>().ok()) else { continue };
        let comm = fs::read_to_string(e.path().join("comm")).unwrap_or_default();
        if comm.trim() != "claude" { continue; }
        if let Ok(cwd) = fs::read_link(e.path().join("cwd")) {
            out.push((pid, cwd.to_string_lossy().into_owned()));
        }
    }
    out.sort();
    out
}

// ------------------------------------------------------------------ //
// Terminal: raw mode + key reading (termios VMIN/VTIME only).

pub struct RawGuard { orig: libc::termios }

impl RawGuard {
    pub fn new() -> Option<Self> {
        unsafe {
            let mut t: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(0, &mut t) != 0 { return None; }
            let orig = t;
            libc::cfmakeraw(&mut t);
            // Keep output post-processing so println! still produces CRLF.
            t.c_oflag |= libc::OPOST;
            t.c_cc[libc::VMIN] = 1;
            t.c_cc[libc::VTIME] = 0;
            libc::tcsetattr(0, libc::TCSANOW, &t);
            Some(RawGuard { orig })
        }
    }
    fn set_vmin_vtime(&self, vmin: u8, vtime: u8) {
        unsafe {
            let mut t: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(0, &mut t) != 0 { return; }
            t.c_cc[libc::VMIN] = vmin;
            t.c_cc[libc::VTIME] = vtime;
            libc::tcsetattr(0, libc::TCSANOW, &t);
        }
    }
}

impl Drop for RawGuard {
    fn drop(&mut self) {
        unsafe { libc::tcsetattr(0, libc::TCSANOW, &self.orig); }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Key { Up, Down, MoveUp, MoveDown, Enter, Quit, Delete, Char(u8), None }

/// Read one key inside a RawGuard. ESC tails are collected with
/// VMIN=0/VTIME=1 (0.1 s) — raw termios timeouts, no fd flag games.
pub fn read_key(raw: &RawGuard) -> Key {
    let mut b = [0u8; 1];
    if io::stdin().read_exact(&mut b).is_err() { return Key::Quit; }
    let mut seq = vec![b[0]];
    if b[0] == 0x1b {
        raw.set_vmin_vtime(0, 1);
        let mut t = [0u8; 1];
        while seq.len() < 8 {
            match io::stdin().read(&mut t) {
                Ok(1) => seq.push(t[0]),
                _ => break,
            }
        }
        raw.set_vmin_vtime(1, 0);
    }
    match seq.as_slice() {
        b"\x1b[A" => Key::Up,
        b"\x1b[B" => Key::Down,
        b"\x1b[1;2A" => Key::MoveUp,
        b"\x1b[1;2B" => Key::MoveDown,
        [0x1b] => Key::Quit,
        [b'k'] => Key::Up,
        [b'j'] => Key::Down,
        [b'K'] => Key::MoveUp,
        [b'J'] => Key::MoveDown,
        [b'\r'] | [b'\n'] => Key::Enter,
        [b'q'] => Key::Quit,
        [b'd'] => Key::Delete,
        [c] => Key::Char(*c),
        _ => Key::None,
    }
}

// ------------------------------------------------------------------ //
// ANSI helpers

pub fn cyan(s: &str) -> String { style::styled(s, Some(6), None, "") }
pub fn green(s: &str) -> String { style::styled(s, Some(2), None, "") }
pub fn dim(s: &str) -> String { style::dim(s) }
pub fn red(s: &str) -> String { style::styled(s, Some(1), None, "") }

/// Emit OSC 7 so the terminal (wezterm / glass) tracks the cwd.
pub fn emit_osc7(path: &str) {
    let host = fs::read_to_string("/proc/sys/kernel/hostname")
        .unwrap_or_default().trim().to_string();
    Crust::set_cwd(&host, &path);
    let _ = io::stdout().flush();
}

// ------------------------------------------------------------------ //
// Interactive picker + resume (shared by the cc and cl binaries).

use std::os::unix::process::CommandExt;

struct Item { id: String, path: String, tags: Vec<String>, exists: bool, running: bool }

pub fn list_bookmarks() {
    let b = load_bookmarks();
    if sessions(&b).is_empty() {
        println!("No bookmarked sessions.");
        println!();
        println!("To bookmark a session, use '/bm tag1 tag2' in Claude Code.");
        return;
    }
    let running = running_session_ids();
    let mut items: Vec<Item> = sessions(&b).iter().map(|(id, e)| {
        let path = e.get("path").and_then(|p| p.as_str()).unwrap_or("").to_string();
        Item {
            id: id.clone(),
            exists: fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false),
            running: running.contains(id),
            tags: entry_tags(e),
            path,
        }
    }).collect();

    let mut index = 0usize;
    println!("Select session (\u{2191}/\u{2193}/j/k move, J/K reorder, Enter select, d delete, q quit):\n");
    print!("{}", seq::HIDE);
    let raw = RawGuard::new();
    let Some(raw) = raw else { print!("{}", seq::SHOW); return };

    let clear_line = format!("{}\r", seq::ERASE_LINE);
    let restore = |n: usize| {
        print!("{}", format!("{}\r\n", seq::ERASE_LINE).repeat(n));
        print!("{}", seq::up(n as u16));
    };

    loop {
        for (i, it) in items.iter().enumerate() {
            print!("{}", clear_line);
            let status = if !it.exists { red(" [NOT FOUND]") }
                else if it.running { green(" \u{25cf}") }
                else { String::new() };
            let line = format!("{} \u{2192} {}{}", cyan(&it.tags.join(", ")), dim(&it.path), status);
            if i == index { println!("\u{25b8} {}", line); } else { println!("  {}", line); }
        }
        print!("{}", seq::up(items.len() as u16));
        let _ = std::io::stdout().flush();

        match read_key(&raw) {
            Key::Up => index = (index + items.len() - 1) % items.len(),
            Key::Down => index = (index + 1) % items.len(),
            Key::MoveUp => if index > 0 {
                items.swap(index, index - 1);
                index -= 1;
                rebuild_and_save(&items);
            },
            Key::MoveDown => if index < items.len() - 1 {
                items.swap(index, index + 1);
                index += 1;
                rebuild_and_save(&items);
            },
            Key::Delete => {
                let old_size = items.len();
                print!("{}{}", seq::down(old_size as u16), clear_line);
                print!("Delete '{}'? (y/n) ", items[index].tags.join(", "));
                let _ = std::io::stdout().flush();
                let confirm = matches!(read_key(&raw), Key::Char(b'y') | Key::Char(b'Y'));
                if confirm {
                    items.remove(index);
                    rebuild_and_save(&items);
                    if index >= items.len() && index > 0 { index = items.len() - 1; }
                    if items.is_empty() {
                        print!("{}", seq::up(old_size as u16));
                        restore(old_size + 1);
                        println!("All bookmarks deleted.");
                        break;
                    }
                    print!("{}{}", clear_line, seq::up(old_size as u16));
                    restore(old_size);
                } else {
                    print!("{}{}", clear_line, seq::up(old_size as u16));
                }
            }
            Key::Enter => {
                restore(items.len());
                let it = &items[index];
                if it.exists {
                    print!("{}", seq::SHOW);
                    let _ = std::io::stdout().flush();
                    drop(raw);
                    resume_session(&it.id, &it.path);
                } else {
                    println!("{}", red(&format!("Directory not found: {}", it.path)));
                }
                break;
            }
            Key::Quit => { restore(items.len()); break; }
            _ => {}
        }
    }
    print!("{}", seq::SHOW);
    let _ = std::io::stdout().flush();
}

fn rebuild_and_save(items: &[Item]) {
    let mut sessions = serde_json::Map::new();
    for it in items {
        sessions.insert(it.id.clone(), json!({ "path": it.path, "tags": it.tags }));
    }
    let mut b = serde_json::Map::new();
    b.insert("version".into(), json!(2));
    b.insert("sessions".into(), Value::Object(sessions));
    save_bookmarks(&b);
}


pub fn resume_session(session_id: &str, path: &str) {
    if !fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false) {
        println!("Error: Directory not found: {}", path);
        std::process::exit(1);
    }
    let mut session_id = session_id.to_string();
    let mut b = load_bookmarks();
    let mut entry = sessions(&b).get(&session_id).cloned();

    // Auto-migrate: if the bookmarked transcript is gone and a newer one
    // exists (context continuation), follow it. A newer transcript while
    // the original still exists is NOT a continuation.
    if !session_id.starts_with("path:") {
        let session_dir = resolve_session_dir(path);
        let newest = detect_session_id(&session_dir, None);
        let original_gone = find_transcript(&session_id).is_none();
        if original_gone {
            if let (Some(newest_id), Some(e)) = (newest, entry.clone()) {
                if newest_id != session_id {
                    println!("{}", dim("Session was continued — following to latest."));
                    let s = sessions_mut(&mut b);
                    s.shift_remove(&session_id);
                    s.insert(newest_id.clone(), e.clone());
                    save_bookmarks(&b);
                    let _ = fs::remove_file(resume_dir().join(format!("{}.json", session_id)));
                    session_id = newest_id;
                    entry = Some(e);
                }
            }
        }
    }

    // Breadcrumb so cc-bookmark can track continuations after context resets.
    let tags = entry.as_ref().map(|e| entry_tags(e)).unwrap_or_default();
    if entry.is_some() && !session_id.starts_with("path:") {
        let _ = fs::create_dir_all(resume_dir());
        let crumb = resume_dir().join(format!("{}.json", session_id));
        if let Ok(s) = serde_json::to_string(&tags) {
            let _ = fs::write(&crumb, s);
            fix_ownership(&crumb);
        }
    }

    emit_osc7(path);
    println!("Resuming session in: {}", path);
    let _ = std::env::set_current_dir(path);
    let mut cmd = std::process::Command::new("claude");
    if session_id.starts_with("path:") {
        cmd.arg("-c");
    } else {
        cmd.arg("--resume").arg(&session_id);
    }
    cmd.env("CC_SESSION_ID", &session_id);
    if !tags.is_empty() {
        cmd.env("CC_RESUME_TAGS", tags.join(","));
    }
    let err = cmd.exec();
    eprintln!("Failed to exec claude: {}", err);
    std::process::exit(1);
}

