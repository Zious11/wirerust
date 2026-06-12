Output docs/demo-evidence/STORY-110/AC-003-early-exit-guard.gif
Output docs/demo-evidence/STORY-110/AC-003-early-exit-guard.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-003: Early-exit guard in on_data extended to include self.dnp3.is_none().
# Without this, on_data silently drops data when only a DNP3 analyzer is active.
# Traces to: BC-2.15.021 invariant 4; STORY-110 AC-003.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build 2>/dev/null"
Enter
Sleep 15s
Show

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_early_exit_guard_includes_dnp3 -- --nocapture"
Enter
Sleep 5s
