use rand;
use crate::emulator::bus::Bus;
use rand::Rng;


pub const PROGRAM_START: u16 = 0x200;

pub struct Cpu {
    vx: [u8; 16],
    // Vx where x is 0...F; 16 Registers
    i: u16,
    // where I is a Register
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    // Stack pointer; Pointing to top of stack
    random: rand::rngs::ThreadRng,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            vx: [0; 16],
            i: 0,
            pc: PROGRAM_START,
            stack: [0; 16],
            sp: 0,
            random: rand::thread_rng(),
        }
    }

    pub fn run_instruction(&mut self, bus: &mut Bus) {
        let hi = bus.ram_read_byte(self.pc) as u16;
        let lo = bus.ram_read_byte(self.pc + 1) as u16;

        let instruction = (hi << 8) | lo;

        let nnn = instruction & 0x0FFF;
        let kk = instruction & 0x00FF;
        let n = instruction & 0x000F;
        let x = (instruction & 0x0F00) >> 8;
        let y = (instruction & 0x00F0) >> 4;

        match (instruction & 0xF000) >> 12 {
            0x0 => {
                match kk {
                    0xE0 => {
                        bus.clear_screen();
                        self.pc += 2;
                    }
                    0xEE => {
                        self.pc = self.stack[self.sp];
                        self.sp -= 1;
                    }
                    _ => {
                        panic!("Couldn't recognize instruction[0x00**] {:#X} at PC={:#X}", instruction, self.pc);
                    }
                }
            }
            0x1 => {
                self.pc = nnn;
            }
            0x2 => {
                self.sp += 1;
                self.stack[self.sp] = self.pc; // TODO CHECK IF +2 needed
                self.pc = nnn;
            }
            0x3 => {
                if self.vx[x] == kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x4 => {
                if self.vx[x] != kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x5 => {
                if self.vx[x] == self.vx[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x6 => {
                self.vx[x] = kk;
                self.pc += 2;
            }
            0x7 => {
                self.vx[x] += kk;
                self.pc += 2;
            }
            0x8 => {
                match n {
                    0x0 => {
                        self.vx[x] = self.vx[y];
                    }
                    0x1 => {
                        self.vx[x] |= self.vx[y];
                    }
                    0x2 => {
                        self.vx[x] &= self.vx[y];
                    }
                    0x3 => {
                        self.vx[x] ^= self.vx[y];
                    }
                    0x4 => {
                        let sum: u16 = self.vx[x] + self.vx[y];
                        if sum > 0xFF {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[x] = sum as u8;
                        }
                    }
                    0x5 => {
                        if self.vx[x] > self.vx[y] {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[0xF] = 0;
                        }

                        self.vx[x] = self.vx[y] - self.vx[x];
                    }
                    0x6 => {
                        let lsb = self.vx[x] & 0x1;
                        if lsb == 1 {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[0xf] = 0
                        }

                        self.vx[x] /= 2;
                    }
                    0x7 => {
                        if self.vx[x] < self.vx[y] {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[0xF] = 0;
                        }

                        self.vx[x] = self.vx[y] - self.vx[y];
                    }
                    0xE => {
                        let msb = (self.vx[x] & 0b10000000) >> 7;

                        if msb == 1 {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[0xF] = 0;
                        }

                        self.vx[x] *= 2;
                    }
                    _ => {
                        panic!("Couldn't recognize instruction[0x8xy*] {:#X} at PC={:#X}", instruction, self.pc);
                    }
                }
                self.pc += 2;
            }
            0x9 => {
                match n {
                    0x0 => {
                        if self.vx[x] != self.vx[y] {
                            self.pc += 2;
                        }

                        self.pc += 2;
                    }
                    _ => {
                        panic!("Couldn't recognize instruction[0x9xy*] {:#X} at PC={:#X}", instruction, self.pc);
                    }
                }
            }
            0xA => {
                self.i = nnn;
                self.pc += 2;
            }
            0xB => {
                self.pc = nnn + self.vx[0];
            }
            0xC => {
                self.vx[x] = kk & self.random.gen_range(0, 256);
                self.pc += 2;
            }
            0xD => {
                let x_coord = self.vx[x];
                let y_coord = self.vx[y];
                let mut has_overwritten_pixel = false;
                for i in 0..n {
                    let byte = bus.ram_read_byte(self.i + i);
                    if bus.draw_byte_at_coord(x_coord, y_coord, byte) {
                        has_overwritten_pixel = true;
                    }
                }

                if has_overwritten_pixel {
                    self.vx[0xF] = 1;
                } else {
                    self.vx[0xF] = 0;
                }

                self.pc += 2;
            }
            0xE => {
                match kk {
                    0x9E => {
                        if bus.is_key_pressed(self.vx[x]) {
                            self.pc += 2;
                        }

                        self.pc += 2;
                    }
                    0xA1 => {
                        if !bus.is_key_pressed(self.vx[x]) {
                            self.pc += 2;
                        }

                        self.pc += 2;
                    }
                    _ => {
                        panic!("Couldn't recognize instruction[0xEx**] {:#X} at PC={:#X}", instruction, self.pc);
                    }
                }
            }
            0xF => {
                match kk {
                    0x07 => {
                        self.vx[x] = bus.get_delay_timer();
                        self.pc += 2;
                    }
                    0x0A => {
                        if let Some(val) = bus.get_key_pressed() {
                            self.vx[x] = val;
                            self.pc += 2;
                        }
                    }
                    0x15 => {
                        bus.set_delay_timer(self.vx[x]);
                        self.pc += 2;
                    }
                    0x18 => {
                        // TODO: Sound Timer [ST]
                        self.pc += 2;
                    }
                    0x1E => {
                        self.i += self.vx[x];
                        self.pc += 2;
                    }
                    0x29 => {
                        self.i = self.vx[x] * 5;
                        self.pc += 2;
                    }
                    0x33 => {
                        bus.ram_write_byte(self.i, self.vx[x] / 100);
                        bus.ram_write_byte(self.i + 1, (self.vx[x] / 10) % 10);
                        bus.ram_write_byte(self.i + 2, self.vx[x] % 10);
                        self.pc += 2;
                    }
                    0x55 => {
                        for i in 0..x + 1 {
                            bus.ram_write_byte(i + self.i, self.vx[i]);
                        }
                        self.pc += 2;
                    }
                    0x65 => {
                        for i in 0..x + 1 {
                            self.vx[i] = bus.ram_read_byte(i + self.i);
                        }
                        self.pc += 2;
                    }
                    _ => {
                        panic!("Couldn't recognize instruction[0xFx**] {:#X} at PC={:#X}", instruction, self.pc);
                    }
                }
            }
            _ => {
                panic!("Couldn't recognize instruction {:#X} at PC={:#X}", instruction, self.pc);
            }
        }
    }
}