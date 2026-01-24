//! VGA Text Mode Buffer Driver
//! 
//! This module provides access to the VGA text buffer for displaying text
//! on screen. In the agentic future, this will be one of many output channels
//! managed by the Action Agent.

use core::fmt;
use spin::Mutex;
use core::ptr;

/// Standard VGA text buffer address
const VGA_BUFFER_ADDR: usize = 0xb8000;

/// VGA text buffer dimensions
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// Lazy-initialized global writer
pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Cyan, Color::Black),
});

/// VGA color codes
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Combined foreground and background color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Create a new color code
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character with its color attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// Writer for outputting text to the VGA buffer
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
}

impl Writer {
    /// Get buffer pointer
    #[inline]
    fn buffer(&self) -> *mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT] {
        VGA_BUFFER_ADDR as *mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]
    }

    /// Write to a specific position using volatile writes
    #[inline]
    fn write_char(&mut self, row: usize, col: usize, ch: ScreenChar) {
        unsafe {
            let buffer = self.buffer();
            ptr::write_volatile(&mut (*buffer)[row][col], ch);
        }
    }

    /// Read from a specific position using volatile reads
    #[inline]
    fn read_char(&self, row: usize, col: usize) -> ScreenChar {
        unsafe {
            let buffer = self.buffer();
            ptr::read_volatile(&(*buffer)[row][col])
        }
    }

    /// Write a single byte
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.write_char(row, col, ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Write a string
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Move to new line
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.read_char(row, col);
                self.write_char(row - 1, col, character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clear a row
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.write_char(row, col, blank);
        }
    }

    /// Clear the entire screen
    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Clear the entire screen
pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

/// Print macro
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Println macro
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Internal print function
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
