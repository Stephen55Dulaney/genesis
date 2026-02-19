//! Archimedes - The Daily Ambition Agent
//!
//! Archimedes helps co-create daily ambitions through conversation.
//! Two personas:
//! - Voice Archimedes: Conversational partner (future: voice integration)
//! - Silent Archimedes: Background processor that generates structured documents
//!
//! During Agent-First Boot:
//! - Loads today's ambition from storage
//! - Organizes desktop around the ambition
//! - Prepares workspace aligned with purpose

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use super::{Agent, AgentId, AgentState, AgentContext};
use super::message::{Message, MessageKind, FeedbackType, SystemEvent};
use super::prompts::character_ids;
use super::prompts::library::with_library;
use crate::serial_println;
use crate::storage::filesystem;

/// Archimedes - The Daily Ambition Agent
#[derive(Debug)]
pub struct Archimedes {
    /// Agent ID
    id: AgentId,
    /// Current state
    state: AgentState,
    /// Character ID for prompt lookup
    character_id: u32,
    /// Today's ambition statement
    today_ambition: Option<String>,
    /// Ambition commitments
    commitments: Vec<String>,
    /// Workspace folders created
    workspace_folders: Vec<String>,
    /// Counter for periodic memory theme scanning
    memory_scan_counter: u64,
    /// Whether boot continuity check has been performed
    boot_memory_checked: bool,
    /// Whether current ambition has been saved to memory
    ambition_saved: bool,
}

impl Archimedes {
    /// Create a new Archimedes agent
    pub fn new(id: AgentId) -> Self {
        serial_println!("[ARCHIMEDES] Creating Archimedes the Daily Ambition Agent...");
        
        // Get Archimedes's prompt from the library
        let cert_badge = with_library(|lib| {
            lib.get_active(character_ids::ARCHIMEDES_VOICE)
                .map(|p| p.certification.badge())
                .unwrap_or("⬜")
        }).unwrap_or("⬜");
        
        serial_println!("[ARCHIMEDES] Certification: {} Archimedes", cert_badge);
        
        Archimedes {
            id,
            state: AgentState::Initializing,
            character_id: character_ids::ARCHIMEDES_VOICE,
            today_ambition: None,
            commitments: Vec::new(),
            workspace_folders: Vec::new(),
            memory_scan_counter: 0,
            boot_memory_checked: false,
            ambition_saved: false,
        }
    }
    
    /// Load today's ambition from storage
    fn load_today_ambition(&mut self) {
        
        // Try to load today's ambition file
        // Format: /storage/agents/archimedes/daily_ambitions/YYYY-MM-DD.txt
        // For now, we'll use a simple date format
        let today_path = "/storage/agents/archimedes/daily_ambitions/today.txt";
        
        if let Ok(content) = filesystem::read_file_string(today_path) {
            serial_println!("[ARCHIMEDES] Loaded today's ambition from storage");
            // Parse commitments (simple parsing for now)
            self.parse_ambition(&content);
            self.today_ambition = Some(content);
        } else {
            serial_println!("[ARCHIMEDES] No ambition file found - will create new one");
            // Create default ambition
            self.today_ambition = Some(String::from("Today, I want us to build something amazing together."));
            self.commitments = vec![
                String::from("YOU: Set clear goals"),
                String::from("AI: Support with tools and insights"),
                String::from("COLLAB: Work together"),
            ];
        }
    }
    
    /// Parse ambition document to extract commitments
    fn parse_ambition(&mut self, content: &str) {
        self.commitments.clear();
        
        // Simple parsing: look for "Key Commitments" section
        let lines: Vec<&str> = content.lines().collect();
        let mut in_commitments = false;
        
        for line in lines {
            if line.contains("Key Commitments") || line.contains("Commitments") {
                in_commitments = true;
                continue;
            }
            
            if in_commitments && line.starts_with("-") {
                let commitment = line.trim_start_matches("-").trim().to_string();
                if !commitment.is_empty() {
                    self.commitments.push(commitment);
                }
            }
            
            // Stop at next section
            if in_commitments && (line.starts_with("#") || line.starts_with("##")) {
                break;
            }
        }
    }
    
    /// Create workspace folders aligned with ambition
    fn create_workspace_folders(&mut self) {
        serial_println!("[ARCHIMEDES] Creating workspace folders aligned with ambition...");
        
        // Create today's workspace structure
        let folders = vec![
            "/workspaces/today",
            "/workspaces/today/focus",
            "/workspaces/today/resources",
            "/workspaces/today/output",
        ];
        
        for folder in &folders {
            if let Err(_) = filesystem::create_dir(folder) {
                // Directory might already exist, that's okay
            }
            self.workspace_folders.push((*folder).to_string());
        }
        
        serial_println!("[ARCHIMEDES] Workspace folders created: {} folders", folders.len());
    }
    
