//! Prompt Library - Storage and Retrieval System
//!
//! The library manages all prompts in the system, supporting:
//! - Version control (multiple versions of same character)
//! - A/B testing (run two versions side-by-side)
//! - Evolution tracking (parent-child relationships)
//! - Academy sync (certification updates)

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use crate::serial_println;

use super::{Prompt, PromptId, PromptRole, CertificationLevel};
use super::characters;

/// The central prompt library
pub struct PromptLibrary {
    /// All prompts indexed by their ID
    prompts: BTreeMap<PromptId, Prompt>,
    /// Active prompt for each character (character_id -> prompt_id)
    active_prompts: BTreeMap<u32, PromptId>,
    /// A/B test configurations (character_id -> [variant_a_id, variant_b_id])
    ab_tests: BTreeMap<u32, (PromptId, PromptId)>,
    /// Evolution history (prompt_id -> Vec<child_prompt_ids>)
    evolution_tree: BTreeMap<PromptId, Vec<PromptId>>,
}

impl PromptLibrary {
    /// Create a new empty library
    pub fn new() -> Self {
        PromptLibrary {
            prompts: BTreeMap::new(),
            active_prompts: BTreeMap::new(),
            ab_tests: BTreeMap::new(),
            evolution_tree: BTreeMap::new(),
        }
    }
    
    /// Initialize with all built-in character prompts
    pub fn with_defaults() -> Self {
        let mut library = Self::new();
        
        serial_println!("[PROMPT_LIBRARY] Loading built-in character prompts...");
        
        // Load all character prompts
        library.register(characters::sam_orchestrator());
        library.register(characters::archimedes_voice());
        library.register(characters::archimedes_silent());
        library.register(characters::thomas_tester());
        library.register(characters::pete_backend());
        library.register(characters::sentinel_security());
        library.register(characters::scout_researcher());
        library.register(characters::scribe_documenter());
        
        serial_println!("[PROMPT_LIBRARY] Loaded {} character prompts", library.prompts.len());
        
        library
    }
    
    /// Register a new prompt in the library
    pub fn register(&mut self, prompt: Prompt) {
        let id = prompt.id;
        let character_id = id.character_id;
        let is_active = prompt.is_active;
        let name = prompt.name.clone();
        
        // Track evolution
        if let Some(parent_id) = prompt.parent_id {
            self.evolution_tree
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(id);
        }
        
        // Store the prompt
        self.prompts.insert(id, prompt);
        
        // Set as active if flagged
        if is_active {
            self.active_prompts.insert(character_id, id);
        }
        
        serial_println!(
            "[PROMPT_LIBRARY] Registered: {} v{}",
            name,
            id.version_string()
        );
    }
    
    /// Get the active prompt for a character
    pub fn get_active(&self, character_id: u32) -> Option<&Prompt> {
        self.active_prompts
            .get(&character_id)
            .and_then(|id| self.prompts.get(id))
    }
    
    /// Get a specific prompt version
    pub fn get(&self, id: &PromptId) -> Option<&Prompt> {
        self.prompts.get(id)
    }
    
    /// Get mutable reference to a prompt
    pub fn get_mut(&mut self, id: &PromptId) -> Option<&mut Prompt> {
        self.prompts.get_mut(id)
    }
    
    /// Set a prompt as the active version for its character
    pub fn set_active(&mut self, id: PromptId) -> bool {
        // Check if prompt exists first
        if !self.prompts.contains_key(&id) {
            return false;
        }
        
        let character_id = id.character_id;
        
        // Deactivate current active (if different from new)
        if let Some(&old_id) = self.active_prompts.get(&character_id) {
            if old_id != id {
                if let Some(old_prompt) = self.prompts.get_mut(&old_id) {
                    old_prompt.is_active = false;
                }
            }
        }
        
        // Now activate the new one
        if let Some(prompt) = self.prompts.get_mut(&id) {
            prompt.is_active = true;
            self.active_prompts.insert(character_id, id);
            
            serial_println!(
                "[PROMPT_LIBRARY] Activated: {} v{}",
                prompt.name,
                id.version_string()
            );
            true
        } else {
            false
        }
    }
    
