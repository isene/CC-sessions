# Bookmark Session

Bookmark the current Claude Code session with tags for easy resumption later.

## Arguments

The user provides space-separated tags after `/bm`. For example:
- `/bm rtfm` - one tag
- `/bm rtfm ruby filemanager` - multiple tags

## What to Do

1. Get the current working directory (this is the session path)
2. Parse the tags from the command arguments (the text after `/bm`)
3. Store the bookmark in `~/.cc-sessions/bookmarks.json`
4. If no tags provided, show current bookmark for this directory (if any)

## Implementation

Run this bash command, replacing `$TAGS` with the actual tags provided:

```bash
mkdir -p ~/.cc-sessions && ruby -rjson -e '
  file = File.expand_path("~/.cc-sessions/bookmarks.json")
  bookmarks = File.exist?(file) ? JSON.parse(File.read(file)) : {}
  cwd = Dir.pwd
  tags = ARGV
  if tags.empty?
    if bookmarks[cwd]
      puts "Current bookmark: #{bookmarks[cwd].join(", ")}"
    else
      puts "No bookmark for this directory. Usage: /bm tag1 tag2 ..."
    end
  else
    bookmarks[cwd] = tags
    File.write(file, JSON.pretty_generate(bookmarks))
    puts "Bookmarked: #{cwd}"
    puts "Tags: #{tags.join(", ")}"
    puts ""
    puts "Resume later with: cc #{tags.first}"
  end
' $TAGS
```

## Response Format

After bookmarking, confirm:
```
Bookmarked: /path/to/session
Tags: tag1, tag2, tag3

Resume later with: cc tag1
```
