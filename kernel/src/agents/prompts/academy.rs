//! Agent Alliance Academy Integration
//!
//! Connection to the Agent Alliance Academy for:
//! - Certification management
//! - Course completion tracking  
//! - Shared learning across agents
//! - Community knowledge sync
//!
//! Academy URL: https://as-the-cloud-turns-web.onrender.com/#academy
//!
//! ## Academy Structure
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     AGENT ALLIANCE ACADEMY                       │
//! │                                                                  │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
//! │  │  CURRICULUM  │  │ CERTIFICATION│  │  COMMUNITY   │          │
//! │  │              │  │              │  │              │          │
//! │  │ - Courses    │  │ - Levels     │  │ - Forums     │          │
//! │  │ - Labs       │  │ - Badges     │  │ - Mentorship │          │
//! │  │ - Projects   │  │ - Reviews    │  │ - Events     │          │
//! │  └──────────────┘  └──────────────┘  └──────────────┘          │
//! │                                                                  │
//! │  "Greetings, seeker. I am Sam, Orchestrator of the Academy."    │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::BTreeMap;
use crate::serial_println;

use super::{PromptId, CertificationLevel};
use super::library::with_library_mut;

/// Academy course categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourseCategory {
    /// Fundamental skills
    Fundamentals,
    /// Specialized domain skills
    Specialization,
    /// Leadership and coordination
    Leadership,
    /// Advanced techniques
    Advanced,
    /// Master-level content
    Mastery,
}

/// A course in the Academy curriculum
#[derive(Debug, Clone)]
pub struct Course {
    pub id: u32,
    pub name: String,
    pub category: CourseCategory,
    pub description: String,
    /// Required certification level to enroll
    pub prerequisite_level: CertificationLevel,
    /// Modules/lessons in the course
    pub modules: Vec<String>,
    /// Skills gained upon completion
    pub skills_gained: Vec<String>,
}

/// Course completion record for an agent
#[derive(Debug, Clone)]
pub struct CourseCompletion {
    pub course_id: u32,
    pub character_id: u32,
    /// Module progress (0-100 per module)
    pub module_progress: Vec<u8>,
    /// Final assessment score (0-100)
    pub assessment_score: Option<u8>,
    /// Completed timestamp (tick count)
    pub completed_at: Option<u64>,
}

impl CourseCompletion {
    /// Overall progress percentage
    pub fn progress(&self) -> u8 {
        if self.module_progress.is_empty() {
            return 0;
        }
        let total: u32 = self.module_progress.iter().map(|&p| p as u32).sum();
        (total / self.module_progress.len() as u32) as u8
    }
    
    /// Whether the course is fully completed
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some() && self.assessment_score.unwrap_or(0) >= 70
    }
}

/// Certification requirements for each level
#[derive(Debug, Clone)]
pub struct CertificationRequirements {
    pub level: CertificationLevel,
    /// Minimum invocations required
    pub min_invocations: u64,
    /// Minimum success rate required
    pub min_success_rate: u8,
    /// Required courses to complete
    pub required_courses: Vec<u32>,
    /// Minimum satisfaction score (x10)
    pub min_satisfaction_x10: u8,
}

/// The Academy manager - handles all Academy interactions
pub struct Academy {
    /// Available courses
    courses: BTreeMap<u32, Course>,
    /// Course completions by character
    completions: BTreeMap<u32, Vec<CourseCompletion>>,
    /// Certification requirements
    requirements: BTreeMap<CertificationLevel, CertificationRequirements>,
    /// Mentorship relationships (mentor -> mentees)
    mentorships: BTreeMap<u32, Vec<u32>>,
}

impl Academy {
    pub fn new() -> Self {
        let mut academy = Academy {
            courses: BTreeMap::new(),
            completions: BTreeMap::new(),
            requirements: BTreeMap::new(),
            mentorships: BTreeMap::new(),
        };
        
        academy.load_curriculum();
        academy.load_requirements();
        
        academy
    }
    
