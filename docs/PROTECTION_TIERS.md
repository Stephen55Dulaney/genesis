# Genesis Protection Tiers

*Designed with Jeff (Momentum CEO) — February 19, 2026*

## Philosophy

> "You need to define what it is. It has to be the ultimate shared service."
> — Jeff, on Tier 1 (Core)

> "I let you do it, but only in this corner over here."
> — Jeff, on Tier 3 (Maintained)

Genesis agents write all the code. Stephen doesn't write Rust — the agents do.
That's the whole point. But not all changes carry equal risk.

A tweak to a prompt template is low-stakes: if it's wrong, fix it next session.
A change to the kernel boot sequence can triple-fault the CPU and brick the system.
Both get written by agents. The difference is **how much we plan before we act.**

These tiers define the level of ceremony — conversation, risk assessment, and
verification — required before a change lands. They are NOT about blocking agents
from writing code. They're about making sure we think before we touch the stove.

## The Five Tiers

### Tier 1: Core (Plan Before You Touch)

**Rule: Discuss the change. Understand the risk. Agree it's worth doing. Then do it carefully.**

The core is the foundation. If it breaks, nothing works. Changes here require:

- [ ] **Discussion**: Stephen and the agent talk through what's changing and why
- [ ] **Risk assessment**: What breaks if this goes wrong? Can we recover?
- [ ] **Scope check**: Is this the minimum change needed? No drive-by refactors.
- [ ] **Build verification**: Compile succeeds after the change
- [ ] **Rollback plan**: We know how to undo it (git revert, etc.)

| Path | What It Is |
|------|-----------|
| `kernel/src/main.rs` | Kernel entry point, boot sequence |
| `kernel/src/interrupts.rs` | IDT & interrupt handlers |
| `kernel/src/memory.rs` | Page table management |
| `kernel/src/allocator.rs` | Heap allocator |
| `kernel/src/serial.rs` | COM1 UART (bridge communication) |
| `kernel/src/agents/mod.rs` | Core Agent trait definition |
| `kernel/src/agents/protection.rs` | Protection system itself |
| `Cargo.toml` | Workspace build configuration |
| `kernel/Cargo.toml` | Kernel dependencies |
| `x86_64-genesis.json` | Custom target specification |
| `rust-toolchain.toml` | Rust toolchain pinning |
| `.genesis-manifest.json` | Provenance tracking |

**Why:** These are the "ultimate shared service." Every agent, every message, every
tick depends on these. A bad allocator change kills the whole system. A bad
interrupt handler triple-faults the CPU. We CAN change them — we just think first.

### Tier 2: Guarded (Talk It Through)

**Rule: Discuss the approach. Verify it doesn't break the contract other code depends on.**

These files define how agents communicate and orchestrate. Changes here ripple
to all agents. The ceremony is lighter than Core but still collaborative:

- [ ] **Discuss approach**: What are we changing and why?
- [ ] **Impact check**: Which agents/features does this affect?
- [ ] **Build verification**: Compile succeeds

| Path | What It Is |
|------|-----------|
| `kernel/src/agents/supervisor.rs` | Sam — orchestrates all agents |
| `kernel/src/agents/message.rs` | Message types & routing |
| `kernel/src/storage/mod.rs` | Storage system interface |
| `kernel/src/storage/filesystem.rs` | In-memory VFS |
| `kernel/src/storage/memory_store.rs` | BM25-lite searchable memory |
| `kernel/src/shell.rs` | Interactive command shell |
| `tools/genesis-bridge.py` | QEMU serial ↔ Telegram ↔ LLM |

**Why:** The Supervisor routes every message. The bridge connects Genesis to the
outside world. Changes here can subtly break things that only show up at runtime.
A quick conversation before coding catches most problems.

### Tier 3: Maintained (Build and Verify)

**Rule: Make the change, make sure it compiles, spot-check periodically.**

This is Jeff's "I let you do it, but only in this corner" tier. Agent implementations,
prompt evolution, and UI code live here. Agents can improve their own code —
and should. Thomas validating his own test patterns, Archimedes refining his
ambition parsing, GUI tweaks — all normal maintenance.

