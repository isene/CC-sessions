# Changelog

## [1.4.1] - 2026-03-03

### Fixed
- `cc-bookmark` query mode now proactively migrates bookmarks when a context continuation is detected (was only printing tags without migrating)
- Hook now detects new session IDs by comparing newest `.jsonl` against a stamp, instead of only running once per `CC_SESSION_ID`

### Changed
- Hook outputs "Bookmark auto-migrated" when migration happens during a continuation
- Migration prints "Bookmark migrated:" instead of "Current bookmark:" so the hook can distinguish migration from no-op

## [1.4.0] - 2026-03-03

### Fixed
- Resuming a session that had a context continuation now auto-follows to the latest session ID instead of jumping back to the old one
- Re-bookmarking (`/bm`) in a continued session now uses the detected (newest) session ID instead of the stale env var

### Changed
- `resume_session` detects newer `.jsonl` files and auto-migrates the bookmark before resuming
- `cc-bookmark` prefers detected session ID over `CC_SESSION_ID` env var in bookmark mode
- Old bookmarks and breadcrumbs cleaned up automatically during migration

## [1.3.0] - 2026-02-27

### Added
- Running session indicator (green `●`) in `cc -l` interactive list
- Resume breadcrumbs to track session continuations across context resets
- `CC_SESSION_ID` and `CC_RESUME_TAGS` env vars passed to resumed sessions
- `exclude_id` support in session detection for sibling session discovery

### Changed
- `cc-bookmark` now checks env vars first for reliable session identification
- `/bm?` auto-migrates bookmark to new session ID after context reset

## [1.2.0] - 2026-02-19

### Added
- Session ID-based bookmarks (v2 format) for reliable session resume
- Auto-migration from v1 path-based bookmarks to v2 session IDs
- OSC 7 escape sequence on session resume for wezterm CWD tracking
- Reorder bookmarks with J/K (Shift+j/k) in interactive list
- `--resume <session_id>` used for direct session resume

### Changed
- Bookmarks now keyed by session UUID instead of directory path
- `cc-bookmark` updated to detect and store session IDs
- Fallback to `path:<dir>` key when session ID unavailable

## [1.1.4] - 2025-02-10

### Fixed
- Bookmark path resolution when claude changes directory during session
- `cc -C` now resolves original session directory via ~/.claude/projects/

## [1.1.3] - 2025-02-04

### Added
- `cc -C` / `cc --current` to show currently running Claude Code sessions
- Shows PID, working directory, and tags for each running session

## [1.1.2] - 2025-01-23

### Added
- `/bm?` command to check current bookmark status

## [1.1.1] - 2025-01-23

### Changed
- Improved help text formatting
- Updated README with new features

## [1.1.0] - 2025-01-23

### Added
- Interactive session picker with `cc -l` (no external dependencies)
- Delete bookmarks with `d` key in list (with confirmation)
- Delete bookmarks from CLI with `cc -d <tag>`
- Vim-style navigation (j/k) in addition to arrow keys

### Changed
- `cc -l` now shows interactive list with hidden cursor
- Zero external dependencies (removed tty-prompt)

## [1.0.2] - 2025-01-23

### Added
- `cc-bookmark` helper script for reliable permission matching
- SVG logo

### Changed
- `/bm` command now uses `cc-bookmark` script (auto-accepts without prompts)
- Updated README with badges and proper logo placement

## [1.0.1] - 2025-01-23

### Added
- Auto-add permission for `/bm` command on first run (no confirmation prompt)

## [1.0.0] - 2025-01-23

### Added
- Initial release
- `cc` command to manage Claude Code sessions
- `/bm` slash command for bookmarking sessions with tags
- `cc <tag>` to resume session by tag
- `cc -l` to list all bookmarked sessions
- `cc -h` for help
- Auto-install of `/bm` command on first run
