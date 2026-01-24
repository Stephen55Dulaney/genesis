# Genesis Serial Bridge - Usage Guide

The Serial Bridge connects your Genesis OS (running in QEMU) to external AI services like Gemini, enabling agents to access cloud intelligence.

## Quick Start

### 1. Install Dependencies

```bash
pip install google-generativeai
```

### 2. Set Your Gemini API Key

```bash
export GEMINI_API_KEY="your-api-key-here"
```

### 3. Build Genesis Kernel

```bash
cd /Users/stephendulaney/genesis/kernel
cargo bootimage --target x86_64-unknown-none
```

### 4. Run the Bridge

```bash
cd /Users/stephendulaney/genesis/tools
python3 genesis-bridge.py
```

The bridge will:
- Start QEMU with Genesis
- Listen for agent requests
- Process video analysis (Scout)
- Send LLM responses back to Genesis

## Features

### Video Analysis (Scout)

In the Genesis shell, request video analysis:

```
genesis> scout video /Users/stephendulaney/Desktop/quantum-videos/2026-01-23\ 12-43-19.mp4
```

Scout will:
1. Send the request to the bridge
2. Bridge calls Gemini multimodal API
3. Gemini analyzes the video (transcript, insights, connections)
4. Results sent back to Genesis shell

### Agent LLM Requests

When agents need cloud intelligence, they send requests via serial:
- Thomas can request test analysis
- Archimedes can request conversation context
- Any agent can use the bridge for cloud processing

## Architecture

```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────┐
│   Genesis OS    │◄───────►│  Serial Bridge   │◄───────►│   Gemini    │
│   (QEMU)        │  Serial │   (Python)       │   HTTP   │     API     │
└─────────────────┘         └──────────────────┘         └─────────────┘
     │                              │
     │                              │
     ▼                              ▼
  Agents                      Video Processing
  (Thomas,                    (Multimodal)
   Scout, etc.)
```

## Example Session

```
$ python3 genesis-bridge.py
==================================================
  GENESIS AGENTIC BRIDGE INITIALIZING
==================================================

[✓] Gemini API ready

[*] Bridge active. Genesis is booting...
[*] Type commands in your terminal to interact with the Genesis shell.
[*] Scout can request video analysis with: scout video /path/to/video.mp4

| GENESIS | [BOOT] Serial port initialized
| GENESIS | [BOOT] Genesis Awakening...
...
genesis> breathe Today I want us to build the graphics system

| GENESIS | [SUPERVISOR] Setting living ambition (the soul)...
| GENESIS | [HEARTBEAT] Pulsing ambition DNA to all agents...

genesis> scout video /Users/stephendulaney/Desktop/quantum-videos/2026-01-23\ 12-43-19.mp4

[*] Scout requested video analysis: /Users/stephendulaney/Desktop/quantum-videos/2026-01-23 12-43-19.mp4
[*] Analyzing video...
[*] Sending analysis back to Genesis...

| GENESIS | Transcript: Conversation about Genesis OS architecture...
| GENESIS | Key Insights:
| GENESIS |   * Daily Ambition is the soul, Genesis is the body
| GENESIS |   * Agents need purpose DNA at birth
| GENESIS |   * Feedback loop enriches understanding
```

## Troubleshooting

**Bridge not receiving input:**
- Check that QEMU is using `-serial stdio`
- Verify stdin/stdout are properly piped

**Gemini API errors:**
- Verify `GEMINI_API_KEY` is set
- Check API quota/limits
- Bridge falls back to simulation mode if API unavailable

**"API_KEY_HTTP_REFERRER_BLOCKED" error:**
- Your Gemini API key has HTTP referrer restrictions enabled
- **Fix Option 1 (Recommended):** Create a new API key without restrictions:
  1. Go to https://aistudio.google.com/apikey
  2. Create a new API key
  3. Do NOT enable "HTTP referrer restrictions"
  4. Use the new key: `export GEMINI_API_KEY="new-key-here"`
- **Fix Option 2:** Configure your existing key:
  1. Go to https://aistudio.google.com/apikey
  2. Edit your API key
  3. Add `localhost` or `*` to allowed referrers
  4. Or disable referrer restrictions entirely

**Video analysis fails:**
- Verify video file path is correct
- Check file permissions
- Ensure video format is supported by Gemini

## Next Steps

- Add Ultravox integration for voice conversations
- Implement persistent context storage
- Add support for multiple LLM providers
- Create agent-specific prompt templates

