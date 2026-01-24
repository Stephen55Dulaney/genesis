# Resolution and Clarity Improvements

## Current Status

**Mode 13h (320x200)**
- Resolution: 320x200 pixels
- Colors: 256 colors
- Font: 8x8 pixels per character
- Character spacing: 4 pixels
- Line spacing: 3 pixels

## Immediate Improvements (Applied)

✅ **Increased character spacing** from 3px to 4px
✅ **Increased line spacing** from 2px to 3px
✅ **Fixed duplicate output** in console

## Option 1: Further Spacing Increases

If text is still too cramped, we can:
- Increase character spacing to 5px or 6px
- Increase line spacing to 4px or 5px
- Trade-off: Less text fits on screen, but more readable

## Option 2: Switch to VGA Mode 12h (640x480)

**Benefits:**
- **4x more pixels** (640x480 vs 320x200)
- Much clearer text rendering
- More screen real estate
- Better for desktop layouts

**Trade-offs:**
- Only 16 colors (vs 256 colors)
- More complex VGA register programming
- Requires rewriting graphics code

### Implementation Steps for Mode 12h

1. **Update constants:**
   ```rust
   pub const WIDTH: u32 = 640;
   pub const HEIGHT: u32 = 480;
   ```

2. **Create `switch_to_mode_12h()` function:**
   - Program VGA registers for 640x480 mode
   - Set up 16-color palette
   - Update framebuffer addressing

3. **Update graphics context:**
   - Adjust for new resolution
   - Update drawing functions
   - Scale fonts if needed

4. **Update desktop layout:**
   - More space for zones
   - Better console overlay
   - More readable text

## Option 3: Scale Font Rendering

We could render each font pixel as a 2x2 block:
- Characters appear 16x16 instead of 8x8
- More visible, but uses more screen space
- Easier to read at current resolution

## Recommendation

**For immediate improvement:** Use the increased spacing (already applied)

**For best clarity:** Switch to Mode 12h (640x480)
- Would require ~2-3 hours of work
- Much better readability
- Worth it for a professional OS

**Quick test:** Try the current 4px spacing first. If still too cramped, we can:
1. Increase to 5-6px spacing
2. Or implement Mode 12h

## Current Text Readability

With 4px character spacing and 3px line spacing:
- Characters should be clearly separated
- Lines should be distinct
- Console should be readable
- Desktop zones should be legible

If still hard to read, Mode 12h is the best solution.
