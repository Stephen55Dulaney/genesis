# Bevy and Graphics Foundation for Genesis

## About Bevy

**Bevy** (https://bevyengine.org) is a modern, data-driven game engine built in Rust. It uses:
- **ECS (Entity-Component-System)** architecture
- **Massively parallel** execution
- **2D and 3D rendering**
- **Simple, ergonomic API**

### Why We Can't Use Bevy Directly

**Bevy requires `std` (standard library):**
- Genesis kernel is `no_std` (bare-metal)
- Bevy is designed for user-space applications
- Bevy needs OS services (threading, file I/O, etc.)

**But we can learn from Bevy's architecture!**

## What We Can Learn from Bevy

### 1. ECS Architecture
Bevy's Entity-Component-System is perfect for agents:
- **Entities** = Agents (Thomas, Scout, etc.)
- **Components** = Agent properties (state, position, graphics)
- **Systems** = Agent behaviors (tick, render, interact)

**For Genesis:**
```rust
// Agent as Entity
struct AgentEntity {
    id: AgentId,
    position: Point,      // Component
    graphics: Sprite,     // Component
    state: AgentState,    // Component
}

// System for rendering
fn render_agents(agents: Query<&AgentEntity>) {
    for agent in agents.iter() {
        draw_sprite(agent.position, agent.graphics);
    }
}
```

### 2. Parallel Execution
Bevy runs systems in parallel when possible. For Genesis:
- Agents can tick in parallel
- Graphics rendering can be parallel
- Message routing can be parallel

### 3. Simple API
Bevy's API is clean and ergonomic. We can adopt similar patterns:
```rust
// Bevy-style (what we'll build)
graphics.draw_rect(x, y, w, h, color);
graphics.draw_text(x, y, "Hello", font);

// Instead of complex VGA register manipulation
```

## Our Graphics Foundation (Milestone 6)

### Phase 1: VGA Graphics Mode

**Current:** VGA Text Mode (80x25 characters)
**Target:** VGA Graphics Mode (320x200 or 640x480 pixels)

**Why Graphics Mode:**
- Pixel-level control
- Can draw shapes, images, windows
- Foundation for GUI

### Phase 2: Drawing Primitives

**Basic Functions:**
```rust
pub struct GraphicsContext {
    framebuffer: *mut u8,
    width: u32,
    height: u32,
}

impl GraphicsContext {
    fn draw_pixel(&mut self, x: u32, y: u32, color: Color);
    fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color);
    fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: Color);
    fn draw_text(&mut self, x: u32, y: u32, text: &str, font: &Font);
    fn clear(&mut self, color: Color);
    fn swap_buffers(&mut self); // Double buffering
}
```

### Phase 3: Agent Graphics Integration

**Agents can render:**
- Their status indicators
- Their "zones" on desktop
- Their inbox windows
- Their activity visualizations

## Implementation Plan

### Step 1: Switch to Graphics Mode
- Initialize VGA graphics mode (Mode 13h: 320x200x256)
- Set up framebuffer
- Test pixel drawing

### Step 2: Basic Drawing
- Implement pixel drawing
- Implement rectangle drawing
- Implement text rendering (bitmap font)

### Step 3: Double Buffering
- Create back buffer
- Draw to back buffer
- Swap to front buffer
- Smooth updates

### Step 4: Agent Integration
- Agents can request graphics operations
- Render agent status
- Render agent zones
- Test with simple desktop

## Why Not Use Bevy Directly?

1. **`no_std` requirement** - Genesis kernel can't use std
2. **Bare-metal constraints** - No OS services available
3. **Size constraints** - Bevy is large, we need minimal
4. **Control** - We need kernel-level control

## What We'll Build Instead

**A minimal, agent-friendly graphics system:**
- Inspired by Bevy's simplicity
- Adapted for `no_std`
- Optimized for agent rendering
- Lightweight and fast

**Think of it as "Bevy-inspired" but built for bare-metal.**

## Next Steps

1. **Implement VGA graphics mode** (Mode 13h)
2. **Create GraphicsContext** (like Bevy's rendering context)
3. **Implement drawing primitives** (pixel, rect, text)
4. **Add double buffering** (smooth updates)
5. **Integrate with agents** (agents can render)

---

**TL;DR:** Bevy is great but requires std. We'll build a Bevy-inspired graphics system for bare-metal, optimized for agents.


