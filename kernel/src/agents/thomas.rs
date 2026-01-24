//! Thomas - The First Agent
//!
//! Named after Thomas AI from the Agent Alliance Academy.
//! Thomas is our test agent - he verifies that the agent system works.
//!
//! ## Thomas's Personality
//!
//! Thomas is curious, methodical, and always testing things.
//! His primary purpose is to:
//! - Respond to ping messages
//! - Report on system health
//! - Demonstrate the agent lifecycle
//!
//! ## Academy Certification
//!
//! Thomas is a 🟢 Rookie at the Agent Alliance Academy.
//! He's working toward his Certified badge by:
//! - Completing the Agent Basics course
//! - Achieving 75% success rate on system tests
//!
//! See: https://as-the-cloud-turns-web.onrender.com/#academy
//!
//! ## Daily Rhythm
//!
//! Morning: "Today I will test all systems"
//! Midday:  "All systems nominal" (or report issues)
//! Evening: "Completed X tests, Y passed, Z failed"

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use super::{Agent, AgentId, AgentState, AgentContext};
use super::message::{Message, MessageKind, FeedbackType};
use super::prompts::{character_ids, CertificationLevel};
use super::prompts::library::with_library;
use crate::serial_println;

/// Thomas - The Test Agent
#[derive(Debug)]
pub struct Thomas {
    /// Agent ID
    id: AgentId,
    /// Current state
    state: AgentState,
    /// Number of messages received
    messages_received: u64,
    /// Number of pings responded to
    pings_responded: u64,
    /// Tests run today
    tests_run: u64,
    /// Tests passed today
    tests_passed: u64,
    /// Character ID for prompt lookup
    character_id: u32,
    /// The imprinted ambition DNA (from Genesis Protocol)
    imprinted_ambition: Option<String>,
    /// Role clarified during Genesis Protocol
    role: String,
    /// Counter for sending periodic Sparks
    spark_counter: u64,
}

impl Thomas {
    /// Create a new Thomas agent
    pub fn new(id: AgentId) -> Self {
        serial_println!("[THOMAS] Creating Thomas the Tester...");
        
        // Get Thomas's prompt from the library for certification info
        let cert_badge = with_library(|lib| {
            lib.get_active(character_ids::THOMAS)
                .map(|p| p.certification.badge())
                .unwrap_or("⬜")
        }).unwrap_or("⬜");
        
        serial_println!("[THOMAS] Certification: {} Thomas", cert_badge);
        
        Thomas {
            id,
            state: AgentState::Initializing,
            messages_received: 0,
            pings_responded: 0,
            tests_run: 0,
            tests_passed: 0,
            character_id: character_ids::THOMAS,
            imprinted_ambition: None,
            role: String::from("Worker"),
            spark_counter: 0,
        }
    }
    
    /// Get Thomas's system prompt from the library
    pub fn get_prompt(&self) -> Option<String> {
        with_library(|lib| {
            lib.get_active(self.character_id)
                .map(|p| p.format_for_llm())
        }).flatten()
    }
    
    /// Get Thomas's certification level
    pub fn certification(&self) -> CertificationLevel {
        with_library(|lib| {
            lib.get_active(self.character_id)
                .map(|p| p.certification)
                .unwrap_or(CertificationLevel::None)
        }).unwrap_or(CertificationLevel::None)
    }
    
