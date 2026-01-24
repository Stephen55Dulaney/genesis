//! VGA Graphics Mode Driver
//!
//! Provides pixel-level graphics for Genesis OS.
//! Uses VGA Mode 12h: 640x480 pixels, 16 colors (4x more pixels than Mode 13h!)
//!
//! Inspired by Bevy's simple API, adapted for bare-metal.

extern crate alloc;

use core::ptr;
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;

// Import serial macros for debug output
use crate::serial_println;

/// VGA Mode 12h: 640x480, 16 colors
pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;
pub const FRAMEBUFFER_ADDR: usize = 0xA0000;

/// Color palette (256 colors)
/// Mode 13h uses a palette - we'll use standard VGA colors
#[allow(dead_code)] // Some colors not used yet but available for future use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
    // More colors available in palette...
}

/// Graphics context for rendering
pub struct GraphicsContext {
    /// Front buffer (displayed)
    front_buffer: *mut u8,
    /// Back buffer (drawn to)
    back_buffer: *mut u8,
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Current clear color
    clear_color: u8,
}

impl GraphicsContext {
    /// Create a new graphics context
    /// 
    /// # Safety
    /// This function is unsafe because it directly accesses VGA memory.
    /// Should only be called once during kernel initialization.
    pub unsafe fn new() -> Self {
        // Initialize Mode 13h via VGA ports
        // This is a simplified version - full implementation would set all VGA registers
        
        // Set VGA Mode 13h (320x200x256)
        // Port 0x3C2: Miscellaneous Output Register
        // Port 0x3D4/0x3D5: CRT Controller
        // Port 0x3C4/0x3C5: Sequencer
        // Port 0x3CE/0x3CF: Graphics Controller
        // Port 0x3C0: Attribute Controller
        
        // For now, we'll assume Mode 13h is already set or will be set by bootloader
        // In a full implementation, we'd program the VGA registers here
        
        GraphicsContext {
            front_buffer: FRAMEBUFFER_ADDR as *mut u8,
            back_buffer: ptr::null_mut(), // Will allocate later for double buffering
            width: WIDTH,
            height: HEIGHT,
            clear_color: Color::Black as u8,
        }
    }
    
    /// Initialize VGA Mode 12h
    /// 
    /// This sets up the VGA hardware for graphics mode.
    /// Mode 12h: 640x480 pixels, 16 colors, planar framebuffer at 0xA0000
    /// 
    /// # Safety
    /// Direct hardware access - must be called during kernel init.
    pub unsafe fn init_mode_12h(&mut self) {
        // Actually switch to VGA Mode 12h by programming hardware registers
        switch_to_mode_12h();
        
        // Clear the framebuffer
        self.clear(Color::Black);
        
        // Draw a test pattern to verify graphics mode is working
        self.draw_test_pattern();
    }
    
    /// Initialize VGA Mode 13h (legacy - kept for compatibility)
    /// 
    /// # Safety
    /// Direct hardware access - must be called during kernel init.
    pub unsafe fn init_mode_13h(&mut self) {
        self.init_mode_12h(); // Use Mode 12h instead
    }
    
    /// Draw a test pattern to verify graphics is working
    pub fn draw_test_pattern(&mut self) {
        // Clear screen
        self.clear(Color::Black);
        
        // Draw colored rectangles in corners (larger for 640x480)
        let corner_size = 100u32;
        self.draw_rect(0, 0, corner_size, corner_size, Color::Red as u8);
        self.draw_rect(self.width - corner_size, 0, corner_size, corner_size, Color::Green as u8);
        self.draw_rect(0, self.height - corner_size, corner_size, corner_size, Color::Blue as u8);
        self.draw_rect(self.width - corner_size, self.height - corner_size, corner_size, corner_size, Color::Yellow as u8);
        
        // Draw a centered rectangle
        let center_x = self.width / 2 - 80;
        let center_y = self.height / 2 - 40;
        self.draw_rect_outline(center_x, center_y, 160, 80, Color::Cyan as u8);
        
        // Draw text (larger spacing for readability)
        self.draw_text(20, 120, "GENESIS", Color::White as u8);
        self.draw_text(20, 140, "Mode 12h: 640x480 Graphics", Color::LightCyan as u8);
        self.draw_text(20, 160, "4x More Pixels - Much Clearer!", Color::LightGreen as u8);
    }
    