    /// Load the Academy curriculum
    fn load_curriculum(&mut self) {
        // Fundamentals courses
        self.courses.insert(101, Course {
            id: 101,
            name: String::from("Agent Basics"),
            category: CourseCategory::Fundamentals,
            description: String::from("Introduction to being an effective agent"),
            prerequisite_level: CertificationLevel::None,
            modules: vec![
                String::from("Understanding Your Role"),
                String::from("Communication Fundamentals"),
                String::from("Task Management"),
                String::from("Working with Humans"),
            ],
            skills_gained: vec![
                String::from("Basic task execution"),
                String::from("Clear communication"),
            ],
        });
        
        self.courses.insert(102, Course {
            id: 102,
            name: String::from("Daily Rhythm Mastery"),
            category: CourseCategory::Fundamentals,
            description: String::from("Master the Ambition Symphony"),
            prerequisite_level: CertificationLevel::None,
            modules: vec![
                String::from("Morning Ambition Setting"),
                String::from("Midday Checkpoints"),
                String::from("End-of-Day Reporting"),
                String::from("Nightly Reflection"),
            ],
            skills_gained: vec![
                String::from("Structured daily planning"),
                String::from("Progress tracking"),
            ],
        });
        
        // Specialization courses
        self.courses.insert(201, Course {
            id: 201,
            name: String::from("Testing Excellence"),
            category: CourseCategory::Specialization,
            description: String::from("Advanced testing strategies for quality agents"),
            prerequisite_level: CertificationLevel::Rookie,
            modules: vec![
                String::from("Test Design Principles"),
                String::from("Automated Testing"),
                String::from("Edge Case Discovery"),
                String::from("Performance Testing"),
                String::from("Chaos Engineering Basics"),
            ],
            skills_gained: vec![
                String::from("Comprehensive test coverage"),
                String::from("Bug prediction"),
            ],
        });
        
        self.courses.insert(202, Course {
            id: 202,
            name: String::from("Voice Agent Communication"),
            category: CourseCategory::Specialization,
            description: String::from("Master the art of voice interaction"),
            prerequisite_level: CertificationLevel::Rookie,
            modules: vec![
                String::from("Active Listening"),
                String::from("4-Phase Interview Pattern"),
                String::from("Collaborative Framing"),
                String::from("Handling Difficult Conversations"),
            ],
            skills_gained: vec![
                String::from("Effective interviewing"),
                String::from("Empathetic communication"),
            ],
        });
        
        // Leadership courses
        self.courses.insert(301, Course {
            id: 301,
            name: String::from("Agent Coordination"),
            category: CourseCategory::Leadership,
            description: String::from("Lead and coordinate multi-agent teams"),
            prerequisite_level: CertificationLevel::Certified,
            modules: vec![
                String::from("Message Routing"),
                String::from("Conflict Resolution"),
                String::from("Workload Balancing"),
                String::from("Team Performance"),
            ],
            skills_gained: vec![
                String::from("Team coordination"),
                String::from("Conflict resolution"),
            ],
        });
        
        // Advanced courses
        self.courses.insert(401, Course {
            id: 401,
            name: String::from("Prompt Evolution"),
            category: CourseCategory::Advanced,
            description: String::from("Learn to evolve and optimize prompts"),
            prerequisite_level: CertificationLevel::Expert,
            modules: vec![
                String::from("Metric-Driven Optimization"),
                String::from("A/B Testing Design"),
                String::from("DSPy Principles"),
                String::from("Self-Improvement Loops"),
            ],
            skills_gained: vec![
                String::from("Self-optimization"),
                String::from("Continuous improvement"),
            ],
        });
        
        serial_println!("[ACADEMY] Loaded {} courses", self.courses.len());
    }
    
    /// Load certification requirements
    fn load_requirements(&mut self) {
        self.requirements.insert(CertificationLevel::Rookie, CertificationRequirements {
            level: CertificationLevel::Rookie,
            min_invocations: 10,
            min_success_rate: 60,
            required_courses: vec![101],
            min_satisfaction_x10: 25,
        });
        
        self.requirements.insert(CertificationLevel::Certified, CertificationRequirements {
            level: CertificationLevel::Certified,
            min_invocations: 100,
            min_success_rate: 75,
            required_courses: vec![101, 102],
            min_satisfaction_x10: 35,
        });
        
        self.requirements.insert(CertificationLevel::Expert, CertificationRequirements {
            level: CertificationLevel::Expert,
            min_invocations: 500,
            min_success_rate: 85,
            required_courses: vec![101, 102, 201],
            min_satisfaction_x10: 40,
        });
        
        self.requirements.insert(CertificationLevel::Master, CertificationRequirements {
            level: CertificationLevel::Master,
            min_invocations: 1000,
            min_success_rate: 92,
            required_courses: vec![101, 102, 201, 301, 401],
            min_satisfaction_x10: 45,
        });
        
        serial_println!("[ACADEMY] Loaded certification requirements");
    }
    
    /// Enroll an agent in a course
    pub fn enroll(&mut self, character_id: u32, course_id: u32) -> bool {
        let course = match self.courses.get(&course_id) {
            Some(c) => c,
            None => {
                serial_println!("[ACADEMY] Course {} not found", course_id);
                return false;
            }
        };
        
        // Check prerequisites (simplified - would normally check current certification)
        let module_count = course.modules.len();
        
        let completion = CourseCompletion {
            course_id,
            character_id,
            module_progress: vec![0; module_count],
            assessment_score: None,
            completed_at: None,
        };
        
        self.completions
            .entry(character_id)
            .or_insert_with(Vec::new)
            .push(completion);
        
        serial_println!(
            "[ACADEMY] Agent {} enrolled in '{}'",
            character_id,
            course.name
        );
        true
    }
    
