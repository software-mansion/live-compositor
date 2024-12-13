use compositor_render::scene;

use super::util::*;

impl TryFrom<RGBAColor> for scene::RGBAColor {
    type Error = TypeError;
    fn try_from(value: RGBAColor) -> std::result::Result<Self, Self::Error> {
        let s = &value.0.trim();
        match s.chars().next() {
            Some('#') => {
                match s.len() {
                    7 => parse_hex(s),
                    9 => parse_hex_rgba(s),
                    _ => Err(TypeError::new("Invalid format. Color has to be in #RRGGBB or #RRGGBBAA format.")),
                }
            },
            Some('r') => {
                if s.starts_with("rgb(") {
                    parse_rgb(s)
                } else if s.starts_with("rgba(") {
                    parse_rgba(s)
                } else {
                    Err(TypeError::new("Invalid format. Expected rgb() or rgba() format."))
                }
            },
            _ => Err(TypeError::new("Unsupported color format.")),
        }
    }
}

fn parse_hex(s: &str) -> Result<scene::RGBAColor, TypeError> {
    let (r, g, b) = (&s[1..3], &s[3..5], &s[5..7]);
    let a = "ff"; // default full opacity
    parse_color_components(r, g, b, a)
}

fn parse_hex_rgba(s: &str) -> Result<scene::RGBAColor, TypeError> {
    let (r, g, b, a) = (&s[1..3], &s[3..5], &s[5..7], &s[7..9]);
    parse_color_components(r, g, b, a)
}

fn parse_rgb(s: &str) -> Result<scene::RGBAColor, TypeError> {
    let inner_s = s.trim_start_matches("rgb(").trim_end_matches(')');
    let parts: Vec<&str> = inner_s.split(',').collect();
    if parts.len() != 3 {
        return Err(TypeError::new("Invalid RGB format."))
    }
    let (r, g, b) = (parts[0].trim(), parts[1].trim(), parts[2].trim());
    let a = "255"; // default full opacity
    parse_color_components_from_decimal(r, g, b, a)
}

fn parse_rgba(s: &str) -> Result<scene::RGBAColor, TypeError> {
    let inner_s = s.trim_start_matches("rgba(").trim_end_matches(')');
    let parts: Vec<&str> = inner_s.split(',').collect();
    if parts.len() != 4 {
        return Err(TypeError::new("Invalid RGBA format."))
    }
    let (r, g, b) = (parts[0].trim(), parts[1].trim(), parts[2].trim());
    let alpha = parts[3].trim();
    let a = (alpha.parse::<f32>().map_err(|_| TypeError::new("Alpha channel parsing failed."))? * 255.0).round() as u8;
    parse_color_components_from_decimal(r, g, b, &a.to_string())
}

fn parse_color_components(r: &str, g: &str, b: &str, a: &str) -> Result<scene::RGBAColor, TypeError> {
    Ok(scene::RGBAColor(
        parse_color_channel(r)?,
        parse_color_channel(g)?,
        parse_color_channel(b)?,
        parse_color_channel(a)?,
    ))
}

fn parse_color_components_from_decimal(r: &str, g: &str, b: &str, a: &str) -> Result<scene::RGBAColor, TypeError> {
    Ok(scene::RGBAColor(
        r.parse::<u8>().map_err(|_| TypeError::new("Red channel is not a valid decimal."))?,
        g.parse::<u8>().map_err(|_| TypeError::new("Green channel is not a valid decimal."))?,
        b.parse::<u8>().map_err(|_| TypeError::new("Blue channel is not a valid decimal."))?,
        a.parse::<u8>().map_err(|_| TypeError::new("Alpha channel is not a valid decimal."))?,
    ))
}

fn parse_color_channel(value: &str) -> Result<u8, TypeError> {
    u8::from_str_radix(value, 16).map_err(|_err| {
        TypeError::new(
            "Invalid format. Color representation is not a valid hexadecimal number.",
        )
    })
}