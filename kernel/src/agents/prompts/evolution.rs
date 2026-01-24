//! DSPy-Inspired Prompt Evolution System
//!
//! This module provides tools for automatically optimizing prompts
//! based on measured performance. Inspired by DSPy's approach of
//! treating prompts as programs that can be optimized.
//!
//! ## The Evolution Cycle
//!
//! ```text
//! ┌────────────┐     ┌────────────┐     ┌────────────┐
//! │  Deploy    │────▶│  Measure   │────▶│  Analyze   │
//! │  Prompt    │     │  Metrics   │     │  Results   │
//! └────────────┘     └────────────┘     └────────────┘
//!       ▲                                      │
//!       │            ┌────────────┐            │
//!       └────────────│  Generate  │◀───────────┘
//!                    │  Variant   │
//!                    └────────────┘
//! ```
//!
//! ## Optimization Strategies
//!
//! 1. **A/B Testing**: Compare two variants head-to-head
//! 2. **Incremental Refinement**: Small improvements over time
//! 3. **Template Mutation**: Vary specific sections
//! 4. **Metric-Driven Selection**: Promote best performers

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::serial_println;

use super::{Prompt, PromptId, CertificationLevel};
use super::library::{PromptLibrary, with_library_mut};

/// An optimization experiment
#[derive(Debug, Clone)]
pub struct Experiment {
    /// Unique experiment ID
    pub id: u64,
    /// Character being optimized
    pub character_id: u32,
    /// Baseline prompt version
    pub baseline_id: PromptId,
    /// Variant prompt version
    pub variant_id: PromptId,
    /// Number of trials to run
    pub target_trials: u64,
    /// Current trial count
    pub current_trials: u64,
    /// Experiment status
    pub status: ExperimentStatus,
    /// What we're testing
    pub hypothesis: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperimentStatus {
    /// Experiment is collecting data
    Running,
    /// Enough data collected, awaiting analysis
    Complete,
    /// Analysis done, winner selected
    Concluded,
    /// Experiment was cancelled
    Cancelled,
}

/// Result of analyzing an experiment
#[derive(Debug, Clone)]
pub struct ExperimentResult {
    pub experiment_id: u64,
    pub winner: PromptId,
    pub loser: PromptId,
    pub baseline_success_rate: u8,
    pub variant_success_rate: u8,
    pub confidence: f32, // Simplified confidence score
    pub recommendation: String,
}

/// Optimization suggestion for a prompt
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub prompt_id: PromptId,
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub priority: u8, // 1-10, higher = more important
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionType {
    /// Success rate is low
    LowSuccessRate,
    /// Completion time is slow
    SlowCompletion,
    /// Low user satisfaction
    LowSatisfaction,
    /// Not enough data
    InsufficientData,
    /// Ready for certification upgrade
    CertificationReady,
    /// Performance regression detected
    Regression,
}

/// The Evolution Engine - manages prompt optimization
pub struct EvolutionEngine {
    /// Active experiments
    experiments: BTreeMap<u64, Experiment>,
    /// Next experiment ID
    next_experiment_id: u64,
    /// Historical results
    results: Vec<ExperimentResult>,
    /// Optimization suggestions
    suggestions: Vec<OptimizationSuggestion>,
}

impl EvolutionEngine {
    pub fn new() -> Self {
        EvolutionEngine {
            experiments: BTreeMap::new(),
            next_experiment_id: 1,
            results: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    /// Start a new optimization experiment
    pub fn start_experiment(
        &mut self,
        library: &mut PromptLibrary,
        baseline_id: PromptId,
        variant_prompt: &str,
        hypothesis: &str,
        target_trials: u64,
    ) -> Option<u64> {
        // Get baseline prompt
        let baseline = library.get(&baseline_id)?;
        
        // Create variant (evolved version)
        let variant = baseline.evolve(variant_prompt, false);
        let variant_id = variant.id;
        
        // Register variant in library
        library.register(variant);
        
        // Start A/B test
        library.start_ab_test(baseline_id, variant_id);
        
        // Create experiment record
        let experiment_id = self.next_experiment_id;
        self.next_experiment_id += 1;
        
        let experiment = Experiment {
            id: experiment_id,
            character_id: baseline_id.character_id,
            baseline_id,
            variant_id,
            target_trials: target_trials,
            current_trials: 0,
            status: ExperimentStatus::Running,
            hypothesis: String::from(hypothesis),
        };
        
        self.experiments.insert(experiment_id, experiment);
        
        serial_println!(
            "[EVOLUTION] Started experiment {}: Testing hypothesis '{}'",
            experiment_id,
            hypothesis
        );
        
        Some(experiment_id)
    }
    
    /// Record a trial result for an experiment
    pub fn record_trial(
        &mut self,
        experiment_id: u64,
        was_variant: bool,
        success: bool,
        completion_ticks: u64,
        satisfaction: Option<u8>,
    ) {
        if let Some(experiment) = self.experiments.get_mut(&experiment_id) {
            experiment.current_trials += 1;
            
            // Update metrics in library
            with_library_mut(|library| {
                let prompt_id = if was_variant {
                    experiment.variant_id
                } else {
                    experiment.baseline_id
                };
                
                if let Some(prompt) = library.get_mut(&prompt_id) {
                    if success {
                        prompt.metrics.record_success(completion_ticks);
                    } else {
                        prompt.metrics.record_failure();
                    }
                    
                    if let Some(rating) = satisfaction {
                        prompt.metrics.record_satisfaction(rating);
                    }
                }
            });
            
            // Check if experiment is complete
            if experiment.current_trials >= experiment.target_trials {
                experiment.status = ExperimentStatus::Complete;
                serial_println!(
                    "[EVOLUTION] Experiment {} complete - {} trials collected",
                    experiment_id,
                    experiment.current_trials
                );
            }
        }
    }
    
    /// Analyze a completed experiment and select winner
    pub fn conclude_experiment(&mut self, experiment_id: u64) -> Option<ExperimentResult> {
        let experiment = self.experiments.get_mut(&experiment_id)?;
        
        if experiment.status != ExperimentStatus::Complete {
            serial_println!("[EVOLUTION] Experiment {} not ready for analysis", experiment_id);
            return None;
        }
        
        // Get metrics from library
        let (baseline_rate, variant_rate) = with_library_mut(|library| {
            let baseline = library.get(&experiment.baseline_id)
                .map(|p| p.metrics.success_rate())
                .unwrap_or(0);
            let variant = library.get(&experiment.variant_id)
                .map(|p| p.metrics.success_rate())
                .unwrap_or(0);
            (baseline, variant)
        }).unwrap_or((0, 0));
        
        // Determine winner
        let (winner, loser) = if variant_rate > baseline_rate {
            (experiment.variant_id, experiment.baseline_id)
        } else {
            (experiment.baseline_id, experiment.variant_id)
        };
        
        // Calculate simple confidence (difference in rates)
        let confidence = if baseline_rate == variant_rate {
            0.5
        } else {
            let diff = (baseline_rate as f32 - variant_rate as f32).abs();
            (diff / 100.0).min(1.0)
        };
        
        // Build recommendation
        let recommendation = if variant_rate > baseline_rate + 5 {
            String::from("Strong evidence to adopt variant. Deploy as new active.")
        } else if baseline_rate > variant_rate + 5 {
            String::from("Baseline performs better. Keep current version.")
        } else {
            String::from("No significant difference. Consider other factors.")
        };
        
        // Promote winner
        with_library_mut(|library| {
            library.set_active(winner);
            library.end_ab_test(experiment.character_id);
        });
        
        experiment.status = ExperimentStatus::Concluded;
        
        let result = ExperimentResult {
            experiment_id,
            winner,
            loser,
            baseline_success_rate: baseline_rate,
            variant_success_rate: variant_rate,
            confidence,
            recommendation,
        };
        
        serial_println!(
            "[EVOLUTION] Experiment {} concluded: Winner v{} ({}% vs {}%)",
            experiment_id,
            winner.version_string(),
            baseline_rate.max(variant_rate),
            baseline_rate.min(variant_rate)
        );
        
        self.results.push(result.clone());
        Some(result)
    }
    
    /// Analyze all prompts and generate optimization suggestions
    pub fn analyze_all(&mut self, library: &PromptLibrary) -> Vec<OptimizationSuggestion> {
        self.suggestions.clear();
        
        for character_id in 1..=8 {
            if let Some(prompt) = library.get_active(character_id) {
                self.analyze_prompt(prompt);
            }
        }
        
        // Sort by priority
        self.suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        serial_println!(
            "[EVOLUTION] Generated {} optimization suggestions",
            self.suggestions.len()
        );
        
        self.suggestions.clone()
    }
    
    /// Analyze a single prompt for optimization opportunities
    fn analyze_prompt(&mut self, prompt: &Prompt) {
        let metrics = &prompt.metrics;
        
        // Check for insufficient data
        if metrics.invocations < 10 {
            self.suggestions.push(OptimizationSuggestion {
                prompt_id: prompt.id,
                suggestion_type: SuggestionType::InsufficientData,
                description: String::from("Need more usage data (< 10 invocations)"),
                priority: 3,
            });
            return;
        }
        
        // Check success rate
        let success_rate = metrics.success_rate();
        if success_rate < 70 {
            self.suggestions.push(OptimizationSuggestion {
                prompt_id: prompt.id,
                suggestion_type: SuggestionType::LowSuccessRate,
                description: alloc::format!(
                    "Low success rate ({}%). Consider prompt refinement.",
                    success_rate
                ),
                priority: 8,
            });
        }
        
        // Check satisfaction
        let satisfaction = metrics.avg_satisfaction_x10();
        if satisfaction < 35 && metrics.satisfaction_count > 5 {
            self.suggestions.push(OptimizationSuggestion {
                prompt_id: prompt.id,
                suggestion_type: SuggestionType::LowSatisfaction,
                description: alloc::format!(
                    "Low user satisfaction ({}/5). Review agent personality.",
                    satisfaction / 10
                ),
                priority: 7,
            });
        }
        
        // Check for certification upgrade eligibility
        if success_rate >= 90 
            && metrics.invocations >= 100 
            && prompt.certification < CertificationLevel::Expert 
        {
            self.suggestions.push(OptimizationSuggestion {
                prompt_id: prompt.id,
                suggestion_type: SuggestionType::CertificationReady,
                description: String::from("High performance! Consider certification upgrade."),
                priority: 5,
            });
        }
    }
    
    /// Get active experiments
    pub fn active_experiments(&self) -> Vec<&Experiment> {
        self.experiments
            .values()
            .filter(|e| e.status == ExperimentStatus::Running)
            .collect()
    }
    
    /// Get experiment by ID
    pub fn get_experiment(&self, id: u64) -> Option<&Experiment> {
        self.experiments.get(&id)
    }
    
    /// Get historical results
    pub fn get_results(&self) -> &[ExperimentResult] {
        &self.results
    }
}

/// Global evolution engine
static EVOLUTION: spin::Mutex<Option<EvolutionEngine>> = spin::Mutex::new(None);

/// Initialize the evolution engine
pub fn init() {
    let mut engine = EVOLUTION.lock();
    if engine.is_none() {
        *engine = Some(EvolutionEngine::new());
        serial_println!("[EVOLUTION] Engine initialized");
    }
}

/// Access the evolution engine
pub fn with_engine<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&EvolutionEngine) -> R,
{
    EVOLUTION.lock().as_ref().map(f)
}

/// Access the evolution engine mutably
pub fn with_engine_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut EvolutionEngine) -> R,
{
    EVOLUTION.lock().as_mut().map(f)
}

