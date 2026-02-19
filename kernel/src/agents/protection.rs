//! Protection Tiers for Genesis Agent Governance
//!
//! Defines five tiers that control the **level of planning and scrutiny**
//! required before a change is made. Agents write ALL the code (Stephen
//! doesn't write Rust) — the tiers define how much we think first.
//!
//! ```text
//!   Tier 1: Core       — Discuss, assess risk, agree, then change carefully.
//!   Tier 2: Guarded    — Talk through the approach, verify impact.
//!   Tier 3: Maintained — Build and verify. Agents self-improve here.
//!   Tier 4: Playground — Full autonomy. Experiment freely.
//!   Tier 5: Sandbox    — Secrets. Don't touch, don't log, don't expose.
//! ```
//!
//! Agents are ENCOURAGED to propose improvements at every tier. The system
//! is designed to self-improve. The tiers just ensure we don't self-improve
//! out of existence.
//!
//! See: docs/PROTECTION_TIERS.md for the full specification.

use alloc::string::String;
use core::fmt;

/// The five protection tiers, ordered by required ceremony level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProtectionTier {
    /// Tier 1: Core — Discuss, assess risk, agree, then change carefully.
    Core = 1,
    /// Tier 2: Guarded — Talk through the approach, verify impact.
    Guarded = 2,
    /// Tier 3: Maintained — Build and verify. Agents self-improve here.
    Maintained = 3,
    /// Tier 4: Playground — Full autonomy. Experiment freely.
    Playground = 4,
    /// Tier 5: Sandbox — Secrets. Don't touch, don't log, don't expose.
    Sandbox = 5,
}

impl ProtectionTier {
    /// Short human-readable name for display.
    pub fn name(&self) -> &'static str {
        match self {
            ProtectionTier::Core => "Core",
            ProtectionTier::Guarded => "Guarded",
            ProtectionTier::Maintained => "Maintained",
            ProtectionTier::Playground => "Playground",
            ProtectionTier::Sandbox => "Sandbox",
        }
    }

    /// Badge for display (matches Academy certification style).
    pub fn badge(&self) -> &'static str {
        match self {
            ProtectionTier::Core => "🔴",
            ProtectionTier::Guarded => "🟠",
            ProtectionTier::Maintained => "🟡",
            ProtectionTier::Playground => "🟢",
            ProtectionTier::Sandbox => "⛔",
        }
    }

    /// Description of what ceremony is required at this tier.
    pub fn ceremony(&self) -> &'static str {
        match self {
            ProtectionTier::Core => "Discuss risk, agree on approach, verify build",
            ProtectionTier::Guarded => "Talk through approach, check impact, verify build",
            ProtectionTier::Maintained => "Build and verify, spot-check later",
            ProtectionTier::Playground => "Full autonomy, provenance tracked",
            ProtectionTier::Sandbox => "Do not read, modify, or expose",
        }
    }

    /// Whether this tier requires discussion with Stephen before changes.
    pub fn requires_discussion(&self) -> bool {
        matches!(self, ProtectionTier::Core | ProtectionTier::Guarded)
    }

    /// Whether agents can freely modify files at this tier.
    pub fn allows_autonomous_change(&self) -> bool {
        matches!(self, ProtectionTier::Maintained | ProtectionTier::Playground)
    }

    /// Whether agents should avoid touching files at this tier entirely.
    pub fn is_restricted(&self) -> bool {
        matches!(self, ProtectionTier::Sandbox)
    }

    /// Whether agents should proactively recommend improvements here.
    /// True for all tiers except Sandbox — the system should self-improve.
    pub fn encourage_recommendations(&self) -> bool {
        !matches!(self, ProtectionTier::Sandbox)
    }
}

impl fmt::Display for ProtectionTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Tier {} ({})", self.badge(), *self as u8, self.name())
    }
}

/// The type of change an agent wants to make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// Reading file contents (allowed everywhere except Sandbox)
    Read,
    /// Proposing a modification (always allowed, tier sets ceremony)
    Modify,
    /// Creating a new file
    Create,
    /// Deleting a file
    Delete,
    /// Recommending a refactor or improvement
    Recommend,
    /// Accessing secret/credential content
    AccessSecret,
}

