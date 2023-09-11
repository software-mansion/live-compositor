use std::{fmt::Display, str::FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};

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

    use crate::util::coord::{Coord, CoordParseError};

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
