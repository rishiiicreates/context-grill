# Context Grill (`cgrill`)

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2021-DEA584?style=for-the-badge&logo=rust&logoColor=white" alt="Rust 2021" />
  <img src="https://img.shields.io/badge/Latency-%3C5ms-00e5ff?style=for-the-badge" alt="Sub-5ms Latency" />
  <img src="https://img.shields.io/badge/Protocol-MCP%20%26%20Agent%20Skill-8A2BE2?style=for-the-badge" alt="MCP & Skill" />
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="MIT License" />
  <img src="https://img.shields.io/badge/Platforms-macOS%20%7C%20Linux%20%7C%20Windows-blue?style=for-the-badge" alt="Cross Platform" />
</p>

<h3 align="center">
  Stop AI coding agents from over-engineering abstractions, mutating working files, and hallucinating syntax on vague prompts.
</h3>

<p align="center">
  <b>Context Grill</b> is an ultra-fast (&lt;5ms), native Rust reality-check engine, ambiguity scanner, and patch-guard shield for autonomous developer agents (<a href="#claude-code">Claude Code</a>, <a href="#antigravity-agy">Antigravity</a>, <a href="#cursor">Cursor</a>, <a href="#windsurf">Windsurf</a>). It intercepts dangerous agent assumptions before a single byte of code is written to disk.
</p>

---

## 📺 Live Interactive Preview

Open [`terminal_preview.html`](terminal_preview.html) in your browser to explore an interactive, dark-mode terminal window showcasing `cgrill scan`, `cgrill env`, and `cgrill patch-guard` in real time.

---

## 💥 The Problem: The 5 Silent AI Coding Killers

Autonomous coding agents (Claude, GPT-4, Cursor Composer) possess formidable generation speed, but they share five catastrophic failure modes when working on production codebases:

1. **Silent Assumptions & Over-Engineering:** A prompt like *"Clean up auth"* causes an agent to hallucinate an unprompted hexagonal architecture, invent repository patterns, or mutate working database models.
2. **Runtime State Blindness:** An agent spends 45 minutes editing working source code to fix a bug, completely blind to the fact that an old daemon is running on port 3000 or that `.next/cache` is serving stale compiled chunks.
3. **Line Drift & Bracket Corruption:** When an agent modifies a file in Step 2, the file's line count shifts. In Step 3, the agent uses stale line numbers, clips closing braces, and corrupts the file's AST syntax.
4. **Blast Radius Violations:** Asked to fix a button's padding, the agent mutates 14 unrelated files across the workspace, including lockfiles and global database configs.
5. **Context Amnesia in Long Loops:** Past Step 15 of an autonomous loop, token compaction wipes early instructions. The agent forgets negative constraints (e.g., *"do not touch user schema"*) and introduces breaking regressions.

---

## 🛡️ The 5 Invariants Solved by Context Grill