    /// Prepare desktop layout based on ambition
    fn prepare_desktop_layout(&self) -> String {
        // Return layout description for supervisor
        // In full implementation, this would return DesktopLayout struct
        format!("Focus zone with ambition: \"{}\"", 
            self.today_ambition.as_ref().unwrap_or(&String::from("No ambition set")))
    }
}

impl Agent for Archimedes {
    fn id(&self) -> AgentId {
        self.id
    }
    
    fn name(&self) -> &str {
        "Archimedes"
    }
    
    fn state(&self) -> AgentState {
        self.state
    }
    
    fn init(&mut self) {
        serial_println!("[ARCHIMEDES] Initializing Daily Ambition Agent...");
        self.state = AgentState::Initializing;
        
        // Load today's ambition
        self.load_today_ambition();
        
        // Show ambition if loaded
        if let Some(ref ambition) = self.today_ambition {
            serial_println!("[ARCHIMEDES] Today's Ambition: \"{}\"", ambition);
            if !self.commitments.is_empty() {
                serial_println!("[ARCHIMEDES] Commitments: {} items", self.commitments.len());
            }
        } else {
            serial_println!("[ARCHIMEDES] No ambition loaded - ready to create new one");
        }
        
        self.state = AgentState::Ready;
        serial_println!("[ARCHIMEDES] Ready! \"What do we want to accomplish today?\"");
    }
    
    fn tick(&mut self, ctx: &mut AgentContext) -> AgentState {
        // Process messages
        for msg in ctx.inbox.iter() {
            self.receive(msg);

            // Handle environment setup
            if let MessageKind::SystemEvent(SystemEvent::EnvironmentSetup) = &msg.kind {
                serial_println!("[ARCHIMEDES] Environment setup: Organizing workspace...");

                // Create workspace folders
                self.create_workspace_folders();

                // Send feedback about workspace preparation
                let feedback = Message::new(
                    self.id,
                    None, // To supervisor
                    MessageKind::Feedback(FeedbackType::Resource {
                        description: format!("Workspace prepared with {} folders", self.workspace_folders.len()),
                        location: String::from("/workspaces/today"),
                    }),
                );
                ctx.outbox.push(feedback);

                serial_println!("[ARCHIMEDES] Workspace organized around ambition!");
            }

            // Listen for Heartbeat (ambition DNA)
            if let MessageKind::Heartbeat(ref ambition) = &msg.kind {
                serial_println!("[ARCHIMEDES] Received heartbeat: \"{}\"", ambition);
                // Update today's ambition if it changed
                if self.today_ambition.as_ref() != Some(ambition) {
                    self.today_ambition = Some(ambition.clone());
                    self.parse_ambition(ambition);
                    serial_println!("[ARCHIMEDES] Ambition updated from heartbeat");
                    // Reset ambition_saved so the new ambition gets persisted
                    self.ambition_saved = false;
                    // Search memory for related past insights
                    let first_word = ambition.split_whitespace().next().unwrap_or("ambition");
                    let search = Message::new(
                        self.id,
                        None,
                        MessageKind::MemorySearch {
                            query: String::from(first_word),
                        },
                    );
                    ctx.outbox.push(search);
                    serial_println!("[ARCHIMEDES] Searching memory for past insights related to: {}", first_word);
                }
            }

            // Handle MemoryResults — surface connections from past insights
            if let MessageKind::MemoryResults { ref results } = &msg.kind {
                if results.len() >= 2 {
                    let first = &results[0].1;
                    let second = &results[1].1;
                    let connection = Message::new(
                        self.id,
                        None,
                        MessageKind::Feedback(FeedbackType::Connection {
                            from: first.clone(),
                            to: second.clone(),
                            pattern: String::from("Archimedes found a link between past insights"),
                        }),
                    );
                    ctx.outbox.push(connection);
                    serial_println!("[ARCHIMEDES] Found connection between {} past entries", results.len());
                    serial_println!("[NOTIFY] Archimedes found a connection between {} past insights", results.len());
                }
            }
        }

        // === Proactive behaviors ===

        // Boot continuity: search memory for previous session's ambition (once)
        if !self.boot_memory_checked {
            self.boot_memory_checked = true;
            serial_println!("[ARCHIMEDES] Searching memory for previous session's ambition...");
            let search = Message::new(
                self.id,
                None,
                MessageKind::MemorySearch {
                    query: String::from("ambition today accomplish"),
                },
            );
            ctx.outbox.push(search);
        }

        // Save current ambition to memory (once per ambition)
        if !self.ambition_saved {
            if let Some(ref ambition) = self.today_ambition {
                self.ambition_saved = true;
                let store = Message::new(
                    self.id,
                    None,
                    MessageKind::MemoryStore {
                        content: format!("Ambition: {}", ambition),
                        kind: String::from("observation"),
                    },
                );
                ctx.outbox.push(store);
                serial_println!("[ARCHIMEDES] Saved current ambition to memory store");
                serial_println!("[NOTIFY] Archimedes saved ambition: {}", ambition);
            }
        }

        // Theme scan: search memory for keywords from current ambition (every 2000 ticks)
        self.memory_scan_counter += 1;
        if self.memory_scan_counter >= 2000 {
            self.memory_scan_counter = 0;
            if let Some(ref ambition) = self.today_ambition {
                // Extract a keyword from the ambition for theme searching
                let keyword = ambition.split_whitespace()
                    .filter(|w| w.len() > 3)
                    .next()
                    .unwrap_or("ambition");
                serial_println!("[ARCHIMEDES] Scanning memory for themes related to: {}", keyword);
                let search = Message::new(
                    self.id,
                    None,
                    MessageKind::MemorySearch {
                        query: String::from(keyword),
                    },
                );
                ctx.outbox.push(search);
            }
        }

        self.state
    }
    
