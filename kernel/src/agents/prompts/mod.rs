//! Prompt Library for Genesis Agents
//!
//! This is where agent intelligence lives - not just static text, but
//! evolving, learning, certified prompts that grow smarter over time.
//!
//! ## Philosophy
//!
//! Prompts are the soul of an agent. They define:
//! - Personality and voice
//! - Expertise and capabilities
//! - Behavioral patterns
//! - Collaboration style
//!
//! ## The DSPy-Inspired Evolution System
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    PROMPT EVOLUTION CYCLE                        │
//! │                                                                  │
//! │   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐ │
//! │   │ Deploy   │───▶│ Measure  │───▶│ Optimize │───▶│ Certify  │ │
//! │   │ v1.0     │    │ Metrics  │    │ Prompt   │    │ Academy  │ │
//! │   └──────────┘    └──────────┘    └──────────┘    └──────────┘ │
//! │        ▲                                               │        │
//! │        └───────────────────────────────────────────────┘        │
//! │                    Continuous Improvement                        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Academy Integration
//!
//! Agents earn certifications from the Agent Alliance Academy:
//! - Rookie: Basic functionality
//! - Certified: Proven performance
//! - Expert: Advanced capabilities
//! - Master: Teaching other agents
//!
//! See: https://as-the-cloud-turns-web.onrender.com/#academy

pub mod library;
pub mod evolution;
pub mod characters;
pub mod academy;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;

/// Unique identifier for a prompt version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PromptId {
    /// Character/agent this prompt belongs to
    pub character_id: u32,
    /// Version number (semantic versioning: major.minor.patch encoded)
    pub version: u32,
}

impl PromptId {
    pub fn new(character_id: u32, major: u8, minor: u8, patch: u8) -> Self {
        let version = ((major as u32) << 16) | ((minor as u32) << 8) | (patch as u32);
        PromptId { character_id, version }
    }
    
    pub fn major(&self) -> u8 {
        ((self.version >> 16) & 0xFF) as u8
    }
    
    pub fn minor(&self) -> u8 {
        ((self.version >> 8) & 0xFF) as u8
    }
    
    pub fn patch(&self) -> u8 {
        (self.version & 0xFF) as u8
    }
    
    pub fn version_string(&self) -> String {
        alloc::format!("{}.{}.{}", self.major(), self.minor(), self.patch())
    }
}

/// The type/role of an agent prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptRole {
    /// System orchestrator (like Sam)
    Orchestrator,
    /// Research/Scout agent
    Researcher,
    /// Testing/QA agent (like Thomas)
    Tester,
    /// Voice conversational agent (like Archimedes Voice)
    VoiceAgent,
    /// Background processor (like Silent Archimedes)
    BackgroundProcessor,
    /// Frontend development specialist
    FrontendDev,
    /// Backend development specialist
    BackendDev,
    /// Security & Quality specialist (like Sentinel)
    SecurityQuality,
    /// System maintenance specialist
    SystemMaintenance,
    /// Custom/specialized role
    Custom,
}

/// Academy certification levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CertificationLevel {
    /// No certification yet
    None = 0,
    /// Basic functionality verified
    Rookie = 1,
    /// Proven performance in production
    Certified = 2,
    /// Advanced capabilities demonstrated
    Expert = 3,
    /// Can teach and mentor other agents
    Master = 4,
}

impl CertificationLevel {
    pub fn badge(&self) -> &'static str {
        match self {
            CertificationLevel::None => "⬜",
            CertificationLevel::Rookie => "🟢",
            CertificationLevel::Certified => "🔵",
            CertificationLevel::Expert => "🟣",
            CertificationLevel::Master => "🟡",
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            CertificationLevel::None => "Uncertified",
            CertificationLevel::Rookie => "Rookie",
            CertificationLevel::Certified => "Certified",
            CertificationLevel::Expert => "Expert",
            CertificationLevel::Master => "Master",
        }
    }
}

/// Performance metrics for prompt optimization (DSPy-style)
#[derive(Debug, Clone, Default)]
pub struct PromptMetrics {
    /// Total times this prompt was used
    pub invocations: u64,
    /// Successful completions
    pub successes: u64,
    /// Failures or errors
    pub failures: u64,
    /// User satisfaction ratings (sum for averaging)
    pub satisfaction_sum: u64,
    /// Number of satisfaction ratings received
    pub satisfaction_count: u64,
    /// Task completion time (sum for averaging, in ticks)
    pub completion_time_sum: u64,
    /// Number of completion time measurements
    pub completion_time_count: u64,
}

impl PromptMetrics {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a successful invocation
    pub fn record_success(&mut self, completion_ticks: u64) {
        self.invocations += 1;
        self.successes += 1;
        self.completion_time_sum += completion_ticks;
        self.completion_time_count += 1;
    }
    
    /// Record a failed invocation
    pub fn record_failure(&mut self) {
        self.invocations += 1;
        self.failures += 1;
    }
    
