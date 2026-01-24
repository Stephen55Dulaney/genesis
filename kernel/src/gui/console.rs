//! Graphics Console Overlay
//!
//! Provides a visible text console overlay in graphics mode so users can
//! interact with Genesis directly within the GUI, not just the terminal.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;

/// Maximum number of lines in console output history
const MAX_OUTPUT_LINES: usize = 10;

/// Console state for graphics mode overlay
pub struct GraphicsConsole {
    /// Current input buffer (what user is typing)
    input_buffer: String,
    /// Output history (last N lines of command output)
    output_lines: Vec<String>,
    /// Prompt string
    prompt: String,
    /// Console height in pixels
    height: u32,
    /// Console Y position (from bottom)
    y_position: u32,
}

impl GraphicsConsole {
    /// Create a new graphics console
    pub fn new(height: u32) -> Self {
        GraphicsConsole {
            input_buffer: String::new(),
            output_lines: Vec::new(),
            prompt: String::from("genesis> "),
            height,
            y_position: 0, // Will be set based on screen height
        }
    }

    /// Set the Y position (from top of screen)
    pub fn set_y_position(&mut self, y: u32) {
        self.y_position = y;
    }

    /// Get Y position
    pub fn y_position(&self) -> u32 {
        self.y_position
    }

    /// Get height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Update the input buffer (what user is typing)
    pub fn set_input_buffer(&mut self, buffer: &str) {
        self.input_buffer = String::from(buffer);
    }

    /// Add a line to output history
    pub fn add_output_line(&mut self, line: String) {
        self.output_lines.push(line);
        // Keep only last N lines
        if self.output_lines.len() > MAX_OUTPUT_LINES {
            self.output_lines.remove(0);
        }
    }

    /// Clear output history
    pub fn clear_output(&mut self) {
        self.output_lines.clear();
    }

    /// Get current input buffer
    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    /// Get output lines
    pub fn output_lines(&self) -> &[String] {
        &self.output_lines
    }

    /// Get prompt
    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    /// Render the console overlay to graphics
    /// 
    /// This should be called from within a graphics::with_graphics closure
    /// to avoid nested mutex locking.
    pub fn render_to_graphics(&self, gfx: &mut super::graphics::GraphicsContext, screen_width: u32, screen_height: u32) {
        // Console is at the bottom of the screen
        let console_y = screen_height - self.height;
        
        // Draw console background (dark gray for visibility)
        gfx.draw_rect(0, console_y, screen_width, self.height, 8); // Dark gray background (color 8)
        
        // Draw a bright white top border (3 pixels thick for visibility)
        gfx.draw_rect(0, console_y, screen_width, 3, 15); // White top border
        
        // Also draw outline for extra visibility
        gfx.draw_rect_outline(0, console_y, screen_width, self.height, 15); // White border
        
        // Render output lines (above input line)
        let mut y_offset = console_y + 5;
        let line_height = 8;
        
        for line in self.output_lines.iter().rev().take(MAX_OUTPUT_LINES) {
            if y_offset + line_height > console_y + self.height - 12 {
                break; // Don't overlap input line
            }
            gfx.draw_text(5, y_offset, line, 15); // White text
            y_offset += line_height;
        }
        
        // Draw input line at bottom
        let input_y = console_y + self.height - 10;
        let input_text = format!("{}{}", self.prompt, self.input_buffer);
        
        // Truncate input text if too long to fit on screen
        let max_chars = (screen_width / 8).saturating_sub(2); // Leave room for cursor
        let display_text = if input_text.len() > max_chars as usize {
            &input_text[input_text.len().saturating_sub(max_chars as usize)..]
        } else {
            &input_text
        };
        
        // Draw prompt + input (bright cyan for visibility)
        gfx.draw_text(5, input_y, display_text, 11); // Light cyan
        
        // Draw cursor (blinking would require timer, for now just show underscore)
        let cursor_x = 5 + (display_text.len() as u32 * 8); // 8 pixels per char
        gfx.draw_text(cursor_x, input_y, "_", 15); // White cursor
    }
}

/// Global graphics console instance
lazy_static! {
    pub static ref GRAPHICS_CONSOLE: Mutex<Option<GraphicsConsole>> = Mutex::new(None);
}

/// Initialize graphics console
pub fn init(_screen_width: u32, screen_height: u32) {
    let console_height = 120; // 120 pixels tall (increased for 640x480 resolution)
    let mut console = GraphicsConsole::new(console_height);
    console.set_y_position(screen_height - console_height);
    *GRAPHICS_CONSOLE.lock() = Some(console);
}

/// Update console input buffer
pub fn update_input_buffer(buffer: &str) {
    use crate::serial_println;
    serial_println!("[CONSOLE] update_input_buffer called with: '{}'", buffer);
    
    if let Some(ref mut console) = GRAPHICS_CONSOLE.lock().as_mut() {
        console.set_input_buffer(buffer);
        serial_println!("[CONSOLE] Input buffer updated successfully");
    } else {
        serial_println!("[CONSOLE] ERROR: Graphics console not initialized!");
    }
}

/// Add output line to console
pub fn add_output_line(line: String) {
    if let Some(ref mut console) = GRAPHICS_CONSOLE.lock().as_mut() {
        console.add_output_line(line);
    }
}

/// Clear console output
pub fn clear_output() {
    if let Some(ref mut console) = GRAPHICS_CONSOLE.lock().as_mut() {
        console.clear_output();
    }
}

/// Render the console overlay (internal - use render_to_graphics from within graphics context)
fn render_internal(screen_width: u32, screen_height: u32) {
    if let Some(ref console) = GRAPHICS_CONSOLE.lock().as_ref() {
        // This should only be called from within graphics::with_graphics
        use super::graphics;
        graphics::with_graphics(|gfx| {
            console.render_to_graphics(gfx, screen_width, screen_height);
        });
    }
}

/// Render the console overlay (public wrapper)
pub fn render(screen_width: u32, screen_height: u32) {
    render_internal(screen_width, screen_height);
}

/// Render console directly to graphics context (for use within graphics::with_graphics)
pub fn render_to_graphics(gfx: &mut super::graphics::GraphicsContext, screen_width: u32, screen_height: u32) {
    if let Some(ref console) = GRAPHICS_CONSOLE.lock().as_ref() {
        console.render_to_graphics(gfx, screen_width, screen_height);
    }
}

/// Check if console is initialized
pub fn is_initialized() -> bool {
    GRAPHICS_CONSOLE.lock().is_some()
}
