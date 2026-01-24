# Genesis OS - Test Plan

## Overview
This document outlines the test plan for Genesis OS, including expected outputs for each command and automated test procedures.

---

## Boot Sequence Tests

### Test 1: Boot Screen Display
**Command:** Boot Genesis (automatic)

**Expected Output:**
- QEMU window shows boot screen with:
  - ASCII art "GENESIS"
  - "AGENTIC OPERATING SYSTEM"
  - "Genesis Awakening..."
  - "QuantumDynamX.com"
  - "Where Agents, Classical, Quantum & Humans Collaborate"
- Terminal shows:
  ```
  [BOOT] Serial port initialized
  [BOOT] VGA buffer at 0xb8000
  [BOOT] Screen cleared
  [BOOT] Boot screen displayed
  [BOOT] Showing boot screen briefly...
  [BOOT] Continuing initialization...
  ```

**Pass Criteria:**
- ✅ Boot screen appears in QEMU window
- ✅ Boot screen displays for ~0.5 seconds
- ✅ Boot continues without freezing
- ✅ Serial output shows all boot messages

---

### Test 2: Graphics Initialization
**Command:** Boot Genesis (automatic)

**Expected Output:**
- Terminal shows:
  ```
  [GRAPHICS] Initializing graphics system...
  [GRAPHICS] Graphics system initialized
  [GRAPHICS] Double buffering enabled (or unavailable)
  [GRAPHICS] Test pattern drawn
  [GRAPHICS] Graphics ready
  ```

**Pass Criteria:**
- ✅ Graphics system initializes
- ✅ Double buffering attempts (may fail if heap full)
- ✅ No panic during graphics init

---

### Test 3: Agent System Initialization
**Command:** Boot Genesis (automatic)

**Expected Output:**
- Terminal shows:
  ```
  [SUPERVISOR] Initializing Agent Supervisor (Sam)...
  [SUPERVISOR] Loading Prompt Library...
  [SUPERVISOR] Starting Evolution Engine...
  [SUPERVISOR] Connecting to Agent Alliance Academy...
  [THOMAS] Creating Thomas the Tester...
  [THOMAS] Certification: 🟢 Rookie Thomas
  [SUPERVISOR] Agent registered: Thomas (ID: 1)
  ```

**Pass Criteria:**
- ✅ Supervisor initializes
- ✅ Prompt library loads
- ✅ Thomas agent created
- ✅ Agent registered successfully

---

## Shell Command Tests

### Test 4: `help` Command
**Command:** `help`

**Expected Output:**
```
Available commands:
  help      - Show this help message
  clear     - Clear the screen
  status    - Show agent status
  academy   - Show Academy certifications
  ping      - Ping all agents
  ambition  - Trigger morning ambitions
  report    - Trigger end-of-day report
  thomas    - Talk to Thomas specifically
  whoami    - Show current user info
  breathe [text] - Set the living ambition (the soul)
  heartbeat - View current ambition pulse
  insights  - View collected Sparks and Connections
  scout video [path] - Request video analysis (via bridge)
  test      - Trigger Thomas to run tests and send a Spark
  haiku     - Ask TypeWrite to generate a haiku (tests LLM connection)
  graphics  - Test graphics rendering (draw test pattern)
```

**Pass Criteria:**
- ✅ All commands listed
- ✅ Descriptions clear
- ✅ No errors

---

### Test 5: `status` Command
**Command:** `status`

**Expected Output:**
```
Agent Supervisor: [ ONLINE ]
Active Agents: [ 1 ]
Memory Tier System: [ ONLINE - Warm Tier ]
Tick: [current tick number]

Agent Status:
  AgentId(1) [Thomas]: Ready
```

**Pass Criteria:**
- ✅ Supervisor shows ONLINE
- ✅ At least 1 agent (Thomas) shown
- ✅ Agent status is Ready
- ✅ Tick counter increments

---

### Test 6: `ping` Command
**Command:** `ping`

**Expected Output:**
```
Pinging all agents...
(Responses will appear as agents process messages)
```

**Then after a few ticks:**
```
[THOMAS] Sent pong to AgentId(0)
```

**Pass Criteria:**
- ✅ Ping command executes
- ✅ Thomas responds with pong
- ✅ Pong message appears in serial output

