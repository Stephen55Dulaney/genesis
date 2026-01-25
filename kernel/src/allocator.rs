//! Heap Allocator for Genesis
//!
//! This module sets up the kernel heap, enabling dynamic memory allocation.
//! With this, we can use `Vec`, `String`, `Box`, and other heap-allocated types!
//!
//! ## How the Heap Works
//!
//! ```text
//! Virtual Memory Layout:
//! 
//! 0x0000_0000_0000 ─────────────────────
//!                   │ (unmapped/guard)  │
//! 0x0000_4444_0000 ─────────────────────
//!                   │                   │
//!                   │   KERNEL HEAP     │  ← We allocate here!
//!                   │   (512 KiB)       │
//!                   │                   │
//! 0x0000_4444_0000 + HEAP_SIZE ─────────
//!                   │ (rest of memory)  │
//! ```
//!
//! ## For Genesis Agents
//!
//! Each agent will have its own memory allocation patterns:
//! - Working memory: frequently accessed, kept in heap
//! - Short-term: recent context, may be evicted
//! - Long-term: persistent state, eventually stored to disk
//! - Archival: historical data, compressed/cloud

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use linked_list_allocator::LockedHeap;
use crate::serial_println;

/// Start address of the kernel heap
/// We pick an address that's unlikely to conflict with other mappings
pub const HEAP_START: usize = 0x_4444_4444_0000;

/// Size of the kernel heap (512 KiB to start - we can grow later)
pub const HEAP_SIZE: usize = 512 * 1024;

/// The global allocator instance
/// 
/// This is what Rust uses when you call `Box::new()`, `Vec::push()`, etc.
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the kernel heap.
///
/// Maps the heap pages and initializes the allocator.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    serial_println!("[HEAP] Initializing kernel heap...");
    serial_println!("[HEAP] Start: 0x{:X}", HEAP_START);
    serial_println!("[HEAP] Size:  {} KiB", HEAP_SIZE / 1024);
    
    // Calculate the page range we need to map
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Map each page in the heap range
    let page_count = page_range.count();
    serial_println!("[HEAP] Mapping {} pages...", page_count);
    
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    // Initialize the allocator with the heap region
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    serial_println!("[HEAP] Heap initialized successfully!");
    serial_println!("[HEAP] You can now use Vec, String, Box, etc.");
    
    Ok(())
}

/// Get heap usage statistics
pub fn heap_stats() -> (usize, usize) {
    let allocator = ALLOCATOR.lock();
    let used = allocator.used();
    let free = allocator.free();
    (used, free)
}

