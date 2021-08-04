use crate::emulator::keyboard::Keyboard;
use crate::emulator::ram::Ram;
use crate::emulator::display::Display;
use std::time;

pub struct Bus {
    ram: Ram,
    keyboard: Keyboard,
    display: Display,
    delay_timer: u8,
    delay_timer_set_time: time::Instant,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: Ram::new(),
            keyboard: Keyboard::new(),
            display: Display::new(),
            delay_timer: 0,
            delay_timer_set_time: time::Instant::now(),
        }
    }

    pub fn ram_read_byte(&self, address: u16) -> u8 {
        self.ram.read_byte(address)
    }

    pub fn ram_write_byte(&mut self, address: u16, value: u8) {
        self.ram.write_byte(address, value)
    }

    pub fn clear_screen(&mut self) {
        self.display.clear();
    }

    pub fn set_key_screen(&mut self, key: Option<u8>) {
        self.keyboard.set_key_pressed(key);
    }

    pub fn is_key_pressed(&self, key_code: u8) -> bool {
        self.keyboard.is_key_pressed(key_code)
    }

    pub fn get_key_pressed(&self) -> Option<u8> {
        self.keyboard.get_key_pressed()
    }

    pub fn set_delay_timer(&mut self, value: u8) {
        self.delay_timer_set_time = time::Instant::now();
        self.delay_timer = value;
    }

    pub fn get_delay_timer(&self) -> u8 {
        let diff = time::Instant::now() - self.delay_timer_set_time;
        let ms = diff.as_millis() as u64;
        let ticks = ms / 16;
        if ticks >= self.delay_timer as u64 {
            0
        } else {
            self.delay_timer - ticks as u8
        }
    }

    pub fn draw_byte_at_coord(&mut self, x: usize, y: usize, byte: u8) -> bool {
        self.display.draw_byte_at_coord(x as u8, y as u8, byte)
    }

    pub fn get_display_buffer(&self) -> &[u8] {
        self.display.get_display_buffer()
    }

    pub fn draw_screen(&self) {
        self.display.draw_screen();
    }
}
