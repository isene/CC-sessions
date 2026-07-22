#!/bin/bash
#
# Install CC-sessions (Rust)
#
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Building CC-sessions..."
cd "$SCRIPT_DIR"
PATH="/usr/bin:$PATH" cargo build --release

if [ -d "$HOME/bin" ]; then
    ln -sf "$SCRIPT_DIR/target/release/cc" "$HOME/bin/cc"
    ln -sf "$SCRIPT_DIR/target/release/cl" "$HOME/bin/cl"
    ln -sf "$SCRIPT_DIR/target/release/cc-bookmark" "$HOME/bin/cc-bookmark"
    echo "Created symlinks: ~/bin/cc, ~/bin/cl, ~/bin/cc-bookmark"
else
    echo "~/bin does not exist. Add $SCRIPT_DIR/target/release to your PATH."
fi

# First run installs the /bm command + permission
"$SCRIPT_DIR/target/release/cc" -h > /dev/null
echo "Done."
