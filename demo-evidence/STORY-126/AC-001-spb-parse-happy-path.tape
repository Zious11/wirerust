Output .factory/demo-evidence/STORY-126/AC-001-spb-parse-happy-path.gif
Output .factory/demo-evidence/STORY-126/AC-001-spb-parse-happy-path.webm

Require bash

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 480
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-001 / AC-002 / AC-003: SPB parse happy path
# Fixture: SHB + IDB(Ethernet) + SPB(14-byte Ethernet frame)
# Demonstrates: pcapng with Simple Packet Block is parsed without panic or error;
# SPB uses interface 0 from the preceding IDB; timestamp_secs=0, timestamp_usecs=0.
# Traces to: BC-2.01.013 AC-001, AC-002, AC-003; STORY-126.

Hide
Type "python3 /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-126/mk_spb_fixtures.py"
Enter
Sleep 3s
Show

Type "# AC-001/002/003: SHB+IDB+SPB(14-byte Ethernet) parsed without error"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-126/target/release/wirerust summary /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-126/spb-happy.pcapng --no-color"
Enter
Sleep 4s
