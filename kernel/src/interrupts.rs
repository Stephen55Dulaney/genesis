//! Interrupt Handling for Genesis
//!
//! This module sets up the Interrupt Descriptor Table (IDT) and handlers
//! for CPU exceptions and hardware interrupts.
//!
//! ## How Interrupts Work
//!
//! 1. Something happens (key press, timer tick, error)
//! 2. CPU looks up handler address in the IDT
//! 3. CPU saves current state and jumps to handler
//! 4. Handler runs (we process the event)
//! 5. Handler returns, CPU restores state
//!
//! ## For Genesis Agents
//!
//! Interrupts are the lowest-level "events" in the system. In the future,
//! agents could subscribe to interrupt types - the Perception Agent might
//! handle keyboard interrupts, while the Compute Router handles timer ticks.

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{println, serial_println};
use spin::{Lazy, Mutex};
use pic8259::ChainedPics;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyState};
// SHELL is accessed via crate::shell::Shell::push_char

/// PIC offset - we remap hardware interrupts to start at 32
const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Hardware interrupt numbers
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,           // IRQ 0 → Interrupt 32
    Keyboard = PIC_1_OFFSET + 1,    // IRQ 1 → Interrupt 33
    Serial1 = PIC_1_OFFSET + 4,     // IRQ 4 → Interrupt 36
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

/// The global Interrupt Descriptor Table
static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    
    // CPU Exceptions
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.double_fault.set_handler_fn(double_fault_handler);
    
    // Hardware Interrupts
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
    idt[InterruptIndex::Serial1.as_u8()].set_handler_fn(serial_interrupt_handler);
    
    idt
});

/// The two PICs (master and slave) that handle hardware interrupts
pub static PICS: Mutex<ChainedPics> = 
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Keyboard decoder - translates scancodes to characters
/// 
/// The PS/2 keyboard sends "scancodes" (raw numbers like 0x1E for 'A').
/// This decoder knows the US keyboard layout and converts them.
static KEYBOARD: Lazy<Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>>> = Lazy::new(|| {
    Mutex::new(Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    ))
});

/// Initialize the IDT and enable interrupts
pub fn init() {
    serial_println!("[INIT] Loading Interrupt Descriptor Table...");
    IDT.load();
    serial_println!("[INIT] IDT loaded successfully");
    
    serial_println!("[INIT] Initializing PICs...");
    unsafe { PICS.lock().initialize() };
    serial_println!("[INIT] PICs initialized (offset={})", PIC_1_OFFSET);
    
    serial_println!("[INIT] Enabling CPU interrupts...");
    x86_64::instructions::interrupts::enable();
    serial_println!("[INIT] Interrupts ENABLED - hardware can now talk to us!");
}

// ============================================================================
// Exception Handlers
// ============================================================================

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT");
    println!("{:#?}", stack_frame);
    serial_println!("[EXCEPTION] Breakpoint at {:?}", stack_frame.instruction_pointer);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("EXCEPTION: DOUBLE FAULT");
    println!("{:#?}", stack_frame);
    serial_println!("[FATAL] Double fault!");
    serial_println!("{:#?}", stack_frame);
    
    loop {
        x86_64::instructions::hlt();
    }
}

