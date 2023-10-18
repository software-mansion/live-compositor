#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RGBColor(pub u8, pub u8, pub u8);

impl RGBColor {
    pub const BLACK: Self = Self(0, 0, 0);

    pub fn to_yuv(&self) -> (f32, f32, f32) {
        let r = self.0 as f32 / 255.0;
        let g = self.1 as f32 / 255.0;
        let b = self.2 as f32 / 255.0;
        (
            ((0.299 * r) + (0.587 * g) + (0.114 * b)).clamp(0.0, 1.0),
            (((-0.168736 * r) - (0.331264 * g) + (0.5 * b)) + (128.0 / 255.0)).clamp(0.0, 1.0),
            (((0.5 * r) + (-0.418688 * g) + (-0.081312 * b)) + (128.0 / 255.0)).clamp(0.0, 1.0),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RGBAColor(pub u8, pub u8, pub u8, pub u8);
