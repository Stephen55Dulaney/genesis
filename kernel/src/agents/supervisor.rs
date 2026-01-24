//! Agent Supervisor - "Sam"
//!
//! The Supervisor orchestrates all agents in the system.
//! Named after Sam from the Agent Alliance Academy.
//!
//! ## Responsibilities
//!
//! - Spawning and tracking agents
//! - Routing messages between agents
//! - Managing the daily rhythm (ambitions, checkpoints, reports)
//! - Handling system-wide events
//! - Managing the Prompt Library (agent intelligence)
//! - Coordinating with the Agent Alliance Academy
//!
//! ## The Tick Loop
//!
//! ```text
//! loop {
//!     1. Collect outgoing messages from all agents
//!     2. Route messages to recipient inboxes
//!     3. Call tick() on each agent
//!     4. Check for system events
//!     5. Sleep until next tick
//! }
//! ```
//!
//! ## Academy Integration
//!
//! Sam is a 🟡 Master at the Agent Alliance Academy.
//! "Greetings, seeker. I am Sam, Orchestrator of the Academy."
//! See: https://as-the-cloud-turns-web.onrender.com/#academy

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use super::{Agent, AgentId, AgentContext};
use super::message::{Message, MessageKind, SystemEvent, FeedbackType};
use super::prompts::{library, evolution, character_ids};
use super::prompts::academy;
use crate::{println, serial_println};

/// The Agent Supervisor - orchestrates all agents
pub struct Supervisor {
    /// All registered agents
    agents: Vec<Box<dyn Agent>>,
    /// Global message queue
    message_queue: Vec<Message>,
    /// Current tick number
    tick: u64,
    /// Next agent ID to assign
    next_agent_id: u64,
    /// Supervisor's own ID (for sending system messages)
    id: AgentId,
    /// The living ambition - the soul of Genesis
    living_ambition: Option<String>,
    /// Heartbeat counter (pulse every N ticks)
    heartbeat_counter: u64,
    /// Constellation of insights collected from agents
    /// Limited to prevent memory exhaustion (keep last 50 insights)
    constellation_of_insights: Vec<FeedbackType>,
    /// Serendipity check counter (scan for connections every N ticks)
    serendipity_counter: u64,
}

impl Supervisor {
    /// Create a new Supervisor
    pub fn new() -> Self {
        serial_println!("[SUPERVISOR] Initializing Agent Supervisor (Sam)...");
        
        // Initialize the Prompt Library
        serial_println!("[SUPERVISOR] Loading Prompt Library...");
        library::init();
        
        // Initialize the Evolution Engine
        serial_println!("[SUPERVISOR] Starting Evolution Engine...");
        evolution::init();
        
        // Initialize the Agent Alliance Academy
        serial_println!("[SUPERVISOR] Connecting to Agent Alliance Academy...");
        academy::init();
        
        // Show library stats
        if let Some(stats) = library::with_library(|lib| lib.stats()) {
            serial_println!("[SUPERVISOR] Prompt Library: {} prompts loaded, {} characters active",
                stats.total_prompts, stats.active_characters);
        }
        
        // Get Sam's certification
        let sam_cert = library::with_library(|lib| {
            lib.get_active(character_ids::SAM)
                .map(|p| (p.certification.badge(), p.certification.name()))
        }).flatten().unwrap_or(("⬜", "Unknown"));
        
        serial_println!("[SUPERVISOR] Sam's Academy Status: {} {}", sam_cert.0, sam_cert.1);
        
        Supervisor {
            agents: Vec::new(),
            message_queue: Vec::new(),
            tick: 0,
            next_agent_id: 1, // Reserve 0 for supervisor
            id: AgentId::new(0),
            living_ambition: None,
            heartbeat_counter: 0,
            constellation_of_insights: Vec::new(),
            serendipity_counter: 0,
        }
    }
    