    /// Draw a single pixel
    /// 
    /// For Mode 12h (640x480), uses planar graphics mode:
    /// - Each pixel is 4 bits (one per color plane)
    /// - Memory organized in 4 planes
    /// - Each byte represents 8 pixels horizontally
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        // Mode 12h uses planar graphics (4 color planes)
        // Calculate byte offset: (y * bytes_per_row) + (x / 8)
        // bytes_per_row = width / 8 = 640 / 8 = 80
        let bytes_per_row = self.width / 8;
        let byte_offset = (y * bytes_per_row + (x / 8)) as usize;
        let bit_position = 7 - (x % 8); // Bit 7 is leftmost pixel
        let bit_mask = 1u8 << bit_position;
        
        unsafe {
            use x86_64::instructions::port::Port;
            
            let buffer = if self.back_buffer.is_null() {
                self.front_buffer
            } else {
                self.back_buffer
            };
            
            // Graphics Controller ports
            let mut gc_index: Port<u8> = Port::new(0x3CE);
            let mut gc_data: Port<u8> = Port::new(0x3CF);
            
            // Set bit mask to write only this pixel
            gc_index.write(0x08);
            gc_data.write(bit_mask);
            
            // Use set/reset to write color to all planes
            gc_index.write(0x00); // Set/reset register
            gc_data.write(color & 0x0F); // Color value (4 bits)
            
            gc_index.write(0x01); // Enable set/reset
            gc_data.write(0x0F); // Enable for all 4 planes
            
            // Read-modify-write cycle (required for planar mode)
            let _current_byte = *buffer.add(byte_offset);
            *buffer.add(byte_offset) = 0xFF; // Write all bits (set/reset handles color)
            
            // Restore graphics controller state
            gc_index.write(0x01);
            gc_data.write(0x00); // Disable set/reset
            
            gc_index.write(0x08);
            gc_data.write(0xFF); // Reset bit mask
        }
    }
    
    /// Draw a filled rectangle
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u8) {
        for py in y..(y + h).min(self.height) {
            for px in x..(x + w).min(self.width) {
                self.draw_pixel(px, py, color);
            }
        }
    }
    
    /// Draw a rectangle outline
    pub fn draw_rect_outline(&mut self, x: u32, y: u32, w: u32, h: u32, color: u8) {
        // Top and bottom lines
        for px in x..(x + w).min(self.width) {
            self.draw_pixel(px, y, color);
            if y + h > 0 {
                self.draw_pixel(px, (y + h - 1).min(self.height - 1), color);
            }
        }
        
        // Left and right lines
        for py in y..(y + h).min(self.height) {
            self.draw_pixel(x, py, color);
            if x + w > 0 {
                self.draw_pixel((x + w - 1).min(self.width - 1), py, color);
            }
        }
    }
    
    /// Clear the screen with a color
    pub fn clear(&mut self, color: Color) {
        self.clear_color = color as u8;
        let color_value = color as u8 & 0x0F; // Mode 12h uses 4-bit colors
        
        unsafe {
            use x86_64::instructions::port::Port;
            
            let buffer = if self.back_buffer.is_null() {
                self.front_buffer
            } else {
                self.back_buffer
            };
            
            // For Mode 12h planar mode, clear using set/reset
            let mut gc_index: Port<u8> = Port::new(0x3CE);
            let mut gc_data: Port<u8> = Port::new(0x3CF);
            
            // Set/reset value = clear color
            gc_index.write(0x00);
            gc_data.write(color_value);
            
            // Enable set/reset for all planes
            gc_index.write(0x01);
            gc_data.write(0x0F);
            
            // Set bit mask to write all bits
            gc_index.write(0x08);
            gc_data.write(0xFF);
            
            // Clear all bytes (each byte = 8 pixels)
            let bytes_per_row = self.width / 8; // 640 / 8 = 80
            let total_bytes = (bytes_per_row * self.height) as usize;
            
            for i in 0..total_bytes {
                let _ = *buffer.add(i); // Read (required for write mode)
                *buffer.add(i) = 0xFF; // Write (set/reset applies color)
            }
            
            // Restore graphics controller
            gc_index.write(0x01);
            gc_data.write(0x00); // Disable set/reset
        }
    }
    
    /// Swap front and back buffers (double buffering)
    /// 
    /// If double buffering is enabled, this copies the back buffer to the front buffer.
    /// Otherwise, it's a no-op.
    pub fn swap_buffers(&mut self) {
        if !self.back_buffer.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(
                    self.back_buffer,
                    self.front_buffer,
                    (self.width * self.height) as usize,
                );
            }
        }
    }
    
    /// Enable double buffering
    /// 
    /// Allocates a back buffer for smooth rendering.
    /// Requires heap allocation to be initialized.
    pub fn enable_double_buffering(&mut self) -> Result<(), &'static str> {
        use alloc::alloc::{alloc, Layout};
        
        let size = (self.width * self.height) as usize;
        let layout = Layout::from_size_align(size, 1)
            .map_err(|_| "Invalid layout")?;
        
        unsafe {
            let buffer = alloc(layout);
            if buffer.is_null() {
                return Err("Failed to allocate back buffer");
            }
            self.back_buffer = buffer;
        }
        
        Ok(())
    }
    
    /// Get width
    pub fn width(&self) -> u32 {
        self.width
    }
    
    /// Get height
    pub fn height(&self) -> u32 {
        self.height
    }
    
    /// Draw text using a simple bitmap font
    /// 
    /// Uses the current font from the font library, respecting font spacing
    pub fn draw_text(&mut self, x: u32, y: u32, text: &str, color: u8) {
        use super::fonts;
        let font = fonts::get_font();
        let char_width = font.char_width;
        let spacing = font.spacing;
        let char_height = font.char_height;
        
        let mut cursor_x = x;
        let mut cursor_y = y;
        
        for ch in text.chars() {
            match ch {
                '\n' => {
                    cursor_x = x;
                    cursor_y += char_height + 3; // Line height with extra spacing for readability
                }
                '\r' => {
                    cursor_x = x;
                }
                ' ' => {
                    // Space character - just advance cursor
                    cursor_x += char_width + spacing;
                }
                _ => {
                    self.draw_char(cursor_x, cursor_y, ch, color);
                    cursor_x += char_width + spacing; // Character width + spacing
                    
                    // Wrap if we go off screen
                    if cursor_x + char_width > self.width {
                        cursor_x = x;
                        cursor_y += char_height + 3; // Extra line spacing for readability
                    }
                }
            }
        }
    }
    
    /// Draw a single character using a simple bitmap font
    fn draw_char(&mut self, x: u32, y: u32, ch: char, color: u8) {
        // Simple 8x8 bitmap font (ASCII subset)
        // Each character is represented as 8 bytes, each byte is a row of 8 pixels
        let font_data = get_char_bitmap(ch);
        
        for (row, &byte) in font_data.iter().enumerate() {
            let py = y + row as u32;
            if py >= self.height {
                break;
            }
            
            for col in 0..8 {
                let px = x + col;
                if px >= self.width {
                    break;
                }
                
                // Check if bit is set (bit 7 is leftmost pixel)
                if (byte >> (7 - col)) & 1 != 0 {
                    self.draw_pixel(px, py, color);
                }
            }
        }
    }
}

