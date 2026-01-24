# Agent-First Boot Sequence - Technical Design

## Philosophy: Agents Before GUI

In traditional operating systems:
1. Boot → Hardware init
2. Load kernel
3. Start services
4. Show GUI
5. User opens apps
6. Apps see files

**In Genesis:**
1. Boot → Hardware init
2. Load kernel
3. **Agents wake up FIRST**
4. **Agents organize environment**
5. **Agents prepare workspace**
6. **GUI appears, already organized**

---

## Boot Sequence Phases

### Phase 1: Kernel Initialization (Current)
```
[BOOT] Hardware init
[BOOT] Memory management
[BOOT] Interrupts
[BOOT] Drivers (keyboard, serial, VGA)
```

### Phase 2: Agent Awakening (Next)
```
[AGENTS] Supervisor initializes
[AGENTS] Load agent registry from persistent storage
[AGENTS] Parallel agent boot:
  ├─ Archimedes: Load daily ambition
  ├─ Sentinel: Security scan
  ├─ Scout: Resource discovery
  ├─ TypeWrite: Workspace prep
  └─ Thomas: Health check
[AGENTS] Agents report readiness
```

### Phase 3: Environment Setup (Before GUI)
```
[SETUP] Agents analyze current state
[SETUP] Agents organize files/folders
[SETUP] Agents create workspace layouts
[SETUP] Agents prepare relevant resources
[SETUP] Agents set up agent "inboxes"
[SETUP] Desktop layout computed
```

### Phase 4: GUI Rendering (Final)
```
[GUI] Graphics mode initialized
[GUI] Desktop rendered from agent layout
[GUI] Agent status displays appear
[GUI] User sees organized environment
[GUI] Ready for interaction
```

---

## Technical Implementation

### 1. Agent Boot Orchestration

**Location:** `kernel/src/agents/supervisor.rs`

**New Methods:**
```rust
impl Supervisor {
    /// Boot sequence: Wake all agents before GUI
    pub fn agent_boot_sequence(&mut self) -> BootResult {
        // Phase 1: Load agent registry
        let agents = self.load_agent_registry();
        
        // Phase 2: Parallel agent initialization
        for agent_config in agents {
            let agent = self.create_agent(agent_config);
            agent.awaken(); // Agent-specific boot logic
            self.register(agent);
        }
        
        // Phase 3: Wait for all agents ready
        self.wait_for_agents_ready();
        
        // Phase 4: Agents begin environment setup
        self.trigger_environment_setup();
        
        BootResult::Success
    }
    
    /// Agents organize the environment
    fn trigger_environment_setup(&mut self) {
        // Broadcast: "Organize your domain"
        self.broadcast(MessageKind::SystemEvent(
            SystemEvent::EnvironmentSetup
        ));
    }
}
```

### 2. Agent Environment Setup Actions

**Each agent implements:**
```rust
impl Agent for Archimedes {
    fn handle_environment_setup(&mut self) {
        // 1. Load daily ambition from storage
        let ambition = self.load_daily_ambition();
        
        // 2. Create workspace folders
        self.create_workspace_folders(&ambition);
        
        // 3. Load relevant documents
        self.load_relevant_documents(&ambition);
        
        // 4. Prepare desktop layout
        self.prepare_desktop_layout(&ambition);
        
        // 5. Report completion
        self.send_feedback(FeedbackType::Resource {
            description: "Workspace prepared",
            location: "/workspaces/today"
        });
    }
}
```

### 3. Desktop Layout System

**New Module:** `kernel/src/gui/desktop.rs`

```rust
pub struct DesktopLayout {
    /// Desktop elements (icons, folders, widgets)
    elements: Vec<DesktopElement>,
    /// Agent-defined zones
    zones: Vec<Zone>,
    /// Focus area (from daily ambition)
    focus_area: Option<FocusArea>,
}

pub enum DesktopElement {
    /// File/folder icon
    FileIcon { path: String, position: Point },
    /// Agent inbox widget
    AgentInbox { agent_id: AgentId, position: Point },
    /// Status widget
    StatusWidget { content: String, position: Point },
    /// Workspace folder
    WorkspaceFolder { name: String, position: Point },
}

pub struct Zone {
    name: String,
    purpose: ZonePurpose,
    elements: Vec<DesktopElement>,
    agent_owner: Option<AgentId>,
}

pub enum ZonePurpose {
    Focus,      // Today's work
    Resources,  // Scout's research
    Writing,    // TypeWrite's space
    Security,   // Sentinel's area
    Testing,    // Thomas's ground
}
```

### 4. Graphics Foundation

**New Module:** `kernel/src/gui/graphics.rs`

```rust
pub struct GraphicsContext {
    framebuffer: *mut u8,
    width: u32,
    height: u32,
    pitch: u32,
}

impl GraphicsContext {
    /// Initialize VGA graphics mode
    pub fn init_graphics_mode() -> Self;
    
    /// Draw pixel
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color);
    
    /// Draw rectangle
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color);
    
    /// Draw text
    pub fn draw_text(&mut self, x: u32, y: u32, text: &str, color: Color);
    
    /// Swap buffers (double buffering)
    pub fn swap_buffers(&mut self);
}
```

### 5. Persistent Storage

