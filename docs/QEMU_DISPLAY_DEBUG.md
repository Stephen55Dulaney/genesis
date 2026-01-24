# QEMU Display Debugging Guide

## Problem: QEMU Window Stuck on Boot Screen

**Symptoms:**
- Boot sequence completes successfully (visible in terminal)
- Graphics mode initializes (Mode 12h)
- Desktop renders
- BUT: QEMU window still shows boot screen ("Genesis Awakening...")

## Root Cause Analysis

QEMU displays TWO separate memory regions:
1. **Text Buffer** (0xb8000) - VGA text mode, 80x25 characters
2. **Graphics Framebuffer** (0xA0000) - VGA graphics mode, 640x480 pixels

**The Issue:** After switching to graphics mode, QEMU might still be displaying the text buffer instead of the graphics framebuffer.

## Debugging Steps

### 1. Check Serial Output

Look for these messages in order:
- `[VGA] Mode 12h initialization complete!` ✅ (appears)
- `[VGA] Clearing graphics framebuffer...` ❌ (MISSING - code not executing!)
- `[DESKTOP] Graphics buffer swapped - QEMU window should update!` ❌ (MISSING)

### 2. Why Framebuffer Clear Code Isn't Executing

The framebuffer clear code is after `[VGA] Mode 12h initialization complete!` but the debug message `[VGA] Clearing graphics framebuffer...` doesn't appear. This suggests:
- Code might not be compiled (check for errors)
- Function might be returning early
- Silent panic/error

### 3. QEMU Display Behavior

QEMU with `-vga std` should automatically switch between:
- Text mode: Shows 0xb8000
- Graphics mode: Shows 0xA0000

But it might need explicit signaling or the text buffer cleared.

## Solutions Implemented

### Solution 1: Clear Text Buffer After Graphics Init
After switching to graphics mode, clear the text buffer so QEMU doesn't show it:
```rust
vga_buffer::clear_screen(); // Clear text buffer at 0xb8000
```

### Solution 2: Force Framebuffer Clear
Clear graphics framebuffer immediately after Mode 12h switch (if code executes).

### Solution 3: Explicit Buffer Swap
Force buffer swap after desktop render to ensure QEMU sees the graphics.

## Next Steps

1. **Rebuild** and check if `[VGA] Clearing graphics framebuffer...` appears
2. **If it appears**: Framebuffer clear is working, but QEMU might need text buffer cleared too
3. **If it doesn't appear**: Check compilation errors or function flow

## QEMU Command Flags

Current flags in `genesis-bridge.py`:
```python
"-display", "default",  # Show graphics window
"-vga", "std",         # Standard VGA emulation
```

These should be sufficient for Mode 12h support.

## Expected Behavior After Fix

1. Boot screen appears briefly (text mode)
2. Graphics mode switches (black screen)
3. Desktop + console appear (graphics mode)
4. QEMU window updates dynamically
