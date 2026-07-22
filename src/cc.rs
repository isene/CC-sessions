//! cc — Claude Code session manager.
//!
//!   cc              Continue session in current dir, or start new
//!   cc <tag>        Resume session bookmarked with <tag>
//!   cc -l, --list   Interactive list of bookmarked sessions
//!   cc -C, --current  Show currently running Claude Code sessions
//!   cc -d, --delete <tag>  Delete bookmark matching <tag>
//!   cc -h, --help   Show help

use cc_sessions::*;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;

fn main() {
    ensure_setup();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("-h") | Some("--help") => show_help(),
        Some("-l") | Some("--list") => list_bookmarks(),
        Some("-C") | Some("--current") => show_current_sessions(),
        Some("-d") | Some("--delete") => match args.get(1) {
            Some(tag) => delete_bookmark_by_tag(tag),
            None => {
                println!("Usage: cc -d <tag>");
                println!("Delete bookmark matching the given tag.");
                std::process::exit(1);
            }
        },
        None => continue_or_start(),
        Some(tag) => resume_by_tag(tag),
    }
}

fn ensure_setup() {
    let _ = fs::create_dir_all(config_dir());
    // Install the /bm command definition on first run. The source ships
    // next to the repo the binary was built from: walk up from the
    // executable until a commands/bm.md appears.
    let dest = home().join(".claude/commands/bm.md");
    if !dest.exists() {
        if let Some(src) = find_bm_source() {
            let _ = fs::create_dir_all(dest.parent().unwrap());
            if fs::copy(&src, &dest).is_ok() {
                println!("Installed /bm command to {}", dest.display());
                println!("You can now use '/bm tag1 tag2' in Claude Code to bookmark sessions.");
                println!();
            }
        }
    }
    ensure_permission();
}

fn find_bm_source() -> Option<PathBuf> {
    let mut dir = std::env::current_exe().ok()?;
    for _ in 0..5 {
        dir = dir.parent()?.to_path_buf();
        let cand = dir.join("commands/bm.md");
        if cand.is_file() { return Some(cand); }
    }
    None
}

fn ensure_permission() {
    const PERM: &str = "Bash(cc-bookmark:*)";
    let settings_file = home().join(".claude/settings.json");
    let mut settings: Value = fs::read_to_string(&settings_file).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));
    let Some(obj) = settings.as_object_mut() else { return };
    let perms = obj.entry("permissions").or_insert_with(|| json!({}));
    let Some(perms) = perms.as_object_mut() else { return };
    let allow = perms.entry("allow").or_insert_with(|| json!([]));
    let Some(arr) = allow.as_array_mut() else { return };
    if !arr.iter().any(|v| v.as_str() == Some(PERM)) {
        arr.push(json!(PERM));
        if let Some(dir) = settings_file.parent() { let _ = fs::create_dir_all(dir); }
        if let Ok(s) = serde_json::to_string_pretty(&settings) {
            let _ = fs::write(&settings_file, s);
            println!("Added auto-accept permission for /bm command");
            println!();
        }
    }
}

fn delete_bookmark_by_tag(tag: &str) {
    match find_session_by_tag(tag) {
        Some(m) => {
            let mut b = load_bookmarks();
            sessions_mut(&mut b).shift_remove(&m.id);
            save_bookmarks(&b);
            println!("Deleted bookmark: {}", m.path);
            println!("  (was tagged: {})", tag);
        }
        None => {
            println!("No session found with tag '{}'", tag);
            std::process::exit(1);
        }
    }
}

fn show_current_sessions() {
    let procs = running_claude_procs();
    if procs.is_empty() {
        println!("No Claude Code sessions currently running.");
        return;
    }
    let b = load_bookmarks();
    println!("Running sessions:\n");
    for (pid, cwd) in procs {
        let session_dir = resolve_session_dir(&cwd);
        let tags = detect_session_id(&session_dir, None)
            .and_then(|sid| sessions(&b).get(&sid).map(entry_tags))
            .unwrap_or_default();
        let tag_str = if tags.is_empty() { dim("(no tags)") } else { green(&tags.join(", ")) };
        println!("  PID {}: {}", cyan(&pid.to_string()), session_dir);
        println!("         Tags: {}", tag_str);
        println!();
    }
}

fn show_help() {
    print!("{}", r#"CC - Claude Code Session Manager

Easily bookmark and resume Claude Code sessions with tags.

USAGE:
  cc                     Continue session in current dir, or start new
  cc <tag>               Resume session bookmarked with <tag>
  cc -l, --list          Interactive list of bookmarked sessions
  cc -C, --current       Show currently running Claude Code sessions
  cc -d, --delete <tag>  Delete bookmark matching <tag>
  cc -h, --help          Show this help

BOOKMARKING (inside Claude Code):
  /bm tag1 tag2   Bookmark current session with one or more tags

EXAMPLES:
  cc rtfm         Resume session tagged 'rtfm'
  cc              Continue in current dir or start fresh
  cc -l           Show interactive list (d to delete, Enter to select)
  cc -C           Show running sessions with their tags
  cc -d rtfm      Delete bookmark tagged 'rtfm'

FILES:
  ~/.cc-sessions/bookmarks.json   Bookmark storage
  ~/.claude/commands/bm.md        The /bm command definition

FIRST RUN:
  On first run, cc installs the /bm command to ~/.claude/commands/
  This enables the '/bm' command inside Claude Code sessions.
"#);
}

fn resume_by_tag(tag: &str) {
    match find_session_by_tag(tag) {
        Some(m) => resume_session(&m.id, &m.path),
        None => {
            println!("No session found with tag '{}'", tag);
            println!();
            println!("Available tags:");
            let b = load_bookmarks();
            if sessions(&b).is_empty() {
                println!("  (none - use '/bm tag1 tag2' in Claude Code to bookmark)");
            } else {
                let mut all: Vec<String> = sessions(&b).values()
                    .flat_map(entry_tags).collect();
                all.sort(); all.dedup();
                for t in all { println!("  {}", t); }
            }
            std::process::exit(1);
        }
    }
}

fn continue_or_start() {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    emit_osc7(&cwd);
    let has_session = {
        let cd = PathBuf::from(&cwd).join(".claude");
        cd.is_dir() && dir_has_any_file(&cd)
    };
    let mut cmd = std::process::Command::new("claude");
    if has_session { cmd.arg("-c"); }
    let err = cmd.exec();
    eprintln!("Failed to exec claude: {}", err);
    std::process::exit(1);
}

fn dir_has_any_file(dir: &std::path::Path) -> bool {
    let Ok(rd) = fs::read_dir(dir) else { return false };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_file() { return true; }
        if p.is_dir() && dir_has_any_file(&p) { return true; }
    }
    false
}
