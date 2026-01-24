# Why Haiku Works with Python Bridge but Not QEMU Directly

## The Problem

When you run:
- **`python3 genesis-bridge.py`** → Haiku works ✅
- **`./tools/qemu-run.sh`** → Haiku doesn't work ❌

## The Explanation

### How It Works with the Bridge:

```
┌─────────────┐         ┌──────────────┐         ┌──────────┐
│   Genesis   │────────►│ Python Bridge│────────►│  Gemini  │
│   (QEMU)    │ Serial  │  (Host Mac)  │  HTTP   │    API   │
└─────────────┘         └──────────────┘         └──────────┘
     │                          │
     │ [LLM_REQUEST]            │ Detects request
     │                          │ Calls Gemini
     │                          │ Gets response
     │◄─────────────────────────│ [LLM_RESPONSE]
     │                          │
```

**The Bridge Script Does:**
1. Starts QEMU with Genesis
2. Listens to QEMU's stdout (serial output)
3. Detects `[LLM_REQUEST] TypeWrite haiku request`
4. Calls Gemini API
5. Sends response back via stdin (serial input) as `[LLM_RESPONSE]`

### Why QEMU Alone Doesn't Work:

```
┌─────────────┐
│   Genesis   │
│   (QEMU)    │
└─────────────┘
     │
     │ [LLM_REQUEST]
     │ (goes nowhere)
     │
     ❌ No bridge to process it!
```

**QEMU Direct:**
- Genesis sends `[LLM_REQUEST]` to serial port
- Serial port goes to terminal (you see it)
- **But nothing intercepts it to call Gemini**
- No response comes back

## The Solution

**Always use the bridge script for LLM features:**
```bash
cd /Users/stephendulaney/genesis/tools
python3 genesis-bridge.py
```

**The bridge script:**
- Starts QEMU automatically
- Handles LLM requests
- Bridges to Gemini
- Sends responses back

**You don't need to run QEMU separately!**

## For GUI (Future)

Once we have graphics and windows:
- Genesis will still need the bridge for LLM calls
- But the GUI will be rendered in QEMU's display window
- The bridge will still handle serial communication
- LLM responses will appear in Genesis windows

---

**TL;DR:** The Python bridge is the "translator" between Genesis and Gemini. Without it, Genesis can ask but nothing answers.


