//! cc-bookmark — helper behind the /bm command inside Claude Code.
//! Bookmarks the current session with tags, keyed by session id, and
//! tracks context continuations (Claude creates a new session id when
//! context resets; the bookmark follows).

use cc_sessions::*;
use serde_json::json;
use std::fs;

fn main() {
    let _ = fs::create_dir_all(config_dir());
    let mut bookmarks = load_bookmarks();

    // Set by the `cc` wrapper — the session id we resumed.
    let mut original_id = std::env::var("CC_SESSION_ID").ok();

    // Use the bookmark's stored path (not the shell cwd) when we know
    // the session — bash cwd drifts during a session.
    let mut cwd: Option<String> = None;
    if let Some(oid) = &original_id {
        if let Some(entry) = sessions(&bookmarks).get(oid) {
            if let Some(p) = entry.get("path").and_then(|p| p.as_str()) {
                cwd = Some(resolve_session_dir(p));
            }
        }
    }
    let mut cwd = cwd.unwrap_or_else(|| {
        let pwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        resolve_session_dir(&pwd)
    });

    let mut session_id = detect_session_id(&cwd, None);

    // Claude Code 2.1+ exports the running session's id — authoritative.
    if let Ok(current_id) = std::env::var("CLAUDE_CODE_SESSION_ID") {
        if let Some(launch) = session_launch_dir(&current_id) {
            cwd = launch;
        }
        session_id = Some(current_id.clone());
        // Context continuation while running under `cc`: migrate the old entry.
        if let Some(oid) = original_id.clone() {
            if oid != current_id {
                if let Some(old) = sessions_mut(&mut bookmarks).shift_remove(&oid) {
                    sessions_mut(&mut bookmarks)
                        .entry(current_id.clone()).or_insert(old);
                    save_bookmarks(&bookmarks);
                    let _ = fs::remove_file(resume_dir().join(format!("{}.json", oid)));
                }
            }
        }
        original_id = Some(current_id);
    }

    let mut tags: Vec<String> = std::env::args().skip(1).collect();
    // Reserved words = query mode, not tag names.
    if tags.len() == 1
        && matches!(tags[0].to_lowercase().as_str(), "?" | "query" | "status" | "show") {
        tags.clear();
    }

    if tags.is_empty() {
        query_mode(&mut bookmarks, original_id.as_deref(), session_id.as_deref(), &cwd);
    } else {
        bookmark_mode(&mut bookmarks, original_id.as_deref(), session_id.as_deref(), &cwd, tags);
    }
}

fn query_mode(bookmarks: &mut serde_json::Map<String, serde_json::Value>,
              original_id: Option<&str>, session_id: Option<&str>, cwd: &str) {
    if let Some(oid) = original_id {
        if let Some(entry) = sessions(bookmarks).get(oid).cloned() {
            // Continued only if the original transcript is GONE and a
            // newer one exists. A newer sibling alone is NOT continuation.
            let original_gone = !session_file_exists(oid, cwd);
            match session_id {
                Some(sid) if original_gone && sid != oid => {
                    let tags = entry_tags(&entry);
                    let s = sessions_mut(bookmarks);
                    s.shift_remove(oid);
                    s.insert(sid.to_string(), entry);
                    save_bookmarks(bookmarks);
                    let _ = fs::remove_file(resume_dir().join(format!("{}.json", oid)));
                    println!("Bookmark migrated: {}", tags.join(", "));
                }
                _ => println!("Current bookmark: {}", entry_tags(&entry).join(", ")),
            }
            return;
        }
        // Session id changed (compaction/reset): bookmark sits on the
        // original id. Recover tags from env or breadcrumb.
        let resume_tags: Option<Vec<String>> = std::env::var("CC_RESUME_TAGS").ok()
            .map(|s| s.split(',').map(str::to_string).collect())
            .or_else(|| {
                let crumb = resume_dir().join(format!("{}.json", oid));
                fs::read_to_string(&crumb).ok()
                    .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
            });
        if let Some(rtags) = resume_tags {
            if let Some(new_id) = detect_session_id(cwd, Some(oid)) {
                if let Some(existing) = sessions(bookmarks).get(&new_id) {
                    println!("Current bookmark: {}", entry_tags(existing).join(", "));
                } else {
                    sessions_mut(bookmarks).insert(new_id,
                        json!({ "path": cwd, "tags": rtags }));
                    save_bookmarks(bookmarks);
                    let _ = fs::remove_file(resume_dir().join(format!("{}.json", oid)));
                    println!("Current bookmark: {}", rtags.join(", "));
                }
                return;
            }
        }
        println!("No bookmark for this session. Usage: /bm tag1 tag2 ...");
    } else if let Some(sid) = session_id {
        match sessions(bookmarks).get(sid) {
            Some(entry) => println!("Current bookmark: {}", entry_tags(entry).join(", ")),
            None => println!("No bookmark for this session. Usage: /bm tag1 tag2 ..."),
        }
    } else {
        println!("Could not detect session ID. Usage: /bm tag1 tag2 ...");
    }
}

fn bookmark_mode(bookmarks: &mut serde_json::Map<String, serde_json::Value>,
                 original_id: Option<&str>, session_id: Option<&str>, cwd: &str,
                 tags: Vec<String>) {
    // Use CC_SESSION_ID while its transcript still exists (the real
    // session); fall back to the newest transcript only after a true
    // continuation removed the original.
    let key = match (original_id, session_id) {
        (Some(oid), _) if session_file_exists(oid, cwd) => oid.to_string(),
        (Some(oid), Some(sid)) if sid != oid => sid.to_string(),
        (oid, sid) => sid.or(oid).map(str::to_string)
            .unwrap_or_else(|| format!("path:{}", cwd)),
    };
    if let Some(oid) = original_id {
        if oid != key && sessions(bookmarks).contains_key(oid) {
            sessions_mut(bookmarks).shift_remove(oid);
            let _ = fs::remove_file(resume_dir().join(format!("{}.json", oid)));
        }
    }
    sessions_mut(bookmarks).insert(key, json!({ "path": cwd, "tags": tags }));
    save_bookmarks(bookmarks);
    println!("Bookmarked: {}", cwd);
    println!("Tags: {}", tags.join(", "));
    println!();
    println!("Resume later with: cc {}", tags.first().map(String::as_str).unwrap_or(""));
}