---

### Test 7: `breathe` Command
**Command:** `breathe Today I want to build amazing things`

**Expected Output:**
```
[SUPERVISOR] Setting living ambition (the soul)...
[HEARTBEAT] Pulsing ambition DNA to all agents...
[THOMAS] Received heartbeat: "Today I want to build amazing things"
[THOMAS] Re-imprinted with new ambition DNA
```

**Pass Criteria:**
- ✅ Ambition is set
- ✅ Heartbeat message sent
- ✅ Thomas receives and processes heartbeat
- ✅ Ambition stored correctly

---

### Test 8: `heartbeat` Command
**Command:** `heartbeat`

**Expected Output (if ambition set):**
```
Current Living Ambition (the soul):
  "Today I want to build amazing things"

Heartbeat pulsing every ~100 ticks
```

**Expected Output (if no ambition):**
```
No living ambition set yet.
Use 'breathe [ambition]' to set the soul of Genesis.
```

**Pass Criteria:**
- ✅ Shows current ambition if set
- ✅ Shows helpful message if not set
- ✅ Formatting correct

---

### Test 9: `test` Command
**Command:** `test`

**Expected Output:**
```
Triggering Thomas to run tests...
Test request sent. Run 'insights' to see the Spark!
```

**Then check insights:**
```
genesis> insights
Constellation of Insights (1 total):

  [  1] ✨ SPARK
       Content: [test insight content]
       Context: System test result summary
```

**Pass Criteria:**
- ✅ Test command executes
- ✅ Thomas receives request
- ✅ Spark is generated
- ✅ Spark appears in insights

---

### Test 10: `insights` Command
**Command:** `insights`

**Expected Output (if insights exist):**
```
Constellation of Insights (X total):

  [  1] ✨ SPARK
       Content: [content]
       Context: [context]
  
  [  2] 🔗 CONNECTION
       From: [source]
       To: [destination]
       Pattern: [pattern]
```

**Expected Output (if no insights):**
```
No insights collected yet.
Agents will send Sparks and Connections as they work.
```

**Pass Criteria:**
- ✅ Shows all collected insights
- ✅ Correct formatting
- ✅ Categories displayed (SPARK, CONNECTION, RESOURCE, FEELING)
- ✅ Helpful message if empty

---

### Test 11: `haiku` Command (Requires Bridge)
**Command:** `haiku`

**Expected Output:**
```
Asking TypeWrite to generate a haiku...
(Sending request to Serial Bridge for Gemini processing)
[LLM_REQUEST] TypeWrite haiku request
```

**Then bridge responds:**
```
[*] TypeWrite requested a haiku from Gemini...
[*] Sending haiku back to Genesis...

TypeWrite says:

[haiku text here]
```

**Pass Criteria:**
- ✅ Request sent to bridge
- ✅ Bridge receives request
- ✅ Gemini generates haiku
- ✅ Haiku displayed correctly
- ✅ Multi-line formatting preserved

---

### Test 12: `graphics` Command
**Command:** `graphics`

**Expected Output:**
```
Drawing graphics test pattern...
Graphics test pattern drawn!
(Check QEMU display window to see graphics)
```

**Visual Check (QEMU Window):**
- Red rectangle (top-left corner)
- Green rectangle (top-right corner)
- Blue rectangle (bottom-left corner)
- Yellow rectangle (bottom-right corner)
- Cyan rectangle outline (center)
- Text: "GENESIS", "Graphics Mode Active", "Milestone 6: Graphics Foundation"

**Pass Criteria:**
- ✅ Command executes
- ✅ Test pattern drawn
- ✅ Colored rectangles visible
- ✅ Text visible
- ✅ No kernel panic

---

### Test 13: `clear` Command
**Command:** `clear`

**Expected Output:**
- VGA text screen clears
- Cursor returns to top-left
- Prompt reappears

**Pass Criteria:**
- ✅ Screen clears completely
- ✅ No artifacts left
- ✅ Prompt reappears

---

### Test 14: `whoami` Command
**Command:** `whoami`

**Expected Output:**
```
User: Stephen Dulaney
Role: Genesis Architect
Location: QuantumDynamX Lab
```

**Pass Criteria:**
- ✅ User info displayed
- ✅ Formatting correct

