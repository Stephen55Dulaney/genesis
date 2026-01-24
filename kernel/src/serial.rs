//! Serial Port Driver (COM1)
//! 
//! This module provides output to the serial port, which appears in your
//! terminal when running QEMU with `-serial stdio`. Essential for debugging!
//!
//! ## How Serial Communication Works
//! 
//! The 16550 UART (Universal Asynchronous Receiver/Transmitter) is a chip
//! that converts parallel data to serial and vice versa. On x86 PCs:
//! 
//! - COM1 is at I/O port 0x3F8
//! - We write bytes one at a time
//! - The chip handles timing and transmission
//!
//! ## Why This Matters for Genesis
//! 
//! In the agentic future, serial could be one channel agents use to
//! communicate with external systems - a simple, reliable protocol.

use spin::Mutex;
use x86_64::instructions::port::Port;
use core::fmt;

/// COM1 base port address
const COM1: u16 = 0x3F8;

/// Global serial port instance
pub static SERIAL1: Mutex<SerialPort> = Mutex::new(SerialPort::new(COM1));

/// A serial port for text output
pub struct SerialPort {
    /// Data port - where we send/receive bytes
    data: Port<u8>,
    /// Interrupt enable register
    int_enable: Port<u8>,
    /// FIFO control register  
    fifo_ctrl: Port<u8>,
    /// Line control register (data bits, stop bits, parity)
    line_ctrl: Port<u8>,
    /// Modem control register
    modem_ctrl: Port<u8>,
    /// Line status register - tells us if we can send
    line_status: Port<u8>,
}

impl SerialPort {
    /// Create a new serial port at the given base address
    pub const fn new(base: u16) -> Self {
        SerialPort {
            data: Port::new(base),
            int_enable: Port::new(base + 1),
            fifo_ctrl: Port::new(base + 2),
            line_ctrl: Port::new(base + 3),
            modem_ctrl: Port::new(base + 4),
            line_status: Port::new(base + 5),
        }
    }

    /// Initialize the serial port
    /// 
    /// This configures:
    /// - Baud rate: 38400 (fast enough for debugging)
    /// - Data bits: 8
    /// - Stop bits: 1  
    /// - Parity: None
    /// - FIFO enabled
    pub fn init(&mut self) {
        unsafe {
            // Disable interrupts
            self.int_enable.write(0x00);

            // Enable DLAB (Divisor Latch Access Bit) to set baud rate
            self.line_ctrl.write(0x80);

            // Set divisor to 3 (38400 baud)
            // Divisor = 115200 / desired_baud
            // 115200 / 38400 = 3
            self.data.write(0x03);      // Low byte
            self.int_enable.write(0x00); // High byte

            // 8 bits, no parity, one stop bit (8N1)
            self.line_ctrl.write(0x03);

            // Enable FIFO, clear them, 14-byte threshold
            self.fifo_ctrl.write(0xC7);

            // IRQs enabled, RTS/DSR set
            self.modem_ctrl.write(0x0B);

            // Enable interrupts (for future use)
            self.int_enable.write(0x01);
        }
    }

    /// Check if the transmit buffer is empty (safe to send)
    fn is_transmit_empty(&mut self) -> bool {
        unsafe { self.line_status.read() & 0x20 != 0 }
    }

    /// Check if there is data waiting to be read
    pub fn is_receive_empty(&mut self) -> bool {
        unsafe { self.line_status.read() & 1 == 0 }
    }

    /// Read a single byte from the serial port
    pub fn receive(&mut self) -> u8 {
        // Wait for data to be available
        while self.is_receive_empty() {
            core::hint::spin_loop();
        }
        
        unsafe {
            self.data.read()
        }
    }

    /// Try to read a byte without blocking
    pub fn try_receive(&mut self) -> Option<u8> {
        if self.is_receive_empty() {
            None
        } else {
            unsafe { Some(self.data.read()) }
        }
    }

    /// Send a single byte
    pub fn send(&mut self, byte: u8) {
        // Wait for transmit buffer to be empty
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        
        unsafe {
            self.data.write(byte);
        }
    }

    /// Send a string
    pub fn send_string(&mut self, s: &str) {
        for byte in s.bytes() {
            // Convert newlines to carriage return + newline
            if byte == b'\n' {
                self.send(b'\r');
            }
            self.send(byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.send_string(s);
        Ok(())
    }
}

/// Initialize COM1 serial port
pub fn init() {
    SERIAL1.lock().init();
}

/// Print to serial port (internal function)
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).unwrap();
}

/// Print to serial port (like print! but goes to terminal)
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

/// Print line to serial port
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