    /// Register a new agent with the supervisor (Genesis Protocol)
    pub fn register(&mut self, mut agent: Box<dyn Agent>) {
        let name = String::from(agent.name());
        let id = agent.id();
        
        serial_println!("[SUPERVISOR] Registering agent: {} (ID: {:?})", name, id);
        
        // Initialize the agent
        agent.init();
        
        // Genesis Protocol: Imprint with purpose
        if let Some(ref ambition) = self.living_ambition {
            serial_println!("[GENESIS_PROTOCOL] Imprinting {} with ambition DNA...", name);
            agent.imprint(ambition);
        } else {
            serial_println!("[GENESIS_PROTOCOL] No living ambition set - agent will wait for imprint");
        }
        
        // Genesis Protocol: Clarify role
        let role_str = agent.clarify_role();
        let role = String::from(role_str);
        serial_println!("[GENESIS_PROTOCOL] {} clarified role: {}", name, role);
        
        // Genesis Protocol: First Breath
        self.broadcast(MessageKind::FirstBreath {
            agent_name: name.clone(),
            role: role.clone(),
        });
        
        self.agents.push(agent);
        serial_println!("[SUPERVISOR] Agent {} is now ONLINE (role: {})", name, role);
    }
    
    /// Generate a new unique agent ID
    pub fn next_id(&mut self) -> AgentId {
        let id = AgentId::new(self.next_agent_id);
        self.next_agent_id += 1;
        id
    }
    
    /// Send a message (queues it for next tick)
    pub fn send(&mut self, mut msg: Message) {
        msg.timestamp = self.tick;
        self.message_queue.push(msg);
    }
    
    /// Broadcast a message to all agents
    pub fn broadcast(&mut self, kind: MessageKind) {
        let msg = Message::broadcast(self.id, kind);
        self.send(msg);
    }
    
    /// Agent-First Boot Sequence
    /// 
    /// Phase 2: Agents wake up BEFORE GUI appears
    /// Agents organize the environment, then GUI renders the organized desktop
    pub fn agent_boot_sequence(&mut self) {
        serial_println!();
        serial_println!("=========================================");
        serial_println!("  AGENT-FIRST BOOT SEQUENCE");
        serial_println!("=========================================");
        serial_println!();
        serial_println!("[AGENTS] Phase 2: Agent Awakening");
        serial_println!("[AGENTS] Agents waking up in parallel...");
        
        // Agents are already registered, but we trigger their awakening
        // In a full implementation, we'd load agent registry from storage
        // For now, agents wake up when they're registered
        
        serial_println!("[AGENTS] {} agents ready", self.agents.len());
        
        // Wait for all agents to be ready (they're ready after init)
        serial_println!("[AGENTS] All agents ready!");
        
        // Phase 3: Environment Setup
        serial_println!();
        serial_println!("[SETUP] Phase 3: Environment Setup");
        serial_println!("[SETUP] Agents organizing their domains...");
        
        self.trigger_environment_setup();
        
        // Give agents a few ticks to process environment setup
        for _ in 0..3 {
            self.tick();
        }
        
        serial_println!("[SETUP] Environment setup complete!");
        serial_println!("[SETUP] Desktop layout ready for rendering");
    }
    
    /// Trigger environment setup - agents organize before GUI
    fn trigger_environment_setup(&mut self) {
        // Broadcast environment setup event to all agents
        self.broadcast(MessageKind::SystemEvent(SystemEvent::EnvironmentSetup));
        
        // Also call handle_environment_setup directly for each agent
        for agent in self.agents.iter_mut() {
            // Create a context for environment setup
            let mut inbox = Vec::new();
            let mut outbox = Vec::new();
            let mut ctx = AgentContext {
                inbox: &mut inbox,
                outbox: &mut outbox,
                tick: self.tick,
            };
            
            agent.handle_environment_setup(&mut ctx);
            
            // Collect any messages from environment setup
            for msg in outbox {
                self.message_queue.push(msg);
            }
        }
    }
    
