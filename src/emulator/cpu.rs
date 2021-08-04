use rand;
use crate::emulator::bus::Bus;
use rand::Rng;
use std::fmt;

pub const PROGRAM_START: u16 = 0x200;

pub struct Cpu {
    vx: [u16; 16],
    // Vx where x is 0...F; 16 Registers
    i: u16,
    // where I is a Register
    pc: u16,
    stack: [u16; 16],
    sp: usize,
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
        let n = instruction & 0x00F;
        let x = ((instruction & 0x0F00) >> 8) as usize;
        let y = ((instruction & 0x00F0) >> 4) as usize;

        match (instruction & 0xF000) >> 12 {
            0x0 => {
                match kk {
                    0xE0 => {
                        bus.clear_screen();
                        self.pc += 2;
                    }
                    0xEE => {
                        self.pc = self.stack[self.sp as usize];
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
                self.stack[self.sp] = self.pc + 2;
                self.pc = nnn;
            }
            0x3 => {
                if self.vx[x as usize] == kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x4 => {
                if self.vx[x as usize] != kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x5 => {
                if self.vx[x as usize] == self.vx[y as usize] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x6 => {
                self.vx[x as usize] = kk;
                self.pc += 2;
            }
            0x7 => {
                self.vx[x as usize] = self.vx[x as usize].wrapping_add(kk);
                self.pc += 2;
            }
            0x8 => {
                match n {
                    0x0 => {
                        self.vx[x as usize] = self.vx[y as usize];
                    }
                    0x1 => {
                        self.vx[x as usize] |= self.vx[y as usize];
                    }
                    0x2 => {
                        self.vx[x as usize] &= self.vx[y as usize];
                    }
                    0x3 => {
                        self.vx[x as usize] ^= self.vx[y as usize];
                    }
                    0x4 => {
                        let sum: u16 = self.vx[x as usize] + self.vx[y as usize];
                        if sum > 0xFF {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[x as usize] = sum;
                        }
                    }
                    0x5 => {
                        if self.vx[x as usize] > self.vx[y as usize] {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[0xF] = 0;
                        }

                        self.vx[x as usize] = (self.vx[y as usize] as i8 - self.vx[x as usize] as i8) as u16;
                    }
                    0x6 => {
                        let lsb = self.vx[x as usize] & 0x1;
                        if lsb == 1 {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[0xf] = 0
                        }

                        self.vx[x as usize] /= 2;
                    }
                    0x7 => {
                        if self.vx[x as usize] < self.vx[y as usize] {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[0xF] = 0;
                        }

                        self.vx[x as usize] = (self.vx[y as usize] as i8 - self.vx[y as usize] as i8) as u16;
                    }
                    0xE => {
                        let msb = (self.vx[x as usize] & 0b1000_0000) >> 7;

                        if msb == 1 {
                            self.vx[0xF] = 1;
                        } else {
                            self.vx[0xF] = 0;
                        }

                        self.vx[x as usize] *= 2;
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
                        if self.vx[x as usize] != self.vx[y as usize] {
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
                self.vx[x as usize] = kk & self.random.gen_range(0..256);
                self.pc += 2;
            }
            0xD => {
                let x_coord = self.vx[x as usize];
                let y_coord = self.vx[y as usize];
                let mut has_overwritten_pixel = false;
                for i in 0..n {
                    let byte = bus.ram_read_byte(self.i + i);
                    if bus.draw_byte_at_coord(x_coord as usize, (y_coord + i as u16) as usize, byte) {
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
                        if bus.is_key_pressed(self.vx[x as usize] as u8) {
                            self.pc += 2;
                        }

                        self.pc += 2;
                    }
                    0xA1 => {
                        if !bus.is_key_pressed(self.vx[x as usize] as u8) {
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
                        self.vx[x as usize] = bus.get_delay_timer() as u16;
                        self.pc += 2;
                    }
                    0x0A => {
                        if let Some(val) = bus.get_key_pressed() {
                            self.vx[x as usize] = val as u16;
                            self.pc += 2;
                        }
                    }
                    0x15 => {
                        bus.set_delay_timer(self.vx[x as usize] as u8);
                        self.pc += 2;
                    }
                    0x18 => {
                        // TODO: Sound Timer [ST]
                        self.pc += 2;
                    }
                    0x1E => {
                        self.i += self.vx[x as usize];
                        self.pc += 2;
                    }
                    0x29 => {
                        self.i = self.vx[x as usize] * 5;
                        self.pc += 2;
                    }
                    0x33 => {
                        bus.ram_write_byte(self.i, (self.vx[x as usize] / 100) as u8);
                        bus.ram_write_byte(self.i + 1, ((self.vx[x as usize] / 10) % 10) as u8);
                        bus.ram_write_byte(self.i + 2, (self.vx[x as usize] % 10) as u8);
                        self.pc += 2;
                    }
                    0x55 => {
                        for i in 0..x + 1 {
                            bus.ram_write_byte((i + self.i as usize) as u16, self.vx[i as usize] as u8);
                        }
                        self.i += x as u16 + 1;
                        self.pc += 2;
                    }
                    0x65 => {
                        for i in 0..x + 1 {
                            self.vx[i as usize] = bus.ram_read_byte((i + self.i as usize) as u16) as u16;
                        }
                        self.i += (x + 1) as u16;
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

// pub struct Cpu {
//     vx: [u8; 16],
//     pc: u16,
//     i: u16,
//     ret_stack: Vec<u16>,
//     rng: rand::rngs::ThreadRng,
// }
//
//
// impl Cpu {
//     pub fn new() -> Cpu {
//         Cpu {
//             vx: [0; 16],
//             pc: PROGRAM_START,
//             i: 0,
//             ret_stack: Vec::<u16>::new(),
//             rng: rand::thread_rng(),
//         }
//     }
//
//     pub fn run_instruction(&mut self, bus: &mut Bus) {
//         let hi = bus.ram_read_byte(self.pc) as u16;
//         let lo = bus.ram_read_byte(self.pc + 1) as u16;
//         let instruction: u16 = (hi << 8) | lo;
//         // println!(
//         //     "Instruction read {:#X}:{:#X}: hi{:#X} lo:{:#X} ",
//         //     self.pc,
//         //     instruction,
//         //     hi,
//         //     lo
//         // );
//
//         let nnn = instruction & 0x0FFF;
//         let nn = (instruction & 0x0FF) as u8;
//         let n = (instruction & 0x00F) as u8;
//         let x = ((instruction & 0x0F00) >> 8) as u8;
//         let y = ((instruction & 0x00F0) >> 4) as u8;
//         //println!("nnn={:?}, nn={:?}, n={:?} x={}, y={}", nnn, nn, n, x, y);
//
//         match (instruction & 0xF000) >> 12 {
//             0x0 => {
//                 match nn {
//                     0xE0 => {
//                         bus.clear_screen();
//                         self.pc += 2;
//                     }
//                     0xEE => {
//                         //return from subroutine
//                         let addr = self.ret_stack.pop().unwrap();
//                         self.pc = addr;
//                     }
//                     _ => panic!(
//                         "Unrecognized 0x00** instruction {:#X}:{:#X}",
//                         self.pc,
//                         instruction
//                     ),
//                 }
//             }
//             0x1 => {
//                 //goto nnn;
//                 self.pc = nnn;
//             }
//             0x2 => {
//                 //Call subroutine at address NNN
//                 self.ret_stack.push(self.pc + 2);
//                 self.pc = nnn;
//             }
//             0x3 => {
//                 //if(Vx==NN)
//                 let vx = self.read_reg_vx(x);
//                 if vx == nn {
//                     self.pc += 4;
//                 } else {
//                     self.pc += 2;
//                 }
//             }
//             0x4 => {
//                 //Skip next instruction if(Vx!=NN)
//                 let vx = self.read_reg_vx(x);
//                 if vx != nn {
//                     self.pc += 4;
//                 } else {
//                     self.pc += 2;
//                 }
//             }
//             0x5 => {
//                 //Skip next instruction if(Vx==Vy)
//                 let vx = self.read_reg_vx(x);
//                 let vy = self.read_reg_vx(y);
//                 if vx == vy {
//                     self.pc += 4;
//                 } else {
//                     self.pc += 2;
//                 }
//             }
//             0x6 => {
//                 //vx = nn
//                 self.write_reg_vx(x, nn);
//                 self.pc += 2;
//             }
//             0x7 => {
//                 let vx = self.read_reg_vx(x);
//                 self.write_reg_vx(x, vx.wrapping_add(nn));
//                 self.pc += 2;
//             }
//             0x8 => {
//                 let vy = self.read_reg_vx(y);
//                 let vx = self.read_reg_vx(x);
//
//                 match n {
//                     0x0 => {
//                         // Vx=Vy
//                         self.write_reg_vx(x, vy);
//                     }
//                     0x2 => {
//                         // Vx=Vx&Vy
//                         self.write_reg_vx(x, vx & vy);
//                     }
//                     0x3 => {
//                         // Vx=Vx^Vy
//                         self.write_reg_vx(x, vx ^ vy);
//                     }
//                     0x4 => {
//                         //	Vx += Vy
//                         let sum: u16 = vx as u16 + vy as u16;
//                         self.write_reg_vx(x, sum as u8);
//                         if sum > 0xFF {
//                             self.write_reg_vx(0xF, 1);
//                         }
//                     }
//                     0x5 => {
//                         let diff: i8 = vx as i8 - vy as i8;
//                         self.write_reg_vx(x, diff as u8);
//                         if diff < 0 {
//                             self.write_reg_vx(0xF, 1);
//                         } else {
//                             self.write_reg_vx(0xF, 0);
//                         }
//                     }
//                     0x6 => {
//                         // Vx=Vx>>1
//                         self.write_reg_vx(0xF, vx & 0x1);
//                         self.write_reg_vx(x, vx >> 1);
//                     }
//                     0x7 => {
//                         let diff: i8 = vy as i8 - vx as i8;
//                         self.write_reg_vx(x, diff as u8);
//                         if diff < 0 {
//                             self.write_reg_vx(0xF, 1);
//                         } else {
//                             self.write_reg_vx(0xF, 0);
//                         }
//                     }
//                     0xE => {
//                         // VF is the most significant bit value.
//                         // SHR Vx
//                         self.write_reg_vx(0xF, (vx & 0x80) >> 7);
//                         self.write_reg_vx(x, vx << 1);
//                     }
//                     _ => panic!(
//                         "Unrecognized 0x8XY* instruction {:#X}:{:#X}",
//                         self.pc,
//                         instruction
//                     ),
//                 };
//
//                 self.pc += 2;
//             }
//             0x9 => {
//                 //skips the next instruction if(Vx!=Vy)
//                 let vx = self.read_reg_vx(x);
//                 let vy = self.read_reg_vx(y);
//                 if vx != vy {
//                     self.pc += 4;
//                 } else {
//                     self.pc += 2;
//                 }
//             }
//             0xA => {
//                 self.i = nnn;
//                 self.pc += 2;
//             }
//             0xB => {
//                 self.pc = self.read_reg_vx(0) as u16 + nnn;
//             }
//             0xC => {
//                 // Vx=rand() & NN
//                 let number = self.rng.gen_range(0..255);
//                 self.write_reg_vx(x, number & nn);
//                 self.pc += 2;
//             }
//             0xD => {
//                 //draw(Vx,Vy,N)
//                 let vx = self.read_reg_vx(x);
//                 let vy = self.read_reg_vx(y);
//                 self.debug_draw_sprite(bus, vx, vy, n);
//                 self.pc += 2;
//             }
//             0xE => {
//                 match nn {
//                     0xA1 => {
//                         // if(key()!=Vx) then skip the next instruction
//                         let key = self.read_reg_vx(x);
//                         if !bus.is_key_pressed(key) {
//                             self.pc += 4;
//                         } else {
//                             self.pc += 2;
//                         }
//                     }
//                     0x9E => {
//                         // if(key()==Vx) then skip the next instruction
//                         let key = self.read_reg_vx(x);
//                         if bus.is_key_pressed(key) {
//                             self.pc += 4;
//                         } else {
//                             self.pc += 2;
//                         }
//                     }
//                     _ => panic!(
//                         "Unrecognized 0xEX** instruction {:#X}:{:#X}",
//                         self.pc,
//                         instruction
//                     ),
//                 };
//             }
//             0xF => {
//                 match nn {
//                     0x07 => {
//                         self.write_reg_vx(x, bus.get_delay_timer());
//                         self.pc += 2;
//                     }
//                     0x0A => {
//                         if let Some(val) = bus.get_key_pressed() {
//                             self.write_reg_vx(x, val);
//                             self.pc += 2;
//                         }
//                     }
//                     0x15 => {
//                         bus.set_delay_timer(self.read_reg_vx(x));
//                         self.pc += 2;
//                     }
//                     0x18 => {
//                         // TODO Sound timer
//                         self.pc += 2;
//                     }
//                     0x1E => {
//                         //I +=Vx
//                         let vx = self.read_reg_vx(x);
//                         self.i += vx as u16;
//                         self.pc += 2;
//                     }
//                     0x29 => {
//                         //i == sprite address for character in Vx
//                         //Multiply by 5 because each sprite has 5 lines, each line
//                         //is 1 byte.
//                         self.i = self.read_reg_vx(x) as u16 * 5;
//                         self.pc += 2;
//                     }
//                     0x33 => {
//                         let vx = self.read_reg_vx(x);
//                         bus.ram_write_byte(self.i, vx / 100);
//                         bus.ram_write_byte(self.i + 1, (vx % 100) / 10);
//                         bus.ram_write_byte(self.i + 2, vx % 10);
//                         self.pc += 2;
//                     }
//                     0x55 => {
//                         for index in 0..x + 1 {
//                             let value = self.read_reg_vx(index);
//                             bus.ram_write_byte(self.i + index as u16, value);
//                         }
//                         self.i += x as u16 + 1;
//                         self.pc += 2;
//                     }
//                     0x65 => {
//                         for index in 0..x + 1 {
//                             let value = bus.ram_read_byte(self.i + index as u16);
//                             self.write_reg_vx(index, value);
//                         }
//                         self.i += x as u16 + 1;
//                         self.pc += 2;
//                     }
//                     _ => panic!(
//                         "Unrecognized 0xF instruction {:#X}:{:#X}",
//                         self.pc,
//                         instruction
//                     ),
//                 }
//             }
//
//             _ => panic!("Unrecognized instruction {:#X}:{:#X}", self.pc, instruction),
//         }
//     }
//
//     fn debug_draw_sprite(&mut self, bus: &mut Bus, x: u8, y: u8, height: u8) {
//         let mut should_set_vf = false;
//         for sprite_y in 0..height {
//             let b = bus.ram_read_byte(self.i + sprite_y as u16);
//             if bus.draw_byte_at_coord(x as usize, (y + sprite_y) as usize, b) {
//                 should_set_vf = true;
//             }
//         }
//         if should_set_vf {
//             self.write_reg_vx(0xF, 1);
//         } else {
//             self.write_reg_vx(0xF, 0);
//         }
//     }
//
//     pub fn write_reg_vx(&mut self, index: u8, value: u8) {
//         self.vx[index as usize] = value;
//     }
//
//     pub fn read_reg_vx(&mut self, index: u8) -> u8 {
//         self.vx[index as usize]
//     }
// }