    /// Record user satisfaction (1-5 scale)
    pub fn record_satisfaction(&mut self, rating: u8) {
        self.satisfaction_sum += rating as u64;
        self.satisfaction_count += 1;
    }
    
    /// Success rate as percentage (0-100)
    pub fn success_rate(&self) -> u8 {
        if self.invocations == 0 {
            return 0;
        }
        ((self.successes * 100) / self.invocations) as u8
    }
    
    /// Average satisfaction (0-5 scale, returned as 0-50 for precision)
    pub fn avg_satisfaction_x10(&self) -> u8 {
        if self.satisfaction_count == 0 {
            return 0;
        }
        ((self.satisfaction_sum * 10) / self.satisfaction_count) as u8
    }
    
    /// Average completion time in ticks
    pub fn avg_completion_time(&self) -> u64 {
        if self.completion_time_count == 0 {
            return 0;
        }
        self.completion_time_sum / self.completion_time_count
    }
}

/// A prompt template that can be evolved and optimized
#[derive(Debug, Clone)]
pub struct Prompt {
    /// Unique identifier for this prompt version
    pub id: PromptId,
    /// Human-readable name
    pub name: String,
    /// The role/type of this prompt
    pub role: PromptRole,
    /// Academy certification level
    pub certification: CertificationLevel,
    /// The actual prompt text (system prompt)
    pub system_prompt: String,
    /// Optional personality/voice description
    pub personality: Option<String>,
    /// Specific capabilities this prompt enables
    pub capabilities: Vec<String>,
    /// Performance metrics for optimization
    pub metrics: PromptMetrics,
    /// Parent prompt ID (for evolution tracking)
    pub parent_id: Option<PromptId>,
    /// Whether this is the active/deployed version
    pub is_active: bool,
}

impl Prompt {
    /// Create a new prompt
    pub fn new(
        character_id: u32,
        name: &str,
        role: PromptRole,
        system_prompt: &str,
    ) -> Self {
        Prompt {
            id: PromptId::new(character_id, 1, 0, 0),
            name: String::from(name),
            role,
            certification: CertificationLevel::None,
            system_prompt: String::from(system_prompt),
            personality: None,
            capabilities: Vec::new(),
            metrics: PromptMetrics::new(),
            parent_id: None,
            is_active: true,
        }
    }
    
    /// Add a personality description
    pub fn with_personality(mut self, personality: &str) -> Self {
        self.personality = Some(String::from(personality));
        self
    }
    
    /// Add capabilities
    pub fn with_capabilities(mut self, capabilities: &[&str]) -> Self {
        self.capabilities = capabilities.iter().map(|s| String::from(*s)).collect();
        self
    }
    
    /// Set certification level
    pub fn with_certification(mut self, level: CertificationLevel) -> Self {
        self.certification = level;
        self
    }
    
    /// Create an evolved version of this prompt
    pub fn evolve(&self, new_system_prompt: &str, bump_minor: bool) -> Self {
        let new_id = if bump_minor {
            PromptId::new(
                self.id.character_id,
                self.id.major(),
                self.id.minor() + 1,
                0,
            )
        } else {
            PromptId::new(
                self.id.character_id,
                self.id.major(),
                self.id.minor(),
                self.id.patch() + 1,
            )
        };
        
        Prompt {
            id: new_id,
            name: self.name.clone(),
            role: self.role,
            certification: self.certification, // Inherit certification
            system_prompt: String::from(new_system_prompt),
            personality: self.personality.clone(),
            capabilities: self.capabilities.clone(),
            metrics: PromptMetrics::new(), // Fresh metrics for new version
            parent_id: Some(self.id),
            is_active: false, // New versions start inactive
        }
    }
    
    /// Format the full prompt for LLM consumption
    pub fn format_for_llm(&self) -> String {
        let mut full_prompt = self.system_prompt.clone();
        
        if let Some(ref personality) = self.personality {
            full_prompt = alloc::format!(
                "{}\n\n## Personality\n{}",
                full_prompt,
                personality
            );
        }
        
        if !self.capabilities.is_empty() {
            let caps = self.capabilities.join("\n- ");
            full_prompt = alloc::format!(
                "{}\n\n## Capabilities\n- {}",
                full_prompt,
                caps
            );
        }
        
        full_prompt
    }
}

/// Character IDs for built-in agents
pub mod character_ids {
    pub const SAM: u32 = 1;          // Orchestrator
    pub const ARCHIMEDES_VOICE: u32 = 2;   // Voice agent
    pub const ARCHIMEDES_SILENT: u32 = 3;  // Background processor
    pub const THOMAS: u32 = 4;        // Tester
    pub const PETE: u32 = 5;          // Backend dev
    pub const SENTINEL: u32 = 6;      // Security & quality
    pub const SCOUT: u32 = 7;         // Researcher
    pub const SCRIBE: u32 = 8;        // Note-taker/documenter
}