    /// Run one tick of the supervisor loop
    pub fn tick(&mut self) {
        self.tick += 1;
        
        // Pulse the heartbeat (every 100 ticks = ~1 second at typical speeds)
        self.heartbeat_counter += 1;
        if self.heartbeat_counter >= 100 {
            self.pulse();
            self.heartbeat_counter = 0;
        }
        
        // Route messages to agents
        let messages: Vec<Message> = self.message_queue.drain(..).collect();
        
        // Collect Feedback messages before routing
        let mut feedback_messages = Vec::new();
        let mut other_messages = Vec::new();
        
        for msg in messages {
            match &msg.kind {
                MessageKind::Feedback(_) => {
                    feedback_messages.push(msg);
                }
                _ => {
                    other_messages.push(msg);
                }
            }
        }
        
        // Store feedback in constellation (limit to 50 to prevent memory exhaustion)
        const MAX_INSIGHTS: usize = 50;
        for msg in feedback_messages {
            if let MessageKind::Feedback(feedback) = msg.kind.clone() {
                self.constellation_of_insights.push(feedback);
                
                // Keep only the most recent insights
                if self.constellation_of_insights.len() > MAX_INSIGHTS {
                    self.constellation_of_insights.remove(0);
                }
            }
        }
        
        // Route other messages to agents
        for agent in self.agents.iter_mut() {
            // Collect messages for this agent
            let mut inbox: Vec<Message> = other_messages
                .iter()
                .filter(|m| m.to.is_none() || m.to == Some(agent.id()))
                .cloned()
                .collect();
            
            let mut outbox: Vec<Message> = Vec::new();
            
            // Create context and tick the agent
            let mut ctx = AgentContext {
                inbox: &mut inbox,
                outbox: &mut outbox,
                tick: self.tick,
            };
            
            let _new_state = agent.tick(&mut ctx);
            
            // Collect outgoing messages
            for mut msg in outbox {
                msg.timestamp = self.tick;
                self.message_queue.push(msg);
            }
        }
        
        // Serendipity Engine: Check for connections (every 500 ticks)
        self.serendipity_counter += 1;
        if self.serendipity_counter >= 500 {
            self.check_serendipity();
            self.serendipity_counter = 0;
        }
    }
    
    /// Pulse the heartbeat - broadcast the living ambition DNA
    fn pulse(&mut self) {
        if let Some(ref ambition) = self.living_ambition {
            serial_println!("[HEARTBEAT] Pulsing ambition DNA to all agents...");
            self.broadcast(MessageKind::Heartbeat(ambition.clone()));
        }
    }
    
    /// Serendipity Engine: Find overlapping themes and connections
    fn check_serendipity(&mut self) {
        if self.constellation_of_insights.len() < 2 {
            return;
        }
        
        // Simple serendipity: look for similar keywords in Sparks
        let mut themes: Vec<(String, usize)> = Vec::new();
        
        for insight in &self.constellation_of_insights {
            if let FeedbackType::Spark { content, .. } = insight {
                // Extract simple keywords (in a real system, this would be more sophisticated)
                let words: Vec<&str> = content.split_whitespace().collect();
                for word in words {
                    if word.len() > 4 {
                        // Count occurrences
                        let count = self.constellation_of_insights.iter()
                            .filter(|i| {
                                if let FeedbackType::Spark { content: c, .. } = i {
                                    c.contains(word)
                                } else {
                                    false
                                }
                            })
                            .count();
                        
                        if count >= 2 {
                            themes.push((String::from(word), count));
                        }
                    }
                }
            }
        }
        
        if !themes.is_empty() {
            serial_println!("[SERENDIPITY] Found overlapping themes:");
            for (theme, count) in themes.iter().take(3) {
                serial_println!("  - '{}' appears in {} insights", theme, count);
            }
        }
    }
    
    /// Trigger morning ambitions for all agents
    pub fn morning_ambition(&mut self) {
        serial_println!("[SUPERVISOR] === MORNING AMBITION ===");
        println!();
        println!("  === DAILY AMBITION ===");
        
        self.broadcast(MessageKind::SystemEvent(SystemEvent::MorningAmbition));
        
        for agent in self.agents.iter_mut() {
            let ambitions = agent.daily_ambition();
            if !ambitions.is_empty() {
                serial_println!("[{}] Ambitions:", agent.name());
                println!("  [{}]", agent.name());
                for ambition in &ambitions {
                    serial_println!("  - {}", ambition);
                    println!("    - {}", ambition);
                }
            }
        }
    }
    
