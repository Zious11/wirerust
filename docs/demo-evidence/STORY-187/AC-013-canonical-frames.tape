# STORY-187 AC-187-013: canonical, independently-sourced classic S7comm Setup
# Communication frame pair (Job request + Ack_Data response), and the committed
# fixture pcap end-to-end regression (BC-2.21.002/006/008, policy
# DF-CANONICAL-FRAME-HOLDOUT-001)
#
# Demonstrates:
#   - test_BC_2_21_006_canonical_setup_communication_job_frame_on_data: canonical
#     Setup Communication Job (function code 0xF0) PDU, wrapped in real RFC 1006
#     TPKT + class-0 COTP DT (02 F0 80) framing, dispatches and classifies the
#     flow Classic with zero Findings
#   - test_BC_2_21_008_canonical_ack_vector_verbatim: the BC-2.21.008 canonical Ack
#     12-byte vector taken verbatim (no byte substituted)
#   - test_BC_2_21_008_canonical_ack_data_setup_communication_response_on_data: the
#     canonical Ack_Data (0x03) response frame -- header_len == 12, error_class ==
#     Some(0), error_code == Some(0), parameter block begins at data[12] (corrected
#     2026-09-24 ruling, not the pre-ruling data[10] assumption)
#   - test_BC_2_21_002_setup_comm_fixture_pcap_well_formed_no_findings: the
#     committed tests/fixtures/s7comm-setup-comm.pcap capture (synthetic, generator-
#     produced per ADR-014 Decision 7) read via PcapSource::from_file and fed
#     through S7commAnalyzer::on_data end-to-end -- zero Findings,
#     session_established == true, classified_protocol == Some(Classic)
#
# Canonical byte-sequence provenance (policy DF-CANONICAL-FRAME-HOLDOUT-001):
# primary source cnblogs, "Siemens S7 Communication Protocol Reference Compilation"
# (https://www.cnblogs.com/crcce-dncs/p/10659087.html), corroborated by two
# independent sources -- Yiqisoft (https://www.yiqisoft.cn/blogs/IoT-Gateway/363.html)
# and the Inductive Automation KB ("Loggers - Device Connections: Siemens") --
# permitted as test-vector sources per ADR-014 Decision 4's 2026-09-24
# reconciliation note (STORY-187 adversarial pass 5, F-40, human-ratified). Parser
# design and field semantics continue to derive from Decision 4's prose sources
# (Wireshark wiki, Kleinmann & Wool 2014, Orange-Cyberdefense catalog) and
# permitted design references (icsnpp-s7comm, libs7comm, python-snap7).
#
# Traces to: BC-2.21.002 postcondition 3, BC-2.21.006, BC-2.21.008 postconditions
# 1-2; policy DF-CANONICAL-FRAME-HOLDOUT-001
# Test: cargo test --test s7comm_analyzer_tests canonical
# Test: cargo test --test s7comm_analyzer_tests fixture_pcap

Output AC-013-canonical-frames.gif
Output AC-013-canonical-frames.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 560
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# AC-187-013: canonical Setup Communication Job/Ack_Data frames + fixture pcap end-to-end"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests canonical 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 500ms

Type "cargo test --test s7comm_analyzer_tests fixture_pcap 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 2s
