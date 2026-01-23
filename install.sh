#!/bin/bash
#
# Install CC-sessions
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing CC-sessions..."

# Create symlink in ~/bin
if [ -d "$HOME/bin" ]; then
    ln -sf "$SCRIPT_DIR/bin/cc" "$HOME/bin/cc"
    echo "Created symlink: ~/bin/cc -> $SCRIPT_DIR/bin/cc"
else
    echo "~/bin does not exist. Add $SCRIPT_DIR/bin to your PATH manually:"
    echo "  export PATH=\"$SCRIPT_DIR/bin:\$PATH\""
fi

# Install the /bm command
mkdir -p "$HOME/.claude/commands"
cp "$SCRIPT_DIR/commands/bm.md" "$HOME/.claude/commands/bm.md"
echo "Installed /bm command to ~/.claude/commands/bm.md"

echo
echo "Done! You can now:"
echo "  - Use 'cc' to manage Claude Code sessions"
echo "  - Use '/bm tag1 tag2' inside Claude Code to bookmark sessions"