/// Get bitmap data for a character (8x8 font)
/// Returns 8 bytes, each representing a row of pixels
/// Uses the current font from the font library
fn get_char_bitmap(ch: char) -> [u8; 8] {
    use super::fonts;
    fonts::get_glyph(ch)
}

/// Switch VGA to Mode 12h (640x480, 16 colors)
/// 
/// This programs all the VGA registers needed for Mode 12h graphics mode.
/// Mode 12h uses planar graphics (4 color planes) for 16 colors.
/// # Safety
/// Direct hardware access - only call during initialization.
unsafe fn switch_to_mode_12h() {
    use x86_64::instructions::port::Port;
    use crate::serial_println;
    
    serial_println!("[VGA] Switching to Mode 12h (640x480, 16 colors)...");
    
    // VGA Mode 12h register values (640x480, 16 colors)
    
    // Miscellaneous Output Register
    serial_println!("[VGA] Setting Miscellaneous Output Register...");
    let mut misc_port: Port<u8> = Port::new(0x3C2);
    misc_port.write(0xE7); // Mode 12h value
    serial_println!("[VGA] MISC register set");
    
    // Sequencer registers (0x3C4/0x3C5)
    let mut seq_index: Port<u8> = Port::new(0x3C4);
    let mut seq_data: Port<u8> = Port::new(0x3C5);
    
    let seq_regs: [(u8, u8); 5] = [
        (0x00, 0x03), // Reset register
        (0x01, 0x01), // Clocking mode
        (0x02, 0x0F), // Map mask (all planes)
        (0x03, 0x00), // Character map select
        (0x04, 0x06), // Memory mode (planar)
    ];
    
    for (index, value) in seq_regs.iter() {
        seq_index.write(*index);
        seq_data.write(*value);
    }
    
    // Unlock CRT Controller registers
    serial_println!("[VGA] Unlocking CRT Controller registers...");
    let mut crtc_index: Port<u8> = Port::new(0x3D4);
    let mut crtc_data: Port<u8> = Port::new(0x3D5);
    
    crtc_index.write(0x11);
    let val = crtc_data.read();
    crtc_data.write(val & 0x7F); // Unlock
    serial_println!("[VGA] CRT Controller unlocked");
    
    // CRT Controller values for Mode 12h (640x480)
    serial_println!("[VGA] Configuring CRT Controller registers...");
    let crtc_regs: [(u8, u8); 25] = [
        (0x00, 0x5F), // Horizontal total
        (0x01, 0x4F), // Horizontal display end (640/8 - 1)
        (0x02, 0x50), // Start horizontal blanking
        (0x03, 0x82), // End horizontal blanking
        (0x04, 0x54), // Start horizontal retrace
        (0x05, 0x80), // End horizontal retrace
        (0x06, 0x0D), // Vertical total (low)
        (0x07, 0x3E), // Overflow (vertical total high)
        (0x08, 0x00), // Preset row scan
        (0x09, 0x40), // Maximum scan line (16 pixels per character)
        (0x0A, 0x00), // Cursor start
        (0x0B, 0x00), // Cursor end
        (0x0C, 0x00), // Start address high
        (0x0D, 0x00), // Start address low
        (0x0E, 0x00), // Cursor location high
        (0x0F, 0x00), // Cursor location low
        (0x10, 0xEA), // Vertical retrace start
        (0x11, 0x8C), // Vertical retrace end
        (0x12, 0xDF), // Vertical display end (480 lines)
        (0x13, 0x28), // Offset (80 bytes per row for 640 pixels)
        (0x14, 0x00), // Underline location
        (0x15, 0xE7), // Start vertical blanking
        (0x16, 0x04), // End vertical blanking
        (0x17, 0xE3), // Mode control
        (0x18, 0xFF), // Line compare
    ];
    
    for (index, value) in crtc_regs.iter() {
        crtc_index.write(*index);
        crtc_data.write(*value);
    }
    
    // Graphics Controller registers (0x3CE/0x3CF)
    let mut gc_index: Port<u8> = Port::new(0x3CE);
    let mut gc_data: Port<u8> = Port::new(0x3CF);
    
    let gc_regs: [(u8, u8); 9] = [
        (0x00, 0x00), // Set/reset
        (0x01, 0x00), // Enable set/reset
        (0x02, 0x00), // Color compare
        (0x03, 0x00), // Data rotate
        (0x04, 0x00), // Read map select
        (0x05, 0x10), // Graphics mode (read mode 0, write mode 0)
        (0x06, 0x0E), // Misc graphics
        (0x07, 0x05), // Color don't care
        (0x08, 0xFF), // Bit mask
    ];
    
    for (index, value) in gc_regs.iter() {
        gc_index.write(*index);
        gc_data.write(*value);
    }
    serial_println!("[VGA] Graphics Controller configured");
    
    // Attribute Controller registers (0x3C0)
    serial_println!("[VGA] Configuring Attribute Controller...");
    let mut input_status: Port<u8> = Port::new(0x3DA);
    let mut attr_port: Port<u8> = Port::new(0x3C0);
    
    // Reset attribute controller flip-flop
    let _ = input_status.read();
    
    // Attribute Controller palette (16 colors)
    for i in 0u8..16 {
        attr_port.write(i);
        attr_port.write(i); // Identity mapping
    }
    
    // Attribute Controller mode registers
    attr_port.write(0x10); attr_port.write(0x01); // Mode control
    attr_port.write(0x11); attr_port.write(0x00); // Overscan
    attr_port.write(0x12); attr_port.write(0x0F); // Color plane enable
    attr_port.write(0x13); attr_port.write(0x00); // Horizontal panning
    attr_port.write(0x14); attr_port.write(0x00); // Color select
    
    // Enable video
    attr_port.write(0x20);
    serial_println!("[VGA] Attribute Controller configured");
    
    // Set up 16-color palette
    serial_println!("[VGA] Setting up 16-color palette...");
    setup_palette_16color();
    serial_println!("[VGA] Mode 12h initialization complete!");
    
    // CRITICAL: Clear the graphics framebuffer immediately after mode switch
    // This ensures QEMU displays graphics mode, not the old text buffer
    serial_println!("[VGA] STEP 1: About to clear graphics framebuffer...");
    
    // Use the existing gc_index and gc_data (they're still in scope)
    serial_println!("[VGA] STEP 2: Configuring graphics controller for clearing...");
    gc_index.write(0x00); // Set/reset register
    gc_data.write(0x00); // Black (color 0)
    
    gc_index.write(0x01); // Enable set/reset
    gc_data.write(0x0F); // Enable for all 4 planes
    
    gc_index.write(0x08); // Bit mask
    gc_data.write(0xFF); // All bits
    
    serial_println!("[VGA] STEP 3: Calculating framebuffer size...");
    let framebuffer = FRAMEBUFFER_ADDR as *mut u8;
    let bytes_per_row = WIDTH / 8; // 640 / 8 = 80 bytes per row
    let total_bytes = (bytes_per_row * HEIGHT) as usize;
    serial_println!("[VGA] STEP 4: Framebuffer: {} bytes total", total_bytes);
    
    serial_println!("[VGA] STEP 5: Clearing framebuffer (this may take a moment)...");
    // Clear entire framebuffer
    unsafe {
        for i in 0..total_bytes {
            let _ = *framebuffer.add(i); // Read (required for write mode)
            *framebuffer.add(i) = 0xFF; // Write (set/reset applies black)
        }
    }
    
    serial_println!("[VGA] STEP 6: Restoring graphics controller state...");
    // Restore graphics controller
    gc_index.write(0x01);
    gc_data.write(0x00); // Disable set/reset
    
    serial_println!("[VGA] STEP 7: SUCCESS! Graphics framebuffer cleared - QEMU should show graphics!");
}