**New Module:** `kernel/src/storage/filesystem.rs`

```rust
pub struct FileSystem {
    // Simple file system for agent data
}

impl FileSystem {
    /// Read file
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>>;
    
    /// Write file
    pub fn write_file(&mut self, path: &str, data: &[u8]) -> Result<()>;
    
    /// List directory
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>>;
    
    /// Create directory
    pub fn create_dir(&mut self, path: &str) -> Result<()>;
}
```

**Agent Data Structure:**
```
/storage/
  /agents/
    /archimedes/
      daily_ambitions/
        2026-01-23.txt
      workspace_layouts/
        default.json
    /scout/
      resources/
        articles/
        videos/
    /sentinel/
      security_logs/
```

---

## Agent-Specific Boot Actions

### Archimedes (Daily Ambition Agent)
1. **Load Ambition:**
   - Read `/storage/agents/archimedes/daily_ambitions/today.txt`
   - If missing, create default ambition
   - Set as `living_ambition` in Supervisor

2. **Create Workspace:**
   - `/workspaces/today/` - Main workspace
   - `/workspaces/today/focus/` - Priority items
   - `/workspaces/today/resources/` - Supporting materials
   - `/workspaces/today/output/` - Today's creations

3. **Prepare Desktop:**
   - Focus zone with ambition text
   - Quick access to workspace folders
   - Ambition progress indicator

### Scout (Research Agent)
1. **Scan Resources:**
   - Check `/storage/downloads/` for new files
   - Organize by topic/project
   - Create learning playlists

2. **Prepare Research Zone:**
   - Desktop area for research materials
   - Quick links to relevant resources
   - "Learn More" suggestions

### Sentinel (Security Agent)
1. **Security Audit:**
   - Scan system state
   - Check for anomalies
   - Prepare security dashboard

2. **Organize by Security:**
   - Safe zones (trusted files)
   - Sandbox zones (untrusted)
   - Privacy controls

### TypeWrite (Content Agent)
1. **Prepare Writing Space:**
   - Templates ready
   - Documents organized by project
   - Collaboration spaces

2. **Desktop Writing Zone:**
   - Quick notes area
   - Document templates
   - Writing tools

### Thomas (Guardian/Tester)
1. **Health Check:**
   - System performance
   - Memory usage
   - Agent status

2. **Testing Ground:**
   - Development tools
   - Debug console
   - Monitoring dashboard

---

## Boot Sequence Code Flow

```rust
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Phase 1: Kernel init (existing)
    serial::init();
    memory::init(...);
    interrupts::init();
    
    // Phase 2: Agent awakening (NEW)
    let mut supervisor = Supervisor::new();
    supervisor.agent_boot_sequence();
    
    // Phase 3: Environment setup (NEW)
    supervisor.trigger_environment_setup();
    supervisor.wait_for_setup_complete();
    
    // Phase 4: Graphics init (NEW)
    let mut graphics = GraphicsContext::init_graphics_mode();
    
    // Phase 5: Render desktop from agent layout (NEW)
    let desktop_layout = supervisor.get_desktop_layout();
    render_desktop(&mut graphics, &desktop_layout);
    
    // Phase 6: Main loop (existing + GUI)
    loop {
        // Handle input
        handle_input(&mut supervisor);
        
        // Update GUI
        update_gui(&mut graphics, &supervisor);
        
        // Agent ticks
        supervisor.tick();
    }
}
```

---

## Key Design Decisions

### 1. Agents Boot in Parallel
- Not sequential
- Each agent has independent boot logic
- Supervisor coordinates, doesn't micromanage

### 2. Environment Setup Before GUI
- Agents organize files/folders
- Desktop layout computed
- GUI just renders what agents prepared

### 3. Desktop Layout as Agent Output
- Agents define their zones
- Supervisor combines into layout
- GUI renders layout

### 4. Persistent Storage Required
- Agents need to remember state
- Ambitions persist across boots
- File organization persists

### 5. Graphics Mode Before GUI
- Need graphics primitives first
- Then window manager
- Then desktop rendering

---

## Implementation Order

1. **Graphics Foundation** (Week 1)
   - VGA graphics mode
   - Basic drawing functions
   - Test rendering

2. **Persistent Storage** (Week 1-2)
   - Simple file system
   - Agent data storage
   - Read/write files

3. **Agent Boot Sequence** (Week 2)
   - Agent awakening logic
   - Parallel boot
   - Environment setup triggers

4. **Desktop Layout System** (Week 2-3)
   - Layout data structures
   - Agent zone definitions
   - Layout rendering

5. **GUI Integration** (Week 3)
   - Combine graphics + layout
   - Render desktop
   - Test agent-first boot

---

## Success Criteria

✅ **Agent-First Boot Works When:**
- Agents boot before GUI appears
- Agents organize files/folders
- Desktop shows organized layout on first render
- User sees purpose-driven environment immediately

✅ **Graphics Foundation Works When:**
- Can draw pixels, shapes, text
- Smooth updates (double buffering)
- Agents can render their zones

✅ **Desktop Layout Works When:**
- Agents define zones
- Layout combines agent zones
- GUI renders layout correctly

---

*This is the QuantumDynamX way: Agents first, humans see the result.*