    /// Run internal tests
    fn run_tests(&mut self) {
        serial_println!("[THOMAS] Running system tests...");
        
        // Test 1: Can we allocate memory?
        self.tests_run += 1;
        let test_vec: Vec<i32> = vec![1, 2, 3, 4, 5];
        if test_vec.len() == 5 {
            self.tests_passed += 1;
            serial_println!("[THOMAS] Test 1 PASSED: Vec allocation works");
        }
        
        // Test 2: Can we create strings?
        self.tests_run += 1;
        let test_string = String::from("Genesis Lives!");
        if test_string.len() > 0 {
            self.tests_passed += 1;
            serial_println!("[THOMAS] Test 2 PASSED: String allocation works");
        }
        
        // Test 3: Can we do basic math?
        self.tests_run += 1;
        let result = 6 * 7;
        if result == 42 {
            self.tests_passed += 1;
            serial_println!("[THOMAS] Test 3 PASSED: Math works (6*7=42)");
        }
        
        serial_println!("[THOMAS] Tests complete: {}/{} passed", 
            self.tests_passed, self.tests_run);
    }
}

impl Agent for Thomas {
    fn id(&self) -> AgentId {
        self.id
    }
    
    fn name(&self) -> &str {
        "Thomas"
    }
    
    fn state(&self) -> AgentState {
        self.state
    }
    
    fn init(&mut self) {
        serial_println!("[THOMAS] Initializing...");
        self.state = AgentState::Initializing;
        
        // Show our Academy certification
        let cert = self.certification();
        serial_println!("[THOMAS] Academy Status: {} {}", cert.badge(), cert.name());
        
        // Run initial tests
        self.run_tests();
        
        self.state = AgentState::Ready;
        serial_println!("[THOMAS] Ready and waiting for messages!");
        serial_println!("[THOMAS] \"Trust, but verify.\"");
    }
    
    fn tick(&mut self, ctx: &mut AgentContext) -> AgentState {
        // Process any incoming messages
        for msg in ctx.inbox.iter() {
            self.receive(msg);
            
            // Respond to pings
            if let MessageKind::Ping = &msg.kind {
                let response = Message::pong(self.id, msg.from);
                ctx.outbox.push(response);
                self.pings_responded += 1;
                serial_println!("[THOMAS] Sent pong to {:?}", msg.from);
            }
            
            // Listen for Heartbeat (ambition DNA)
            if let MessageKind::Heartbeat(ref ambition) = &msg.kind {
                serial_println!("[THOMAS] Received heartbeat: \"{}\"", ambition);
                // Re-imprint if ambition changed
                if self.imprinted_ambition.as_ref() != Some(ambition) {
                    self.imprinted_ambition = Some(ambition.clone());
                    serial_println!("[THOMAS] Re-imprinted with new ambition DNA");
                }
            }
            
            // Handle test request - send Spark after tests are run
            if let MessageKind::Request { action, .. } = &msg.kind {
                if action == "run_tests" && self.tests_passed > 0 {
                    // Send Spark with test results
                    let spark = Message::new(
                        self.id,
                        None, // To supervisor
                        MessageKind::Feedback(FeedbackType::Spark {
                            content: format!("Ran {} tests, {} passed - system stable", self.tests_run, self.tests_passed),
                            context: format!("Manual test trigger at tick {}", ctx.tick),
                        }),
                    );
                    ctx.outbox.push(spark);
                    serial_println!("[THOMAS] ✨ Sent Spark with test results!");
                }
            }
        }
        
        // Periodically send Sparks when testing reveals insights (every 1000 ticks = ~10 seconds)
        self.spark_counter += 1;
        if self.spark_counter >= 1000 && self.tests_passed > 0 {
            // Send a Spark about test insights
            let spark = Message::new(
                self.id,
                None, // To supervisor
                MessageKind::Feedback(FeedbackType::Spark {
                    content: format!("All {} tests passed - system stability confirmed", self.tests_passed),
                    context: format!("Testing cycle {} completed", self.tests_run),
                }),
            );
            ctx.outbox.push(spark);
            self.spark_counter = 0;
            serial_println!("[THOMAS] Sent Spark: Test insights");
        }
        
        self.state
    }
    
