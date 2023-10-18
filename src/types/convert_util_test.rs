use crate::types::{
    util::{Coord, RGBAColor, RGBColor},
    TypeError,
};
use compositor_common::util::{colors, coord};

#[test]
fn test_rgb_serialization() {
    fn test_case(color: colors::RGBColor, expected: &str) {
        assert_eq!(RGBColor::from(color), RGBColor(expected.to_string()));
    }

    test_case(colors::RGBColor(0, 0, 0), "#000000");
    test_case(colors::RGBColor(1, 2, 3), "#010203");
    test_case(colors::RGBColor(1, 255, 3), "#01FF03");
}

#[test]
fn test_rgb_deserialization() {
    fn test_case(color: &str, expected: Result<colors::RGBColor, TypeError>) {
        assert_eq!(
            colors::RGBColor::try_from(RGBColor(color.to_string())),
            expected
        );
    }
    test_case("#000000", Ok(colors::RGBColor(0, 0, 0)));
    test_case("#010203", Ok(colors::RGBColor(1, 2, 3)));
    test_case("#01FF03", Ok(colors::RGBColor(1, 255, 3)));
    test_case("#FFffFF", Ok(colors::RGBColor(255, 255, 255)));
    test_case(
        "#00000G",
        Err(TypeError::new(
            "Invalid format. Color representation is not a valid hexadecimal number.",
        )),
    );
    test_case(
        "#000",
        Err(TypeError::new(
            "Invalid format. Color has to be in #RRGGBB format.",
        )),
    );
}

#[test]
fn test_rgba_serialization() {
    fn test_case(color: colors::RGBAColor, expected: &str) {
        assert_eq!(RGBAColor::from(color), RGBAColor(expected.to_string()));
    }
    test_case(colors::RGBAColor(0, 0, 0, 0), "#00000000");
    test_case(colors::RGBAColor(1, 2, 3, 4), "#01020304");
    test_case(colors::RGBAColor(1, 255, 3, 4), "#01FF0304");
}

#[test]
fn test_rgba_deserialization() {
    fn test_case(color: &str, expected: Result<colors::RGBAColor, TypeError>) {
        assert_eq!(
            colors::RGBAColor::try_from(RGBAColor(color.to_string())),
            expected
        );
    }
    test_case("#00000000", Ok(colors::RGBAColor(0, 0, 0, 0)));
    test_case("#01020304", Ok(colors::RGBAColor(1, 2, 3, 4)));
    test_case("#01FF0304", Ok(colors::RGBAColor(1, 255, 3, 4)));
    test_case("#FFffFFff", Ok(colors::RGBAColor(255, 255, 255, 255)));
    test_case(
        "#0000000G",
        Err(TypeError::new(
            "Invalid format. Color representation is not a valid hexadecimal number.",
        )),
    );
    test_case(
        "#000",
        Err(TypeError::new(
            "Invalid format. Color has to be in #RRGGBBAA format.",
        )),
    );
}

#[test]
fn test_coords_serialization() {
    fn test_case_str(coord: coord::Coord, expected: &str) {
        assert_eq!(Coord::from(coord), Coord::String(expected.to_string()));
    }
    test_case_str(coord::Coord::Pixel(-31), "-31px");
    test_case_str(coord::Coord::Percent(67), "67%");
}

#[test]
fn test_coords_deserialization() {
    fn test_case_str(coord: &str, expected: Result<coord::Coord, TypeError>) {
        assert_eq!(
            coord::Coord::try_from(Coord::String(coord.to_string())),
            expected
        );
    }

    const ERROR_MESSAGE: &str = "Invalid format. Coord definition can only be specified as number (pixels count), number with `px` suffix (pixels count) or number with `%` suffix (percents count)";

    test_case_str("100", Ok(coord::Coord::Pixel(100)));
    test_case_str("2137px", Ok(coord::Coord::Pixel(2137)));
    test_case_str("-420px", Ok(coord::Coord::Pixel(-420)));
    test_case_str("69%", Ok(coord::Coord::Percent(69)));
    test_case_str("-1337%", Ok(coord::Coord::Percent(-1337)));

    test_case_str("-1-337%", Err(TypeError::new(ERROR_MESSAGE.to_string())));
    test_case_str("1x", Err(TypeError::new(ERROR_MESSAGE.to_string())));
}