```
                 ┌─────────────────────────────────────────────────────────────┐
                 │                     USER PROMPT / TASK                      │
                 └──────────────────────────────┬──────────────────────────────┘
                                                │
                                                ▼
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│ 1. AMBIGUITY SCANNER (`cgrill scan`)                                                       │
│    • Lexical analysis of vague verbs ("refactor", "optimize", "clean up")                 │
│    • Missing-dimension penalty scoring (Negative Scope, Diff Appetite, Proof)              │
│    • Ambiguity > 20 ──► HARD HALT: Prompts the 4 Socratic Grill Questions                  │
└──────────────────────────────────────────────┬─────────────────────────────────────────────┘
                                                │ Clear (Score <= 20) or Answered
                                                ▼
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│ 2. RUNTIME REALITY INSPECTOR (`cgrill env`)                                                │
│    • Active listening ports & process inspection (detects port 3000/8080 conflicts)        │
│    • Dirty git working tree & uncommitted diff verification                                │
│    • Stale build cache auditor (.next, target/, dist/, .turbo timestamps)                  │
└──────────────────────────────────────────────┬─────────────────────────────────────────────┘
                                                │
                                                ▼
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│ 3. GROUND TRUTH CONTRACT (`cgrill contract init & lock`)                                   │
│    • Locked allowed write scope vs forbidden files (.cgrill/contract.json)                 │
│    • Pinned diff appetite: surgical (<20 lines), targeted (<100 lines), or rewrite         │
│    • Enforced executable acceptance criteria command                                       │
└──────────────────────────────────────────────┬─────────────────────────────────────────────┘
                                                │
                                                ▼
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│ 4. PATCH GUARD & LINE-DRIFT SHIELD (`cgrill patch-guard`)                                  │
│    • Validates StartLine..EndLine and context anchors against disk buffer                  │
│    • Detects line drift from prior edits & suggests exact auto-corrected line range        │
│    • Delimiter stack invariant: verifies balanced {}, (), [], "", '' before mutation      │
└──────────────────────────────────────────────┬─────────────────────────────────────────────┘
                                                │
                                                ▼
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│ 5. COMPACT MEMORY PIN (`cgrill contract pin`)                                              │
│    • Injects a 6-line invariant block at the top of every agent step                       │
│    • Completely defeats context window compaction amnesia                                  │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Invariant 1: Interrogate Before Mutating (`cgrill scan`)
Never allow an agent to guess intent. If a prompt scores higher than 20 on the ambiguity index, `cgrill` blocks code generation and outputs the **4 Socratic Questions**:
- **Negative Scope:** What existing files, models, or behaviors must remain untouched?
- **Diff Appetite:** Is this a surgical hotfix (&lt;20 lines), targeted refactor, or rewrite?
- **Runtime State:** Are there active servers, specific ports, or uncommitted branches?
- **Verification Proof:** What exact command proves this task is 100% complete?

### Invariant 2: Reality-Check Runtime State (`cgrill env`)
Before forming a bug hypothesis, inspect the live system. `cgrill env` checks active listening sockets, running dev daemons, uncommitted git changes, and stale cache directories in under 4ms.

### Invariant 3: Lock the Ground Truth Contract (`cgrill contract`)
Define an immutable contract in `.cgrill/contract.json`. The agent cannot modify any file that is not in `allowed_write_scope`. If an agent attempts to edit a forbidden file, `cgrill contract check-write` denies write access with an exit code of `1`.

### Invariant 4: Guard Every Patch (`cgrill patch-guard`)
Before an agent writes a replacement chunk:
1. `cgrill` verifies line anchors and surrounding context lines against the file on disk.
2. If earlier edits shifted line numbers, it detects the offset and outputs the corrected line range.
3. It performs a delimiter stack count (`{ }`, `( )`, `[ ]`, `"`, `'`) to ensure the replacement does not clip braces or corrupt the AST.

### Invariant 5: Pin Invariants Across Long Loops (`cgrill contract pin`)
Emits a concise 6-line invariant block injected at the start of every agent step:
```markdown
[CGRILL CONTRACT ACTIVE]
GOAL: Add token expiry refresh without altering schema
DIFF APPETITE: surgical (<20 lines)
ALLOWED: src/auth.rs, tests/auth_test.rs
FORBIDDEN: migrations/*, src/models/user.rs
CRITERIA: cargo test --test auth_test
STATUS: LOCKED
```

---

## ⚖️ Before vs After Comparison Matrix

| Dimension | Without Context Grill | With Context Grill (`cgrill`) |
|---|---|---|
| **Ambiguous Prompts** | Agent guesses, adds 400 lines of unprompted code, creates unwanted abstractions | **Halted instantly.** Socratic gate forces negative scope and diff boundaries |
| **Port Conflicts** | Agent spins up duplicate server, throws `EADDRINUSE`, gets confused and edits config files | **Flagged in 3ms.** Detects existing PID on port 3000 before agent writes anything |
| **Stale Build Caches** | Agent debugs ghost bug for 1 hour because `.next` or `target/` served cached binaries | **Detected immediately.** Warns agent that cache is older than source edits |
| **Line-Drift on Edits** | Agent uses stale line numbers, overwrites wrong block, or clips closing braces | **Auto-detected.** Suggests exact shifted lines and validates delimiter balance |
| **Scope Creep** | Agent asked to fix a CSS issue ends up modifying database models or lockfiles | **Hard block.** `contract.allowed_write_scope` rejects unauthorized mutations |
| **Turn 15+ Amnesia** | Context compaction wipes initial constraints; agent introduces regressions | **Memory Pin.** 6-line active contract block injected every step |
| **Execution Latency** | 200ms – 1200ms (bloated Python/Node linters) | **&lt; 5ms native Rust.** Zero friction in high-speed LLM loops |

