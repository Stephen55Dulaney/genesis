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
use crate::storage::memory_store::{self, MemoryKind};

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
    /// Rhythm counter for periodic checkpoint and report cycles
    rhythm_counter: u64,
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
            rhythm_counter: 0,
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
                // Auto-store feedback in persistent memory
                let source = alloc::format!("agent-{}", msg.from.0);
                match &feedback {
                    FeedbackType::Spark { content, context } => {
                        memory_store::store_with_timestamp(
                            &alloc::format!("{} (context: {})", content, context),
                            MemoryKind::Spark,
                            &source,
                            self.tick,
                        );
                    }
                    FeedbackType::Connection { from, to, pattern } => {
                        memory_store::store_with_timestamp(
                            &alloc::format!("{} -> {}: {}", from, to, pattern),
                            MemoryKind::Connection,
                            &source,
                            self.tick,
                        );
                    }
                    FeedbackType::Resource { description, location } => {
                        memory_store::store_with_timestamp(
                            &alloc::format!("{} (at: {})", description, location),
                            MemoryKind::Resource,
                            &source,
                            self.tick,
                        );
                    }
                    FeedbackType::Feeling { tag, intensity } => {
                        memory_store::store_with_timestamp(
                            &alloc::format!("{} (intensity: {})", tag, intensity),
                            MemoryKind::Feeling,
                            &source,
                            self.tick,
                        );
                    }
                }

                self.constellation_of_insights.push(feedback);

                // Keep only the most recent insights
                if self.constellation_of_insights.len() > MAX_INSIGHTS {
                    self.constellation_of_insights.remove(0);
                }
            }
        }
        
        // Handle memory messages (supervisor intercepts these)
        let mut routable_messages = Vec::new();
        for msg in other_messages {
            match &msg.kind {
                MessageKind::MemoryStore { content, kind } => {
                    let mem_kind = MemoryKind::from_str(kind)
                        .unwrap_or(MemoryKind::Observation);
                    let source = alloc::format!("agent-{}", msg.from.0);
                    let id = memory_store::store_with_timestamp(
                        content, mem_kind, &source, self.tick,
                    );
                    serial_println!("[MEMORY_STORE] Agent {} stored memory #{}", msg.from.0, id);
                    // Send confirmation back
                    let reply = Message::new(
                        self.id,
                        Some(msg.from),
                        MessageKind::MemoryResults {
                            results: alloc::vec![(id, content.clone())],
                        },
                    );
                    self.message_queue.push(reply);
                }
                MessageKind::MemorySearch { query } => {
                    let search_results = memory_store::search(query);
                    let results: Vec<(u64, String)> = search_results.iter()
                        .take(10)
                        .filter_map(|(id, _score)| {
                            memory_store::get(*id).map(|e| {
                                let preview = if e.content.len() > 80 {
                                    let s: String = e.content.chars().take(77).collect();
                                    alloc::format!("{}...", s)
                                } else {
                                    e.content.clone()
                                };
                                (*id, preview)
                            })
                        })
                        .collect();
                    serial_println!("[MEMORY_STORE] Agent {} searched '{}', {} results",
                        msg.from.0, query, results.len());
                    let reply = Message::new(
                        self.id,
                        Some(msg.from),
                        MessageKind::MemoryResults { results },
                    );
                    self.message_queue.push(reply);
                }
                _ => {
                    routable_messages.push(msg);
                }
            }
        }

        // Route other messages to agents
        for agent in self.agents.iter_mut() {
            // Collect messages for this agent
            let mut inbox: Vec<Message> = routable_messages
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

        // Daily rhythm: periodic checkpoint and status reports
        self.rhythm_counter += 1;

        // Midday checkpoint (every 10,000 ticks)
        if self.rhythm_counter % 10_000 == 0 {
            serial_println!("[CHECKPOINT] Periodic checkpoint at tick {}", self.tick);
            serial_println!("[NOTIFY] Checkpoint at tick {}: querying all agents", self.tick);
            for agent in self.agents.iter() {
                let progress = agent.checkpoint();
                if !progress.is_empty() {
                    serial_println!("[CHECKPOINT] [{}]:", agent.name());
                    for item in &progress {
                        serial_println!("[CHECKPOINT]   - {}", item);
                    }
                }
            }
        }

        // Status report (every 20,000 ticks)
        if self.rhythm_counter % 20_000 == 0 {
            serial_println!("[REPORT] Periodic status report at tick {}", self.tick);
            for agent in self.agents.iter() {
                let report = agent.eod_report();
                if !report.is_empty() {
                    serial_println!("[REPORT] [{}]:", agent.name());
                    for item in &report {
                        serial_println!("[REPORT]   - {}", item);
                    }
                }
            }
        }
    }
    
    /// Pulse the heartbeat - broadcast the living ambition DNA
    fn pulse(&mut self) {
        if let Some(ref ambition) = self.living_ambition {
            serial_println!("[HEARTBEAT] Pulsing ambition DNA to all agents...");
            self.broadcast(MessageKind::Heartbeat(ambition.clone()));
        }
    }
    
    /// Serendipity Engine: Find overlapping themes using memory search
    fn check_serendipity(&mut self) {
        let stats = memory_store::stats();
        if stats.entry_count < 2 {
            return;
        }

        // Use the memory store's top keywords to find themes
        if !stats.top_keywords.is_empty() {
            let mut themes_found = false;
            for (keyword, count) in stats.top_keywords.iter().take(5) {
                if *count >= 2 {
                    if !themes_found {
                        serial_println!("[SERENDIPITY] Found overlapping themes in memory:");
                        themes_found = true;
                    }
                    // Search memory for this keyword to find connections
                    let results = memory_store::search(keyword);
                    serial_println!("  - '{}' appears in {} entries ({} search hits)",
                        keyword, count, results.len());

                    // Get content previews of the first two matching entries
                    if results.len() >= 2 {
                        let first_preview = memory_store::get(results[0].0)
                            .map(|e| if e.content.len() > 60 {
                                let s: String = e.content.chars().take(57).collect();
                                alloc::format!("{}...", s)
                            } else {
                                e.content.clone()
                            });
                        let second_preview = memory_store::get(results[1].0)
                            .map(|e| if e.content.len() > 60 {
                                let s: String = e.content.chars().take(57).collect();
                                alloc::format!("{}...", s)
                            } else {
                                e.content.clone()
                            });

                        if let (Some(from), Some(to)) = (first_preview, second_preview) {
                            // Broadcast connection to all agents
                            let connection_msg = Message::broadcast(
                                self.id,
                                MessageKind::Feedback(FeedbackType::Connection {
                                    from: from.clone(),
                                    to: to.clone(),
                                    pattern: alloc::format!("Serendipity: theme '{}' links these insights", keyword),
                                }),
                            );
                            self.message_queue.push(connection_msg);
                            serial_println!("[SERENDIPITY] Broadcasted connection to agents for theme '{}'", keyword);
                            serial_println!("[NOTIFY] Serendipity: theme '{}' links insights across agents", keyword);
                        }
                    }
                }
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