/// Switch VGA to Mode 13h (320x200, 256 colors) - Legacy function
/// 
/// This programs all the VGA registers needed for graphics mode.
/// # Safety
/// Direct hardware access - only call during initialization.
unsafe fn switch_to_mode_13h() {
    // Redirect to Mode 12h for better clarity
    switch_to_mode_12h();
}

/// Set up the standard VGA 16-color palette for Mode 12h
unsafe fn setup_palette_16color() {
    use x86_64::instructions::port::Port;
    
    serial_println!("[PALETTE] Setting up 16-color palette...");
    let mut palette_index: Port<u8> = Port::new(0x3C8);
    let mut palette_data: Port<u8> = Port::new(0x3C9);
    
    palette_index.write(0); // Start at color 0
    
    // Standard VGA 16 colors
    let standard_colors: [(u8, u8, u8); 16] = [
        (0x00, 0x00, 0x00), // 0: Black
        (0x00, 0x00, 0x2A), // 1: Blue
        (0x00, 0x2A, 0x00), // 2: Green
        (0x00, 0x2A, 0x2A), // 3: Cyan
        (0x2A, 0x00, 0x00), // 4: Red
        (0x2A, 0x00, 0x2A), // 5: Magenta
        (0x2A, 0x15, 0x00), // 6: Brown
        (0x2A, 0x2A, 0x2A), // 7: Light Gray
        (0x15, 0x15, 0x15), // 8: Dark Gray
        (0x15, 0x15, 0x3F), // 9: Light Blue
        (0x15, 0x3F, 0x15), // 10: Light Green
        (0x15, 0x3F, 0x3F), // 11: Light Cyan
        (0x3F, 0x15, 0x15), // 12: Light Red
        (0x3F, 0x15, 0x3F), // 13: Pink
        (0x3F, 0x3F, 0x15), // 14: Yellow
        (0x3F, 0x3F, 0x3F), // 15: White
    ];
    
    for (r, g, b) in standard_colors.iter() {
        palette_data.write(*r);
        palette_data.write(*g);
        palette_data.write(*b);
    }
    serial_println!("[PALETTE] 16-color palette configured");
}