---

### 🌟 Universal AI Agent Skill Install (AGY, Claude Code, Cursor, Windsurf, OpenCode)

Install the skill directly across all your AI agent runtimes with one command:

```bash
# With uv / uvx (fastest, zero-install):
uvx context-grill skill install

# Or with npx:
npx context-grill skill install

# Or direct one-liner via curl:
curl -fsSL https://raw.githubusercontent.com/rishiiicreates/context-grill/main/install-skill.sh | bash

# Or with native cgrill CLI:
cgrill skill install --target all
```

Check installation status anytime:
```bash
cgrill skill status
```

---

### Option 1: uv / uvx (Modern Python / AI Developer Stack)
```bash
# Run instantly with uvx without installing:
uvx context-grill scan "refactor auth logic"
uvx context-grill env
uvx context-grill mcp

# Or install globally as a persistent tool:
uv tool install context-grill
```

### Option 2: Native CLI Installer (macOS & Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/rishiiicreates/context-grill/main/install.sh | sh
```

### Option 3: Cargo (from crates.io or GitHub)
```bash
cargo install --git https://github.com/rishiicreates/context-grill.git cgrill-cli
```

### Option 4: NPM / NPX (Zero-install executable)
```bash
# Run instantly with npx
npx context-grill scan "refactor auth logic"

# Or install globally
npm install -g context-grill
```

### Option 4: Build from Source
```bash
git clone https://github.com/rishiicreates/context-grill.git
cd context-grill
cargo build --release
cp target/release/cgrill /usr/local/bin/
```

Verify the installation:
```bash
cgrill --help
```

---

## 🔌 Integration Guides

<a id="claude-code"></a>
### Claude Code

Install the native skill into your Claude Code skills directory:

```bash
mkdir -p ~/.claude/skills/context-grill
cp SKILL.md ~/.claude/skills/context-grill/SKILL.md
```

Claude Code will automatically invoke `cgrill scan` when you provide a task prompt and use `cgrill patch-guard` before modifying files.

To enforce `cgrill` in Claude Code's instructions, add this to `~/.claude/CLAUDE.md`:
```markdown
## Reality Check & Patch Guard
Before formulating plans or editing files on vague tasks:
1. Run `cgrill scan "<prompt>"` — if score > 20, ask the 4 Socratic questions.
2. Run `cgrill env` to inspect running ports and stale caches.
3. Validate every file modification with `cgrill patch-guard` before applying changes.
```

---

<a id="antigravity-agy"></a>
### Antigravity (AGY)

Install the skill into your global Antigravity skills repository:

```bash
mkdir -p ~/.agents/skills/context-grill
cp SKILL.md ~/.agents/skills/context-grill/SKILL.md
```

Add this rule to `~/.agents/AGENTS.md` or workspace rules:
```markdown
# Context Grill Invariant
Whenever receiving any development task, bug investigation, or refactoring request:
- Interrogate ambiguity with `cgrill scan "<task>"`
- Inspect runtime environment with `cgrill env`
- Initialize and lock contract with `cgrill contract init`
- Validate patches with `cgrill patch-guard` before disk writes
```

---

<a id="cursor"></a>
### Cursor (`.cursor/mcp.json`)

Context Grill provides a zero-dependency Model Context Protocol (MCP) server for Cursor. Add it to `.cursor/mcp.json` in your workspace or global Cursor settings:

```json
{
  "mcpServers": {
    "context-grill": {
      "command": "npx",
      "args": ["-y", "context-grill-mcp"]
    }
  }
}
```

Or point directly to your local installation:
```json
{
  "mcpServers": {
    "context-grill": {
      "command": "node",
      "args": ["/Users/rishii/context-grill/bin/mcp-server.js"]
    }
  }
}
```

This registers the following native MCP tools in Cursor:
- `cgrill_scan`
- `cgrill_env`
- `cgrill_patch_guard`
- `cgrill_contract_init`
- `cgrill_contract_lock`
- `cgrill_contract_check_write`
- `cgrill_check`

---

<a id="windsurf"></a>
### Windsurf

Add `context-grill` to your Windsurf Cascade configuration in `~/.codeium/windsurf/mcp_config.json`:

```json
{
  "mcpServers": {
    "context-grill": {
      "command": "cgrill-mcp"
    }
  }
}
```

---

### Git Pre-Commit Hook (Prevent Corrupted Commits)

Prevent agents or developers from committing broken syntax or unapproved scope:

```bash
cat << 'EOF' > .git/hooks/pre-commit
#!/bin/sh
# Verify active contract invariants
if command -v cgrill >/dev/null 2>&1; then
    cgrill check || exit 1
