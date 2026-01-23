# Bookmark Session

Bookmark the current Claude Code session with tags for easy resumption later.

## Arguments

The user provides space-separated tags after `/bm`. For example:
- `/bm rtfm` - one tag
- `/bm rtfm ruby filemanager` - multiple tags

## What to Do

Run the cc-bookmark command with the provided tags:

```bash
cc-bookmark $TAGS
```

Replace `$TAGS` with the actual tags provided by the user.

## Example

User types: `/bm rtfm ruby`

Run: `cc-bookmark rtfm ruby`