---

## Automated Test Script

### Manual Test Sequence
Run these commands in order and verify outputs:

```bash
# 1. Boot Genesis
cd /Users/stephendulaney/genesis/tools
python3 genesis-bridge.py

# 2. Wait for genesis> prompt, then run:
help
status
ping
breathe Today I want to test Genesis thoroughly
heartbeat
test
insights
whoami
graphics
clear
haiku
```

### Expected Test Results Summary

| Test | Command | Expected Result | Status |
|------|---------|----------------|--------|
| Boot | (automatic) | Boot screen + initialization | ⬜ |
| Help | `help` | All commands listed | ⬜ |
| Status | `status` | Agent status shown | ⬜ |
| Ping | `ping` | Pong response | ⬜ |
| Breathe | `breathe [text]` | Ambition set + heartbeat | ⬜ |
| Heartbeat | `heartbeat` | Current ambition shown | ⬜ |
| Test | `test` | Spark generated | ⬜ |
| Insights | `insights` | Insights displayed | ⬜ |
| Haiku | `haiku` | Haiku from Gemini | ⬜ |
| Graphics | `graphics` | Test pattern drawn | ⬜ |
| Clear | `clear` | Screen cleared | ⬜ |
| Whoami | `whoami` | User info shown | ⬜ |

---

## Regression Tests

### Test 15: Memory Leak Check
**Procedure:**
1. Run `test` command 100 times
2. Check `insights` - should show max 50 insights (oldest removed)
3. Monitor for kernel panic

**Pass Criteria:**
- ✅ No kernel panic
- ✅ Insights limited to 50
- ✅ Oldest insights removed when limit reached

---

### Test 16: Agent Heartbeat
**Procedure:**
1. Set ambition with `breathe`
2. Wait ~100 ticks (or run commands)
3. Check serial output for heartbeat messages

**Expected Output:**
```
[HEARTBEAT] Pulsing ambition DNA to all agents...
[THOMAS] Received heartbeat: "[ambition text]"
```

**Pass Criteria:**
- ✅ Heartbeat pulses every ~100 ticks
- ✅ All agents receive heartbeat
- ✅ Ambition DNA propagated correctly

---

### Test 17: Bridge Communication
**Procedure:**
1. Run `haiku` command
2. Verify bridge receives request
3. Verify response comes back
4. Verify formatting preserved

**Pass Criteria:**
- ✅ Request sent correctly
- ✅ Bridge processes request
- ✅ Response received
- ✅ Multi-line formatting works

---

## Performance Tests

### Test 18: Boot Time
**Measure:** Time from QEMU start to `genesis>` prompt

**Target:** < 5 seconds

**Pass Criteria:**
- ✅ Boot completes in reasonable time
- ✅ No excessive delays

---

### Test 19: Graphics Performance
**Measure:** Time to draw test pattern

**Target:** < 100ms

**Pass Criteria:**
- ✅ Graphics render quickly
- ✅ No visible lag

---

## Failure Scenarios

### Test 20: Invalid Command
**Command:** `invalid_command`

**Expected Output:**
```
Unknown command: invalid_command
Type 'help' for a list of commands.
```

**Pass Criteria:**
- ✅ Error message clear
- ✅ Helpful suggestion provided
- ✅ No kernel panic

---

### Test 21: Bridge Not Running
**Procedure:** Run `haiku` without bridge script

**Expected Output:**
- Request sent but no response
- No error (graceful degradation)

**Pass Criteria:**
- ✅ No kernel panic
- ✅ System continues normally

---

## Test Checklist

Use this checklist when testing:

- [ ] Boot sequence completes
- [ ] Boot screen displays correctly
- [ ] Graphics initialize
- [ ] Agents initialize
- [ ] Shell prompt appears
- [ ] All commands work
- [ ] LLM integration works (haiku)
- [ ] Graphics rendering works
- [ ] No kernel panics
- [ ] No memory leaks
- [ ] Performance acceptable

---

## Quick Test Command

Run this sequence to test everything quickly:

```
help
status
ping
breathe Testing Genesis OS
heartbeat
test
insights
graphics
haiku
```

---

*Last Updated: After Milestone 6 (Graphics Foundation)*
*Next Review: After Agent-First Boot implementation*