| Path | What It Is |
|------|-----------|
| `kernel/src/agents/thomas.rs` | Thomas agent implementation |
| `kernel/src/agents/archimedes.rs` | Archimedes agent implementation |
| `kernel/src/agents/prompts/` | Prompt library, evolution, academy |
| `kernel/src/gui/` | Graphics subsystem (desktop, console, fonts) |
| `kernel/src/vga_buffer.rs` | Text mode VGA |
| `tools/genesis_memory_persist.py` | Disk persistence system |
| `tools/genesis_vision_system.py` | Vision capabilities |
| `tools/qemu-run.sh` | Launch scripts |
| `tools/qemu-debug.sh` | Debug scripts |
| `tools/test-genesis.sh` | Test scripts |

**Process:**
1. Agent makes the change
2. Build verification (does it compile?)
3. Thomas runs validation if available
4. Stephen spot-checks in weekly review

**Why:** Risk is contained to the agent's own domain. If Thomas breaks his own
pattern detection, only Thomas suffers. Easy to notice, easy to fix.

### Tier 4: Playground (Just Do It)

**Rule: Full autonomy. Create, modify, delete freely. Manifest tracks provenance.**

This is the innovation space. New libraries, documentation, session logs,
experiments. Nothing here is critical infrastructure. If something doesn't
work, delete it and try again.

| Path | What It Is |
|------|-----------|
| `lib/` | Agent-generated libraries (vision, robot bridge) |
| `docs/` | Documentation (specs, guides, history) |
| `docs/history/` | Historical articles |
| `docs/specs/` | Specification drafts |
| `assets/` | Character artwork |
| `sessions/` | Session reflection logs |
| `memory/state/` | Runtime state files |
| `memory/checkpoints/` | Point-in-time snapshots |
| `memory/learnings.md` | Codified learning patterns |
| `tools/BRIDGE_README.md` | Bridge documentation |

**Why:** Innovation requires freedom. The manifest tracks who wrote what, so
there's always an audit trail. But there's no gate.

### Tier 5: Sandbox (Handle With Care)

**Rule: Secrets and credentials. Never commit, never log, never expose.**

This tier is different — it's not about planning, it's about safety. API keys
and credentials live here. The rule is simple: don't let them leak.

| Path | What It Is |
|------|-----------|
| `memory/secrets/` | API keys (Gemini, etc.) |
| `tools/.env.telegram` | Bot credentials |
| Any `*.secret`, `*.key` | Credential files |
| `sandbox/` (future) | Isolated experimental code |

**Why:** An agent that accidentally logs an API key to Telegram is a security
incident. The `.gitignore` protects against git commits. The tier makes it
explicit: don't read these, don't reference these, don't output these.

## How This Works In Practice

This is a **conversation-based governance system**, not an access control list.
Stephen doesn't write Rust. The agents do. The tiers define how much we talk
before the agents start typing.

```
Tier 1 (Core):       "Let's discuss this. What's the risk? OK, go ahead."
Tier 2 (Guarded):    "What are you changing and why? Makes sense, do it."
Tier 3 (Maintained): "Go ahead, I'll spot-check later."
Tier 4 (Playground): "Do whatever you want."
Tier 5 (Sandbox):    "Don't touch the secrets."
```

### The Core Change Checklist

When making a Tier 1 change, work through this:

1. **What** — What exactly is changing? (file, function, behavior)
2. **Why** — What problem does this solve or feature does it enable?
3. **Risk** — What's the worst case if this goes wrong?
4. **Blast radius** — What else depends on this code?
5. **Minimum change** — Is this the smallest change that solves the problem?
6. **Verification** — How do we know it works? (compile, test, QEMU boot)
7. **Rollback** — How do we undo it? (git revert hash)

### Autonomous Agent Governance (Future)

When agents operate via the bridge (Telegram → Claude/Gemini → tools → commit),
they should be tier-aware:

- Before modifying a file, check its tier
- Tier 1-2: Flag to Stephen via Telegram before proceeding
- Tier 3: Proceed, report in daily checkpoint
- Tier 4: Proceed silently
- Tier 5: Refuse the operation

## Future Considerations

1. **Automated tier checking in the bridge**: `genesis-bridge.py` could detect
   which files a proposed change touches and flag the tier before applying.

2. **Per-agent tier awareness**: As agents earn higher Academy certifications,
   they demonstrate more trustworthiness for higher-tier changes.

3. **Runtime enforcement**: The `protection.rs` module in the kernel provides
   tier lookups so agents can self-check before proposing changes.

4. **Audit log**: Every tier-1/2 change logged to memory store with the
   checklist answers for post-hoc review.

---

*"The core is the core. The question isn't whether agents can change it —
they have to, because I don't write Rust. The question is how much we think
about it first."*
*— Stephen, February 2026*
