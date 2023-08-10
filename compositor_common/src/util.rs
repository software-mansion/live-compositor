use std::{fmt::Display, num::ParseIntError, str::FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr)]
pub struct RGBColor(pub u8, pub u8, pub u8);

impl Display for RGBColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:#02X}{:#02X}{:#02X}", self.0, self.1, self.2)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr)]
pub struct RGBAColor(pub u8, pub u8, pub u8, pub u8);

impl Display for RGBAColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:#02X}{:#02X}{:#02X}{:#02X}",
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

    #[error("Invalid format, Color representation is not a valid hexadecimal number")]
    HexNumberParseError(#[from] ParseIntError),
}
