//! Agent Framework for Genesis
//!
//! This is the heart of Genesis - where "agentic" meets "operating system."
//! 
//! ## Philosophy
//!
//! Traditional OS: Processes are passive, waiting to be scheduled.
//! Genesis: Agents are active, goal-driven entities that:
//!   - Have their own ambitions (daily goals)
//!   - Communicate via messages
//!   - Reflect on their performance
//!   - Manage their own memory tiers
//!   - Evolve their prompts over time (DSPy-style)
//!   - Earn certifications from the Agent Alliance Academy
//!
//! ## The Daily Rhythm (Ambition Symphony)
//!
//! ```text
//! MORNING:  agent.daily_ambition() → Set goals for the day
//! MIDDAY:   agent.checkpoint()     → Report progress
//! EVENING:  agent.eod_report()     → Summarize accomplishments
//! NIGHT:    agent.reflect()        → Learn and optimize
//! ```
//!
//! ## Architecture
//!
//! ```text
//!                    ┌─────────────────┐
//!                    │   Supervisor    │
//!                    │   (Sam)         │
//!                    └────────┬────────┘
//!                             │
//!          ┌──────────────────┼──────────────────┐
//!          │                  │                  │
//!     ┌────┴────┐       ┌────┴────┐       ┌────┴────┐
//!     │ Thomas  │       │ Scout   │       │ Scribe  │
//!     │ (Test)  │       │(Research)│      │ (Notes) │
//!     └─────────┘       └─────────┘       └─────────┘
//! ```
//!
//! ## Prompt Library & Evolution
//!
//! Every agent has a prompt that defines its personality and capabilities.
//! Prompts can evolve through:
//! - A/B testing (compare variants)
//! - Metric-driven optimization
//! - Academy certification
//!
//! See: https://as-the-cloud-turns-web.onrender.com/#academy

pub mod message;
pub mod supervisor;
pub mod thomas;
pub mod archimedes;
pub mod prompts;
pub mod protection;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;

/// Unique identifier for an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub u64);

impl AgentId {
    pub fn new(id: u64) -> Self {
        AgentId(id)
    }
}

/// The current state of an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    /// Agent is initializing
    Initializing,
    /// Agent is ready and idle
    Ready,
    /// Agent is actively processing
    Running,
    /// Agent is waiting for a message or event
    Waiting,
    /// Agent has completed its current task
    Completed,
    /// Agent encountered an error
    Error,
    /// Agent is shutting down
    ShuttingDown,
}

/// Context passed to agents during their tick
pub struct AgentContext<'a> {
    /// Messages waiting for this agent
    pub inbox: &'a mut Vec<message::Message>,
    /// Outbox for sending messages
    pub outbox: &'a mut Vec<message::Message>,
    /// Current tick number (for timing)
    pub tick: u64,
}

/// The core Agent trait - what makes something an "agent" in Genesis
///
/// Every agent must implement this trait. It defines the lifecycle
/// and behavior of an autonomous entity in the system.
pub trait Agent: Send + Debug {
    /// Get the agent's unique identifier
    fn id(&self) -> AgentId;
    
    /// Get the agent's name (for display/logging)
    fn name(&self) -> &str;
    
    /// Get the agent's current state
    fn state(&self) -> AgentState;
    
    /// Initialize the agent (called once at startup)
    fn init(&mut self);
    
    /// Main processing loop - called each tick by the supervisor
    /// 
    /// This is where the agent does its work:
    /// - Check inbox for messages
    /// - Process tasks
    /// - Send messages to outbox
    /// - Update state
    fn tick(&mut self, ctx: &mut AgentContext) -> AgentState;
    
    /// Handle a received message
    fn receive(&mut self, msg: &message::Message);
    
    /// Shutdown the agent gracefully
    fn shutdown(&mut self);
    
    // =========================================================================
    // Daily Rhythm Methods (The Ambition Symphony)
    // =========================================================================
    
    /// Morning: Set the agent's ambitions for the day
    /// Returns a list of goals/intentions
    fn daily_ambition(&mut self) -> Vec<String> {
        Vec::new() // Default: no specific ambitions
    }
    
    /// Midday: Report progress on ambitions
    /// Returns status update on each goal
    fn checkpoint(&self) -> Vec<String> {
        Vec::new() // Default: no report
    }
    
    /// Evening: Summarize accomplishments
    /// Returns bullet points of what was achieved
    fn eod_report(&self) -> Vec<String> {
        Vec::new() // Default: no report
    }
    
    /// Night: Reflect on performance and learn
    /// This is where agents can adjust their behavior
    fn reflect(&mut self) {
        // Default: no reflection
    }
    
    // =========================================================================
    // Protection Tier & Self-Improvement
    // =========================================================================

    /// The highest tier this agent autonomously self-improves at.
    ///
    /// Agents are ENCOURAGED to propose improvements at every tier — the
    /// system should get better over time. This method returns the tier
    /// where the agent can make changes without discussion. Higher tiers
    /// (lower numbers) require conversation with Stephen first.
    ///
    /// Default: Tier 4 (Playground) — experiment freely.
    /// Thomas overrides to Tier 3 (Maintained) — can refactor his own code.
    fn max_write_tier(&self) -> protection::ProtectionTier {
        protection::ProtectionTier::Playground
    }

    // =========================================================================
    // Journal — "As the Kernel Turns"
    // =========================================================================

    /// Write a first-person journal entry reflecting on the agent's experience.
    /// Called periodically by the supervisor. Returns None if the agent has
    /// nothing to say right now.
    fn journal_entry(&self, _tick: u64) -> Option<String> {
        None
    }

    // =========================================================================
    // Genesis Protocol - The Soul-Body Connection
    // =========================================================================
    
    /// Imprint the agent with the day's purpose DNA
    /// 
    /// This is called during agent "birth" - the first thing an agent does
    /// is connect to the Genesis Core and receive the day's Ambition Heartbeat.
    /// This imprints the core purpose onto the agent.
    fn imprint(&mut self, _ambition: &str) {
        // Default: agents can override to store the ambition
    }
    
    /// Agent determines its role based on the current ambition
    /// 
    /// The agent queries: "Given this ambition, what is my primary function today?"
    /// Returns a role like "Explorer", "Builder", "Synthesizer", or "Guardian".
    fn clarify_role(&mut self) -> &str {
        // Default: generic role
        "Worker"
    }
    
    /// Handle environment setup - agents organize their domain before GUI
    /// 
    /// Called during Phase 3 of boot sequence. Agents should:
    /// - Organize files/folders in their domain
    /// - Create workspace layouts
    /// - Prepare resources
    /// - Set up their zones
    fn handle_environment_setup(&mut self, _ctx: &mut AgentContext) {
        // Default: no environment setup
        // Agents can override to organize their domain
    }
}

