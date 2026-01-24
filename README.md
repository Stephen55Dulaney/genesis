# 🧬 Genesis

**An agentic operating system built from scratch in Rust.**

Agents wake before the GUI, organize your desktop around daily ambitions, and collaborate with humans at the kernel level.

Part of the [QuantumDynamX](https://quantumdynamx.com) vision: *Where Agents, Classical, Quantum & Humans Collaborate Together.*

---

## 🌟 Vision

Traditional OS: Apps are tools you launch after boot.

**Genesis**: Agents are citizens of the kernel. They wake first. They organize your world. They ask: *"What do WE want to accomplish today?"*

---

## ✨ Features

### Agent-First Boot Sequence
- Agents wake during boot, **before** the GUI appears
- Agents organize your workspace around your daily ambition
- Desktop renders the organized environment

### Daily Ambition System (Archimedes)
- **Voice Archimedes**: Conversational partner for morning planning
- **Silent Archimedes**: Generates structured ambition documents
- Split-screen layout: Conversation (left) + Ambition Statement (right)

### Agent Framework
- **Supervisor (Sam)**: Orchestrates all agents
- **Thomas (Guardian)**: Tests and monitors system integrity
- **Archimedes (Co-Creator)**: Daily ambition and workspace organization

### Living Ambition Heartbeat
- The daily ambition pulses through the system like a heartbeat
- Agents receive the "ambition DNA" and align their work
- Feedback loop: Sparks, Connections, Resources, Feelings

### Prompt Library & Academy
- Agent prompts managed centrally
- DSPy-style prompt evolution
- Agent Alliance Academy certifications

---

## 🛠 Technical Stack

- **Language**: Rust (no_std, bare-metal)
- **Target**: x86_64, runs in QEMU
- **Graphics**: VGA Mode 13h (320x200, 256 colors)
- **LLM Bridge**: Python serial bridge to Gemini API

---

## 🚀 Quick Start

### Prerequisites
- Rust nightly toolchain
- QEMU (x86_64)
- Python 3 (for LLM bridge)

### Build & Run

```bash
# Build the kernel
cargo build

# Create bootable image
cargo bootimage

# Run with LLM bridge (recommended)
python tools/genesis-bridge.py

# Or run standalone
./tools/qemu-run.sh
```

### Shell Commands

```
genesis> help          # Show all commands
genesis> archimedes    # Talk to Archimedes
genesis> ambition      # Trigger morning ambitions
genesis> desktop       # Show split-screen desktop
genesis> heartbeat     # View current ambition pulse
genesis> insights      # View collected Sparks
```

---

## 📁 Project Structure

```
genesis/
├── kernel/
│   ├── src/
│   │   ├── main.rs           # Kernel entry point
│   │   ├── agents/           # Agent framework
│   │   │   ├── supervisor.rs # Agent orchestration
│   │   │   ├── thomas.rs     # Guardian agent
│   │   │   ├── archimedes.rs # Daily Ambition agent
│   │   │   └── prompts/      # Prompt library
│   │   ├── gui/              # Graphics system
│   │   │   ├── graphics.rs   # VGA Mode 13h driver
│   │   │   └── desktop.rs    # Desktop layout
│   │   ├── shell.rs          # Interactive shell
│   │   └── storage/          # File system
│   └── Cargo.toml
├── tools/
│   ├── genesis-bridge.py     # LLM bridge script
│   └── qemu-run.sh           # QEMU launcher
└── docs/                     # Documentation
```

---

## 🎯 Roadmap

- [x] Bare-metal Rust kernel
- [x] VGA text mode
- [x] Serial debugging
- [x] Keyboard input
- [x] Memory management & heap
- [x] Agent framework
- [x] Prompt library
- [x] Interactive shell
- [x] Graphics foundation (Mode 13h)
- [x] Agent-First Boot sequence
- [x] Daily Ambition integration
- [x] Split-screen desktop layout
- [ ] VGA mode switching (text ↔ graphics)
- [ ] Window manager
- [ ] Voice integration (Ultravox)
- [ ] Quantum computing integration (IBM Qiskit)

---

## 🤝 Part of QuantumDynamX

Genesis is part of the **QuantumDynamX** ecosystem:

- **[Agent Alliance Academy](https://as-the-cloud-turns-web.onrender.com/#academy)** - Where agents learn and earn certifications
- **As the Cloud Turns** - The story of our agents
- **Watson Wheeler Institute** - Research hub

---

## 📜 License

MIT OR Apache-2.0

---

*"What do WE want to accomplish today?"* — Archimedes 🦉

