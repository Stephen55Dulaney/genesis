//! Interactive Shell for Genesis
//! 
//! This module provides a simple command-line interface for interacting
//! with the Genesis kernel and its agents.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::{print, println};
use crate::agents::supervisor::Supervisor;
use spin::{Mutex, Lazy};
use crossbeam_queue::ArrayQueue;

/// Print to both VGA text mode and serial (so bridge can see it)
/// Text mode is reliable and allocation-free for input/output
macro_rules! shell_print {
    () => {
        {
            println!();
            serial_println!();
        }
    };
    ($($arg:tt)*) => {
        {
            println!($($arg)*);
            serial_println!($($arg)*);
        }
    };
}

// Unused for now - keeping for future use
#[allow(unused_macros)]
/// Print without newline to both VGA and serial
macro_rules! shell_print_no_nl {
    ($($arg:tt)*) => {
        print!($($arg)*);
        crate::serial_print!($($arg)*);
    };
}

/// The maximum length of a command line
const MAX_COMMAND_LEN: usize = 128;

/// A queue for incoming characters from interrupts (keyboard/serial)
pub static INPUT_QUEUE: Lazy<ArrayQueue<char>> = Lazy::new(|| ArrayQueue::new(128));

lazy_static::lazy_static! {
    /// Global shell instance
    pub static ref SHELL: Mutex<Shell> = Mutex::new(Shell::new());
}

/// The state of the shell
pub struct Shell {
    /// Input buffer for the current command
    buffer: String,
    /// Prompt string
    prompt: &'static str,
}

impl Shell {
    /// Create a new shell
    pub fn new() -> Self {
        Shell {
            buffer: String::with_capacity(MAX_COMMAND_LEN),
            prompt: "genesis> ",
        }
    }

    /// Initialize the shell and print the first prompt
    pub fn init(&self) {
        use crate::serial_println;
        
        println!();
        println!("=========================================");
        println!("  GENESIS INTERACTIVE SHELL [READY]");
        println!("  Type 'help' for commands");
        println!("=========================================");
        print!("{}", self.prompt);
        
        // Also print to serial so bridge can see it
        serial_println!();
        serial_println!("=========================================");
        serial_println!("  GENESIS INTERACTIVE SHELL [READY]");
        serial_println!("  Type 'help' for commands");
        serial_println!("=========================================");
        crate::serial_print!("{}", self.prompt);
    }

    /// Add a character to the input queue (safe to call from interrupts)
    pub fn push_char(c: char) {
        if let Err(_) = INPUT_QUEUE.push(c) {
            // Queue is full, ignore for now
        }
    }

    /// Process all characters in the queue
    pub fn process_input(&mut self, supervisor: &mut Supervisor) {
        while let Some(c) = INPUT_QUEUE.pop() {
            self.handle_char(c, supervisor);
        }
    }

    /// Handle a single character input
    fn handle_char(&mut self, c: char, supervisor: &mut Supervisor) {
        match c {
            '\n' | '\r' => {
                use crate::serial_println;
                
                println!(); // New line on screen
                serial_println!(); // Also to serial
                
                // Check if this is a bridge response (not a command)
                if self.buffer.starts_with("[LLM_RESPONSE]") {
                    // Display the response instead of executing it
                    let response = self.buffer.strip_prefix("[LLM_RESPONSE]").unwrap_or(&self.buffer).trim();
                    // shell_print! macro already adds to console, so no need to add again
                    shell_print!("{}", response);
                } else {
                    // Normal command execution
                    self.execute_command(supervisor);
                }
                
                self.buffer.clear();
                print!("{}", self.prompt);
                crate::serial_print!("{}", self.prompt); // Also to serial
            }
            '\u{08}' | '\u{7f}' => {
                use crate::serial_print;
                // Handle backspace
                if !self.buffer.is_empty() {
                    self.buffer.pop();
                    // Move cursor back, print space, move cursor back again (on both VGA and serial)
                    print!("\u{08} \u{08}");
                    serial_print!("\u{08} \u{08}"); // Also update serial output
                    
                    // Text mode input - no graphics console needed
                }
            }
            _ => {
                if self.buffer.len() < MAX_COMMAND_LEN {
                    self.buffer.push(c);
                    print!("{}", c);
                    crate::serial_print!("{}", c); // Also to serial
                    
                    // Text mode input - reliable, no allocations
                }
            }
        }
    }

