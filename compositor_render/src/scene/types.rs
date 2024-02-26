mod convert;
pub(crate) mod interpolation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlign {
    Left,
    Right,
    Justified,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
    Justified,
}

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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Degree(pub f64);

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct AbsolutePosition {
    pub width: f32,
    pub height: f32,
    pub position_horizontal: HorizontalPosition,
    pub position_vertical: VerticalPosition,
    pub rotation_degrees: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum VerticalPosition {
    TopOffset(f32),
    BottomOffset(f32),
}

#[derive(Debug, Clone, Copy)]
pub enum HorizontalPosition {
    LeftOffset(f32),
    RightOffset(f32),
}

#[derive(Debug, Clone, Copy)]
pub enum InterpolationKind {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    Bounce,
    CubicBezier { x1: f64, y1: f64, x2: f64, y2: f64 },
}
