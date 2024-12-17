use compositor_render::scene;

use crate::types::{util::RGBAColor, TypeError};

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
            "Invalid format. Color has to be in #RRGGBB or #RRGGBBAA format.",
        )),
    );
}
