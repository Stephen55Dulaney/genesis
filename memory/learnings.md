# Genesis Project Learnings

Codified patterns and insights from development sessions.

---

## [2026-02-19] Bot-to-Builder Session

**Pattern**: Tools + Loop = Agency
**Context**: Any LLM API integration where you want autonomous behavior. 6 tool definitions + 1 executor function + 1 while loop (`while stop_reason == "tool_use"`) transforms a chatbot into a builder. Emergent behaviors (multi-model routing, self-testing) appear without explicit programming.
**Evidence**: [Session log](../sessions/2026/02/2026-02-19_bot-to-builder.md)
**Confidence**: 10/10

---

**Pattern**: Validate before declaring success
**Context**: Agent-generated code and agent-reported capabilities. Agents will declare "GENESIS CAN SEE" before processing a real user image. Always validate against real inputs, not synthetic tests.
**Evidence**: Frog identification used hardcoded Britannica URL, not user's actual photo. Image bug hid for hours.
**Confidence**: 10/10

---

**Pattern**: Micro-Task Validation Protocol
**Context**: Any agent building multi-file systems. Decompose → Build one artifact → Thomas validates → Fix or proceed → TypeWrite documents → Repeat. Prevents overnight code dumps that look impressive but contain hardcoded secrets and untested paths.
**Evidence**: 2,301 lines built overnight, API key found in code review
**Confidence**: 9/10

---

**Pattern**: Notification frequency is the first UX problem
**Context**: Any agent system that outputs to human-facing channels (Telegram, Slack, watch notifications). Set tick intervals at human speed (minutes), not machine speed (seconds). The first thing you build with an AI agent is the volume knob.
**Evidence**: Apple Watch buzzing every few seconds from serendipity + health check + spark notifications
**Confidence**: 10/10

---

**Pattern**: Filesystem inbox/outbox for cross-process communication
**Context**: Connecting systems that can't share a process (terminal CLI + QEMU bridge). JSON files in a shared directory with polling is simpler and more robust than sockets/IPC for low-frequency communication. Route by `"to"` field.
**Evidence**: `~/.genesis/inbox/` + `~/.genesis/outbox/` with 5-second poll loop
**Confidence**: 8/10

---

**Pattern**: Pipe-delimited serialization with escape sequences for serial bridges
**Context**: Passing structured data through serial ports (QEMU stdout/stdin). Use `|` delimiter, escape with `\p` for pipes, `\\n` for newlines, `\\\\` for backslashes. Simple, debuggable, no parser dependency.
**Evidence**: Memory persistence protocol in memory_store.rs
**Confidence**: 9/10

---

**Pattern**: Agent provenance tracking with manifest + commit trailers
**Context**: Any repo where AI agents generate code. `.genesis-manifest.json` tracks author, date, prompt, review status per file. `Built-By:` / `Reviewed-By:` commit trailers in git log. Enables audit trail for agent-generated code.
**Evidence**: Caught hardcoded API key during human review step
**Confidence**: 9/10

---

**Pattern**: Check which Python version the target uses
**Context**: macOS with multiple Python installations. `pip3 install` may target 3.9 while the bridge runs on 3.10. Always use `/usr/local/bin/python3 -m pip install` to target the correct interpreter.
**Evidence**: Thomas reported OpenCV not found after pip3 installed it for wrong Python
**Confidence**: 10/10

---

**Pattern**: Bug-driven architecture discovery
**Context**: When a feature fails in an unexpected way, investigate what the agent *tried* to do. The Telegram image bug revealed that Claude was already doing multi-model routing (calling Gemini via run_python for vision tasks) without being programmed to. Failures expose emergent behaviors.
**Evidence**: Frog chain — 5 AI systems deep, improvised by the agent
**Confidence**: 8/10

---

## [2026-02-19] Protection Tiers Implementation

**Pattern**: AI governance = ceremony level, not access control
**Context**: When AI agents are the developers (human doesn't write the language), you can't use traditional permission systems. Instead, define tiers of *ceremony* — how much discussion, risk assessment, and verification happens before a change. Higher stakes = bigger speed bump, not a wall.
**Evidence**: First draft said "human-only" for Core. Stephen corrected: "I don't write Rust." Redesigned as conversation-based governance.
**Confidence**: 10/10

---

**Pattern**: The architect doesn't need to write the language
**Context**: Genesis OS governance design. Stephen architects, Jeff advises on governance, agents write the Rust. Three roles, three skills. The protection system must respect that the human's value is in design decisions and risk judgment, not in typing code.
**Evidence**: [Session log](../sessions/2026/02/2026-02-19_protection-tiers.md)
**Confidence**: 10/10

---

**Pattern**: Self-improvement needs guardrails, not walls
**Context**: Any system where AI agents can modify their own code. Agents SHOULD proactively refactor and improve — that's the point. But the higher the stakes, the more they should plan first. "We don't want them to self-improve out of existence." Speed bumps (checklists, discussion, build verification) scale with blast radius.
**Evidence**: Jeff's tiered design — "I let you do it, but only in this corner"
**Confidence**: 9/10

---

**Pattern**: Explore the full architecture before designing governance
**Context**: Before building a protection/permission system, map every file, module, and dependency. The Explore subagent generated a 600+ line architecture report that revealed Genesis had NO existing capability restrictions — critical context for knowing what to build.
**Evidence**: Subagent exploration before protection.rs design
**Confidence**: 9/10