/// Set up the standard VGA 256-color palette (legacy - for Mode 13h)
unsafe fn setup_palette() {
    setup_palette_16color(); // Use 16-color palette for Mode 12h
}

// Safety: GraphicsContext is safe to share and send across threads because:
// 1. All access is protected by a Mutex
// 2. Raw pointers are only accessed through safe methods that check bounds
// 3. VGA framebuffer is a fixed memory location that doesn't move
// 4. The back_buffer is heap-allocated and managed by the Mutex
unsafe impl Send for GraphicsContext {}
unsafe impl Sync for GraphicsContext {}

// Global graphics context
// Note: Doc comment removed - lazy_static! macro doesn't support doc comments
lazy_static! {
    pub static ref GRAPHICS: Mutex<Option<GraphicsContext>> = Mutex::new(None);
}

/// Current VGA mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VgaMode {
    Text,      // Mode 3: 80x25 text mode
    Graphics,  // Mode 13h: 320x200 graphics mode
}

lazy_static! {
    /// Current VGA mode state
    pub static ref CURRENT_MODE: Mutex<VgaMode> = Mutex::new(VgaMode::Text);
}

/// Initialize graphics system
/// 
/// # Safety
/// Must be called during kernel initialization, after heap is initialized.
pub unsafe fn init() {
    serial_println!("[GRAPHICS] Creating graphics context...");
    let mut graphics = GraphicsContext::new();
    
    serial_println!("[GRAPHICS] Initializing Mode 12h...");
    graphics.init_mode_12h(); // Use Mode 12h (640x480) for better clarity
    
    serial_println!("[GRAPHICS] Storing graphics context...");
    *GRAPHICS.lock() = Some(graphics);
    *CURRENT_MODE.lock() = VgaMode::Graphics;
    serial_println!("[GRAPHICS] Graphics system ready!");
}

