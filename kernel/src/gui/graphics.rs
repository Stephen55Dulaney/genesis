//! VGA Graphics Mode Driver
//!
//! Provides pixel-level graphics for Genesis OS.
//! Uses VGA Mode 13h: 320x200 pixels, 256 colors.
//!
//! Inspired by Bevy's simple API, adapted for bare-metal.

extern crate alloc;

use core::ptr;
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;

/// VGA Mode 13h: 320x200, 256 colors
pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 200;
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
    
    /// Initialize VGA Mode 13h
    /// 
    /// This sets up the VGA hardware for graphics mode.
    /// Mode 13h: 320x200 pixels, 256 colors, linear framebuffer at 0xA0000
    /// 
    /// # Safety
    /// Direct hardware access - must be called during kernel init.
    pub unsafe fn init_mode_13h(&mut self) {
        // Actually switch to VGA Mode 13h by programming hardware registers
        switch_to_mode_13h();
        
        // Clear the framebuffer
        self.clear(Color::Black);
        
        // Draw a test pattern to verify graphics mode is working
        self.draw_test_pattern();
    }
    
    /// Draw a test pattern to verify graphics is working
    pub fn draw_test_pattern(&mut self) {
        // Clear screen
        self.clear(Color::Black);
        
        // Draw colored rectangles in corners
        self.draw_rect(0, 0, 50, 50, Color::Red as u8);
        self.draw_rect((self.width - 50) as u32, 0, 50, 50, Color::Green as u8);
        self.draw_rect(0, (self.height - 50) as u32, 50, 50, Color::Blue as u8);
        self.draw_rect((self.width - 50) as u32, (self.height - 50) as u32, 50, 50, Color::Yellow as u8);
        
        // Draw a centered rectangle
        let center_x = self.width / 2 - 40;
        let center_y = self.height / 2 - 20;
        self.draw_rect_outline(center_x, center_y, 80, 40, Color::Cyan as u8);
        
        // Draw text
        self.draw_text(10, 60, "GENESIS", Color::White as u8);
        self.draw_text(10, 70, "Graphics Mode Active", Color::LightCyan as u8);
        self.draw_text(10, 80, "Milestone 6: Graphics Foundation", Color::LightGreen as u8);
    }
    
    /// Draw a single pixel
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let offset = (y * self.width + x) as usize;
        unsafe {
            let buffer = if self.back_buffer.is_null() {
                self.front_buffer
            } else {
                self.back_buffer
            };
            *buffer.add(offset) = color;
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
        let buffer = if self.back_buffer.is_null() {
            self.front_buffer
        } else {
            self.back_buffer
        };
        
        unsafe {
            ptr::write_bytes(buffer, self.clear_color, (self.width * self.height) as usize);
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
    /// Uses a simple 8x8 pixel font (each character is 8 pixels wide)
    pub fn draw_text(&mut self, x: u32, y: u32, text: &str, color: u8) {
        let mut cursor_x = x;
        let mut cursor_y = y;
        
        for ch in text.chars() {
            match ch {
                '\n' => {
                    cursor_x = x;
                    cursor_y += 8; // Line height
                }
                '\r' => {
                    cursor_x = x;
                }
                _ => {
                    self.draw_char(cursor_x, cursor_y, ch, color);
                    cursor_x += 8; // Character width
                    
                    // Wrap if we go off screen
                    if cursor_x + 8 > self.width {
                        cursor_x = x;
                        cursor_y += 8;
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
fn get_char_bitmap(ch: char) -> [u8; 8] {
    // Simple 8x8 bitmap font for ASCII characters
    // This is a minimal implementation - a full font would have all 256 characters
    match ch {
        'A'..='Z' | 'a'..='z' | '0'..='9' | ' ' | '.' | ':' | '-' | '_' | '!' | '?' => {
            // For now, return a simple pattern
            // In a full implementation, we'd have a complete font table
            simple_char_bitmap(ch)
        }
        _ => [0; 8], // Unknown character - blank
    }
}

/// Simple bitmap font generator (placeholder)
/// In a full implementation, this would use a proper font table
fn simple_char_bitmap(ch: char) -> [u8; 8] {
    // Very simple placeholder font - just draws a box for most characters
    // A real font would have proper glyphs for each character
    match ch {
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // Space
        ':' => [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00], // Colon
        '-' => [0x00, 0x00, 0x00, 0x7E, 0x7E, 0x00, 0x00, 0x00], // Dash
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF], // Underscore
        '!' => [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00], // Exclamation
        '?' => [0x3C, 0x66, 0x0C, 0x18, 0x18, 0x00, 0x18, 0x00], // Question mark
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00], // Period
        _ => {
            // Default: simple box pattern for letters/numbers
            [0x7E, 0x81, 0x81, 0x81, 0x81, 0x81, 0x7E, 0x00]
        }
    }
}

/// Switch VGA to Mode 13h (320x200, 256 colors)
/// 
/// This programs all the VGA registers needed for graphics mode.
/// # Safety
/// Direct hardware access - only call during initialization.
unsafe fn switch_to_mode_13h() {
    // VGA Mode 13h register values
    // These are the standard values documented for Mode 13h
    
    // Miscellaneous Output Register
    let mut misc_port: Port<u8> = Port::new(0x3C2);
    misc_port.write(0x63);
    
    // Sequencer registers (0x3C4/0x3C5)
    let mut seq_index: Port<u8> = Port::new(0x3C4);
    let mut seq_data: Port<u8> = Port::new(0x3C5);
    
    // Sequencer values for Mode 13h
    let seq_regs: [(u8, u8); 5] = [
        (0x00, 0x03), // Reset register
        (0x01, 0x01), // Clocking mode
        (0x02, 0x0F), // Map mask
        (0x03, 0x00), // Character map select
        (0x04, 0x0E), // Memory mode
    ];
    
    for (index, value) in seq_regs.iter() {
        seq_index.write(*index);
        seq_data.write(*value);
    }
    
    // Unlock CRT Controller registers
    let mut crtc_index: Port<u8> = Port::new(0x3D4);
    let mut crtc_data: Port<u8> = Port::new(0x3D5);
    
    crtc_index.write(0x11);
    let val = crtc_data.read();
    crtc_data.write(val & 0x7F);
    
    // CRT Controller values for Mode 13h
    let crtc_regs: [(u8, u8); 25] = [
        (0x00, 0x5F), // Horizontal total
        (0x01, 0x4F), // Horizontal display end
        (0x02, 0x50), // Start horizontal blanking
        (0x03, 0x82), // End horizontal blanking
        (0x04, 0x54), // Start horizontal retrace
        (0x05, 0x80), // End horizontal retrace
        (0x06, 0xBF), // Vertical total
        (0x07, 0x1F), // Overflow
        (0x08, 0x00), // Preset row scan
        (0x09, 0x41), // Maximum scan line
        (0x0A, 0x00), // Cursor start
        (0x0B, 0x00), // Cursor end
        (0x0C, 0x00), // Start address high
        (0x0D, 0x00), // Start address low
        (0x0E, 0x00), // Cursor location high
        (0x0F, 0x00), // Cursor location low
        (0x10, 0x9C), // Vertical retrace start
        (0x11, 0x8E), // Vertical retrace end (and lock bit)
        (0x12, 0x8F), // Vertical display end
        (0x13, 0x28), // Offset
        (0x14, 0x40), // Underline location
        (0x15, 0x96), // Start vertical blanking
        (0x16, 0xB9), // End vertical blanking
        (0x17, 0xA3), // Mode control
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
        (0x05, 0x40), // Graphics mode
        (0x06, 0x05), // Misc graphics
        (0x07, 0x0F), // Color don't care
        (0x08, 0xFF), // Bit mask
    ];
    
    for (index, value) in gc_regs.iter() {
        gc_index.write(*index);
        gc_data.write(*value);
    }
    
    // Attribute Controller registers (0x3C0)
    let mut input_status: Port<u8> = Port::new(0x3DA);
    let mut attr_port: Port<u8> = Port::new(0x3C0);
    
    // Reset attribute controller flip-flop by reading input status
    let _ = input_status.read();
    
    // Attribute Controller palette (first 16 registers)
    for i in 0u8..16 {
        attr_port.write(i);
        attr_port.write(i); // Identity mapping
    }
    
    // Attribute Controller mode registers
    attr_port.write(0x10); attr_port.write(0x41); // Mode control
    attr_port.write(0x11); attr_port.write(0x00); // Overscan
    attr_port.write(0x12); attr_port.write(0x0F); // Color plane enable
    attr_port.write(0x13); attr_port.write(0x00); // Horizontal panning
    attr_port.write(0x14); attr_port.write(0x00); // Color select
    
    // Enable video by setting bit 5
    attr_port.write(0x20);
    
    // Set up a basic 256-color palette
    setup_palette();
}

/// Set up the standard VGA 256-color palette
unsafe fn setup_palette() {
    let mut palette_index: Port<u8> = Port::new(0x3C8);
    let mut palette_data: Port<u8> = Port::new(0x3C9);
    
    palette_index.write(0); // Start at color 0
    
    // Standard VGA colors (first 16)
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
    
    // Fill remaining colors with a grayscale ramp for simplicity
    for i in 16u8..=255 {
        let gray = (i / 4) as u8;
        palette_data.write(gray);
        palette_data.write(gray);
        palette_data.write(gray);
    }
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

/// Initialize graphics system
/// 
/// # Safety
/// Must be called during kernel initialization, after heap is initialized.
pub unsafe fn init() {
    let mut graphics = GraphicsContext::new();
    graphics.init_mode_13h();
    *GRAPHICS.lock() = Some(graphics);
}

/// Get mutable reference to graphics context
pub fn with_graphics<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut GraphicsContext) -> R,
{
    GRAPHICS.lock().as_mut().map(f)
}

