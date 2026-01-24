//! Memory Management for Genesis
//!
//! This module handles physical and virtual memory allocation.
//! 
//! ## Memory Hierarchy (Future: 4-Tier Agent Memory)
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  HOT TIER   - GPU VRAM (future)         │
//! ├─────────────────────────────────────────┤
//! │  WARM TIER  - System RAM ← WE ARE HERE  │
//! ├─────────────────────────────────────────┤
//! │  COLD TIER  - SSD/NVMe (future)         │
//! ├─────────────────────────────────────────┤
//! │  DEEP TIER  - Cloud/Quantum (future)    │
//! └─────────────────────────────────────────┘
//! ```
//!
//! For now, we implement basic heap allocation in RAM.
//! This is the foundation for agent state storage.

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTable, PhysFrame, Size4KiB,
        mapper::MapToError, OffsetPageTable,
    },
    PhysAddr, VirtAddr,
};

/// Initialize a new OffsetPageTable.
///
/// # Safety
/// The caller must guarantee that the complete physical memory is mapped
/// to virtual memory at the passed `physical_memory_offset`.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 page table.
///
/// # Safety
/// The caller must guarantee that the complete physical memory is mapped
/// to virtual memory at the passed `physical_memory_offset`.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// # Safety
    /// The caller must guarantee that the passed memory map is valid.
    /// The main requirement is that all frames marked as `USABLE` are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // Get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        
        // Map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        
        // Transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        
        // Create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Create a mapping for a page in the page table.
/// 
/// This is used by the heap allocator to map virtual addresses to physical frames.
pub fn create_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = frame_allocator
        .allocate_frame()
        .ok_or(MapToError::FrameAllocationFailed)?;
    
    let flags = Flags::PRESENT | Flags::WRITABLE;

    unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)?.flush();
    }

    Ok(())
}

