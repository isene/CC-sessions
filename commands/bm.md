# Bookmark Session

Bookmark the current Claude Code session with tags for easy resumption later.

## Arguments

- `/bm tag1 tag2` - Bookmark with tags
- `/bm?` or `/bm ?` - Show current bookmark status

## What to Do

If the argument is `?` or empty after `/bm?`:
```bash
cc-bookmark
```

Otherwise, run with the provided tags:
```bash
cc-bookmark $TAGS
```

Replace `$TAGS` with the actual tags provided by the user.

## Examples

- `/bm rtfm ruby` → Run: `cc-bookmark rtfm ruby`
- `/bm?` → Run: `cc-bookmark` (shows current bookmark)
- `/bm ?` → Run: `cc-bookmark` (shows current bookmark)
