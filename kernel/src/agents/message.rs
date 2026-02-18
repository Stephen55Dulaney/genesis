//! Message System for Agent Communication
//!
//! Agents communicate via messages, not shared memory.
//! This is safer, more scalable, and fits the actor model.
//!
//! ## Message Flow
//!
//! ```text
//! Agent A                    Supervisor                   Agent B
//!    │                           │                           │
//!    │──── Message ─────────────>│                           │
//!    │                           │──── Message ─────────────>│
//!    │                           │                           │
//!    │                           │<──── Reply ───────────────│
//!    │<──── Reply ───────────────│                           │
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use super::AgentId;

/// Priority levels for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Types of feedback agents can send back to the core
#[derive(Debug, Clone)]
pub enum FeedbackType {
    /// A new idea or insight that emerged while doing work
    Spark {
        content: String,
        context: String,
    },
    /// A link to a previous day's ambition or different project
    Connection {
        from: String,
        to: String,
        pattern: String,
    },
    /// A useful article, tool, or data source found
    Resource {
        description: String,
        location: String,
    },
    /// A simple tag describing the agent's state
    Feeling {
        tag: String,
        intensity: u8, // 0-100
    },
}

/// Types of messages agents can send
#[derive(Debug, Clone)]
pub enum MessageKind {
    /// Simple text message
    Text(String),
    
    /// Request for another agent to do something
    Request {
        action: String,
        params: Vec<String>,
    },
    
    /// Response to a request
    Response {
        success: bool,
        data: String,
    },
    
    /// Status update (for daily rhythm)
    StatusUpdate {
        ambitions: Vec<String>,
        progress: Vec<String>,
    },
    
    /// System event (from supervisor)
    SystemEvent(SystemEvent),
    
    /// Keyboard input event
    KeyboardInput(char),
    
    /// Ping (for testing connectivity)
    Ping,
    
    /// Pong (response to ping)
    Pong,
    
    /// The periodic broadcast of the ambition DNA (the heartbeat)
    Heartbeat(String),
    
    /// Feedback from agents back to the core (Sparks, Connections, etc.)
    Feedback(FeedbackType),
    
    /// Agent's first breath announcement
    FirstBreath {
        agent_name: String,
        role: String,
    },

    /// Store something in memory
    MemoryStore {
        content: String,
        kind: String, // "spark", "connection", etc.
    },

    /// Search memory
    MemorySearch {
        query: String,
    },

    /// Search results returned
    MemoryResults {
        results: Vec<(u64, String)>, // (id, content preview)
    },
}

/// System-level events from the supervisor
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// Time for morning ambitions
    MorningAmbition,
    /// Time for midday checkpoint
    MiddayCheckpoint,
    /// Time for end-of-day report
    EndOfDay,
    /// Time for reflection
    NightReflection,
    /// System is shutting down
    Shutdown,
    /// Agent birth protocol - imprinting with purpose
    GenesisProtocol,
    /// Environment setup phase - agents organize before GUI
    EnvironmentSetup,
}

/// A message between agents
#[derive(Debug, Clone)]
pub struct Message {
    /// Unique message ID
    pub id: u64,
    /// Who sent this message
    pub from: AgentId,
    /// Who should receive this message (None = broadcast)
    pub to: Option<AgentId>,
    /// Message priority
    pub priority: Priority,
    /// The actual message content
    pub kind: MessageKind,
    /// Timestamp (tick number when sent)
    pub timestamp: u64,
}

impl Message {
    /// Create a new message
    pub fn new(from: AgentId, to: Option<AgentId>, kind: MessageKind) -> Self {
        static mut NEXT_ID: u64 = 0;
        let id = unsafe {
            NEXT_ID += 1;
            NEXT_ID
        };
        
        Message {
            id,
            from,
            to,
            priority: Priority::Normal,
            kind,
            timestamp: 0, // Will be set by supervisor
        }
    }
    
    /// Create a message with high priority
    pub fn urgent(from: AgentId, to: Option<AgentId>, kind: MessageKind) -> Self {
        let mut msg = Self::new(from, to, kind);
        msg.priority = Priority::High;
        msg
    }
    
    /// Create a broadcast message (to all agents)
    pub fn broadcast(from: AgentId, kind: MessageKind) -> Self {
        Self::new(from, None, kind)
    }
    
    /// Create a ping message
    pub fn ping(from: AgentId, to: AgentId) -> Self {
        Self::new(from, Some(to), MessageKind::Ping)
    }
    
    /// Create a pong response
    pub fn pong(from: AgentId, to: AgentId) -> Self {
        Self::new(from, Some(to), MessageKind::Pong)
    }
}