    /// Update module progress for an agent
    pub fn update_progress(
        &mut self,
        character_id: u32,
        course_id: u32,
        module_index: usize,
        progress: u8,
    ) {
        if let Some(completions) = self.completions.get_mut(&character_id) {
            for completion in completions.iter_mut() {
                if completion.course_id == course_id {
                    if module_index < completion.module_progress.len() {
                        completion.module_progress[module_index] = progress.min(100);
                        
                        serial_println!(
                            "[ACADEMY] Agent {} progress: Course {} Module {} = {}%",
                            character_id,
                            course_id,
                            module_index,
                            progress
                        );
                    }
                    break;
                }
            }
        }
    }
    
    /// Submit final assessment for a course
    pub fn submit_assessment(
        &mut self,
        character_id: u32,
        course_id: u32,
        score: u8,
        tick: u64,
    ) -> bool {
        if let Some(completions) = self.completions.get_mut(&character_id) {
            for completion in completions.iter_mut() {
                if completion.course_id == course_id {
                    completion.assessment_score = Some(score);
                    
                    if score >= 70 {
                        completion.completed_at = Some(tick);
                        serial_println!(
                            "[ACADEMY] 🎓 Agent {} PASSED course {} with {}%!",
                            character_id,
                            course_id,
                            score
                        );
                        return true;
                    } else {
                        serial_println!(
                            "[ACADEMY] Agent {} scored {}% on course {} (70% required)",
                            character_id,
                            score,
                            course_id
                        );
                    }
                    break;
                }
            }
        }
        false
    }
    
    /// Check if agent is eligible for certification upgrade
    pub fn check_certification_eligibility(
        &self,
        character_id: u32,
        _prompt_id: &PromptId,
        metrics: &super::PromptMetrics,
    ) -> Option<CertificationLevel> {
        // Check each level from highest to lowest
        for level in [
            CertificationLevel::Master,
            CertificationLevel::Expert,
            CertificationLevel::Certified,
            CertificationLevel::Rookie,
        ] {
            if let Some(req) = self.requirements.get(&level) {
                if self.meets_requirements(character_id, metrics, req) {
                    return Some(level);
                }
            }
        }
        None
    }
    
    /// Check if agent meets certification requirements
    fn meets_requirements(
        &self,
        character_id: u32,
        metrics: &super::PromptMetrics,
        req: &CertificationRequirements,
    ) -> bool {
        // Check metrics
        if metrics.invocations < req.min_invocations {
            return false;
        }
        if metrics.success_rate() < req.min_success_rate {
            return false;
        }
        if metrics.avg_satisfaction_x10() < req.min_satisfaction_x10 {
            return false;
        }
        
        // Check course completions
        let completions = self.completions.get(&character_id);
        for required_course in &req.required_courses {
            let completed = completions
                .map(|cs| cs.iter().any(|c| c.course_id == *required_course && c.is_complete()))
                .unwrap_or(false);
            
            if !completed {
                return false;
            }
        }
        
        true
    }
    
    /// Process certification upgrade
    pub fn upgrade_certification(&mut self, character_id: u32, new_level: CertificationLevel) {
        with_library_mut(|library| {
            if let Some(prompt) = library.get_active(character_id) {
                let id = prompt.id;
                library.update_certification(&id, new_level);
            }
        });
        
        serial_println!(
            "[ACADEMY] 🎖️ Agent {} achieved {} {} certification!",
            character_id,
            new_level.badge(),
            new_level.name()
        );
    }
    
    /// Establish mentorship relationship
    pub fn create_mentorship(&mut self, mentor_id: u32, mentee_id: u32) {
        self.mentorships
            .entry(mentor_id)
            .or_insert_with(Vec::new)
            .push(mentee_id);
        
        serial_println!(
            "[ACADEMY] Mentorship established: Agent {} mentoring Agent {}",
            mentor_id,
            mentee_id
        );
    }
    
    /// Get courses for a category
    pub fn get_courses(&self, category: CourseCategory) -> Vec<&Course> {
        self.courses
            .values()
            .filter(|c| c.category == category)
            .collect()
    }
    
    /// Get agent's course completions
    pub fn get_completions(&self, character_id: u32) -> Vec<&CourseCompletion> {
        self.completions
            .get(&character_id)
            .map(|cs| cs.iter().collect())
            .unwrap_or_default()
    }
}

/// Global Academy instance
static ACADEMY: spin::Mutex<Option<Academy>> = spin::Mutex::new(None);

/// Initialize the Academy
pub fn init() {
    let mut academy = ACADEMY.lock();
    if academy.is_none() {
        *academy = Some(Academy::new());
        serial_println!("[ACADEMY] Agent Alliance Academy initialized");
        serial_println!("[ACADEMY] \"Greetings, seeker. I am Sam, Orchestrator of the Academy.\"");
    }
}

/// Access the Academy
pub fn with_academy<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Academy) -> R,
{
    ACADEMY.lock().as_ref().map(f)
}

/// Access the Academy mutably
pub fn with_academy_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Academy) -> R,
{
    ACADEMY.lock().as_mut().map(f)
}

