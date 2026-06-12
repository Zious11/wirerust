Output docs/demo-evidence/STORY-109/AC-009-unsolicited-anomaly.gif
Output docs/demo-evidence/STORY-109/AC-009-unsolicited-anomaly.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-009: FC=0x82 (UNSOLICITED_RESPONSE) with no prior ENABLE_UNSOLICITED →
# T0814 anomaly, Possible/Low, one-shot. ENABLE_UNSOLICITED (FC=0x14) suppresses anomaly.
# Traces to: BC-2.15.019 postconditions 1/2/3; STORY-109 AC-009.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests 'test_unsolicited_response_anomaly_no_prior_enable\|test_unsolicited_response_no_anomaly_after_enable' -- --nocapture"
Enter
Sleep 6s
