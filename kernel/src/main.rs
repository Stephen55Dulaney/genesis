//! # Genesis Kernel
//! 
//! The core of Project Genesis - an agentic operating system where AI agents
//! are first-class citizens at the kernel level.
//! 
//! Part of the QuantumDynamX umbrella.
//! "Where Agents, Classical, Quantum & Humans Collaborate Together"

#![no_std]  // Don't link the Rust standard library
#![no_main] // Disable all Rust-level entry points
#![feature(abi_x86_interrupt)]  // Required for interrupt handlers

extern crate alloc;  // Enable heap allocation types (Vec, String, Box)

use alloc::boxed::Box;
use alloc::string::String;
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

// Kernel modules
mod vga_buffer;
mod serial;
mod interrupts;
mod memory;
mod allocator;
mod agents;
mod shell;
mod gui;
mod storage;

use agents::supervisor::Supervisor;
use agents::thomas::Thomas;
use agents::archimedes::Archimedes;

// Define the kernel entry point
entry_point!(kernel_main);

/// Kernel entry point - called by the bootloader
/// 
/// This is where Genesis awakens.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize serial port for debug output
    serial::init();
    
    // Log to serial (appears in your terminal!)
    serial_println!("=====================================");
    serial_println!("  Genesis Kernel - Serial Debug Log");
    serial_println!("=====================================");
    serial_println!();
    serial_println!("[BOOT] Serial port initialized");
    serial_println!("[BOOT] VGA buffer at 0xb8000");
    
    // Clear the screen (but don't draw boot screen to text buffer)
    // We'll draw it in graphics mode instead to avoid QEMU display issues
    vga_buffer::clear_screen();
    serial_println!("[BOOT] Screen cleared");
    serial_println!("[BOOT] Skipping text-mode boot screen - will render in graphics mode");
    
    // Initialize memory management
    serial_println!();
    serial_println!("[MEMORY] Initializing memory management...");
    
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    
    serial_println!("[MEMORY] Page mapper initialized");
    serial_println!("[MEMORY] Frame allocator initialized");
    
    // Initialize the heap
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    
    serial_println!("[MEMORY] Heap initialized");
    
    // Stay in TEXT MODE for reliable shell input
    // Graphics will be initialized on-demand (e.g., when 'desktop' command is run)
    // This avoids VGA mode switching corruption issues
    serial_println!();
    serial_println!("[GRAPHICS] Staying in text mode - graphics available on-demand");
    
    // Display boot screen in text mode (reliable, no mode switching)
    display_boot_screen();
    serial_println!("[BOOT] Boot screen displayed in text mode");
    
    // Brief pause to see boot screen
    delay_ms(1000);
    
    // Clear screen for shell
    vga_buffer::clear_screen();
    serial_println!("[BOOT] Screen cleared - ready for shell");
    
    // Initialize interrupts (IDT + PIC)
    interrupts::init();
    
    // =========================================================================
    // AGENT SYSTEM INITIALIZATION
    // =========================================================================
    
    serial_println!();
    serial_println!("=========================================");
    serial_println!("  INITIALIZING AGENT SYSTEM");
    serial_println!("=========================================");
    serial_println!();
    
    // Create the supervisor (Sam)
    let mut supervisor = Supervisor::new();
    
    // Create agents
    // Archimedes - Daily Ambition Agent (loads ambition, organizes workspace)
    let archimedes_id = supervisor.next_id();
    let archimedes = Archimedes::new(archimedes_id);
    supervisor.register(Box::new(archimedes));
    
    // Thomas - Guardian/Tester Agent
    let thomas_id = supervisor.next_id();
    let thomas = Thomas::new(thomas_id);
    supervisor.register(Box::new(thomas));
    
    // =========================================================================
    // AGENT-FIRST BOOT SEQUENCE
    // =========================================================================
    
    // Phase 2: Agent Awakening (agents wake before GUI)
    // Phase 3: Environment Setup (agents organize before GUI)
    serial_println!();
    supervisor.agent_boot_sequence();
    
    // =========================================================================
    // DESKTOP LAYOUT - Render organized desktop
    // =========================================================================
    
    // Desktop and graphics console will be initialized on-demand when graphics mode is activated
    // (e.g., when 'desktop' command is run)
    serial_println!();
    serial_println!("[DESKTOP] Desktop available on-demand via 'desktop' command");
    console::add_output_line(String::from("Type commands here!"));
    serial_println!("[CONSOLE] Graphics console overlay initialized");
    serial_println!("[CONSOLE] Console should appear at bottom of graphics window");
    
    // Get Archimedes's ambition for desktop display
    // For now, we'll create a default layout - in full implementation,
    // we'd query Archimedes agent for its ambition
    desktop::with_desktop(|layout| {
        // Create split-screen layout: Conversation (left) + Ambition (right)
        layout.create_ambition_layout(
            Some("Today, I want us to build something amazing together."),
            &[
                String::from("YOU: Set clear goals"),
                String::from("AI: Support with tools and insights"),
                String::from("COLLAB: Work together"),
            ]
        );
    });
    
    serial_println!("[DESKTOP] Desktop layout created");
    
    // Render desktop (includes console overlay)
    serial_println!("[DESKTOP] Rendering organized desktop...");
    desktop::render();
    serial_println!("[DESKTOP] Desktop rendered!");
    
    // CRITICAL: Ensure desktop stays visible - render multiple times to force display
    serial_println!("[DESKTOP] Ensuring desktop stays visible...");
    for _ in 0..3 {
        desktop::render();
        delay_ms(100); // Small delay between renders
    }
    serial_println!("[DESKTOP] Desktop should now be visible in QEMU window!");
    
    // Print agent status
    supervisor.print_status();
    
    // Trigger morning ambition!
    serial_println!();
    supervisor.morning_ambition();
    
    // Run a few ticks
    serial_println!();
    serial_println!("[SUPERVISOR] Running tick loop...");
    for _ in 0..5 {
        supervisor.tick();
    }
    
    // End of day report
    serial_println!();
    supervisor.eod_report();
    
    // Update display
    println!();
    println!("  Agent Supervisor:    [ ONLINE ]");
    println!("  Active Agents:       [ {} ]", supervisor.agent_count());
    println!("  Memory Tier System:  [ ONLINE - Warm Tier ]");
    println!();
    println!("  Thomas says: \"All systems nominal!\"");
    println!();
    println!("  >> KEYBOARD INPUT ACTIVE - Type something! <<");

    serial_println!();
    serial_println!("=========================================");
    serial_println!("  GENESIS FULLY OPERATIONAL");
    serial_println!("  Agents: {}", supervisor.agent_count());
    serial_println!("  Tick: {}", supervisor.current_tick());
    serial_println!("=========================================");
    serial_println!();
    serial_println!("[INFO] Press Ctrl+A, X to exit QEMU");

    // Initialize shell
    shell::SHELL.lock().init();
    
    // Main loop - handle interrupts and process shell input
    loop {
        // Poll serial input (fallback if interrupts don't fire)
        // This ensures we get data from the bridge even without serial interrupts
        {
            use crate::serial::SERIAL1;
            let mut serial = SERIAL1.lock();
            while let Some(byte) = serial.try_receive() {
                // Send character to shell queue
                crate::shell::Shell::push_char(byte as char);
            }
        }
        
        // Process shell input (characters from keyboard or serial)
        shell::SHELL.lock().process_input(&mut supervisor);
        
        // Process agent ticks
        supervisor.tick();
        
        // Periodically re-render desktop in graphics mode to keep console visible
        // (This ensures console updates even if render wasn't triggered by input)
        if gui::graphics::current_mode() == gui::graphics::VgaMode::Graphics {
            static mut RENDER_COUNTER: u64 = 0;
            unsafe {
                RENDER_COUNTER += 1;
                // Re-render more frequently (every 100 ticks) to keep desktop visible
                // This prevents the screen from going black
                if RENDER_COUNTER % 100 == 0 {
                    gui::desktop::render();
                }
            }
        }
        
        // Halt the CPU until the next interrupt
        x86_64::instructions::hlt();
    }
}

