# Genesis Display Options

## Current Display Mode

Genesis uses **VGA Mode 13h**, which is:
- **Resolution**: 320x200 pixels (fixed)
- **Colors**: 256 colors
- **Framebuffer**: 0xA0000

This is a standard VGA graphics mode that provides a good balance of resolution and color depth for our needs.

## Font Size Options

### Current Font Settings
- **Character Size**: 8x8 pixels per character
- **Character Spacing**: 3 pixels between characters
- **Line Spacing**: 2 pixels between lines

### Making Text More Readable

**Option 1: Increase Spacing (Current Approach)**
- Already increased from 1px to 3px character spacing
- Increased line spacing from 1px to 2px
- Characters are now more separated and readable

**Option 2: Use Larger Font Size**
- Could create a 10x10 or 12x12 font (but would fit fewer characters)
- Would require new font bitmaps
- Trade-off: Better readability vs. less text on screen

**Option 3: Use Smaller Font Size**
- Could create a 6x6 font (more characters, but harder to read)
- Not recommended for readability

## Window Size Options

### QEMU Window Size

The QEMU window size is controlled by:
1. **VGA Mode**: Mode 13h = 320x200 (fixed)
2. **QEMU Scaling**: QEMU can scale the window, but the internal resolution stays 320x200

### Changing Resolution

To use a different resolution, we'd need to switch VGA modes:

**Mode 12h**: 640x480x16 colors
- Higher resolution, fewer colors
- More screen real estate
- Better for text readability

**Mode X**: 320x240x256 colors (non-standard)
- Slightly taller than Mode 13h
- Requires custom VGA programming

**Current Choice: Mode 13h**
- Good color depth (256 colors)
- Simple to program
- Standard VGA mode
- 320x200 is sufficient for our current needs

### QEMU Scaling

QEMU can scale the window, but this just makes pixels bigger - it doesn't add more pixels. The internal resolution remains 320x200.

## Recommendations

**For Better Readability:**
1. ✅ **Current**: Increased spacing to 3px (done)
2. ✅ **Current**: Increased line spacing to 2px (done)
3. **Future**: Could increase to 4px spacing if still too tight
4. **Future**: Could switch to Mode 12h (640x480) for more pixels

**For Mode Toggling:**
- ✅ Fixed toggle freeze issue (mode state now updates correctly)
- F1 or Escape toggles between text/graphics modes
- Should work reliably now

## Testing Font Spacing

After rebuilding, test with:
- `help` command - should see well-spaced characters
- Console input - should be readable as you type
- Desktop zones - text should be legible

If still too tight, we can increase spacing to 4px or even 5px.
