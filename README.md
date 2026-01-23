# CC-sessions

A simple tool for bookmarking and resuming Claude Code sessions with tags.

## The Problem

Claude Code sessions are tied to directories, making it hard to remember where
you were working on specific projects. This tool lets you tag sessions with
meaningful names and quickly resume them.

## Features

- **Bookmark sessions** with `/bm tag1 tag2` inside Claude Code
- **Resume sessions** with `cc tag` from anywhere
- **List bookmarks** with `cc -l`
- **Auto-install** the `/bm` command on first run

## Installation

```bash
# Clone the repository
git clone https://github.com/isene/CC-sessions.git
cd CC-sessions

# Add to your PATH (add to .zshrc or .bashrc)
export PATH="$HOME/path/to/CC-sessions/bin:$PATH"

# Or create a symlink
ln -sf /path/to/CC-sessions/bin/cc ~/bin/cc
```

## Usage

### Bookmarking Sessions

Inside a Claude Code session, use the `/bm` command:

```
/bm rtfm ruby filemanager
```

This bookmarks the current session with three tags. You can later resume it
using any of those tags.

### Resuming Sessions

```bash
# Resume session tagged 'rtfm'
cc rtfm

# Continue session in current directory (or start new)
cc

# List all bookmarked sessions
cc -l

# Show help
cc -h
```

### First Run

On first run, `cc` automatically installs the `/bm` command to `~/.claude/commands/`.
This enables the `/bm` command inside Claude Code.

## Files

| File | Purpose |
|------|---------|
| `~/.cc-sessions/bookmarks.json` | Stores your bookmarks |
| `~/.claude/commands/bm.md` | The `/bm` command definition |

## Example Workflow

```bash
# Start working on RTFM project
cd ~/projects/rtfm
claude

# Inside Claude Code, bookmark it
/bm rtfm ruby filemanager

# Later, from anywhere
cc rtfm    # Instantly back in that session
```

## Requirements

- Ruby 2.7+
- Claude Code CLI (`claude`)

## License

MIT