    fn receive(&mut self, msg: &Message) {
        match &msg.kind {
            MessageKind::SystemEvent(SystemEvent::EnvironmentSetup) => {
                // Handled in tick()
            }
            MessageKind::Heartbeat(_) => {
                // Handled in tick()
            }
            MessageKind::MemoryResults { .. } => {
                // Handled in tick()
            }
            _ => {
                serial_println!("[ARCHIMEDES] Received: {:?}", msg.kind);
            }
        }
    }
    
    fn shutdown(&mut self) {
        serial_println!("[ARCHIMEDES] Shutting down...");
        self.state = AgentState::ShuttingDown;
    }
    
    // Daily Rhythm
    
    fn daily_ambition(&mut self) -> Vec<String> {
        if let Some(ref ambition) = self.today_ambition {
            vec![ambition.clone()]
        } else {
            vec![String::from("Co-create today's ambition with human partner")]
        }
    }
    
    fn checkpoint(&self) -> Vec<String> {
        vec![
            format!("Ambition loaded: {}", if self.today_ambition.is_some() { "Yes" } else { "No" }),
            format!("Workspace folders: {}", self.workspace_folders.len()),
            format!("Commitments: {}", self.commitments.len()),
        ]
    }
    
    fn eod_report(&self) -> Vec<String> {
        vec![
            format!("Today's ambition: {}", 
                self.today_ambition.as_ref().unwrap_or(&String::from("Not set"))),
            format!("Workspace organized: {} folders", self.workspace_folders.len()),
            format!("Commitments tracked: {}", self.commitments.len()),
        ]
    }
    
    fn reflect(&mut self) {
        serial_println!("[ARCHIMEDES] Reflecting on the day...");
        if let Some(ref ambition) = self.today_ambition {
            serial_println!("[ARCHIMEDES] Today we aimed to: {}", ambition);
        }
    }
    
    // Genesis Protocol
    
    fn imprint(&mut self, ambition: &str) {
        serial_println!("[ARCHIMEDES] Imprinting with ambition DNA...");
        self.today_ambition = Some(String::from(ambition));
        self.parse_ambition(ambition);
        serial_println!("[ARCHIMEDES] \"What do we want to accomplish today?\" - I'm ready to co-create!");
    }
    
    fn clarify_role(&mut self) -> &str {
        "Co-Creator" // Archimedes co-creates ambitions with humans
    }
    
    fn handle_environment_setup(&mut self, _ctx: &mut AgentContext) {
        serial_println!("[ARCHIMEDES] Environment setup: Organizing workspace around ambition...");
        
        // Load ambition if not already loaded
        if self.today_ambition.is_none() {
            self.load_today_ambition();
        }
        
        // Create workspace folders
        self.create_workspace_folders();
        
        serial_println!("[ARCHIMEDES] Desktop layout prepared around ambition");
    }
}

impl Archimedes {
    /// Get today's ambition (for desktop rendering)
    pub fn get_ambition(&self) -> Option<&String> {
        self.today_ambition.as_ref()
    }
    
    /// Get commitments (for desktop rendering)
    pub fn get_commitments(&self) -> &[String] {
        &self.commitments
    }
}

