use image::{Rgba, Rgb, RgbImage};

pub struct Tile {
    // alpha, r, g, b
    data: Vec<Option<[u8; 4]>>,
    pub width: u16,
    pub height: u16,
}

impl Tile {
    fn get_pixel(&self, x: u16, y: u16) -> Option<[u8; 4]> {
        if x > self.width || y > self.height {
            return None;
        }

        let idx = ( (self.data.len() / self.width as usize) * y as usize ) + x as usize;

        self.data[idx]
    }
}

pub struct CharTile {
    pub color: Rgba<u8>,
    pub char: char
}

pub trait ArrayToPixel {
    fn to_pixel(&self, data: [u8; 4]) -> Rgba<u8>;
}
impl ArrayToPixel for [u8; 4] {
    fn to_pixel(&self, data: [u8; 4]) -> Rgba<u8> {
        Rgba(data)
    }
}

pub trait TileToChar {
    fn to_char(&self) -> CharTile  {
        
        // TODO
        CharTile { color: Rgba([255, 255, 255, 255]), char: '.' }
    }
}

fn to_char(tile: Tile) -> char {

    ' '
}
