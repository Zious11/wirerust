#!/usr/bin/env bash
# AC-183-006 RED-path demo: proves check-green-doc-tense flags a violating bin/*.py
# file with exit 1 + Pattern 32, and that the same content in a .rs file is NOT
# scan-eligible (suffix-scoped comment detection, F-008/F-009).
#
# Runs entirely inside throwaway git repos under mktemp -- never touches the
# real wirerust tree. Self-locates the repo root so this script is portable
# after the STORY-183 worktree is cleaned up.
set -uo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
REPO_TOOL="$REPO_ROOT/bin/check-green-doc-tense"
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

echo "=== AC-183-006: RED-path demo (hermetic throwaway repo, not the wirerust tree) ==="
git init --quiet "$SCRATCH"
mkdir -p "$SCRATCH/bin"
cp "$REPO_TOOL" "$SCRATCH/bin/check-green-doc-tense"

# Write a violating .py file (Pattern 32: "currently asserts") into the throwaway repo only.
printf '%s\n' '# currently asserts the implementation is complete' > "$SCRATCH/bin/violating.py"

git -C "$SCRATCH" add bin/violating.py >/dev/null

echo "--- Running: python3 bin/check-green-doc-tense (cwd = throwaway repo) ---"
python3 "$SCRATCH/bin/check-green-doc-tense"
EXIT_CODE=$?
echo "--- exit code: $EXIT_CODE ---"

echo
echo "=== Negative control: same '#' line in a .rs file is NOT scan-eligible (suffix-scoped) ==="
RS_SCRATCH="$(mktemp -d)"
git init --quiet "$RS_SCRATCH"
mkdir -p "$RS_SCRATCH/bin" "$RS_SCRATCH/src"
cp "$REPO_TOOL" "$RS_SCRATCH/bin/check-green-doc-tense"
printf '%s\n' '# currently asserts the implementation is complete' 'fn main() {}' > "$RS_SCRATCH/src/placeholder.rs"
git -C "$RS_SCRATCH" add src/placeholder.rs >/dev/null
echo "--- Running: python3 bin/check-green-doc-tense (cwd = second throwaway repo, .rs file) ---"
python3 "$RS_SCRATCH/bin/check-green-doc-tense"
EXIT_CODE2=$?
echo "--- exit code: $EXIT_CODE2 ---"
rm -rf "$RS_SCRATCH"
