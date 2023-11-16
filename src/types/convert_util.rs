use std::time::Duration;

use compositor_common::util::{align, colors, coord, degree};
use compositor_render::scene::{self};

use super::util::*;

impl From<Resolution> for compositor_common::scene::Resolution {
    fn from(resolution: Resolution) -> Self {
        Self {
            width: resolution.width,
            height: resolution.height,
        }
    }
}

impl From<Resolution> for scene::Size {
    fn from(resolution: Resolution) -> Self {
        Self {
            width: resolution.width as f32,
            height: resolution.height as f32,
        }
    }
}

impl From<compositor_common::scene::Resolution> for Resolution {
    fn from(resolution: compositor_common::scene::Resolution) -> Self {
        Self {
            width: resolution.width,
            height: resolution.height,
        }
    }
}

impl From<Transition> for scene::Transition {
    fn from(transition: Transition) -> Self {
        Self {
            duration: Duration::from_secs_f64(transition.duration_ms / 1000.0),
        }
    }
}

impl From<HorizontalAlign> for align::HorizontalAlign {
    fn from(alignment: HorizontalAlign) -> Self {
        match alignment {
            HorizontalAlign::Left => align::HorizontalAlign::Left,
            HorizontalAlign::Right => align::HorizontalAlign::Right,
            HorizontalAlign::Justified => align::HorizontalAlign::Justified,
            HorizontalAlign::Center => align::HorizontalAlign::Center,
        }
    }
}

impl From<align::HorizontalAlign> for HorizontalAlign {
    fn from(alignment: align::HorizontalAlign) -> Self {
        match alignment {
            align::HorizontalAlign::Left => HorizontalAlign::Left,
            align::HorizontalAlign::Right => HorizontalAlign::Right,
            align::HorizontalAlign::Justified => HorizontalAlign::Justified,
            align::HorizontalAlign::Center => HorizontalAlign::Center,
        }
    }
}

impl From<VerticalAlign> for align::VerticalAlign {
    fn from(alignment: VerticalAlign) -> Self {
        match alignment {
            VerticalAlign::Top => align::VerticalAlign::Top,
            VerticalAlign::Center => align::VerticalAlign::Center,
            VerticalAlign::Bottom => align::VerticalAlign::Bottom,
            VerticalAlign::Justified => align::VerticalAlign::Justified,
        }
    }
}

impl From<align::VerticalAlign> for VerticalAlign {
    fn from(alignment: align::VerticalAlign) -> Self {
        match alignment {
            align::VerticalAlign::Top => VerticalAlign::Top,
            align::VerticalAlign::Center => VerticalAlign::Center,
            align::VerticalAlign::Bottom => VerticalAlign::Bottom,
            align::VerticalAlign::Justified => VerticalAlign::Justified,
        }
    }
}

impl From<Degree> for degree::Degree {
    fn from(value: Degree) -> Self {
        Self(value.0)
    }
}

impl From<degree::Degree> for Degree {
    fn from(degree: degree::Degree) -> Self {
        Self(degree.0)
    }
}

impl TryFrom<Framerate> for compositor_common::Framerate {
    type Error = TypeError;

    fn try_from(framerate: Framerate) -> Result<Self, Self::Error> {
        const ERROR_MESSAGE: &str = "Framerate needs to be an unsigned integer or a string in the \"NUM/DEN\" format, where NUM and DEN are both unsigned integers.";
        match framerate {
            Framerate::String(text) => {
                let Some((num_str, den_str)) = text.split_once('/') else {
                    return Err(TypeError::new(ERROR_MESSAGE));
                };
                let num = num_str
                    .parse::<u32>()
                    .or(Err(TypeError::new(ERROR_MESSAGE)))?;
                let den = den_str
                    .parse::<u32>()
                    .or(Err(TypeError::new(ERROR_MESSAGE)))?;
                Ok(compositor_common::Framerate { num, den })
            }
            Framerate::U32(num) => Ok(compositor_common::Framerate { num, den: 1 }),
        }
    }
}