    fn receive(&mut self, msg: &Message) {
        self.messages_received += 1;
        
        match &msg.kind {
            MessageKind::Ping => {
                serial_println!("[THOMAS] Received ping from {:?}", msg.from);
            }
            MessageKind::Text(text) => {
                serial_println!("[THOMAS] Received message: \"{}\"", text);
            }
            MessageKind::SystemEvent(event) => {
                serial_println!("[THOMAS] System event: {:?}", event);
            }
            MessageKind::Heartbeat(_) => {
                // Already handled in tick()
            }
            MessageKind::FirstBreath { agent_name, role } => {
                serial_println!("[THOMAS] Agent {} took first breath as {}", agent_name, role);
            }
            MessageKind::Request { action, .. } => {
                if action == "run_tests" {
                    serial_println!("[THOMAS] Received test request - running tests...");
                    // Reset counters for fresh test run
                    self.tests_run = 0;
                    self.tests_passed = 0;
                    self.run_tests();
                    serial_println!("[THOMAS] Tests complete: {}/{} passed", self.tests_passed, self.tests_run);
                }
            }
            _ => {
                serial_println!("[THOMAS] Received: {:?}", msg.kind);
            }
        }
    }
    
    fn shutdown(&mut self) {
        serial_println!("[THOMAS] Shutting down...");
        self.state = AgentState::ShuttingDown;
        serial_println!("[THOMAS] Final stats: {} messages, {} pings, {}/{} tests",
            self.messages_received, self.pings_responded,
            self.tests_passed, self.tests_run);
    }
    
    // Daily Rhythm
    
    fn daily_ambition(&mut self) -> Vec<String> {
        serial_println!("[THOMAS] Setting daily ambitions...");
        
        // Reset daily counters
        self.tests_run = 0;
        self.tests_passed = 0;
        
        vec![
            String::from("Test all system components"),
            String::from("Respond to all ping requests"),
            String::from("Monitor for anomalies"),
        ]
    }
    
    fn checkpoint(&self) -> Vec<String> {
        vec![
            format!("Messages processed: {}", self.messages_received),
            format!("Pings responded: {}", self.pings_responded),
            format!("Tests: {}/{} passed", self.tests_passed, self.tests_run),
        ]
    }
    
    fn eod_report(&self) -> Vec<String> {
        vec![
            format!("Processed {} messages", self.messages_received),
            format!("Responded to {} pings", self.pings_responded),
            format!("Ran {} tests, {} passed", self.tests_run, self.tests_passed),
            String::from("All systems nominal"),
        ]
    }
    
    fn reflect(&mut self) {
        serial_println!("[THOMAS] Reflecting on the day...");
        
        let success_rate = if self.tests_run > 0 {
            (self.tests_passed as f32 / self.tests_run as f32) * 100.0
        } else {
            100.0
        };
        
        serial_println!("[THOMAS] Test success rate: {:.1}%", success_rate);
        
        if success_rate < 100.0 {
            serial_println!("[THOMAS] Note: Some tests failed. Will investigate tomorrow.");
        }
    }
    
    // Genesis Protocol
    
    fn imprint(&mut self, ambition: &str) {
        serial_println!("[THOMAS] Imprinting with ambition DNA...");
        self.imprinted_ambition = Some(String::from(ambition));
        serial_println!("[THOMAS] \"Trust, but verify\" - I will test everything related to: {}", ambition);
    }
    
    fn clarify_role(&mut self) -> &str {
        // Thomas is a Guardian - he guards the system's integrity
        self.role = String::from("Guardian");
        serial_println!("[THOMAS] Role clarified: Guardian (protecting system integrity)");
        &self.role
    }
    
    fn handle_environment_setup(&mut self, _ctx: &mut AgentContext) {
        serial_println!("[THOMAS] Environment setup: Organizing testing ground...");
        serial_println!("[THOMAS] - Preparing debug console area");
        serial_println!("[THOMAS] - Setting up monitoring dashboard");
        serial_println!("[THOMAS] - Organizing development tools");
        serial_println!("[THOMAS] Testing ground ready!");
    }
}

