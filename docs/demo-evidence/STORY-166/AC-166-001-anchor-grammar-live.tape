# STORY-166 AC-166-001(a)-(c): opt-in `path:line:anchor` citation grammar,
# live against a throwaway fixture created OUTSIDE the repo tree (a temp
# dir, via mktemp -d). Shows: (a)+(b) a real path:line:anchor citation
# PASSING when the anchor symbol is actually at the cited line; (c) a
# fabricated-symbol citation FAILING with the exact
# "SYMBOL NOT AT LINE: ..." message; and a bare path:line citation (no
# anchor field) still PASSING, confirming backward compatibility.
# Traces: STORY-166 AC-166-001(a), (b), (c).

Output docs/demo-evidence/STORY-166/AC-166-001-anchor-grammar-live.gif
Output docs/demo-evidence/STORY-166/AC-166-001-anchor-grammar-live.webm

Require bash
Require python3

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-166"
Enter
Wait
Type `export VALIDATE_CITATIONS="$PWD/bin/validate-citations"`
Enter
Wait
Type `cd "$(mktemp -d)"`
Enter
Wait
Type "clear"
Enter
Wait
Show

Type "# throwaway fixture (created in a temp dir, NOT the repo tree)"
Enter
Sleep 300ms
Type "cat > sample.py << 'PYEOF'"
Enter
Type "def compute_total(a, b):"
Enter
Type "    return a + b"
Enter
Type "PYEOF"
Enter
Wait
Type "cat sample.py"
Enter
Wait
Sleep 500ms

Type "# (a)+(b): path:line:anchor citation, REAL symbol at the cited line -- PASSES"
Enter
Sleep 300ms
Type "echo 'sample.py:1:compute_total' > citations-pass.txt"
Enter
Wait
Type `WIRERUST_REPO_ROOT="$PWD" $VALIDATE_CITATIONS citations-pass.txt`
Enter
Wait
Sleep 1s

Type "# (c): fabricated symbol at an in-bounds line -- FAILS with SYMBOL NOT AT LINE"
Enter
Sleep 300ms
Type "echo 'sample.py:1:compute_totaal' > citations-fail.txt"
Enter
Wait
Type `WIRERUST_REPO_ROOT="$PWD" $VALIDATE_CITATIONS citations-fail.txt`
Enter
Wait
Sleep 1s

Type "# backward compat: bare path:line citation (no anchor field) still PASSES"
Enter
Sleep 300ms
Type "echo 'sample.py:1' > citations-bare.txt"
Enter
Wait
Type `WIRERUST_REPO_ROOT="$PWD" $VALIDATE_CITATIONS citations-bare.txt`
Enter
Wait
Sleep 2s
