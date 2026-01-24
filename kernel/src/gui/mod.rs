//! GUI Module - Graphics Foundation for Genesis
//!
//! This module provides graphics capabilities for the Genesis OS.
//! Inspired by Bevy's simplicity, adapted for bare-metal `no_std` environment.
//!
//! ## Architecture
//!
//! - **GraphicsContext**: Main rendering context (like Bevy's rendering context)
//! - **VGA Graphics Mode**: Mode 13h (320x200x256 colors)
//! - **Drawing Primitives**: Pixel, rectangle, line, text
//! - **Double Buffering**: Smooth updates

pub mod graphics;
pub mod desktop;
pub mod console;
pub mod fonts;