impl ChangeKind {
    pub fn description(&self) -> &'static str {
        match self {
            ChangeKind::Read => "Read file contents",
            ChangeKind::Modify => "Modify file",
            ChangeKind::Create => "Create new file",
            ChangeKind::Delete => "Delete file",
            ChangeKind::Recommend => "Recommend improvement",
            ChangeKind::AccessSecret => "Access secret/credential",
        }
    }
}

/// Determine the protection tier of a file path.
///
/// Paths are matched from most specific to least specific.
/// Unknown paths default to Tier 4 (Playground).
pub fn file_tier(path: &str) -> ProtectionTier {
    // Normalize: strip leading slashes and "genesis/" prefix
    let path = path.trim_start_matches('/');
    let path = path.strip_prefix("genesis/").unwrap_or(path);

    // ── Tier 5: Sandbox (secrets, credentials) ──────────────────────
    if path.starts_with("memory/secrets/")
        || path == "tools/.env.telegram"
        || path.ends_with(".secret")
        || path.ends_with(".key")
        || path.starts_with("sandbox/")
    {
        return ProtectionTier::Sandbox;
    }

    // ── Tier 1: Core (kernel fundamentals, build config) ────────────
    if path == "kernel/src/main.rs"
        || path == "kernel/src/interrupts.rs"
        || path == "kernel/src/memory.rs"
        || path == "kernel/src/allocator.rs"
        || path == "kernel/src/serial.rs"
        || path == "kernel/src/agents/mod.rs"
        || path == "kernel/src/agents/protection.rs"
        || path == "Cargo.toml"
        || path == "kernel/Cargo.toml"
        || path == "x86_64-genesis.json"
        || path == "rust-toolchain.toml"
        || path == ".genesis-manifest.json"
    {
        return ProtectionTier::Core;
    }

    // ── Tier 2: Guarded (supervisor, messaging, storage, bridge) ────
    if path == "kernel/src/agents/supervisor.rs"
        || path == "kernel/src/agents/message.rs"
        || path.starts_with("kernel/src/storage/")
        || path == "kernel/src/shell.rs"
        || path == "tools/genesis-bridge.py"
    {
        return ProtectionTier::Guarded;
    }

    // ── Tier 3: Maintained (agent impls, prompts, GUI, tool scripts) ─
    if path == "kernel/src/agents/thomas.rs"
        || path == "kernel/src/agents/archimedes.rs"
        || path.starts_with("kernel/src/agents/prompts/")
        || path.starts_with("kernel/src/gui/")
        || path == "kernel/src/vga_buffer.rs"
        || path == "tools/genesis_memory_persist.py"
        || path == "tools/genesis_vision_system.py"
        || path == "tools/qemu-run.sh"
        || path == "tools/qemu-debug.sh"
        || path == "tools/test-genesis.sh"
    {
        return ProtectionTier::Maintained;
    }

    // ── Tier 4: Playground (everything else) ────────────────────────
    ProtectionTier::Playground
}

/// Result of a tier check.
#[derive(Debug, Clone)]
pub struct TierCheck {
    pub path: String,
    pub tier: ProtectionTier,
    pub change: ChangeKind,
    pub proceed: bool,
    pub ceremony: &'static str,
}

/// Check what ceremony is required for a change to a given file.
///
/// Returns a TierCheck that tells the agent:
/// - proceed=true: You can go ahead (but follow the ceremony)
/// - proceed=false: You must not do this (only for Sandbox access)
///
/// Agents are NEVER blocked from proposing improvements — the tier just
/// determines how much discussion happens first.
pub fn check(path: &str, change: ChangeKind) -> TierCheck {
    let tier = file_tier(path);

    let (proceed, ceremony) = match change {
        ChangeKind::AccessSecret => {
            (false, "Secrets must never be accessed or exposed by agents")
        }
        ChangeKind::Read if tier.is_restricted() => {
            (false, "Sandbox files must not be read by agents")
        }
        ChangeKind::Modify | ChangeKind::Create | ChangeKind::Delete if tier.is_restricted() => {
            (false, "Sandbox files must not be modified by agents")
        }
        ChangeKind::Recommend => {
            // Recommendations are ALWAYS welcome at every non-sandbox tier
            if tier.is_restricted() {
                (false, "Cannot recommend changes to sandbox files")
            } else {
                (true, tier.ceremony())
            }
        }
        _ => {
            // All other changes proceed — the tier just sets the ceremony
            (true, tier.ceremony())
        }
    };

    TierCheck {
        path: String::from(path),
        tier,
        change,
        proceed,
        ceremony,
    }
}

