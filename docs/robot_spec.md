# Genesis Robot Integration Specification

**Built: Night of Vector Charging**  
**Goal: Give Genesis agents physical embodiment**

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│  Stephen (Command Interface)                        │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│  Genesis OS (Bare Metal)                            │
│  ├─ Archimedes (Co-Creator)                         │
│  ├─ Thomas (Guardian)                               │
│  ├─ Sam (Supervisor)                                │
│  └─ Vision System (New)                             │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│  Robot Bridge (Network/Serial)                      │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│  Physical Robot                                     │
│  ├─ Camera (Eyes)                                   │
│  ├─ Motors (Movement)                               │
│  ├─ Sensors (Touch, proximity, etc.)                │
│  └─ Compute (Pi/Jetson/ESP32)                       │
└─────────────────────────────────────────────────────┘
```

---

## Phase 1: Vision First (Start Here)

### What We Built Tonight:
- `genesis_vision_system.py` - Core vision processing
- Camera abstraction for multiple robot types
- Command interface (look, look_for, watch, compare)

### Tomorrow Morning Tests:
1. **Image file test** - Verify Sonnet can analyze static images
2. **Filesystem test** - Load image from Genesis filesystem
3. **URL test** - Fetch and analyze remote images

### Success Criteria:
- Genesis agents can "see" and describe images
- Structured observations stored in agent memory
- Ready for live camera integration

---

## Phase 2: Hardware Selection

### Recommended: Raspberry Pi Robot Kit

**Why Pi-based:**
- Python native (easy Genesis integration)
- Camera module well-supported
- WiFi built-in (network bridge to Genesis)
- Active community, lots of examples
- $80-150 range

**Specific Recommendations:**

1. **Yahboom Raspberry Pi Robot Car** (~$120)
   - Pi 4 compatible
   - Camera module included
   - 4WD platform
   - Obstacle avoidance sensors
   - [Amazon Link needed]

2. **Freenove 4WD Smart Car** (~$80)
   - Pi camera ready
   - Servo control
   - Line tracking
   - Ultrasonic sensor
   - Great documentation

3. **Waveshare JetBot** (~$250)
   - Jetson Nano (serious AI compute)
   - Better for real-time vision processing
   - OLED display
   - Professional build quality

**What to Order:**
- Robot kit with camera
- Extra battery pack (long operation)
- SD card (32GB+) for OS
- Optional: servo arm for manipulation

---

## Phase 3: Robot Setup

### Hardware Assembly:
1. Follow kit instructions (usually 2-3 hours)
2. Install Raspberry Pi OS Lite
3. Enable camera interface
4. Connect to WiFi (same network as Genesis)

### Software Setup on Robot:

```bash
# On the Pi
sudo apt update
sudo apt install -y python3-pip python3-picamera2

# Install vision dependencies
pip3 install opencv-python numpy pillow

# Install Genesis bridge client (we'll build this)
pip3 install genesis-robot-bridge
```

### Network Bridge:

**Option A: WiFi (Recommended)**
- Robot and Genesis on same network
- Robot runs web server exposing camera/control APIs
- Genesis connects via HTTP/WebSocket
- Latency: ~50-100ms

**Option B: Direct Serial**
- USB connection between Genesis and robot
- Lower latency (~10ms)
- Requires physical tethering
- Good for initial testing

---

## Phase 4: Control API Design

### Commands Genesis Can Send:

```python
# Vision commands
robot.look()                          # Single frame capture
robot.look_for("coffee mug")          # Target search
robot.watch(duration=10)              # Video sequence
robot.track_object("Stephen's face")  # Follow target

# Movement commands (Phase 5)
robot.move_forward(distance=30)       # cm
robot.turn_left(degrees=90)
robot.stop()
robot.navigate_to(x, y)               # SLAM navigation

