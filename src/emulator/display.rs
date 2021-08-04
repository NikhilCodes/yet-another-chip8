const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Display {
    pub(crate) screen: [u8; WIDTH * HEIGHT],
}

impl Display {
    pub fn new() -> Display {
        Display {
            screen: [0; WIDTH * HEIGHT]
        }
    }

    pub fn calc_index_from_coord(x: usize, y: usize) -> usize {
        y * WIDTH + x
    }

    pub fn clear(&mut self) {
        for pixel in self.screen.iter_mut() {
            *pixel = 0;
        }
    }

    pub fn get_display_buffer(&self) -> &[u8] {
        &self.screen
    }

    pub fn draw_byte_at_coord(&mut self, x: u8, y: u8, byte: u8) -> bool {
        let mut erased = false;
        let mut coord_x = x as usize;
        let mut coord_y = y as usize;
        let mut b = byte;

        for _ in 0..8 {
            coord_x %= WIDTH;
            coord_y %= HEIGHT;
            let index = Display::calc_index_from_coord(coord_x, coord_y);
            let bit = (b & 0b1000_0000) >> 7;
            let prev_value = self.screen[index];
            self.screen[index] ^= bit;

            if prev_value == 1 && self.screen[index] == 0 {
                erased = true;
            }

            coord_x += 1;
            b <<= 1;
        }

        erased
    }

    pub fn draw_screen(&self) {
        println!("{:?}", self.screen);
        // for i in 0..self.screen.len() {
        //     if i % 64 != 0 {
        //         if self.screen[i] != 0 {
        //             print!("X");
        //         } else {
        //             print!("-");
        //         }
        //     } else {
        //         println!();
        //     }
        // }
    }
}
