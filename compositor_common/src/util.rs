use std::{fmt::Display, num::ParseIntError, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Degree(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr)]
pub struct RGBColor(pub u8, pub u8, pub u8);

impl RGBColor {
    pub const BLACK: Self = Self(255, 255, 255);

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

impl Display for RGBColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
}

impl FromStr for RGBColor {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 7 {
            return Err(ColorParseError::InvalidRGBFormat);
        }
        if !s.starts_with('#') {
            return Err(ColorParseError::InvalidColorPrefixFormat);
        }
        let (r, g, b) = (&s[1..3], &s[3..5], &s[5..7]);

        Ok(RGBColor(
            u8::from_str_radix(r, 16)?,
            u8::from_str_radix(g, 16)?,
            u8::from_str_radix(b, 16)?,
        ))
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr, Default,
)]
pub struct RGBAColor(pub u8, pub u8, pub u8, pub u8);

impl Display for RGBAColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02X}{:02X}{:02X}{:02X}",
            self.0, self.1, self.2, self.3
        )
    }
}

impl FromStr for RGBAColor {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 9 {
            return Err(ColorParseError::InvalidRGBAFormat);
        }
        if !s.starts_with('#') {
            return Err(ColorParseError::InvalidColorPrefixFormat);
        }
        let (r, g, b, a) = (&s[1..3], &s[3..5], &s[5..7], &s[7..9]);

        Ok(RGBAColor(
            u8::from_str_radix(r, 16)?,
            u8::from_str_radix(g, 16)?,
            u8::from_str_radix(b, 16)?,
            u8::from_str_radix(a, 16)?,
        ))
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ColorParseError {
    #[error("Invalid format. Color has to be in #RRGGBB format")]
    InvalidRGBFormat,

    #[error("Invalid format. Color has to be in #RRGGBBAA format")]
    InvalidRGBAFormat,

    #[error("Invalid format. Color definition has to start with #")]
    InvalidColorPrefixFormat,

    #[error("Invalid format. Color representation is not a valid hexadecimal number")]
    HexNumberParseError(#[from] ParseIntError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr)]
pub enum Coord {
    Pixel(i32),
    Percent(i32),
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Coord::Pixel(pixels) => write!(f, "{}px", pixels),
            Coord::Percent(percents) => write!(f, "{}%", percents),
        }
    }
}

impl FromStr for Coord {
    type Err = CoordParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(percents) = s.strip_suffix('%') {
            return Ok(Coord::Percent(parse_num(percents)?));
        }

        if let Some(pixels) = s.strip_suffix("px") {
            return Ok(Coord::Pixel(parse_num(pixels)?));
        }

        Ok(Coord::Pixel(parse_num(s)?))
    }
}

fn parse_num(str: &str) -> Result<i32, CoordParseError> {
    str.parse::<i32>()
        .or(Err(CoordParseError::InvalidCoordFormat))
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CoordParseError {
    #[error("Invalid format. Coord definition can only be specified as number (pixels count), number with `px` suffix (pixels count) or number with `%` suffix (percents count)")]
    InvalidCoordFormat,
}

impl Coord {
    pub fn pixels(&self, max_pixels: u32) -> i32 {
        match self {
            Coord::Pixel(pixels) => *pixels,
            Coord::Percent(percent) => max_pixels as i32 * percent / 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::util::{Coord, CoordParseError, RGBAColor, RGBColor};

    #[test]
    fn test_rgb_serialization() {
        assert_eq!(format!("{}", RGBColor(0, 0, 0)), "#000000");
        assert_eq!(format!("{}", RGBColor(1, 2, 3)), "#010203");
        assert_eq!(format!("{}", RGBColor(1, 255, 3)), "#01FF03");
    }

    #[test]
    fn test_rgb_deserialization() {
        assert_eq!(RGBColor::from_str("#000000"), Ok(RGBColor(0, 0, 0)));
        assert_eq!(RGBColor::from_str("#010203"), Ok(RGBColor(1, 2, 3)));
        assert_eq!(RGBColor::from_str("#01FF03"), Ok(RGBColor(1, 255, 3)));
        assert_eq!(RGBColor::from_str("#FFffFF"), Ok(RGBColor(255, 255, 255)));
        assert_eq!(
            RGBColor::from_str("#00000G").unwrap_err().to_string(),
            "Invalid format. Color representation is not a valid hexadecimal number"
        );
        assert_eq!(
            RGBColor::from_str("#000").unwrap_err().to_string(),
            "Invalid format. Color has to be in #RRGGBB format"
        );
    }

    #[test]
    fn test_rgba_serialization() {
        assert_eq!(format!("{}", RGBAColor(0, 0, 0, 0)), "#00000000");
        assert_eq!(format!("{}", RGBAColor(1, 2, 3, 4)), "#01020304");
        assert_eq!(format!("{}", RGBAColor(1, 255, 3, 4)), "#01FF0304");
    }

    #[test]
    fn test_rgba_deserialization() {
        assert_eq!(RGBAColor::from_str("#00000000"), Ok(RGBAColor(0, 0, 0, 0)));
        assert_eq!(RGBAColor::from_str("#01020304"), Ok(RGBAColor(1, 2, 3, 4)));
        assert_eq!(
            RGBAColor::from_str("#01FF0304"),
            Ok(RGBAColor(1, 255, 3, 4))
        );
        assert_eq!(
            RGBAColor::from_str("#FFffFFff"),
            Ok(RGBAColor(255, 255, 255, 255))
        );
        assert_eq!(
            RGBAColor::from_str("#0000000G").unwrap_err().to_string(),
            "Invalid format. Color representation is not a valid hexadecimal number"
        );
        assert_eq!(
            RGBAColor::from_str("#000").unwrap_err().to_string(),
            "Invalid format. Color has to be in #RRGGBBAA format"
        );
    }

    #[test]
    fn test_coords_serialization() {
        assert_eq!(format!("{}", Coord::Pixel(-31)), "-31px");
        assert_eq!(format!("{}", Coord::Percent(67)), "67%");
    }

    #[test]
    fn test_coords_deserialization() {
        assert_eq!(Coord::from_str("100"), Ok(Coord::Pixel(100)));
        assert_eq!(Coord::from_str("2137px"), Ok(Coord::Pixel(2137)));
        assert_eq!(Coord::from_str("-420px"), Ok(Coord::Pixel(-420)));
        assert_eq!(Coord::from_str("69%"), Ok(Coord::Percent(69)));
        assert_eq!(Coord::from_str("-1337%"), Ok(Coord::Percent(-1337)));
        assert_eq!(
            Coord::from_str("-1-337%"),
            Err(CoordParseError::InvalidCoordFormat)
        );
        assert_eq!(
            Coord::from_str("1x"),
            Err(CoordParseError::InvalidCoordFormat)
        );
    }
}
