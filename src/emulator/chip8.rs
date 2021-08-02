use crate::emulator::bus::Bus;
use crate::emulator::cpu::Cpu;

pub struct Chip8 {
    bus: Bus,
    cpu: Cpu,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            bus: Bus::new(),
            cpu: Cpu::new(),
        }
    }
}