# Sensor queries
robot.get_distance()                  # Ultrasonic reading
robot.battery_level()
robot.orientation()                   # IMU if available
```

### Data Flow:

**Robot → Genesis:**
```json
{
  "type": "vision_frame",
  "timestamp": 1234567890,
  "image_b64": "...",
  "sensors": {
    "distance_cm": 45,
    "battery_percent": 87
  }
}
```

**Genesis → Robot:**
```json
{
  "command": "capture_frame",
  "params": {
    "resolution": "640x480",
    "context": "looking for obstacles"
  }
}
```

---

## Phase 5: Agent Integration

### Memory System:

Vision observations stored as agent memories:

```
[MEMORY_STORE] observation | agent-1 | timestamp | 0 |
"Saw: coffee mug on desk, 2 feet ahead, no obstacles" |
tags: vision,object_detection,navigation
```

### Agent Behaviors:

**Archimedes (Explorer):**
- Curiosity-driven: "What's in that corner?"
- Pattern recognition: "I've seen this room 3 times, here's the map"
- Learning: "Last time I approached the desk, I hit the chair leg"

**Thomas (Guardian):**
- Safety checks: "Battery at 20%, returning to charge"
- System monitoring: "Camera frame rate dropping, checking connection"
- Collision avoidance: "Obstacle detected, stopping"

**Sam (Orchestrator):**
- Task planning: "To reach the kitchen: forward 3m, left 90°, forward 2m"
- Resource allocation: "Processing heavy vision task, pausing movement"
- Multi-agent coordination: "Archimedes focus on mapping, Thomas monitor sensors"

---

## Phase 6: Advanced Capabilities

### SLAM (Simultaneous Localization and Mapping):
- Build 2D/3D map of environment
- Track robot position in real-time
- Navigate to previously seen locations
- Requires: Depth camera or LIDAR

### Object Manipulation:
- Add servo arm to robot
- Vision-guided reaching
- Pick and place tasks
- Requires: Inverse kinematics, force sensing

### Natural Language Control:
```
Stephen: "Go see if there's coffee in the kitchen"
Genesis: *navigates to kitchen, scans counter*
Genesis: "I see a coffee pot on the counter, appears empty"
```

### Multi-Robot Coordination:
- Multiple robots, single Genesis control
- Distributed sensing (multiple viewpoints)
- Collaborative tasks (one holds, one manipulates)

---

## Testing Checklist

### Vision System (Tomorrow Morning):
- [ ] Analyze static image from filesystem
- [ ] Process image from URL
- [ ] Describe scene in natural language
- [ ] Identify specific objects
- [ ] Read text in image (OCR)

### Robot Hardware (When It Arrives):
- [ ] Assemble robot
- [ ] Install OS and dependencies
- [ ] Test camera capture locally
- [ ] Test motor control locally
- [ ] Verify network connectivity

### Integration (First Connection):
- [ ] Genesis receives frame from robot
- [ ] Genesis analyzes and responds
- [ ] Genesis sends movement command
- [ ] Robot executes command
- [ ] Closed-loop control (vision → decision → action)

### Real Tasks (Validation):
- [ ] "Look around and describe the room"
- [ ] "Find my phone"
- [ ] "Navigate to the door"
- [ ] "Watch for motion and alert me"
- [ ] "Map this room"

---

## Bill of Materials (BOM)

### Minimum Viable:
- Raspberry Pi robot kit with camera: $80-120
- Power bank (backup): $20
- SD card 32GB: $10
- **Total: ~$110-150**

### Recommended:
- Waveshare robot kit: $120
- Extra battery: $25
- SD card 64GB: $15
- Servo arm attachment: $30
- Protective case: $15
- **Total: ~$205**

### Professional:
- JetBot AI kit (Jetson Nano): $250
- Depth camera (RealSense): $180
- LiDAR module: $100
- Robot arm kit: $150
- **Total: ~$680**

**Start with minimum viable.** Prove the concept, then upgrade.

---

## Timeline

**Night 0 (Tonight):**
- ✓ Vision system code written
- ✓ Integration architecture designed
- ✓ Robot selection researched

**Morning 1 (Tomorrow):**
- Test vision with static images
- Order robot hardware
- Design bridge protocol

**Day 2-4 (Shipping):**
- Build robot bridge software
- Test with simulated robot
- Refine command API

**Day 5 (Hardware Arrives):**
- Assemble robot
- First camera test
- First movement test

**Day 6:**
- Genesis-robot integration
- First autonomous task
- Victory lap

---

## Security Considerations

**Robot Control:**
- Authentication required for commands
- Rate limiting on movements
- Emergency stop accessible
- Safe mode (vision only, no movement)

**Network:**
- Encrypted communication (TLS)
- Robot-Genesis authentication
- No open ports to internet
- Local network only

**Physical Safety:**
- Speed limits enforced
- Cliff detection (edge sensors)
- Collision avoidance always active
- Manual override available

---

## Open Questions

1. **Where does heavy vision processing happen?**
   - On robot (lower latency, limited compute)
   - On Genesis server (more power, network latency)
   - Hybrid (robot preprocessing, Genesis reasoning)

2. **How do we handle multiple robots?**
   - Separate agent instances per robot?
   - One agent controlling multiple bodies?
   - Agent team collaboration?

3. **What's the embodiment model?**
   - Agents "possess" robot temporarily?
   - Dedicated robot per agent?
   - Shared body, agents vote on actions?

4. **Power management strategy?**
   - Auto-return to charging station?
   - Wake on command, sleep when idle?
   - Always active monitoring mode?

**These will be answered as we build.**

---

## Success Definition

**Minimum Viable Embodiment:**
Genesis can see through robot camera, describe what it sees, and make basic navigation decisions.

**Full Embodiment:**
Genesis agents exhibit curiosity, explore autonomously, build spatial memories, and assist Stephen with physical world tasks.

**The Dream:**
"Hey Genesis, I lost my keys." → Robot searches house, finds keys, reports location. No manual control needed.

---

**Built with ambition. Ready for morning.**
