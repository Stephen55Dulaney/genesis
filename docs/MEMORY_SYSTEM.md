# Genesis Memory Persistence System

## 🧠 Overview

Genesis uses a **two-layer memory architecture** to ensure data survives both reboots and hardware failures.

### Layer 1: Local Disk (Reboot Protection)
- **Location:** `~/genesis/memory/`
- **Purpose:** Survive system reboots
- **Speed:** Instant read/write
- **Scope:** Single machine

### Layer 2: GitHub (Disaster Recovery)
- **Location:** GitHub repository
- **Purpose:** Survive hardware failure, enable rollback
- **Speed:** Async backup (5-minute intervals)
- **Scope:** Cloud-backed, version controlled

---

## 📁 Directory Structure

```
~/genesis/
├── memory/
│   ├── state/              # Persistent state (backed up to GitHub)
│   │   ├── daily_ambition.json
│   │   ├── api_key_*.json  (gitignored)
│   │   └── event_*.json
│   ├── checkpoints/        # Point-in-time snapshots
│   │   └── 20260219_065610_memory_system_live/
│   │       ├── manifest.json
│   │       └── *.json (copied state files)
│   ├── secrets/            # Encrypted data (gitignored)
│   │   └── api_key_*.json
│   └── .gitignore          # Protects secrets
├── lib/                    # Persistent libraries
│   ├── vision.py
│   └── robot_bridge.py
└── tools/
    └── genesis_memory_persist.py
```

---

## 🔧 API Usage

### Python API

```python
from tools.genesis_memory_persist import GenesisState

state = GenesisState()

# Store ambition
state.set_ambition("Build something amazing")

# Store API keys (encrypted, gitignored)
state.set_api_key("gemini", "your-key-here")

# Retrieve data
ambition = state.get_ambition()
api_key = state.get_api_key("gemini")

# Log important events
state.log_event("system_boot", {
    "timestamp": "2025-02-19",
    "version": "0.1.0"
})
```

### Command Line

```bash
# Create checkpoint
python3 tools/genesis_memory_persist.py checkpoint "before_major_change"

# List checkpoints
python3 tools/genesis_memory_persist.py list

# Restore from checkpoint
python3 tools/genesis_memory_persist.py restore 20260219_065610_memory_system_live

# Manual GitHub backup
python3 tools/genesis_memory_persist.py backup "Custom message"

# Auto-backup daemon (every 5 minutes)
python3 tools/genesis_memory_persist.py auto-backup
```

---

## 🔐 Security

### What's Backed Up to GitHub
- ✅ Daily ambitions
- ✅ System events
- ✅ Agent memory
- ✅ Configuration (non-secret)
- ✅ Checkpoints

### What's NOT Backed Up (Gitignored)
- ❌ API keys (`api_key_*.json`)
- ❌ Secrets directory
- ❌ `.secret` and `.key` files

**API keys are stored locally only.** If you lose the machine, you'll need to re-enter them.

---

## 🚀 Recovery Scenarios

### Scenario 1: System Reboot
**Problem:** Genesis restarts, loses RAM state.

**Solution:** Automatic
```python
# On startup, agents automatically load from memory/state/
state = GenesisState()
ambition = state.get_ambition()  # Instantly restored
```

### Scenario 2: Code Crash
**Problem:** Bug causes system failure mid-session.

**Solution:** Restore from checkpoint
```bash
# List recent checkpoints
python3 tools/genesis_memory_persist.py list

# Restore to before crash
python3 tools/genesis_memory_persist.py restore 20260219_065610_memory_system_live
```

### Scenario 3: Hardware Failure
**Problem:** Disk dies, machine destroyed.

**Solution:** Clone from GitHub
```bash
# On new machine
git clone git@github.com:yourusername/genesis.git
cd genesis

# All memory backed up to GitHub is restored
ls memory/state/
```

### Scenario 4: Accidental Data Loss
**Problem:** Deleted important state file.

**Solution:** Git rollback
```bash
# See what changed
git log memory/

# Restore from specific commit
git checkout <commit-hash> memory/state/
```

---

## ⚡ Automatic Backup

Genesis can run continuous backup in the background:

```bash
# Start auto-backup daemon (runs every 5 minutes)
nohup python3 tools/genesis_memory_persist.py auto-backup > /tmp/genesis_backup.log 2>&1 &
```

This will:
1. Monitor `memory/state/` for changes
2. Create git commit when changes detected
3. Push to GitHub automatically
4. Skip backup if nothing changed

---

## 🧪 Testing

### Test Local Persistence
```python
from tools.genesis_memory_persist import GenesisState

state = GenesisState()
state.set_ambition("Test ambition")

# Simulate reboot by creating new instance
state2 = GenesisState()
assert state2.get_ambition() == "Test ambition"
print("✅ Local persistence works")
```

### Test Checkpoint/Restore
```bash
# Create checkpoint
python3 tools/genesis_memory_persist.py checkpoint "test_checkpoint"

# Modify state
python3 -c "from tools.genesis_memory_persist import GenesisState; GenesisState().set_ambition('Changed')"

# Restore
python3 tools/genesis_memory_persist.py restore <checkpoint_id>

# Verify restored
python3 -c "from tools.genesis_memory_persist import GenesisState; print(GenesisState().get_ambition())"
```

### Test GitHub Backup
```bash
# Make a change
python3 -c "from tools.genesis_memory_persist import GenesisState; GenesisState().set_ambition('Backup test')"

# Backup to GitHub
python3 tools/genesis_memory_persist.py backup "Test backup"

# Verify on GitHub
git log -1 memory/
```

---

## 📊 Current Status

**Initialized:** 2025-02-19 06:56  
**First Checkpoint:** `20260219_065610_memory_system_live`  
**GitHub Backup:** ✅ Active  

**Stored State:**
- Daily ambition: "Build Genesis DevKit v0.1 — the agent-focused robotics platform"
- Gemini API key: Configured (encrypted)
- DevKit initialization event: Logged

---

## 🔮 Future Enhancements

1. **Encrypted GitHub backups** — Encrypt secrets before pushing
2. **Multi-machine sync** — Share state across devices
3. **Automatic pruning** — Remove old checkpoints after N days
4. **Compression** — Reduce storage for large states
5. **Remote backup locations** — S3, Dropbox, etc.

---

## ⚠️ Important Notes

1. **API keys are local only** — Re-enter after hardware failure
2. **Git must be configured** — Ensure `git push` works
3. **Checkpoints are local** — Not pushed to GitHub (too large)
4. **Auto-backup requires daemon** — Must be running for continuous backup

---

**Genesis Memory System is now operational.** Your agents will never forget. 🧠✨