    /// Start an A/B test between two prompt versions
    pub fn start_ab_test(&mut self, variant_a: PromptId, variant_b: PromptId) -> bool {
        // Must be same character
        if variant_a.character_id != variant_b.character_id {
            serial_println!("[PROMPT_LIBRARY] A/B test failed: different characters");
            return false;
        }
        
        // Both must exist
        if !self.prompts.contains_key(&variant_a) || !self.prompts.contains_key(&variant_b) {
            serial_println!("[PROMPT_LIBRARY] A/B test failed: prompt not found");
            return false;
        }
        
        let character_id = variant_a.character_id;
        self.ab_tests.insert(character_id, (variant_a, variant_b));
        
        serial_println!(
            "[PROMPT_LIBRARY] Started A/B test: v{} vs v{}",
            variant_a.version_string(),
            variant_b.version_string()
        );
        true
    }
    
    /// Get A/B test variant (alternates based on invocation count)
    pub fn get_ab_variant(&self, character_id: u32, invocation: u64) -> Option<&Prompt> {
        self.ab_tests.get(&character_id).and_then(|(a, b)| {
            let id = if invocation % 2 == 0 { a } else { b };
            self.prompts.get(id)
        })
    }
    
    /// End A/B test and select winner based on metrics
    pub fn end_ab_test(&mut self, character_id: u32) -> Option<PromptId> {
        if let Some((a, b)) = self.ab_tests.remove(&character_id) {
            let a_rate = self.prompts.get(&a).map(|p| p.metrics.success_rate()).unwrap_or(0);
            let b_rate = self.prompts.get(&b).map(|p| p.metrics.success_rate()).unwrap_or(0);
            
            let winner = if a_rate >= b_rate { a } else { b };
            
            self.set_active(winner);
            
            serial_println!(
                "[PROMPT_LIBRARY] A/B test winner: v{} ({}% vs {}%)",
                winner.version_string(),
                a_rate.max(b_rate),
                a_rate.min(b_rate)
            );
            
            Some(winner)
        } else {
            None
        }
    }
    
    /// Get all prompts for a character (all versions)
    pub fn get_all_versions(&self, character_id: u32) -> Vec<&Prompt> {
        self.prompts
            .values()
            .filter(|p| p.id.character_id == character_id)
            .collect()
    }
    
    /// Get evolution history for a prompt
    pub fn get_children(&self, id: &PromptId) -> Vec<&Prompt> {
        self.evolution_tree
            .get(id)
            .map(|children| {
                children
                    .iter()
                    .filter_map(|child_id| self.prompts.get(child_id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Update certification for a prompt (synced from Academy)
    pub fn update_certification(&mut self, id: &PromptId, level: CertificationLevel) {
        if let Some(prompt) = self.prompts.get_mut(id) {
            let old_level = prompt.certification;
            prompt.certification = level;
            
            serial_println!(
                "[PROMPT_LIBRARY] {} certification: {} {} -> {} {}",
                prompt.name,
                old_level.badge(),
                old_level.name(),
                level.badge(),
                level.name()
            );
        }
    }
    
    /// Get prompts by role
    pub fn get_by_role(&self, role: PromptRole) -> Vec<&Prompt> {
        self.prompts
            .values()
            .filter(|p| p.role == role && p.is_active)
            .collect()
    }
    
    /// Get all certified prompts (Certified level or above)
    pub fn get_certified(&self) -> Vec<&Prompt> {
        self.prompts
            .values()
            .filter(|p| p.certification >= CertificationLevel::Certified)
            .collect()
    }
    
    /// Summary statistics
    pub fn stats(&self) -> LibraryStats {
        let total = self.prompts.len();
        let active = self.active_prompts.len();
        let certified = self.prompts.values()
            .filter(|p| p.certification >= CertificationLevel::Certified)
            .count();
        let ab_tests = self.ab_tests.len();
        
        LibraryStats {
            total_prompts: total,
            active_characters: active,
            certified_prompts: certified,
            running_ab_tests: ab_tests,
        }
    }
}

/// Library statistics
#[derive(Debug, Clone)]
pub struct LibraryStats {
    pub total_prompts: usize,
    pub active_characters: usize,
    pub certified_prompts: usize,
    pub running_ab_tests: usize,
}

/// Global prompt library singleton
static LIBRARY: Mutex<Option<PromptLibrary>> = Mutex::new(None);

/// Initialize the global prompt library
pub fn init() {
    let mut lib = LIBRARY.lock();
    if lib.is_none() {
        *lib = Some(PromptLibrary::with_defaults());
        serial_println!("[PROMPT_LIBRARY] Global library initialized");
    }
}

/// Access the global library
pub fn with_library<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&PromptLibrary) -> R,
{
    LIBRARY.lock().as_ref().map(f)
}

/// Access the global library mutably
pub fn with_library_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut PromptLibrary) -> R,
{
    LIBRARY.lock().as_mut().map(f)
}

