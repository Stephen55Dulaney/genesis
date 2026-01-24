//! Desktop Layout System
//!
//! Agents define zones, supervisor combines them into a desktop layout.
//! The desktop renders:
//! - Left: Conversation/transcript (Voice Archimedes)
//! - Right: Ambition statement (Silent Archimedes)
//! - Bottom: Agent zones (Focus, Resources, etc.)

use alloc::string::String;
use alloc::vec::Vec;

/// A zone on the desktop defined by an agent
#[derive(Debug, Clone)]
pub struct Zone {
    /// Zone name/identifier
    pub name: String,
    /// Agent that owns this zone
    pub agent: String,
    /// X position (pixels)
    pub x: u32,
    /// Y position (pixels)
    pub y: u32,
    /// Width (pixels)
    pub width: u32,
    /// Height (pixels)
    pub height: u32,
    /// Content to display
    pub content: String,
    /// Background color (VGA color index)
    pub bg_color: u8,
    /// Text color (VGA color index)
    pub text_color: u8,
}

/// Desktop layout combining all agent zones
#[derive(Debug)]
pub struct DesktopLayout {
    /// All zones on the desktop
    zones: Vec<Zone>,
    /// Screen width
    width: u32,
    /// Screen height
    height: u32,
}

impl DesktopLayout {
    /// Create a new desktop layout
    pub fn new(width: u32, height: u32) -> Self {
        DesktopLayout {
            zones: Vec::new(),
            width,
            height,
        }
    }
    
    /// Add a zone to the desktop
    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.push(zone);
    }
    
    /// Create default ambition layout (split screen)
    pub fn create_ambition_layout(&mut self, ambition: Option<&str>, commitments: &[String]) {
        // Left zone: Conversation area (for Voice Archimedes)
        let left_zone = Zone {
            name: String::from("Conversation"),
            agent: String::from("Archimedes"),
            x: 0,
            y: 0,
            width: self.width / 2,
            height: self.height,
            content: String::from("CONVERSATION\n\n[Voice Archimedes]\n\nGood morning! What's your ambition for today?\n\n[Transcript will appear here]"),
            bg_color: 0, // Black
            text_color: 11, // Light Cyan
        };
        self.add_zone(left_zone);
        
        // Right zone: Ambition statement (Silent Archimedes)
        let mut right_content = String::from("TODAY'S AMBITION\n\n");
        
        if let Some(ambition) = ambition {
            right_content.push_str("Today's Ambition Statement:\n");
            right_content.push_str("\"");
            right_content.push_str(ambition);
            right_content.push_str("\"\n\n");
        } else {
            right_content.push_str("No ambition set yet.\n\n");
        }
        
        if !commitments.is_empty() {
            right_content.push_str("Key Commitments:\n");
            for commitment in commitments {
                right_content.push_str("- ");
                right_content.push_str(commitment);
                right_content.push_str("\n");
            }
        }
        
        let right_zone = Zone {
            name: String::from("Ambition Statement"),
            agent: String::from("Silent Archimedes"),
            x: self.width / 2,
            y: 0,
            width: self.width / 2,
            height: self.height,
            content: right_content,
            bg_color: 1, // Blue
            text_color: 14, // Yellow
        };
        self.add_zone(right_zone);
    }
    
    /// Get all zones
    pub fn zones(&self) -> &[Zone] {
        &self.zones
    }
    
    /// Render the desktop layout to graphics
    pub fn render(&self) {
        use super::graphics;
        
        graphics::with_graphics(|gfx| {
            // Clear screen
            gfx.clear(graphics::Color::Black);
            
            // Render each zone
            for zone in &self.zones {
                // Draw zone background
                gfx.draw_rect(zone.x, zone.y, zone.width, zone.height, zone.bg_color);
                
                // Draw zone border
                gfx.draw_rect_outline(zone.x, zone.y, zone.width, zone.height, zone.text_color);
                
                // Draw zone content (text)
                // Simple text rendering: split by lines and draw each
                let lines: Vec<&str> = zone.content.lines().collect();
                let mut y_offset = zone.y + 10; // Start 10 pixels from top
                
                for line in lines {
                    if y_offset + 8 > zone.y + zone.height {
                        break; // Don't overflow zone
                    }
                    
                    // Draw text line
                    gfx.draw_text(zone.x + 5, y_offset, line, zone.text_color);
                    y_offset += 8; // Line height
                }
            }
            
            // Swap buffers to display
            gfx.swap_buffers();
        });
    }
}

/// Global desktop layout
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DESKTOP: Mutex<Option<DesktopLayout>> = Mutex::new(None);
}

/// Initialize desktop layout
pub fn init(width: u32, height: u32) {
    *DESKTOP.lock() = Some(DesktopLayout::new(width, height));
}

/// Get mutable reference to desktop layout
pub fn with_desktop<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut DesktopLayout) -> R,
{
    DESKTOP.lock().as_mut().map(f)
}

/// Render the desktop
pub fn render() {
    DESKTOP.lock().as_ref().map(|desktop| desktop.render());
}

