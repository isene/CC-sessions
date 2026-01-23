#!/bin/bash
#
# Install CC-sessions
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing CC-sessions..."

# Create symlinks in ~/bin
if [ -d "$HOME/bin" ]; then
    ln -sf "$SCRIPT_DIR/bin/cc" "$HOME/bin/cc"
    ln -sf "$SCRIPT_DIR/bin/cc-bookmark" "$HOME/bin/cc-bookmark"
    echo "Created symlinks: ~/bin/cc, ~/bin/cc-bookmark"
else
    echo "~/bin does not exist. Add $SCRIPT_DIR/bin to your PATH manually:"
    echo "  export PATH=\"$SCRIPT_DIR/bin:\$PATH\""
fi

# Install the /bm command
mkdir -p "$HOME/.claude/commands"
cp "$SCRIPT_DIR/commands/bm.md" "$HOME/.claude/commands/bm.md"
echo "Installed /bm command to ~/.claude/commands/bm.md"

# Add auto-accept permission for /bm command
SETTINGS_FILE="$HOME/.claude/settings.json"
BM_PERMISSION='Bash(cc-bookmark:*)'

if [ -f "$SETTINGS_FILE" ]; then
    if ! grep -q "$BM_PERMISSION" "$SETTINGS_FILE" 2>/dev/null; then
        # Use ruby to safely modify JSON
        ruby -rjson -e '
            file = ARGV[0]
            perm = ARGV[1]
            settings = JSON.parse(File.read(file))
            settings["permissions"] ||= {}
            settings["permissions"]["allow"] ||= []
            settings["permissions"]["allow"] << perm unless settings["permissions"]["allow"].include?(perm)
            File.write(file, JSON.pretty_generate(settings))
        ' "$SETTINGS_FILE" "$BM_PERMISSION"
        echo "Added auto-accept permission for /bm command"
    fi
else
    # Create new settings file with permission
    mkdir -p "$HOME/.claude"
    echo '{"permissions":{"allow":["'"$BM_PERMISSION"'"]}}' | ruby -rjson -e 'puts JSON.pretty_generate(JSON.parse(STDIN.read))' > "$SETTINGS_FILE"
    echo "Created ~/.claude/settings.json with /bm permission"
fi

echo
echo "Done! You can now:"
echo "  - Use 'cc' to manage Claude Code sessions"
echo "  - Use '/bm tag1 tag2' inside Claude Code to bookmark sessions"