/// Print a summary of all tiers for the shell `protection` command.
pub fn print_tier_summary() {
    use crate::serial_println;

    serial_println!();
    serial_println!("  === PROTECTION TIERS ===");
    serial_println!();
    serial_println!("  {} Tier 1 - Core:       Discuss risk, agree, then change.", ProtectionTier::Core.badge());
    serial_println!("  {} Tier 2 - Guarded:    Talk through approach, verify.", ProtectionTier::Guarded.badge());
    serial_println!("  {} Tier 3 - Maintained: Build & verify. Self-improve here.", ProtectionTier::Maintained.badge());
    serial_println!("  {} Tier 4 - Playground: Full autonomy. Experiment freely.", ProtectionTier::Playground.badge());
    serial_println!("  {} Tier 5 - Sandbox:    Secrets. Don't touch.", ProtectionTier::Sandbox.badge());
    serial_println!();
    serial_println!("  Agents propose improvements at EVERY tier.");
    serial_println!("  Tiers define how much we plan first, not who can code.");
    serial_println!();
    serial_println!("  See: docs/PROTECTION_TIERS.md");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_files_are_tier_1() {
        assert_eq!(file_tier("kernel/src/main.rs"), ProtectionTier::Core);
        assert_eq!(file_tier("kernel/src/interrupts.rs"), ProtectionTier::Core);
        assert_eq!(file_tier("kernel/src/agents/mod.rs"), ProtectionTier::Core);
        assert_eq!(file_tier("Cargo.toml"), ProtectionTier::Core);
    }

    #[test]
    fn guarded_files_are_tier_2() {
        assert_eq!(file_tier("kernel/src/agents/supervisor.rs"), ProtectionTier::Guarded);
        assert_eq!(file_tier("kernel/src/storage/memory_store.rs"), ProtectionTier::Guarded);
        assert_eq!(file_tier("tools/genesis-bridge.py"), ProtectionTier::Guarded);
    }

    #[test]
    fn maintained_files_are_tier_3() {
        assert_eq!(file_tier("kernel/src/agents/thomas.rs"), ProtectionTier::Maintained);
        assert_eq!(file_tier("kernel/src/agents/prompts/library.rs"), ProtectionTier::Maintained);
        assert_eq!(file_tier("kernel/src/gui/desktop.rs"), ProtectionTier::Maintained);
    }

    #[test]
    fn playground_files_are_tier_4() {
        assert_eq!(file_tier("lib/vision.py"), ProtectionTier::Playground);
        assert_eq!(file_tier("docs/MEMORY_SYSTEM.md"), ProtectionTier::Playground);
        assert_eq!(file_tier("sessions/2026/02/session.md"), ProtectionTier::Playground);
    }

    #[test]
    fn sandbox_files_are_tier_5() {
        assert_eq!(file_tier("memory/secrets/api_key_gemini.json"), ProtectionTier::Sandbox);
        assert_eq!(file_tier("tools/.env.telegram"), ProtectionTier::Sandbox);
        assert_eq!(file_tier("something.secret"), ProtectionTier::Sandbox);
    }

    #[test]
    fn recommendations_always_welcome() {
        let result = check("kernel/src/main.rs", ChangeKind::Recommend);
        assert!(result.proceed);
    }

    #[test]
    fn sandbox_blocks_access() {
        let result = check("memory/secrets/api_key.json", ChangeKind::Read);
        assert!(!result.proceed);
    }

    #[test]
    fn core_changes_proceed_with_ceremony() {
        let result = check("kernel/src/main.rs", ChangeKind::Modify);
        assert!(result.proceed); // Allowed — just needs discussion first
        assert!(result.tier.requires_discussion());
    }
}