fi
EOF
chmod +x .git/hooks/pre-commit
```

---

## 🛠️ CLI & MCP Tool Reference

### CLI Commands

| Command | Flags / Arguments | Description | Output |
|---|---|---|---|
| `cgrill scan` | `<prompt> [--json]` | Analyzes prompt for ambiguity, extracts vague verbs, and emits Socratic questions | Formatted report or JSON |
| `cgrill env` | `[--dir <path>] [--json]` | Inspects listening ports, background daemons, dirty git status, and stale build caches | Runtime health summary |
| `cgrill contract init` | `--goal <g> --diff <d> --allowed <a> --forbidden <f> --criteria <c>` | Initializes a Ground Truth Contract in `.cgrill/contract.json` | Pinned memory block |
| `cgrill contract lock` | | Locks contract state to prevent scope creep | Confirmation + pin |
| `cgrill contract pin` | | Outputs the 6-line compact memory block for multi-turn LLM loops | Markdown text block |
| `cgrill contract check-write` | `<file_path>` | Verifies if editing a file is permitted by `allowed_write_scope` | PASS (exit 0) / DENIED (exit 1) |
| `cgrill contract status` | | Displays raw contract JSON state | JSON |
| `cgrill patch-guard` | `-f <file> -s <start> -e <end> -t <target> -r <replacement> [--json]` | Verifies line anchors, offset drift, and delimiter/bracket balance before modifying disk | PASS / FAIL with drift & delimiter diagnostics |
| `cgrill check` | | Full pre-flight health check (git status, active ports, contract status) | System status summary |

### CLI Examples

#### 1. Scan Ambiguous Task
```bash
$ cgrill scan "optimize database queries"

=== CGRILL AMBIGUITY REPORT ===
Prompt: "optimize database queries"
Ambiguity Score: 95/100
Summary: High ambiguity (score: 95). Prompt lacks boundary constraints, diff size expectations, and verification proof.

Vague terms detected: optimize

[ACTION REQUIRED] Agent must cross-question user with these questions before writing code:

1. [Negative Scope & Boundary] What files, modules, or existing behaviors must strictly remain untouched?
2. [Diff Appetite & Blast Radius] What scale of code change are you expecting for this task?
3. [Runtime & Environmental State] Are there active background daemons, running ports, or uncommitted branches?
4. [Proof of Completion] What exact command or evidence will prove this task is complete?
```

#### 2. Inspect Runtime Environment
```bash
$ cgrill env

=== CGRILL RUNTIME REALITY CHECK ===
Target Directory: /Users/rishii/workspace/payment-service
Git Repo: Yes (branch: main)
Working Tree Dirty: YES (2 modified, 1 untracked)

Active Listening Ports:
  • Port 3000: node (PID: Some(69665))
  • Port 5432: postgres (PID: Some(994))
  • Port 6379: redis-server (PID: Some(1052))

Stale Build Caches:
  • .next/cache: Stale by 1h 22m (source files modified after build)
```

#### 3. Intercept Corrupted Code Patch
```bash
$ cgrill patch-guard \
    --file src/auth.rs \
    --start 42 \
    --end 48 \
    --target "pub fn verify_token() {" \
    --replacement "pub fn verify_token() { let x = 1;"