// ============================================================================
// Hardware Interrupt Handlers
// ============================================================================

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Future: Agent Scheduler would run here
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// Keyboard interrupt handler
/// 
/// This is called every time a key is pressed or released!
/// We decode the scancode and print the character to both VGA and serial.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    serial_println!("[KEYBOARD] Interrupt received!");
    
    // Read the scancode from the keyboard controller
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    serial_println!("[KEYBOARD] Scancode: 0x{:02X}", scancode);
    
    // Try to decode the scancode into a character
    let mut keyboard = KEYBOARD.lock();
    
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        // CRITICAL: Only process key PRESSES, not releases
        // This prevents duplicate character processing and modifier key noise
        if key_event.state != KeyState::Pressed {
            // Key release - ignore it (but still acknowledge interrupt)
            unsafe {
                PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
            }
            return;
        }
        
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    // Check if this is Enter/Return as Unicode (some keyboards send it this way)
                    if character == '\n' || character == '\r' {
                        serial_println!("[KEY] Enter/Return received as Unicode - executing command");
                        crate::shell::Shell::push_char('\n');
                    } else {
                        // Send character to shell queue
                        serial_println!("[KEY] Character received: '{}' (U+{:04X})", character, character as u32);
                        crate::shell::Shell::push_char(character);
                    }
                }
                DecodedKey::RawKey(key) => {
                    // Handle special keys
                    match key {
                        pc_keyboard::KeyCode::Enter => {
                            // CRITICAL: Enter key sends newline to execute command
                            serial_println!("[KEY] Enter key pressed - executing command");
                            crate::shell::Shell::push_char('\n');
                        }
                        pc_keyboard::KeyCode::Backspace => {
                            // Send backspace character to shell
                            crate::shell::Shell::push_char('\u{08}');
                        }
                        pc_keyboard::KeyCode::Escape => {
                            // Toggle between text and graphics mode (Mac-friendly alternative to F1)
                            serial_println!("[KEY] Escape pressed - toggling VGA mode...");
                            unsafe {
                                crate::gui::graphics::toggle_mode();
                            }
                            let current = crate::gui::graphics::current_mode();
                            serial_println!("[MODE] Switched to {:?} mode", current);
                        }
                        pc_keyboard::KeyCode::F1 => {
                            // Toggle between text and graphics mode
                            serial_println!("[KEY] F1 pressed - toggling VGA mode...");
                            unsafe {
                                crate::gui::graphics::toggle_mode();
                            }
                            let current = crate::gui::graphics::current_mode();
                            serial_println!("[MODE] Switched to {:?} mode", current);
                        }
                        pc_keyboard::KeyCode::ArrowLeft | 
                        pc_keyboard::KeyCode::ArrowRight |
                        pc_keyboard::KeyCode::ArrowUp |
                        pc_keyboard::KeyCode::ArrowDown |
                        pc_keyboard::KeyCode::F2 |
                        pc_keyboard::KeyCode::F3 |
                        pc_keyboard::KeyCode::F4 |
                        pc_keyboard::KeyCode::F5 |
                        pc_keyboard::KeyCode::F6 |
                        pc_keyboard::KeyCode::F7 |
                        pc_keyboard::KeyCode::F8 |
                        pc_keyboard::KeyCode::F9 |
                        pc_keyboard::KeyCode::F10 => {
                            serial_println!("[KEY] Special: {:?}", key);
                        }
                        pc_keyboard::KeyCode::F11 => {
                            // Exit fullscreen hint (QEMU fullscreen is controlled by QEMU, not Genesis)
                            serial_println!("[KEY] F11 pressed");
                            serial_println!("[INFO] To exit QEMU fullscreen: Press Ctrl+Alt+F (or Ctrl+Alt+G)");
                            serial_println!("[INFO] Or use Ctrl+A, X to quit QEMU entirely");
                        }
                        pc_keyboard::KeyCode::F12 => {
                            serial_println!("[KEY] Special: {:?}", key);
                        }
                        _ => {
                            // Log unhandled keys to debug Enter key issue
                            serial_println!("[KEY] Unhandled RawKey: {:?}", key);
                        }
                    }
                }
            }
        }
    }
    
    // Tell the PIC we handled the interrupt
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

/// Serial interrupt handler
/// 
/// This is called when data arrives on the serial port (COM1).
extern "x86-interrupt" fn serial_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use crate::serial::SERIAL1;
    
    let mut serial = SERIAL1.lock();
    while let Some(byte) = serial.try_receive() {
        // Send character to shell queue
        crate::shell::Shell::push_char(byte as char);
    }
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Serial1.as_u8());
    }
}
