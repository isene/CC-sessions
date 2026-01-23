# CC-sessions

![Ruby](https://img.shields.io/badge/language-Ruby-red) [![Gem Version](https://badge.fury.io/rb/cc-sessions.svg)](https://badge.fury.io/rb/cc-sessions) ![Unlicense](https://img.shields.io/badge/license-Unlicense-green)

<img src="img/cc-sessions_logo.svg" align="left" width="150" height="150">

A simple tool for bookmarking and resuming Claude Code sessions with tags.

Claude Code sessions are tied to directories, making it hard to remember where
you were working on specific projects. This tool lets you tag sessions with
meaningful names and quickly resume them.

<br clear="left"/>

## Features

- **Bookmark sessions** with `/bm tag1 tag2` inside Claude Code
- **Resume sessions** with `cc tag` from anywhere
- **List bookmarks** with `cc -l`
- **Auto-install** the `/bm` command and permission on first run

## Installation

### From RubyGems (Recommended)

```bash
gem install cc-sessions
```

### From Source

```bash
git clone https://github.com/isene/CC-sessions.git
cd CC-sessions
./install.sh
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

On first run, `cc` automatically:
1. Installs the `/bm` command to `~/.claude/commands/`
2. Adds auto-accept permission to `~/.claude/settings.json`

This enables `/bm` to work without confirmation prompts.

## Files

| File | Purpose |
|------|---------|
| `~/.cc-sessions/bookmarks.json` | Stores your bookmarks |
| `~/.claude/commands/bm.md` | The `/bm` command definition |
| `~/.claude/settings.json` | Permission for auto-accept |

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

This software is released into the public domain under [The Unlicense](https://unlicense.org/).
