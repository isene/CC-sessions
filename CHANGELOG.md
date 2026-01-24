# Changelog

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