/// Switch VGA to text mode (Mode 3: 80x25)
/// 
/// # Safety
/// Direct hardware access - switches VGA registers.
pub unsafe fn switch_to_text_mode() {
    use x86_64::instructions::port::Port;
    
    // VGA Mode 3 (80x25 text mode) register values
    // This is the standard text mode that QEMU boots into
    
    // Miscellaneous Output Register
    let mut misc_port: Port<u8> = Port::new(0x3C2);
    misc_port.write(0x67); // Mode 3 value
    
    // Sequencer registers
    let mut seq_index: Port<u8> = Port::new(0x3C4);
    let mut seq_data: Port<u8> = Port::new(0x3C5);
    
    let seq_regs: [(u8, u8); 5] = [
        (0x00, 0x03), // Reset register
        (0x01, 0x00), // Clocking mode
        (0x02, 0x03), // Map mask
        (0x03, 0x00), // Character map select
        (0x04, 0x02), // Memory mode
    ];
    
    for (index, value) in seq_regs.iter() {
        seq_index.write(*index);
        seq_data.write(*value);
    }
    
    // Unlock CRT Controller
    let mut crtc_index: Port<u8> = Port::new(0x3D4);
    let mut crtc_data: Port<u8> = Port::new(0x3D5);
    
    crtc_index.write(0x11);
    let val = crtc_data.read();
    crtc_data.write(val & 0x7F);
    
    // CRT Controller values for Mode 3
    let crtc_regs: [(u8, u8); 25] = [
        (0x00, 0x5F), // Horizontal total
        (0x01, 0x4F), // Horizontal display end
        (0x02, 0x50), // Start horizontal blanking
        (0x03, 0x82), // End horizontal blanking
        (0x04, 0x55), // Start horizontal retrace
        (0x05, 0x81), // End horizontal retrace
        (0x06, 0xBF), // Vertical total
        (0x07, 0x1F), // Overflow
        (0x08, 0x00), // Preset row scan
        (0x09, 0x4F), // Maximum scan line
        (0x0A, 0x0D), // Cursor start
        (0x0B, 0x0E), // Cursor end
        (0x0C, 0x00), // Start address high
        (0x0D, 0x00), // Start address low
        (0x0E, 0x00), // Cursor location high
        (0x0F, 0x00), // Cursor location low
        (0x10, 0x9C), // Vertical retrace start
        (0x11, 0x8E), // Vertical retrace end
        (0x12, 0x8F), // Vertical display end
        (0x13, 0x28), // Offset
        (0x14, 0x1F), // Underline location
        (0x15, 0x96), // Start vertical blanking
        (0x16, 0xB9), // End vertical blanking
        (0x17, 0xA3), // Mode control
        (0x18, 0xFF), // Line compare
    ];
    
    for (index, value) in crtc_regs.iter() {
        crtc_index.write(*index);
        crtc_data.write(*value);
    }
    
    // Graphics Controller registers
    let mut gc_index: Port<u8> = Port::new(0x3CE);
    let mut gc_data: Port<u8> = Port::new(0x3CF);
    
    let gc_regs: [(u8, u8); 9] = [
        (0x00, 0x00), // Set/reset
        (0x01, 0x00), // Enable set/reset
        (0x02, 0x00), // Color compare
        (0x03, 0x00), // Data rotate
        (0x04, 0x00), // Read map select
        (0x05, 0x10), // Graphics mode (text mode)
        (0x06, 0x0E), // Misc graphics
        (0x07, 0x00), // Color don't care
        (0x08, 0xFF), // Bit mask
    ];
    
    for (index, value) in gc_regs.iter() {
        gc_index.write(*index);
        gc_data.write(*value);
    }
    
    // Attribute Controller
    let mut input_status: Port<u8> = Port::new(0x3DA);
    let mut attr_port: Port<u8> = Port::new(0x3C0);
    
    let _ = input_status.read(); // Reset flip-flop
    
    // Standard text mode palette
    for i in 0u8..16 {
        attr_port.write(i);
        attr_port.write(i);
    }
    
    attr_port.write(0x10); attr_port.write(0x0C); // Mode control
    attr_port.write(0x11); attr_port.write(0x00); // Overscan
    attr_port.write(0x12); attr_port.write(0x0F); // Color plane enable
    attr_port.write(0x13); attr_port.write(0x08); // Horizontal panning
    attr_port.write(0x14); attr_port.write(0x00); // Color select
    
    attr_port.write(0x20); // Enable video
    
    // Update mode state
    *CURRENT_MODE.lock() = VgaMode::Text;
}

