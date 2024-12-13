use compositor_render::scene;

use super::util::*;

impl TryFrom<RGBAColor> for scene::RGBAColor {
    type Error = TypeError;
    fn try_from(value: RGBAColor) -> std::result::Result<Self, Self::Error> {
        let s = &value.0.trim();
        match s.chars().next() {
            Some('#') => match s.len() {
                7 => parse_hex(s),
                9 => parse_hex_rgba(s),
                _ => Err(TypeError::new(
                    "Invalid format. Color has to be in #RRGGBB or #RRGGBBAA format.",
                )),
            },
            Some('r') => {
                if s.starts_with("rgb(") {
                    parse_rgb(s)
                } else if s.starts_with("rgba(") {
                    parse_rgba(s)
                } else {
                    Err(TypeError::new(
                        "Invalid format. Expected rgb() or rgba() format.",
                    ))
                }
            }
            _ => {
                if let Some(rgb_str) = parse_named_color(s) {
                    parse_rgb(rgb_str)
                } else {
                    Err(TypeError::new("Unsupported color format."))
                }
            }
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
        return Err(TypeError::new("Invalid RGB format."));
    }
    let (r, g, b) = (parts[0].trim(), parts[1].trim(), parts[2].trim());
    let a = "255"; // default full opacity
    parse_color_components_from_decimal(r, g, b, a)
}

fn parse_rgba(s: &str) -> Result<scene::RGBAColor, TypeError> {
    let inner_s = s.trim_start_matches("rgba(").trim_end_matches(')');
    let parts: Vec<&str> = inner_s.split(',').collect();
    if parts.len() != 4 {
        return Err(TypeError::new("Invalid RGBA format."));
    }
    let (r, g, b) = (parts[0].trim(), parts[1].trim(), parts[2].trim());
    let alpha = parts[3].trim();
    let a = (alpha
        .parse::<f32>()
        .map_err(|_| TypeError::new("Alpha channel parsing failed."))?
        * 255.0)
        .round() as u8;
    parse_color_components_from_decimal(r, g, b, &a.to_string())
}

fn parse_color_components(
    r: &str,
    g: &str,
    b: &str,
    a: &str,
) -> Result<scene::RGBAColor, TypeError> {
    Ok(scene::RGBAColor(
        parse_color_channel(r)?,
        parse_color_channel(g)?,
        parse_color_channel(b)?,
        parse_color_channel(a)?,
    ))
}

fn parse_color_components_from_decimal(
    r: &str,
    g: &str,
    b: &str,
    a: &str,
) -> Result<scene::RGBAColor, TypeError> {
    Ok(scene::RGBAColor(
        r.parse::<u8>()
            .map_err(|_| TypeError::new("Red channel is not a valid decimal."))?,
        g.parse::<u8>()
            .map_err(|_| TypeError::new("Green channel is not a valid decimal."))?,
        b.parse::<u8>()
            .map_err(|_| TypeError::new("Blue channel is not a valid decimal."))?,
        a.parse::<u8>()
            .map_err(|_| TypeError::new("Alpha channel is not a valid decimal."))?,
    ))
}

fn parse_color_channel(value: &str) -> Result<u8, TypeError> {
    u8::from_str_radix(value, 16).map_err(|_err| {
        TypeError::new("Invalid format. Color representation is not a valid hexadecimal number.")
    })
}

fn parse_named_color(color_name: &str) -> Option<&'static str> {
    match color_name {
        "aliceblue" => Some("rgb(240, 248, 255)"),
        "antiquewhite" => Some("rgb(250, 235, 215)"),
        "aqua" => Some("rgb(0, 255, 255)"),
        "aquamarine" => Some("rgb(127, 255, 212)"),
        "azure" => Some("rgb(240, 255, 255)"),
        "beige" => Some("rgb(245, 245, 220)"),
        "bisque" => Some("rgb(255, 228, 196)"),
        "black" => Some("rgb(0, 0, 0)"),
        "blanchedalmond" => Some("rgb(255, 235, 205)"),
        "blue" => Some("rgb(0, 0, 255)"),
        "blueviolet" => Some("rgb(138, 43, 226)"),
        "brown" => Some("rgb(165, 42, 42)"),
        "burlywood" => Some("rgb(222, 184, 135)"),
        "burntsienna" => Some("rgb(234, 126, 93)"),
        "cadetblue" => Some("rgb(95, 158, 160)"),
        "chartreuse" => Some("rgb(127, 255, 0)"),
        "chocolate" => Some("rgb(210, 105, 30)"),
        "coral" => Some("rgb(255, 127, 80)"),
        "cornflowerblue" => Some("rgb(100, 149, 237)"),
        "cornsilk" => Some("rgb(255, 248, 220)"),
        "crimson" => Some("rgb(220, 20, 60)"),
        "cyan" => Some("rgb(0, 255, 255)"),
        "darkblue" => Some("rgb(0, 0, 139)"),
        "darkcyan" => Some("rgb(0, 139, 139)"),
        "darkgoldenrod" => Some("rgb(184, 134, 11)"),
        "darkgray" => Some("rgb(169, 169, 169)"),
        "darkgreen" => Some("rgb(0, 100, 0)"),
        "darkgrey" => Some("rgb(169, 169, 169)"),
        "darkkhaki" => Some("rgb(189, 183, 107)"),
        "darkmagenta" => Some("rgb(139, 0, 139)"),
        "darkolivegreen" => Some("rgb(85, 107, 47)"),
        "darkorange" => Some("rgb(255, 140, 0)"),
        "darkorchid" => Some("rgb(153, 50, 204)"),
        "darkred" => Some("rgb(139, 0, 0)"),
        "darksalmon" => Some("rgb(233, 150, 122)"),
        "darkseagreen" => Some("rgb(143, 188, 143)"),
        "darkslateblue" => Some("rgb(72, 61, 139)"),
        "darkslategray" => Some("rgb(47, 79, 79)"),
        "darkslategrey" => Some("rgb(47, 79, 79)"),
        "darkturquoise" => Some("rgb(0, 206, 209)"),
        "darkviolet" => Some("rgb(148, 0, 211)"),
        "deeppink" => Some("rgb(255, 20, 147)"),
        "deepskyblue" => Some("rgb(0, 191, 255)"),
        "dimgray" => Some("rgb(105, 105, 105)"),
        "dimgrey" => Some("rgb(105, 105, 105)"),
        "dodgerblue" => Some("rgb(30, 144, 255)"),
        "firebrick" => Some("rgb(178, 34, 34)"),
        "floralwhite" => Some("rgb(255, 250, 240)"),
        "forestgreen" => Some("rgb(34, 139, 34)"),
        "fuchsia" => Some("rgb(255, 0, 255)"),
        "gainsboro" => Some("rgb(220, 220, 220)"),
        "ghostwhite" => Some("rgb(248, 248, 255)"),
        "gold" => Some("rgb(255, 215, 0)"),
        "goldenrod" => Some("rgb(218, 165, 32)"),
        "gray" => Some("rgb(128, 128, 128)"),
        "green" => Some("rgb(0, 128, 0)"),
        "greenyellow" => Some("rgb(173, 255, 47)"),
        "grey" => Some("rgb(128, 128, 128)"),
        "honeydew" => Some("rgb(240, 255, 240)"),
        "hotpink" => Some("rgb(255, 105, 180)"),
        "indianred" => Some("rgb(205, 92, 92)"),
        "indigo" => Some("rgb(75, 0, 130)"),
        "ivory" => Some("rgb(255, 255, 240)"),
        "khaki" => Some("rgb(240, 230, 140)"),
        "lavender" => Some("rgb(230, 230, 250)"),
        "lavenderblush" => Some("rgb(255, 240, 245)"),
        "lawngreen" => Some("rgb(124, 252, 0)"),
        "lemonchiffon" => Some("rgb(255, 250, 205)"),
        "lightblue" => Some("rgb(173, 216, 230)"),
        "lightcoral" => Some("rgb(240, 128, 128)"),
        "lightcyan" => Some("rgb(224, 255, 255)"),
        "lightgoldenrodyellow" => Some("rgb(250, 250, 210)"),
        "lightgray" => Some("rgb(211, 211, 211)"),
        "lightgreen" => Some("rgb(144, 238, 144)"),
        "lightgrey" => Some("rgb(211, 211, 211)"),
        "lightpink" => Some("rgb(255, 182, 193)"),
        "lightsalmon" => Some("rgb(255, 160, 122)"),
        "lightseagreen" => Some("rgb(32, 178, 170)"),
        "lightskyblue" => Some("rgb(135, 206, 250)"),
        "lightslategray" => Some("rgb(119, 136, 153)"),
        "lightslategrey" => Some("rgb(119, 136, 153)"),
        "lightsteelblue" => Some("rgb(176, 196, 222)"),
        "lightyellow" => Some("rgb(255, 255, 224)"),
        "lime" => Some("rgb(0, 255, 0)"),
        "limegreen" => Some("rgb(50, 205, 50)"),
        "linen" => Some("rgb(250, 240, 230)"),
        "magenta" => Some("rgb(255, 0, 255)"),
        "maroon" => Some("rgb(128, 0, 0)"),
        "mediumaquamarine" => Some("rgb(102, 205, 170)"),
        "mediumblue" => Some("rgb(0, 0, 205)"),
        "mediumorchid" => Some("rgb(186, 85, 211)"),
        "mediumpurple" => Some("rgb(147, 112, 219)"),
        "mediumseagreen" => Some("rgb(60, 179, 113)"),
        "mediumslateblue" => Some("rgb(123, 104, 238)"),
        "mediumspringgreen" => Some("rgb(0, 250, 154)"),
        "mediumturquoise" => Some("rgb(72, 209, 204)"),
        "mediumvioletred" => Some("rgb(199, 21, 133)"),
        "midnightblue" => Some("rgb(25, 25, 112)"),
        "mintcream" => Some("rgb(245, 255, 250)"),
        "mistyrose" => Some("rgb(255, 228, 225)"),
        "moccasin" => Some("rgb(255, 228, 181)"),
        "navajowhite" => Some("rgb(255, 222, 173)"),
        "navy" => Some("rgb(0, 0, 128)"),
        "oldlace" => Some("rgb(253, 245, 230)"),
        "olive" => Some("rgb(128, 128, 0)"),
        "olivedrab" => Some("rgb(107, 142, 35)"),
        "orange" => Some("rgb(255, 165, 0)"),
        "orangered" => Some("rgb(255, 69, 0)"),
        "orchid" => Some("rgb(218, 112, 214)"),
        "palegoldenrod" => Some("rgb(238, 232, 170)"),
        "palegreen" => Some("rgb(152, 251, 152)"),
        "paleturquoise" => Some("rgb(175, 238, 238)"),
        "palevioletred" => Some("rgb(219, 112, 147)"),
        "papayawhip" => Some("rgb(255, 239, 213)"),
        "peachpuff" => Some("rgb(255, 218, 185)"),
        "peru" => Some("rgb(205, 133, 63)"),
        "pink" => Some("rgb(255, 192, 203)"),
        "plum" => Some("rgb(221, 160, 221)"),
        "powderblue" => Some("rgb(176, 224, 230)"),
        "purple" => Some("rgb(128, 0, 128)"),
        "rebeccapurple" => Some("rgb(102, 51, 153)"),
        "red" => Some("rgb(255, 0, 0)"),
        "rosybrown" => Some("rgb(188, 143, 143)"),
        "royalblue" => Some("rgb(65, 105, 225)"),
        "saddlebrown" => Some("rgb(139, 69, 19)"),
        "salmon" => Some("rgb(250, 128, 114)"),
        "sandybrown" => Some("rgb(244, 164, 96)"),
        "seagreen" => Some("rgb(46, 139, 87)"),
        "seashell" => Some("rgb(255, 245, 238)"),
        "sienna" => Some("rgb(160, 82, 45)"),
        "silver" => Some("rgb(192, 192, 192)"),
        "skyblue" => Some("rgb(135, 206, 235)"),
        "slateblue" => Some("rgb(106, 90, 205)"),
        "slategray" => Some("rgb(112, 128, 144)"),
        "slategrey" => Some("rgb(112, 128, 144)"),
        "snow" => Some("rgb(255, 250, 250)"),
        "springgreen" => Some("rgb(0, 255, 127)"),
        "steelblue" => Some("rgb(70, 130, 180)"),
        "tan" => Some("rgb(210, 180, 140)"),
        "teal" => Some("rgb(0, 128, 128)"),
        "thistle" => Some("rgb(216, 191, 216)"),
        "tomato" => Some("rgb(255, 99, 71)"),
        "turquoise" => Some("rgb(64, 224, 208)"),
        "violet" => Some("rgb(238, 130, 238)"),
        "wheat" => Some("rgb(245, 222, 179)"),
        "white" => Some("rgb(255, 255, 255)"),
        "whitesmoke" => Some("rgb(245, 245, 245)"),
        "yellow" => Some("rgb(255, 255, 0)"),
        "yellowgreen" => Some("rgb(154, 205, 50)"),
        _ => None,
    }
}
