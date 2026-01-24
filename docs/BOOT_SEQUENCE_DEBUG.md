# Boot Sequence Debug Guide

## Problem: Second Stage Loader Not Initiating

The kernel appears to hang during boot, preventing the second stage loader from continuing.

## Boot Sequence Overview

1. **BIOS/UEFI** → Loads bootloader (first stage)
2. **Bootloader** (`bootloader` crate v0.9) → Loads kernel (second stage)
3. **Kernel Entry** (`kernel_main`) → Initializes system
4. **Graphics Init** → Switches to Mode 12h (640x480)

## Debugging Steps

### 1. Check Serial Output

The kernel outputs detailed debug information to the serial port. Watch for:
- `[BOOT]` messages - Early boot sequence
- `[MEMORY]` messages - Memory initialization
- `[GRAPHICS]` messages - Graphics system initialization
- `[VGA]` messages - VGA mode switching (NEW - added for Mode 12h)

**If serial output stops at a specific point, that's where the hang occurs.**

### 2. Common Hang Points

#### Graphics Initialization (Most Likely)
- **Location**: `kernel/src/gui/graphics.rs::init()`
- **Symptoms**: Serial output stops after `[GRAPHICS] Initializing graphics system...`
- **Possible Causes**:
  - Mode 12h register values incorrect
  - VGA hardware not responding
  - Triple fault during register programming

#### Memory Initialization
- **Location**: `kernel/src/main.rs::kernel_main()` after memory init
- **Symptoms**: Serial output stops after `[MEMORY]` messages
- **Possible Causes**:
  - Invalid memory map
  - Page table corruption

### 3. Mode 12h Register Verification

Mode 12h uses planar graphics (4 color planes). Key registers:
- **MISC**: 0xE7
- **Sequencer**: Planar mode (0x04 = 0x06)
- **CRT Controller**: 640x480 timing
- **Graphics Controller**: Write mode 0, planar

**If Mode 12h fails, the system will hang during VGA register programming.**

### 4. Fallback Strategy

If Mode 12h continues to hang, consider:
1. **Temporary fallback to Mode 13h** (320x200, simpler)
2. **Add timeout/delay** between register writes
3. **Verify QEMU VGA emulation** supports Mode 12h

### 5. QEMU Configuration

Current QEMU flags:
```bash
-drive format=raw,file="$KERNEL_PATH"
-serial stdio
-m 128M
-no-reboot
-no-shutdown
```

**Missing**: `-vga std` flag (may be needed for proper VGA emulation)

### 6. Next Steps

1. **Rebuild with debug output** (already added)
2. **Run and check serial output** - identify exact hang point
3. **If hangs at Mode 12h**: Consider temporary fallback to Mode 13h
4. **If hangs elsewhere**: Check memory initialization

## Serial Output Expected Sequence

```
[BOOT] Serial port initialized
[BOOT] VGA buffer at 0xb8000
[BOOT] Screen cleared
[BOOT] Boot screen displayed
[BOOT] Showing boot screen briefly...
[BOOT] Continuing initialization...
[MEMORY] Initializing memory management...
[MEMORY] Page mapper initialized
[MEMORY] Frame allocator initialized
[MEMORY] Heap initialized
[GRAPHICS] Initializing graphics system...
[GRAPHICS] Creating graphics context...
[GRAPHICS] Initializing Mode 12h...
[VGA] Switching to Mode 12h (640x480, 16 colors)...
[VGA] Setting Miscellaneous Output Register...
[VGA] MISC register set
[VGA] Configuring Sequencer registers...
[VGA] Sequencer configured
[VGA] Unlocking CRT Controller registers...
[VGA] CRT Controller unlocked
[VGA] Configuring CRT Controller registers...
[VGA] CRT Controller configured
[VGA] Configuring Graphics Controller registers...
[VGA] Graphics Controller configured
[VGA] Configuring Attribute Controller...
[VGA] Attribute Controller configured
[PALETTE] Setting up 16-color palette...
[PALETTE] 16-color palette configured
[VGA] Mode 12h initialization complete!
[GRAPHICS] Storing graphics context...
[GRAPHICS] Graphics system ready!
```

**If output stops before completion, note the last message - that's the hang point.**