impl TryFrom<Coord> for coord::Coord {
    type Error = TypeError;

    fn try_from(value: Coord) -> Result<Self, Self::Error> {
        const PARSE_ERROR_MESSAGE: &str = "Invalid format. Coord definition can only be specified as number (pixels count), number with `px` suffix (pixels count) or number with `%` suffix (percents count)";
        fn parse_i32(str: &str) -> Result<i32, TypeError> {
            str.parse::<i32>()
                .or(Err(TypeError::new(PARSE_ERROR_MESSAGE)))
        }
        match value {
            Coord::Number(value) => Ok(coord::Coord::Pixel(value)),
            Coord::String(value) => {
                if let Some(percents) = value.strip_suffix('%') {
                    // TODO: support f64
                    return Ok(coord::Coord::Percent(parse_i32(percents)?));
                }

                if let Some(pixels) = value.strip_suffix("px") {
                    return Ok(coord::Coord::Pixel(parse_i32(pixels)?));
                }

                Ok(coord::Coord::Pixel(parse_i32(&value)?))
            }
        }
    }
}

impl From<coord::Coord> for Coord {
    fn from(value: coord::Coord) -> Self {
        match value {
            coord::Coord::Pixel(value) => Self::String(format!("{value}px")),
            coord::Coord::Percent(value) => Self::String(format!("{value}%")),
        }
    }
}

impl TryFrom<RGBColor> for colors::RGBColor {
    type Error = TypeError;

    fn try_from(value: RGBColor) -> std::result::Result<Self, Self::Error> {
        let s = &value.0;
        if s.len() != 7 {
            return Err(TypeError::new(
                "Invalid format. Color has to be in #RRGGBB format.",
            ));
        }
        if !s.starts_with('#') {
            return Err(TypeError::new(
                "Invalid format. Color definition has to start with #.",
            ));
        }
        let (r, g, b) = (&s[1..3], &s[3..5], &s[5..7]);

        fn parse_color_channel(value: &str) -> Result<u8, TypeError> {
            u8::from_str_radix(value, 16).map_err(|_err| {
                TypeError::new(
                    "Invalid format. Color representation is not a valid hexadecimal number.",
                )
            })
        }

        Ok(Self(
            parse_color_channel(r)?,
            parse_color_channel(g)?,
            parse_color_channel(b)?,
        ))
    }
}

impl From<colors::RGBColor> for RGBColor {
    fn from(value: colors::RGBColor) -> Self {
        RGBColor(format!("#{:02X}{:02X}{:02X}", value.0, value.1, value.2))
    }
}

impl TryFrom<RGBAColor> for colors::RGBAColor {
    type Error = TypeError;

    fn try_from(value: RGBAColor) -> std::result::Result<Self, Self::Error> {
        let s = &value.0;
        if s.len() != 9 {
            return Err(TypeError::new(
                "Invalid format. Color has to be in #RRGGBBAA format.",
            ));
        }
        if !s.starts_with('#') {
            return Err(TypeError::new(
                "Invalid format. Color definition has to start with #.",
            ));
        }
        let (r, g, b, a) = (&s[1..3], &s[3..5], &s[5..7], &s[7..9]);

        fn parse_color_channel(value: &str) -> Result<u8, TypeError> {
            u8::from_str_radix(value, 16).map_err(|_err| {
                TypeError::new(
                    "Invalid format. Color representation is not a valid hexadecimal number.",
                )
            })
        }

        Ok(Self(
            parse_color_channel(r)?,
            parse_color_channel(g)?,
            parse_color_channel(b)?,
            parse_color_channel(a)?,
        ))
    }
}

impl From<colors::RGBAColor> for RGBAColor {
    fn from(value: colors::RGBAColor) -> Self {
        Self(format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            value.0, value.1, value.2, value.3
        ))
    }
}
