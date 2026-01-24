# Genesis Interaction Model

## Current State

Genesis now supports **dual-mode operation**: you can switch between **text mode** and **graphics mode** at runtime.

### Two Display Modes

1. **Text Mode (Mode 3)**
   - 80x25 character display
   - VGA text buffer at `0xB8000`
   - Used for shell commands and terminal output
   - Default mode for QEMU boot

2. **Graphics Mode (Mode 13h)**
   - 320x200 pixel display, 256 colors
   - Framebuffer at `0xA0000`
   - Used for desktop layout and visual elements
   - Shows agent-organized desktop (split-screen: Conversation + Ambition)

### Current Interaction Methods

#### 1. **Keyboard Shortcuts**
Press **F1** or **Escape** at any time to toggle between text and graphics modes.

- **F1**: Standard function key (on Mac: hold `fn` + `F1`)
- **Escape**: Mac-friendly alternative (easier to access on Mac keyboards)
- **In Text Mode**: Press F1/Esc → switches to Graphics Mode
- **In Graphics Mode**: Press F1/Esc → switches to Text Mode

#### 2. **Shell Command: `mode`**
Use the shell command to switch modes programmatically:

```bash
genesis> mode              # Show current mode
genesis> mode text          # Switch to text mode
genesis> mode graphics      # Switch to graphics mode (or 'mode gfx')
```

### How It Works

1. **Text Mode**: 
   - All shell commands work normally
   - Output appears in the VGA text buffer (QEMU terminal window)
   - Serial output also goes to the bridge terminal
   - Commands like `help`, `desktop`, `graphics`, `haiku` all work

2. **Graphics Mode**:
   - Desktop layout is rendered (split-screen with ambition)
   - Graphics commands like `desktop` update the visual display
   - **Note**: Currently, you need to switch back to text mode to type commands
   - Keyboard input still works, but there's no visible text input in graphics mode yet

### Current Limitations

1. **No Text Input in Graphics Mode**
   - When in graphics mode, you can't see where you're typing
   - Solution: Switch to text mode (F1) to run commands
   - **Next Step**: Add text console overlay in graphics mode

2. **No Mouse Support**
   - Can't click on zones or interact with desktop visually
   - **Next Step**: Add PS/2 mouse driver and click detection

3. **Separate Windows**
   - QEMU graphics window is separate from terminal
   - Terminal shows serial output (shell commands)
   - Graphics window shows visual desktop
   - This is expected behavior, but can be confusing

## Next Milestone: Interactive Graphics Console

### Goal
Enable **full interaction** within graphics mode, so you can:
- See your commands as you type them
- Run commands without switching modes
- Eventually click on desktop zones

### Implementation Plan

#### Phase 1: Text Console Overlay (Immediate Next Step)
- Add a text console overlay at the bottom of graphics mode
- Show shell prompt and command input
- Render text using the existing `draw_text` function
- Keep it simple: single line input, scrollable history

**Example Layout:**
```
┌─────────────────────────────────────┐
│  Desktop Zones (Conversation, etc.) │
│                                     │
│                                     │
├─────────────────────────────────────┤
│ genesis> [cursor]                   │ ← Console overlay
└─────────────────────────────────────┘
```

#### Phase 2: Mouse Support
- Add PS/2 mouse driver (similar to keyboard)
- Detect mouse clicks on desktop zones
- Enable click-to-focus zones
- Add visual feedback (highlight on hover)

#### Phase 3: Integrated Shell in Graphics Mode
- Full terminal emulator in graphics mode
- Multi-line command history
- Syntax highlighting (optional)
- Agent status indicators

### Commands That Work in Both Modes

- `help` - Show command list
- `mode` - Switch modes
- `desktop` - Render desktop layout (works in both modes)
- `graphics` - Draw test pattern
- `archimedes` - Show Archimedes info
- `heartbeat` - Show current ambition
- `insights` - Show collected Sparks/Connections
- `haiku` - Generate haiku via LLM
- `scout video [path]` - Analyze video

### Quick Reference

| Action | Method |
|--------|--------|
| Toggle modes | Press **F1** or **Escape** (Mac-friendly) |
| Switch to text | `mode text` |
| Switch to graphics | `mode graphics` |
| Show desktop | `desktop` |
| Get help | `help` |

## Future Vision

The ultimate goal is a **seamless graphical interface** where:
- Agents organize the desktop visually
- You can interact with zones directly (click, drag, resize)
- Commands work naturally within the graphical environment
- Text and graphics coexist harmoniously
- Mouse and keyboard work together for rich interaction

This is the foundation for that vision!