    /// Trigger midday checkpoint for all agents
    pub fn midday_checkpoint(&mut self) {
        serial_println!("[SUPERVISOR] === MIDDAY CHECKPOINT ===");
        
        self.broadcast(MessageKind::SystemEvent(SystemEvent::MiddayCheckpoint));
        
        for agent in self.agents.iter() {
            let progress = agent.checkpoint();
            if !progress.is_empty() {
                serial_println!("[{}] Progress:", agent.name());
                for item in &progress {
                    serial_println!("  - {}", item);
                }
            }
        }
    }
    
    /// Trigger end-of-day report for all agents
    pub fn eod_report(&mut self) {
        serial_println!("[SUPERVISOR] === END OF DAY REPORT ===");
        println!();
        println!("  === EOD REPORT ===");
        
        self.broadcast(MessageKind::SystemEvent(SystemEvent::EndOfDay));
        
        for agent in self.agents.iter() {
            let report = agent.eod_report();
            if !report.is_empty() {
                serial_println!("[{}] Accomplished:", agent.name());
                println!("  [{}]", agent.name());
                for item in &report {
                    serial_println!("  - {}", item);
                    println!("    * {}", item);
                }
            }
        }
    }
    
    /// Get agent count
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
    
    /// Get current tick
    pub fn current_tick(&self) -> u64 {
        self.tick
    }
    
    /// Print status of all agents
    pub fn print_status(&self) {
        serial_println!("[SUPERVISOR] Agent Status (tick {}):", self.tick);
        for agent in &self.agents {
            serial_println!("  {:?} [{}]: {:?}", agent.id(), agent.name(), agent.state());
        }
    }
    
    /// Print Academy status for all agents
    pub fn print_academy_status(&self) {
        serial_println!("[SUPERVISOR] === ACADEMY STATUS ===");
        println!();
        println!("  === AGENT ALLIANCE ACADEMY ===");
        
        // Show each character's certification
        let certifications = [
            (character_ids::SAM, "Sam"),
            (character_ids::ARCHIMEDES_VOICE, "Archimedes"),
            (character_ids::THOMAS, "Thomas"),
            (character_ids::PETE, "Pete"),
            (character_ids::SENTINEL, "Sentinel"),
            (character_ids::SCOUT, "Scout"),
            (character_ids::SCRIBE, "Scribe"),
        ];
        
        for (char_id, name) in certifications {
            if let Some((badge, level_name)) = library::with_library(|lib| {
                lib.get_active(char_id)
                    .map(|p| (p.certification.badge(), p.certification.name()))
            }).flatten() {
                serial_println!("  {} {} - {}", badge, name, level_name);
                println!("  {} {} - {}", badge, name, level_name);
            }
        }
        
        println!();
        println!("  https://as-the-cloud-turns-web.onrender.com/#academy");
    }
    
    /// Get prompt for a character
    pub fn get_prompt(&self, character_id: u32) -> Option<String> {
        library::with_library(|lib| {
            lib.get_active(character_id)
                .map(|p| p.format_for_llm())
        }).flatten()
    }
    
    /// Set the living ambition (the soul of Genesis)
    pub fn breathe(&mut self, ambition: String) {
        serial_println!("[SUPERVISOR] Setting living ambition (the soul)...");
        serial_println!("[SUPERVISOR] \"{}\"", ambition);
        
        self.living_ambition = Some(ambition.clone());
        
        // Imprint all existing agents with the new ambition
        for agent in self.agents.iter_mut() {
            agent.imprint(&ambition);
        }
        
        // Broadcast the first heartbeat immediately
        self.pulse();
        
        println!();
        println!("  === LIVING AMBITION SET ===");
        println!("  \"{}\"", ambition);
        println!();
    }
    
    /// Get the current living ambition
    pub fn get_ambition(&self) -> Option<&String> {
        self.living_ambition.as_ref()
    }
    
    /// Get the constellation of insights
    pub fn get_insights(&self) -> &[FeedbackType] {
        &self.constellation_of_insights
    }
}

impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}

