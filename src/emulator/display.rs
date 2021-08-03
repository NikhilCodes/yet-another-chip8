const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Display {
    screen: [u8; WIDTH * HEIGHT],
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
        let mut mutable_byte = byte;
        let mut mutable_x = x;
        let mut overwritten = false;

        // Get 8 bits from byte and put in display
        for _ in 0..8 {
            mutable_x %= WIDTH;
            let bit = (mutable_byte & 0b1000_0000) >> 7;
            let index = Display::calc_index_from_coord(x as usize, y as usize);
            self.screen[index] ^= bit;

            mutable_x += 1;
            mutable_byte <<= 1;
            if bit != 0 && self.screen[index] == 0 {
                overwritten = true;
            }
        }

        overwritten
    }
}
