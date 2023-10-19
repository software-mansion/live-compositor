#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Coord {
    Pixel(i32),
    Percent(i32),
}

impl Coord {
    pub fn pixels(&self, max_pixels: u32) -> i32 {
        match self {
            Coord::Pixel(pixels) => *pixels,
            Coord::Percent(percent) => max_pixels as i32 * percent / 100,
        }
    }
}
