use std::time::Duration;

use compositor_render::scene;

use super::util::*;

impl From<Resolution> for compositor_render::Resolution {
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

impl From<compositor_render::Resolution> for Resolution {
    fn from(resolution: compositor_render::Resolution) -> Self {
        Self {
            width: resolution.width,
            height: resolution.height,
        }
    }
}

impl TryFrom<Transition> for scene::Transition {
    type Error = TypeError;

    fn try_from(transition: Transition) -> Result<Self, Self::Error> {
        let interpolation_kind = match transition.easing_function.unwrap_or(EasingFunction::Linear)
        {
            EasingFunction::Linear => scene::InterpolationKind::Linear,
            EasingFunction::Bounce => scene::InterpolationKind::Bounce,
            EasingFunction::CubicBezier { points } => {
                if points[0] < 0.0 || points[0] > 1.0 {
                    return Err(TypeError::new(
                        "Control point x1 has to be in the range [0, 1].",
                    ));
                }
                if points[2] < 0.0 || points[2] > 1.0 {
                    return Err(TypeError::new(
                        "Control point x2 has to be in the range [0, 1].",
                    ));
                }

                scene::InterpolationKind::CubicBezier {
                    x1: points[0],
                    y1: points[1],
                    x2: points[2],
                    y2: points[3],
                }
            }
        };

        Ok(Self {
            duration: Duration::from_secs_f64(transition.duration_ms / 1000.0),
            interpolation_kind,
        })
    }
}

impl From<HorizontalAlign> for scene::HorizontalAlign {
    fn from(alignment: HorizontalAlign) -> Self {
        match alignment {
            HorizontalAlign::Left => scene::HorizontalAlign::Left,
            HorizontalAlign::Right => scene::HorizontalAlign::Right,
            HorizontalAlign::Justified => scene::HorizontalAlign::Justified,
            HorizontalAlign::Center => scene::HorizontalAlign::Center,
        }
    }
}

impl From<scene::HorizontalAlign> for HorizontalAlign {
    fn from(alignment: scene::HorizontalAlign) -> Self {
        match alignment {
            scene::HorizontalAlign::Left => HorizontalAlign::Left,
            scene::HorizontalAlign::Right => HorizontalAlign::Right,
            scene::HorizontalAlign::Justified => HorizontalAlign::Justified,
            scene::HorizontalAlign::Center => HorizontalAlign::Center,
        }
    }
}

impl From<VerticalAlign> for scene::VerticalAlign {
    fn from(alignment: VerticalAlign) -> Self {
        match alignment {
            VerticalAlign::Top => scene::VerticalAlign::Top,
            VerticalAlign::Center => scene::VerticalAlign::Center,
            VerticalAlign::Bottom => scene::VerticalAlign::Bottom,
            VerticalAlign::Justified => scene::VerticalAlign::Justified,
        }
    }
}

impl From<scene::VerticalAlign> for VerticalAlign {
    fn from(alignment: scene::VerticalAlign) -> Self {
        match alignment {
            scene::VerticalAlign::Top => VerticalAlign::Top,
            scene::VerticalAlign::Center => VerticalAlign::Center,
            scene::VerticalAlign::Bottom => VerticalAlign::Bottom,
            scene::VerticalAlign::Justified => VerticalAlign::Justified,
        }
    }
}

impl From<Degree> for scene::Degree {
    fn from(value: Degree) -> Self {
        Self(value.0)
    }
}

impl From<scene::Degree> for Degree {
    fn from(degree: scene::Degree) -> Self {
        Self(degree.0)
    }
}

impl TryFrom<Framerate> for compositor_render::Framerate {
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
                Ok(compositor_render::Framerate { num, den })
            }
            Framerate::U32(num) => Ok(compositor_render::Framerate { num, den: 1 }),
        }
    }
}

impl TryFrom<AspectRatio> for (u32, u32) {
    type Error = TypeError;

    fn try_from(text: AspectRatio) -> Result<Self, Self::Error> {
        const ERROR_MESSAGE: &str = "Aspect ratio needs to be a string in the \"W:H\" format, where W and H are both unsigned integers.";
        let Some((v1_str, v2_str)) = text.0.split_once(':') else {
            return Err(TypeError::new(ERROR_MESSAGE));
        };
        let v1 = v1_str
            .parse::<u32>()
            .or(Err(TypeError::new(ERROR_MESSAGE)))?;
        let v2 = v2_str
            .parse::<u32>()
            .or(Err(TypeError::new(ERROR_MESSAGE)))?;
        Ok((v1, v2))
    }
}

impl TryFrom<RGBColor> for scene::RGBColor {
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

impl From<scene::RGBColor> for RGBColor {
    fn from(value: scene::RGBColor) -> Self {
        RGBColor(format!("#{:02X}{:02X}{:02X}", value.0, value.1, value.2))
    }
}

impl TryFrom<RGBAColor> for scene::RGBAColor {
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

impl From<scene::RGBAColor> for RGBAColor {
    fn from(value: scene::RGBAColor) -> Self {
        Self(format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            value.0, value.1, value.2, value.3
        ))
    }
}
