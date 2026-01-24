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
        // Note: In a real bare-metal environment, we'd need to program VGA registers.
        // However, many bootloaders and QEMU may already set up graphics mode.
        // For now, we'll clear the framebuffer and assume Mode 13h is active.
        // In a full implementation, we'd program all VGA registers here.
        
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