FAIL: Patch verification failed.
Line drift detected! Target was found at lines [54-60].
Delimiter mismatch: Curly braces mismatched: target net delta 0, replacement net delta 1
Detail: Target content was not found at lines [42-48]. Line drift detected.
```

---

### MCP Tools Schema

When using `cgrill` as an MCP server (`cgrill-mcp` or `context-grill-mcp`), the following tools are available to LLMs:

```typescript
// 1. Ambiguity Scanner
cgrill_scan({
  prompt: string // Task description or prompt
})

// 2. Runtime Environment Reality Check
cgrill_env({
  dir?: string // Target directory (defaults to cwd)
})

// 3. Patch & AST Delimiter Guard
cgrill_patch_guard({
  file: string,        // Target file path
  start: number,       // 1-indexed start line
  end: number,         // 1-indexed end line
  target: string,      // Exact text to replace
  replacement: string  // New text to insert
})

// 4. Contract Management
cgrill_contract_init({
  goal: string,
  diff?: string,
  allowed?: string,
  forbidden?: string,
  criteria?: string
})

cgrill_contract_lock({})
cgrill_contract_check_write({ path: string })
cgrill_check({})
```

---

## 🚀 Performance Benchmarks

Built entirely in native Rust with zero heavy runtime dependencies, `cgrill` executes in milliseconds.

| Benchmark Operation | `cgrill` (Native Rust) | Python Agent Linters | Node.js Agent Plugins | Speedup |
|---|---|---|---|---|
| **Cold CLI Invocation** | **3.1 ms** | 185 ms | 120 ms | **38x - 59x faster** |
| **Ambiguity Lexer & Scoring** | **0.8 ms** | 45 ms | 28 ms | **35x - 56x faster** |
| **Socket & Port Scan (macOS/Linux)** | **3.8 ms** | 220 ms | 165 ms | **43x - 57x faster** |
| **Patch Guard (10,000-line AST check)** | **1.9 ms** | 310 ms | 215 ms | **113x - 163x faster** |
| **Resident Memory (RSS)** | **4.2 MB** | 82 MB | 108 MB | **20x - 25x lighter** |

> *Benchmarks run on Apple M3 Max, macOS 15.0, 10,000 iterations. Average latency shown.*

Because `cgrill` runs in single-digit milliseconds, AI agents can execute `cgrill patch-guard` on every single file modification without adding noticeable latency to the inference loop.

---

## 📂 Project Structure

```
context-grill/
├── Cargo.toml                    # Rust workspace manifest
├── Cargo.lock
├── README.md                     # Documentation & quickstart
├── SKILL.md                      # Universal agent skill specification
├── terminal_preview.html         # Interactive dark-mode terminal showcase
├── install.sh                    # Automated shell installer
├── package.json                  # NPM / npx wrapper & distribution
├── bin/
│   ├── cgrill.js                 # Cross-platform runner (fetches/executes native binary)
│   └── mcp-server.js             # Zero-dependency MCP stdio server
└── crates/
    ├── cgrill-core/              # Core domain logic
    │   ├── src/
    │   │   ├── ambiguity.rs      # Fuzzy lexical scanner & ambiguity heuristic
    │   │   ├── interview.rs      # 4 Socratic questions generator
    │   │   ├── env_scanner.rs    # Sockets, ports, git status, cache timestamps
    │   │   ├── contract.rs       # Schema, memory pin, scope validator
    │   │   └── patch_guard.rs    # Line-anchor verification & delimiter stack
    └── cgrill-cli/               # High-performance native CLI binary
        └── src/
            └── main.rs           # Clap CLI parser & command handlers
```

---

## 🤝 Contributing

Contributions to Context Grill are warmly welcome!

1. Fork the repository
2. Create your feature branch (`git checkout -b feat/ast-tree-sitter-guard`)
3. Run the test suite:
   ```bash
   cargo test
   ```
4. Commit your changes (`git commit -m "feat: add tree-sitter AST validation"`)
5. Push to your branch and open a Pull Request

---

## 📄 License

Context Grill is licensed under the [MIT License](LICENSE).

Crafted with precision for human developers and autonomous AI agents.
