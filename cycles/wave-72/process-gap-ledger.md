---
document_type: process-gap-ledger
cycle: wave-72
created: 2026-07-09
status: open
owner: state-manager
policy: S-7.02
---

# Wave-72 Process-Gap Ledger

Each item below requires a follow-up story or justified deferral before wave-72 is
CLOSED (S-7.02 / lessons-codification). Items without resolution at wave close must be
carried as explicit deferred findings into the next cycle's session-checkpoints.

---

## F-W72-P1-002 — Advisory-pointer idiom not in sibling-sweep targets

**Class:** process-gap  
**Surfaced:** adversary Pass 1  
**Description:** The advisory-pointer idiom (citing a sibling story by ID when the
finding is out of scope for the current story but relevant to a sibling) is not
included in the sibling-sweep target checklist used by story-writer and adversary
agents. Adversaries have re-derived this requirement independently on multiple waves.  
**Required resolution:** Either codify the idiom in the relevant story-writer or
adversary checklist artifact, or file a justified deferral with a target wave.

---

## F-W72-P1-005 — E-11 empty-BC convention vs. tooling interaction

**Class:** process-gap  
**Surfaced:** adversary Pass 1  
**Description:** E-11 stories use `inputs: []` (empty BC list) because they have no
spec inputs. The canonical input-hash tool correctly produces `d41d8cd` for these, but
the bash hook's divergent algorithm (PG-HASH-HOOK-DIVERGENCE) may emit false-positive
drift warnings on E-11 stories in certain tooling contexts. The interaction between the
E-11 convention and the hook divergence is not explicitly documented in the E-11 story
template.  
**Required resolution:** Add a note to the E-11 story template (or the hook-divergence
advisory guidance) clarifying that E-11 `inputs: []` stories will always produce
`d41d8cd` from the canonical tool and any hook discrepancy is a known false positive.

---

## F-W72-P2-006 — Scheduling body/frontmatter drift

**Class:** process-gap  
**Surfaced:** adversary Pass 2  
**Description:** Wave scheduling metadata (story ordering, dependency edges, wave
assignments) can drift between frontmatter fields and the scheduling body prose within
the same story file, because story amendments update one but not the other. No lint
rule or adversary axis explicitly checks for intra-story frontmatter/body scheduling
consistency.  
**Required resolution:** Add an adversary axis or story-writer lint step that checks
wave/dependency frontmatter fields against the scheduling body section of each story.
Or file a justified deferral identifying this as an acceptable known gap.

---

## F-W72-P3-001 — Source citation-style drift class

**Class:** process-gap  
**Surfaced:** adversary Pass 3  
**Description:** Factory documents (stories, BCs, VP shards) use inconsistent citation
styles when referencing external sources — some use inline URLs, some use footnote
style, some use bare identifiers. This inconsistency accumulates across waves and makes
cross-document traceability harder. No citation-style convention is codified in the
story or BC templates.  
**Required resolution:** Either codify a citation-style convention in the relevant
templates (story-writer, BC authoring guide), or file a justified deferral accepting
the current style diversity as non-blocking.

---

## F-W72-P4-001 — Bootstrap own-PR gate axis unrepresented in DF-* policies

**Class:** process-gap  
**Surfaced:** adversary Pass 4  
**Description:** The "bootstrap own-PR gate" axis (a story that codifies tooling must
itself be gated by the tooling it introduces, where feasible) is not represented as a
named policy in `.factory/policies.yaml`. This axis was surfaced as an adversarial
concern during wave-72 but has no durable machine-enforceable form.  
**Required resolution:** Add a DF-* policy entry capturing the bootstrap own-PR gate
requirement, or file a justified deferral explaining why this is handled adequately by
existing adversary pass coverage.

---

## PG-W72-CODIFICATION-FIDELITY / F-W72-P6-001 — Codification stories must be audited against their motivating defect

**Class:** process-gap (primary — highest priority in this ledger)  
**Surfaced:** adversary Pass 6  
**Description:** Codification stories (E-11 type) are intended to durably fix a
diagnosed process gap. Pass 6 found that a codification story's acceptance criteria
had drifted from the specific defect it was meant to close — the ACs addressed a
related but narrower concern, leaving the original gap partially open. The adversary
pass surfaced this as F-W72-P6-001. Root cause: no gate requires the story-writer to
explicitly cross-reference the motivating defect ID (finding ID or process-gap ID) in
each AC and confirm coverage.  
**Required resolution (mandatory before wave-72 CLOSED):** Add a story-writer rule or
adversary axis requiring that every E-11 codification story's ACs explicitly cite the
motivating defect ID and that at least one AC directly closes the diagnosed root cause.
This is the highest-priority item in this ledger and must not be carried as an open
deferral without an explicit justification reviewed at wave close.
