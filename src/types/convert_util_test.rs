use compositor_render::scene;

use crate::types::{
    util::{RGBAColor, RGBColor},
    TypeError,
};

#[test]
fn test_rgb_serialization() {
    fn test_case(color: scene::RGBColor, expected: &str) {
        assert_eq!(RGBColor::from(color), RGBColor(expected.to_string()));
    }

    test_case(scene::RGBColor(0, 0, 0), "#000000");
    test_case(scene::RGBColor(1, 2, 3), "#010203");
    test_case(scene::RGBColor(1, 255, 3), "#01FF03");
}

#[test]
fn test_rgb_deserialization() {
    fn test_case(color: &str, expected: Result<scene::RGBColor, TypeError>) {
        assert_eq!(
            scene::RGBColor::try_from(RGBColor(color.to_string())),
            expected
        );
    }
    test_case("#000000", Ok(scene::RGBColor(0, 0, 0)));
    test_case("#010203", Ok(scene::RGBColor(1, 2, 3)));
    test_case("#01FF03", Ok(scene::RGBColor(1, 255, 3)));
    test_case("#FFffFF", Ok(scene::RGBColor(255, 255, 255)));
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
    fn test_case(color: scene::RGBAColor, expected: &str) {
        assert_eq!(RGBAColor::from(color), RGBAColor(expected.to_string()));
    }
    test_case(scene::RGBAColor(0, 0, 0, 0), "#00000000");
    test_case(scene::RGBAColor(1, 2, 3, 4), "#01020304");
    test_case(scene::RGBAColor(1, 255, 3, 4), "#01FF0304");
}

#[test]
fn test_rgba_deserialization() {
    fn test_case(color: &str, expected: Result<scene::RGBAColor, TypeError>) {
        assert_eq!(
            scene::RGBAColor::try_from(RGBAColor(color.to_string())),
            expected
        );
    }
    test_case("#00000000", Ok(scene::RGBAColor(0, 0, 0, 0)));
    test_case("#01020304", Ok(scene::RGBAColor(1, 2, 3, 4)));
    test_case("#01FF0304", Ok(scene::RGBAColor(1, 255, 3, 4)));
    test_case("#FFffFFff", Ok(scene::RGBAColor(255, 255, 255, 255)));
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