/// Simple delay function using busy-wait loop
/// Approximate delay - actual time depends on CPU speed
/// 
/// Note: This is a very rough approximation. On QEMU/emulated systems,
/// timing may vary significantly. This is calibrated for QEMU.
fn delay_ms(milliseconds: u64) {
    // Much smaller iteration count for QEMU - emulation is slow
    // Roughly calibrated: ~10k iterations per millisecond in QEMU
    const ITERATIONS_PER_MS: u64 = 10_000;
    let iterations = milliseconds * ITERATIONS_PER_MS;
    
    for _ in 0..iterations {
        // Busy wait - prevents compiler from optimizing away the loop
        core::hint::spin_loop();
    }
}

/// Display the boot screen with ASCII art and status
fn display_boot_screen() {
    println!();
    println!("================================================================");
    println!();
    println!("   ██████╗ ███████╗███╗   ██╗███████╗███████╗██╗███████╗");
    println!("  ██╔════╝ ██╔════╝████╗  ██║██╔════╝██╔════╝██║██╔════╝");
    println!("  ██║  ███╗█████╗  ██╔██╗ ██║█████╗  ███████╗██║███████╗");
    println!("  ██║   ██║██╔══╝  ██║╚██╗██║██╔══╝  ╚════██║██║╚════██║");
    println!("  ╚██████╔╝███████╗██║ ╚████║███████╗███████║██║███████║");
    println!("   ╚═════╝ ╚══════╝╚═╝  ╚═══╝╚══════╝╚══════╝╚═╝╚══════╝");
    println!();
    println!("           A G E N T I C   O P E R A T I N G");
    println!("                   S Y S T E M");
    println!();
    println!("================================================================");
    println!();
    println!("  Genesis Awakening...");
    println!();
    println!("  QuantumDynamX.com");
    println!("  \"Where Agents, Classical, Quantum & Humans Collaborate\"");
    println!();
    println!("================================================================");
}

/// Panic handler - called when a panic occurs
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Print to both VGA and serial so we definitely see it
    println!();
    println!("================================================================");
    println!("  KERNEL PANIC - Genesis encountered an error");
    println!("================================================================");
    println!();
    println!("  {}", info);
    
    serial_println!();
    serial_println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    serial_println!("  KERNEL PANIC");
    serial_println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    serial_println!("{}", info);
    
    loop {
        x86_64::instructions::hlt();
    }
}
