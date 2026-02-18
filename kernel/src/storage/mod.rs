//! Persistent Storage System
//!
//! Provides file system capabilities for Genesis OS.
//! Agents use this to persist state, ambitions, and organize files.
//!
//! For now, we'll use a simple in-memory file system.
//! Later, we can add FAT32 or custom file system support.

pub mod filesystem;
pub mod memory_store;
