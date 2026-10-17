---
name: context-grill
description: Use when receiving any development task, feature request, bug fix, or refactor to interrogate vague requirements, prevent over-engineering, inspect runtime state, protect against line drift, and enforce blast radius limits.
---

# Context Grill (`cgrill`)

An ultra-fast native Rust reality check and context extraction system that eliminates the five primary failure modes of AI coding agents:
1. Silent assumptions and over-engineering
2. Runtime state blindness (ports, daemons, stale caches)
3. Line drift and bracket corruption on file edits
4. Blast radius violations (mutating code outside declared scope)
5. Context amnesia on long multi-turn loops

---

## The 5 Invariants of Context Grill

1. **Interrogate Before Mutating:** If a task prompt has missing constraints or an ambiguity score > 20, you MUST halt and present the 4 Socratic questions. Never guess intent.
2. **Reality-Check Runtime State:** Always check active listening ports, background dev processes, dirty git trees, and stale build caches with `cgrill env` before forming bug hypotheses.
3. **Lock the Contract:** Declare `goal`, `diff_appetite`, `allowed_write_scope`, `forbidden_scope`, and `acceptance_criteria` in `.cgrill/contract.json`. Never touch files outside the allowed scope.
4. **Guard Every Patch:** Before modifying any file on disk, verify line anchors and delimiter balance using `cgrill patch-guard` to prevent line drift and syntax clipping.
5. **Pin Memory Across Turns:** Inject the 6-line `[CGRILL CONTRACT ACTIVE]` block at the start of every step in long execution loops to prevent token compaction amnesia.

---

## Workflow Protocol

### Step 1: Ambiguity Scan
When the user gives a prompt:
```bash
cgrill scan "<user_prompt>"
```
- If Ambiguity Score is **<= 20**: Task is clear. Proceed to Step 2.
- If Ambiguity Score is **> 20**: **STOP IMMEDIATELY.** Do NOT write code or spawn heavy planning agents. Present the 4 Socratic Questions:
  - **Negative Scope:** What existing files or behaviors must remain untouched?
  - **Diff Appetite:** Is this a surgical hotfix (<20 lines), targeted refactor, or full rewrite?
  - **Runtime State:** Are there running servers, specific ports, or uncommitted files?
  - **Verification Proof:** What exact command or test output proves this is done?

### Step 2: Runtime Reality Inspection
Check the active environment:
```bash
cgrill env
```
Inspect:
- Active listening ports (preventing EADDRINUSE collisions)
- Stale build caches in `dist/`, `target/`, `.next/`, or `.turbo`
- Dirty git working tree (uncommitted files that must not be lost)

### Step 3: Initialize & Lock the Contract
Once the user answers the 4 questions:
```bash
cgrill contract init --goal "<goal>" --diff "<appetite>" --allowed "<files>" --forbidden "<files>" --criteria "<command>"
cgrill contract lock
```

Emit the pinned memory block into your working context:
```bash
cgrill contract pin
```

### Step 4: Guard File Edits (Patch Guard)
Before making file modifications:
```bash
cgrill patch-guard --file <path> --start <line> --end <line> --target "<target_text>" --replacement "<new_text>"
```
- If PASS: apply the edit with confidence.
- If LINE DRIFT DETECTED: use the suggested line range provided by `cgrill`.
- If DELIMITER MISMATCH: fix the unclosed brace/bracket/parenthesis before touching disk.

### Step 5: Verification & Evidence
Run the required acceptance command:
```bash
<acceptance_criteria_command>
```
Confirm the validator passes before declaring task complete.