    /// Execute the command currently in the buffer
    fn execute_command(&mut self, supervisor: &mut Supervisor) {
        use crate::serial_println;
        
        let cmd = self.buffer.trim();
        if cmd.is_empty() {
            return;
        }

        serial_println!("[SHELL] Executing: {}", cmd);

        match cmd {
            "help" => {
                shell_print!("Available commands:");
                shell_print!("  help      - Show this help message");
                shell_print!("  clear     - Clear the screen");
                shell_print!("  status    - Show agent status");
                shell_print!("  academy   - Show Academy certifications");
                shell_print!("  ping      - Ping all agents");
                shell_print!("  ambition  - Trigger morning ambitions");
                shell_print!("  report    - Trigger end-of-day report");
                shell_print!("  thomas    - Talk to Thomas specifically");
                shell_print!("  whoami    - Show current user info");
                shell_print!("  breathe [text] - Set the living ambition (the soul)");
                shell_print!("  heartbeat - View current ambition pulse");
                shell_print!("  insights  - View collected Sparks and Connections");
                shell_print!("  scout video [path] - Request video analysis (via bridge)");
                shell_print!("  test      - Trigger Thomas to run tests and send a Spark");
                shell_print!("  haiku     - Ask TypeWrite to generate a haiku (tests LLM connection)");
                shell_print!("  graphics  - Test graphics rendering (draw test pattern)");
                shell_print!("  archimedes - Talk to Archimedes (Daily Ambition Agent)");
                shell_print!("  desktop   - Show split-screen desktop (Conversation + Ambition)");
                shell_print!("  mode      - Switch VGA mode (text/graphics) or show current mode");
                shell_print!("  F1 or Esc - Toggle between text and graphics mode (keyboard shortcut)");
                shell_print!("  F11       - Show fullscreen exit instructions");
                shell_print!();
                shell_print!("QEMU Fullscreen: Press Ctrl+Alt+F (or Ctrl+Alt+G) to exit");
                shell_print!("                 Or Ctrl+A, X to quit QEMU");
                shell_print!();
                if crate::gui::graphics::current_mode() == crate::gui::graphics::VgaMode::Graphics {
                    shell_print!("Note: In graphics mode, you can see and type commands");
                    shell_print!("      in the console overlay at the bottom of the screen!");
                }
            }
            "test" => {
                shell_print!("Triggering Thomas to run tests...");
                // Send a request message to Thomas via broadcast
                supervisor.broadcast(crate::agents::message::MessageKind::Request {
                    action: String::from("run_tests"),
                    params: Vec::new(),
                });
                shell_print!("Test request sent. Run 'insights' to see the Spark!");
            }
            "haiku" => {
                shell_print!("Asking TypeWrite to generate a haiku...");
                shell_print!("(Sending request to Serial Bridge for Gemini processing)");
                // Send to serial bridge - it will detect this and call Gemini
                serial_println!("[LLM_REQUEST] TypeWrite haiku request");
            }
            "graphics" => {
                shell_print!("Drawing graphics test pattern...");
                crate::gui::graphics::with_graphics(|gfx| {
                    gfx.draw_test_pattern();
                    gfx.swap_buffers();
                });
                shell_print!("Graphics test pattern drawn!");
                shell_print!("(Check QEMU display window to see graphics)");
            }
            "archimedes" => {
                shell_print!("=== ARCHIMEDES - Daily Ambition Agent ===");
                shell_print!();
                shell_print!("Archimedes is the Co-Creator at the Agent Alliance Academy.");
                shell_print!("Two personas:");
                shell_print!("  - Voice Archimedes: Conversational partner");
                shell_print!("  - Silent Archimedes: Generates ambition documents");
                shell_print!();
                shell_print!("Daily Ambition Philosophy:");
                shell_print!("  \"What do WE want to accomplish today?\"");
                shell_print!("  Focus on hopes and dreams, not problems.");
                shell_print!("  Use collaborative framing: we, us, together.");
                shell_print!();
                if let Some(ambition) = supervisor.get_ambition() {
                    shell_print!("Today's Ambition: \"{}\"", ambition);
                } else {
                    shell_print!("No ambition set. Use 'breathe [text]' to set one.");
                }
                shell_print!();
                shell_print!("Use 'desktop' to see split-screen layout.");
            }
            "desktop" => {
                shell_print!("Rendering split-screen desktop...");
                shell_print!("  Left: Conversation (Voice Archimedes)");
                shell_print!("  Right: Ambition Statement (Silent Archimedes)");
                shell_print!();
                
                // Get current ambition for display
                let default_ambition = String::from("Today, I want us to build something amazing together.");
                let ambition = supervisor.get_ambition()
                    .unwrap_or(&default_ambition);
                
                // Create commitments from supervisor insights or defaults
                let commitments = vec![
                    String::from("YOU: Set clear goals"),
                    String::from("AI: Support with tools and insights"),
                    String::from("COLLAB: Work together"),
                ];
                
                // Render desktop layout
                use crate::gui::desktop;
                desktop::with_desktop(|layout| {
                    layout.create_ambition_layout(Some(&ambition), &commitments);
                });
                desktop::render();
                
                shell_print!("Desktop rendered! Check QEMU graphics window.");
                shell_print!("(Note: Graphics window is separate from this terminal)");
            }
            "clear" => {
                crate::vga_buffer::clear_screen();
            }
            "status" => {
                supervisor.print_status();
                shell_print!("Agents active: {}", supervisor.agent_count());
            }
            "academy" => {
                supervisor.print_academy_status();
            }
            "ping" => {
                shell_print!("Pinging all agents...");
                supervisor.broadcast(crate::agents::message::MessageKind::Ping);
                // Wait a moment for responses, then check for pongs
                // Note: In a real system, this would be async. For now, we'll check on next tick.
                shell_print!("(Responses will appear as agents process messages)");
            }
            "ambition" => {
                supervisor.morning_ambition();
            }
            "report" => {
                supervisor.eod_report();
            }
            "thomas" => {
                shell_print!("Thomas is a 🟢 Rookie at the Agent Alliance Academy.");
                shell_print!("His motto is: \"Trust, but verify.\"");
                if let Some(_prompt) = supervisor.get_prompt(crate::agents::prompts::character_ids::THOMAS) {
                    serial_println!("[SHELL] Thomas's full prompt sent to bridge.");
                    shell_print!("(Full prompt sent to Serial Bridge for LLM processing)");
                }
            }
            "whoami" => {
                shell_print!("User: stephendulaney");
                shell_print!("Role: Genesis Architect");
                shell_print!("Location: QuantumDynamX Lab");
            }
            "heartbeat" => {
                if let Some(ambition) = supervisor.get_ambition() {
                    shell_print!("Current Living Ambition (the soul):");
                    shell_print!("  \"{}\"", ambition);
                    shell_print!();
                    shell_print!("Heartbeat pulsing every ~100 ticks");
                } else {
                    shell_print!("No living ambition set yet.");
                    shell_print!("Use 'breathe [ambition]' to set the soul of Genesis.");
                }
            }
            "insights" => {
                let insights = supervisor.get_insights();
                if insights.is_empty() {
                    shell_print!("No insights collected yet.");
                    shell_print!("Agents will send Sparks and Connections as they work.");
                } else {
                    shell_print!("Constellation of Insights ({} total):", insights.len());
                    shell_print!();
                    
                    let mut spark_count = 0;
                    let mut connection_count = 0;
                    let mut resource_count = 0;
                    let mut feeling_count = 0;
                    
                    for (i, insight) in insights.iter().enumerate() {
                        match insight {
                            crate::agents::message::FeedbackType::Spark { content, context } => {
                                spark_count += 1;
                                shell_print!("  [{:3}] ✨ SPARK", i + 1);
                                shell_print!("       Content: {}", content);
                                shell_print!("       Context: {}", context);
                            }
                            crate::agents::message::FeedbackType::Connection { from, to, pattern } => {
                                connection_count += 1;
                                shell_print!("  [{:3}] 🔗 CONNECTION", i + 1);
                                shell_print!("       From: {}", from);
                                shell_print!("       To: {}", to);
                                shell_print!("       Pattern: {}", pattern);
                            }
                            crate::agents::message::FeedbackType::Resource { description, location } => {
                                resource_count += 1;
                                shell_print!("  [{:3}] 📚 RESOURCE", i + 1);
                                shell_print!("       Description: {}", description);
                                shell_print!("       Location: {}", location);
                            }
                            crate::agents::message::FeedbackType::Feeling { tag, intensity } => {
                                feeling_count += 1;
                                shell_print!("  [{:3}] 💭 FEELING", i + 1);
                                shell_print!("       Tag: {}", tag);
                                shell_print!("       Intensity: {}%", intensity);
                            }
                        }
                        shell_print!();
                    }
                    
                    shell_print!("Summary:");
                    shell_print!("  Sparks: {}", spark_count);
                    shell_print!("  Connections: {}", connection_count);
                    shell_print!("  Resources: {}", resource_count);
                    shell_print!("  Feelings: {}", feeling_count);
                }
            }
            "mode" => {
                let current = crate::gui::graphics::current_mode();
                shell_print!("Current VGA mode: {:?}", current);
                shell_print!("Press F1 to toggle, or use 'mode text' / 'mode graphics'");
            }
            _ => {
                // Check if it's a "mode" command with argument
                if cmd.starts_with("mode ") {
                    let mode_arg = cmd.strip_prefix("mode ").unwrap_or("").trim();
                    match mode_arg {
                        "text" => {
                            unsafe {
                                crate::gui::graphics::switch_to_text_mode();
                            }
                            shell_print!("Switched to TEXT mode");
                            shell_print!("(VGA text buffer at 0xB8000)");
                        }
                        "graphics" | "gfx" => {
                            unsafe {
                                crate::gui::graphics::switch_to_graphics_mode();
                            }
                            shell_print!("Switched to GRAPHICS mode (Mode 13h)");
                            shell_print!("(Framebuffer at 0xA0000, 320x200)");
                        }
                        _ => {
                            shell_print!("Usage: mode [text|graphics]");
                            shell_print!("  mode text      - Switch to text mode");
                            shell_print!("  mode graphics  - Switch to graphics mode");
                            shell_print!("  mode           - Show current mode");
                        }
                    }
                } else if cmd.starts_with("breathe ") {
                    let ambition = cmd.strip_prefix("breathe ").unwrap_or("").trim();
                    if ambition.is_empty() {
                        shell_print!("Usage: breathe [your ambition text]");
                        shell_print!("Example: breathe Today I want us to build the graphics system");
                    } else {
                        supervisor.breathe(String::from(ambition));
                    }
                } else if cmd.starts_with("scout video ") {
                    // Scout video analysis request
                    let video_path = cmd.strip_prefix("scout video ").unwrap_or("").trim();
                    if video_path.is_empty() {
                        shell_print!("Usage: scout video [path/to/video.mp4]");
                        shell_print!("Example: scout video /Users/stephendulaney/Desktop/quantum-videos/2026-01-23\\ 12-43-19.mp4");
                    } else {
                        shell_print!("Requesting video analysis from Scout...");
                        shell_print!("(Sending request to Serial Bridge for Gemini processing)");
                        // Send to serial bridge
                        serial_println!("[SCOUT] Video analysis requested: {}", video_path);
                    }
                } else {
                    shell_print!("Unknown command: {}", cmd);
                    shell_print!("Type 'help' for a list of commands.");
                }
            }
        }
    }
}