/// Toggle between text and graphics mode
/// 
/// # Safety
/// Direct hardware access - switches VGA registers.
pub unsafe fn toggle_mode() {
    let current = *CURRENT_MODE.lock();
    match current {
        VgaMode::Text => {
            // Switch to graphics mode (Mode 12h)
            // Ensure graphics context exists
            if GRAPHICS.lock().is_none() {
                let mut graphics = GraphicsContext::new();
                graphics.init_mode_12h();
                *GRAPHICS.lock() = Some(graphics);
            } else if let Some(ref mut gfx) = GRAPHICS.lock().as_mut() {
                gfx.init_mode_12h();
            }
            // Update mode state
            *CURRENT_MODE.lock() = VgaMode::Graphics;
        }
        VgaMode::Graphics => {
            // Switch to text mode
            switch_to_text_mode();
            // Mode state is updated in switch_to_text_mode()
        }
    }
}

/// Switch to graphics mode (Mode 12h)
/// 
/// # Safety
/// Direct hardware access - switches VGA registers.
pub unsafe fn switch_to_graphics_mode() {
    // Ensure graphics context exists
    if GRAPHICS.lock().is_none() {
        let mut graphics = GraphicsContext::new();
        graphics.init_mode_12h();
        *GRAPHICS.lock() = Some(graphics);
    } else if let Some(ref mut gfx) = GRAPHICS.lock().as_mut() {
        gfx.init_mode_12h();
    }
    // Update mode state
    *CURRENT_MODE.lock() = VgaMode::Graphics;
}

/// Get current VGA mode
pub fn current_mode() -> VgaMode {
    *CURRENT_MODE.lock()
}

/// Get mutable reference to graphics context
pub fn with_graphics<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut GraphicsContext) -> R,
{
    GRAPHICS.lock().as_mut().map(f)
}

