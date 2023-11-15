use compositor_common::scene::Resolution;

use super::test_case::{TestCase, TestInput};

const DEFAULT_RESOLUTION: Resolution = Resolution {
    width: 640,
    height: 360,
};

pub fn snapshot_tests() -> Vec<TestCase> {
    let mut tests = Vec::new();
    tests.append(&mut base_snapshot_tests());
    tests.append(&mut view_snapshot_tests());
    // tests.append(&mut text_snapshot_tests());
    // tests.append(&mut tiled_layout_tests());
    // tests.append(&mut stretch_to_resolution_tests());
    // tests.append(&mut fill_to_resolution_tests());
    // tests.append(&mut fit_to_resolution_tests());
    // tests.append(&mut fixed_position_layout_tests());
    // tests.append(&mut corners_rounding_tests());
    // tests.append(&mut mirror_image());
    tests
}

//fn mirror_image() -> Vec<TestCase> {
//    let image_renderer = include_str!("../../snapshot_tests/register/image_jpeg.register.json");
//
//    Vec::from([
//        TestCase {
//            name: "mirror_image/vertical",
//            scene_json: include_str!("../../snapshot_tests/mirror_image/vertical.scene.json"),
//            renderers: vec![image_renderer],
//            ..Default::default()
//        },
//        TestCase {
//            name: "mirror_image/horizontal",
//            scene_json: include_str!("../../snapshot_tests/mirror_image/horizontal.scene.json"),
//            renderers: vec![image_renderer],
//            ..Default::default()
//        },
//        TestCase {
//            name: "mirror_image/horizontal-vertical",
//            scene_json: include_str!(
//                "../../snapshot_tests/mirror_image/horizontal_and_vertical.scene.json"
//            ),
//            renderers: vec![image_renderer],
//            ..Default::default()
//        },
//    ])
//}
//
//fn corners_rounding_tests() -> Vec<TestCase> {
//    let input1 = TestInput::new(1);
//    Vec::from([
//        TestCase {
//            name: "corners_rounding/border_radius_50px",
//            scene_json: include_str!(
//                "../../snapshot_tests/corners_rounding/border_radius_50px.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "corners_rounding/border_radius_5px",
//            scene_json: include_str!(
//                "../../snapshot_tests/corners_rounding/border_radius_5px.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//    ])
//}
//
//fn fixed_position_layout_tests() -> Vec<TestCase> {
//    let input1 = TestInput::new(1);
//
//    Vec::from([
//        TestCase {
//            name: "fixed_position_layout/top_right_corner",
//            scene_json: include_str!(
//                "../../snapshot_tests/fixed_position_layout/top_right_corner.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "fixed_position_layout/rotation_30_degree",
//            scene_json: include_str!(
//                "../../snapshot_tests/fixed_position_layout/rotation_30_degree.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "fixed_position_layout/rotation_multiple_loops",
//            scene_json: include_str!(
//                "../../snapshot_tests/fixed_position_layout/rotation_multiple_loops.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "fixed_position_layout/overlapping_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/fixed_position_layout/overlapping_inputs.scene.json"
//            ),
//            inputs: vec![
//                input1.clone(),
//                TestInput::new(2),
//                TestInput::new(3),
//                TestInput::new(4),
//            ],
//            ..Default::default()
//        },
//        // TODO: fix stretch to resolution
//        //TestCase {
//        //    name: "fixed_position_layout/scale_2x",
//        //    scene_json: include_str!(
//        //        "../../snapshot_tests/fixed_position_layout/scale_2x.scene.json"
//        //    ),
//        //    inputs: vec![input1.clone()],
//        //    ..Default::default()
//        //},
//        //TestCase {
//        //    name: "fixed_position_layout/scale_0_5x",
//        //    scene_json: include_str!(
//        //        "../../snapshot_tests/fixed_position_layout/scale_0_5x.scene.json"
//        //    ),
//        //    inputs: vec![input1.clone()],
//        //    ..Default::default()
//        //},
//    ])
//}
//
//pub fn fit_to_resolution_tests() -> Vec<TestCase> {
//    let input1 = TestInput::new(1);
//    Vec::from([
//        TestCase {
//            name: "fit_to_resolution/narrow_column",
//            scene_json: include_str!(
//                "../../snapshot_tests/fit_to_resolution/narrow_column.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "fit_to_resolution/wide_row",
//            scene_json: include_str!("../../snapshot_tests/fit_to_resolution/wide_row.scene.json"),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "fit_to_resolution/green_background",
//            scene_json: include_str!(
//                "../../snapshot_tests/fit_to_resolution/green_background.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "fit_to_resolution/vertical_align_top",
//            scene_json: include_str!(
//                "../../snapshot_tests/fit_to_resolution/vertical_align_top.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "fit_to_resolution/horizontal_align_right",
//            scene_json: include_str!(
//                "../../snapshot_tests/fit_to_resolution/horizontal_align_right.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//    ])
//}
//
//pub fn fill_to_resolution_tests() -> Vec<TestCase> {
//    let input1 = TestInput::new(1);
//    Vec::from([
//        TestCase {
//            name: "fill_to_resolution/narrow_column",
//            scene_json: include_str!(
//                "../../snapshot_tests/fill_to_resolution/narrow_column.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "fill_to_resolution/wide_row",
//            scene_json: include_str!("../../snapshot_tests/fill_to_resolution/wide_row.scene.json"),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//    ])
//}
//
//#[allow(dead_code)]
//pub fn stretch_to_resolution_tests() -> Vec<TestCase> {
//    let input1 = TestInput::new(1);
//    Vec::from([
//        TestCase {
//            name: "stretch_to_resolution/narrow_column",
//            scene_json: include_str!(
//                "../../snapshot_tests/stretch_to_resolution/narrow_column.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "stretch_to_resolution/wide_row",
//            scene_json: include_str!(
//                "../../snapshot_tests/stretch_to_resolution/wide_row.scene.json"
//            ),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//    ])
//}
//
//pub fn tiled_layout_tests() -> Vec<TestCase> {
//    let input1 = TestInput::new(1);
//    let input2 = TestInput::new(2);
//    let input3 = TestInput::new(3);
//    let input4 = TestInput::new(4);
//    let input5 = TestInput::new(5);
//    let input6 = TestInput::new(6);
//    let input7 = TestInput::new(7);
//    let input8 = TestInput::new(8);
//    let input9 = TestInput::new(9);
//    let input10 = TestInput::new(10);
//    let input11 = TestInput::new(11);
//    let input12 = TestInput::new(12);
//    let input13 = TestInput::new(13);
//    let input14 = TestInput::new(14);
//    let input15 = TestInput::new(15);
//    let portrait_resolution = Resolution {
//        width: 360,
//        height: 640,
//    };
//    let portrait_input1 = TestInput::new_with_resolution(1, portrait_resolution);
//    let portrait_input2 = TestInput::new_with_resolution(2, portrait_resolution);
//    let portrait_input3 = TestInput::new_with_resolution(3, portrait_resolution);
//    let portrait_input4 = TestInput::new_with_resolution(4, portrait_resolution);
//    let portrait_input5 = TestInput::new_with_resolution(5, portrait_resolution);
//    let portrait_input6 = TestInput::new_with_resolution(6, portrait_resolution);
//    let portrait_input7 = TestInput::new_with_resolution(7, portrait_resolution);
//    let portrait_input8 = TestInput::new_with_resolution(8, portrait_resolution);
//    let portrait_input9 = TestInput::new_with_resolution(9, portrait_resolution);
//    let portrait_input10 = TestInput::new_with_resolution(10, portrait_resolution);
//    let portrait_input11 = TestInput::new_with_resolution(11, portrait_resolution);
//    let portrait_input12 = TestInput::new_with_resolution(12, portrait_resolution);
//    let portrait_input13 = TestInput::new_with_resolution(13, portrait_resolution);
//    let portrait_input14 = TestInput::new_with_resolution(14, portrait_resolution);
//    let portrait_input15 = TestInput::new_with_resolution(15, portrait_resolution);
//    Vec::from([
//        TestCase {
//            name: "tiled_layout/01_inputs",
//            scene_json: include_str!("../../snapshot_tests/tiled_layout/01_inputs.scene.json"),
//            inputs: vec![input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/02_inputs",
//            scene_json: include_str!("../../snapshot_tests/tiled_layout/02_inputs.scene.json"),
//            inputs: vec![input1.clone(), input2.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/03_inputs",
//            scene_json: include_str!("../../snapshot_tests/tiled_layout/03_inputs.scene.json"),
//            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/04_inputs",
//            scene_json: include_str!("../../snapshot_tests/tiled_layout/04_inputs.scene.json"),
//            inputs: vec![
//                input1.clone(),
//                input2.clone(),
//                input3.clone(),
//                input4.clone(),
//            ],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/05_inputs",
//            scene_json: include_str!("../../snapshot_tests/tiled_layout/05_inputs.scene.json"),
//            inputs: vec![
//                input1.clone(),
//                input2.clone(),
//                input3.clone(),
//                input4.clone(),
//                input5.clone(),
//            ],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/15_inputs",
//            scene_json: include_str!("../../snapshot_tests/tiled_layout/15_inputs.scene.json"),
//            inputs: vec![
//                input1.clone(),
//                input2.clone(),
//                input3.clone(),
//                input4.clone(),
//                input5.clone(),
//                input6.clone(),
//                input7.clone(),
//                input8.clone(),
//                input9.clone(),
//                input10.clone(),
//                input11.clone(),
//                input12.clone(),
//                input13.clone(),
//                input14.clone(),
//                input15.clone(),
//            ],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/01_portrait_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/01_portrait_inputs.scene.json"
//            ),
//            inputs: vec![portrait_input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/02_portrait_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/02_portrait_inputs.scene.json"
//            ),
//            inputs: vec![portrait_input1.clone(), portrait_input2.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/03_portrait_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/03_portrait_inputs.scene.json"
//            ),
//            inputs: vec![
//                portrait_input1.clone(),
//                portrait_input2.clone(),
//                portrait_input3.clone(),
//            ],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/05_portrait_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/05_portrait_inputs.scene.json"
//            ),
//            inputs: vec![
//                portrait_input1.clone(),
//                portrait_input2.clone(),
//                portrait_input3.clone(),
//                portrait_input4.clone(),
//                portrait_input5.clone(),
//            ],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/15_portrait_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/15_portrait_inputs.scene.json"
//            ),
//            inputs: vec![
//                portrait_input1.clone(),
//                portrait_input2.clone(),
//                portrait_input3.clone(),
//                portrait_input4.clone(),
//                portrait_input5.clone(),
//                portrait_input6.clone(),
//                portrait_input7.clone(),
//                portrait_input8.clone(),
//                portrait_input9.clone(),
//                portrait_input10.clone(),
//                portrait_input11.clone(),
//                portrait_input12.clone(),
//                portrait_input13.clone(),
//                portrait_input14.clone(),
//                portrait_input15.clone(),
//            ],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/01_inputs_on_portrait_output",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/01_inputs_on_portrait_output.scene.json"
//            ),
//            inputs: vec![portrait_input1.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/03_inputs_on_portrait_output",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/03_inputs_on_portrait_output.scene.json"
//            ),
//            inputs: vec![
//                portrait_input1.clone(),
//                portrait_input2.clone(),
//                portrait_input3.clone(),
//            ],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/05_portrait_inputs_on_portrait_output",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/05_portrait_inputs_on_portrait_output.scene.json"
//            ),
//            inputs: vec![
//                portrait_input1.clone(),
//                portrait_input2.clone(),
//                portrait_input3.clone(),
//                portrait_input4.clone(),
//                portrait_input5.clone(),
//            ],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/15_portrait_inputs_on_portrait_output",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/15_portrait_inputs_on_portrait_output.scene.json"
//            ),
//            inputs: vec![
//                portrait_input1.clone(),
//                portrait_input2.clone(),
//                portrait_input3.clone(),
//                portrait_input4.clone(),
//                portrait_input5.clone(),
//                portrait_input6.clone(),
//                portrait_input7.clone(),
//                portrait_input8.clone(),
//                portrait_input9.clone(),
//                portrait_input10.clone(),
//                portrait_input11.clone(),
//                portrait_input12.clone(),
//                portrait_input13.clone(),
//                portrait_input14.clone(),
//                portrait_input15.clone(),
//            ],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/align_center_with_03_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/align_center_with_03_inputs.scene.json"
//            ),
//            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/align_top_left_with_03_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/align_top_left_with_03_inputs.scene.json"
//            ),
//            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/align_with_margin_and_padding_with_03_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/align_with_margin_and_padding_with_03_inputs.scene.json"
//            ),
//            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/margin_with_03_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/margin_with_03_inputs.scene.json"
//            ),
//            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/margin_and_padding_with_03_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/margin_and_padding_with_03_inputs.scene.json"
//            ),
//            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
//            ..Default::default()
//        },
//        TestCase {
//            name: "tiled_layout/padding_with_03_inputs",
//            scene_json: include_str!(
//                "../../snapshot_tests/tiled_layout/padding_with_03_inputs.scene.json"
//            ),
//            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
//            ..Default::default()
//        },
//    ])
//}
//
//pub fn text_snapshot_tests() -> Vec<TestCase> {
//    Vec::from([
//        TestCase {
//            name: "text/align_center",
//            scene_json: include_str!("../../snapshot_tests/text/align_center.scene.json"),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/align_right",
//            scene_json: include_str!("../../snapshot_tests/text/align_right.scene.json"),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/bold_text",
//            scene_json: include_str!("../../snapshot_tests/text/bold_text.scene.json"),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/dimensions_fitted_column_with_long_text",
//            scene_json: include_str!(
//                "../../snapshot_tests/text/dimensions_fitted_column_with_long_text.scene.json"
//            ),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/dimensions_fitted_column_with_short_text",
//            scene_json: include_str!(
//                "../../snapshot_tests/text/dimensions_fitted_column_with_short_text.scene.json"
//            ),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/dimensions_fitted",
//            scene_json: include_str!("../../snapshot_tests/text/dimensions_fitted.scene.json"),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/dimensions_fixed",
//            scene_json: include_str!("../../snapshot_tests/text/dimensions_fixed.scene.json"),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/dimensions_fixed_with_overflow",
//            scene_json: include_str!(
//                "../../snapshot_tests/text/dimensions_fixed_with_overflow.scene.json"
//            ),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/red_text_on_blue_background",
//            scene_json: include_str!(
//                "../../snapshot_tests/text/red_text_on_blue_background.scene.json"
//            ),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/wrap_glyph",
//            scene_json: include_str!("../../snapshot_tests/text/wrap_glyph.scene.json"),
//            allowed_error: 325.7,
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/wrap_none",
//            scene_json: include_str!("../../snapshot_tests/text/wrap_none.scene.json"),
//            ..Default::default()
//        },
//        TestCase {
//            name: "text/wrap_word",
//            scene_json: include_str!("../../snapshot_tests/text/wrap_word.scene.json"),
//            allowed_error: 321.8,
//            ..Default::default()
//        },
//    ])
//}

pub fn view_snapshot_tests() -> Vec<TestCase> {
    Vec::from([
        TestCase {
            name: "view/constant_width_views_row",
            outputs: vec![(
                include_str!("../../snapshot_tests/view/constant_width_views_row.scene.json"),
                DEFAULT_RESOLUTION,
            )],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_views_row_with_overflow",
            outputs: vec![(
                include_str!("../../snapshot_tests/view/constant_width_views_row_with_overflow.scene.json"),
                DEFAULT_RESOLUTION,
            )],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/dynamic_width_views_row",
            outputs: vec![(
                include_str!("../../snapshot_tests/view/dynamic_width_views_row.scene.json"),
                DEFAULT_RESOLUTION,
            )],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/dynamic_and_constant_width_views_row",
            outputs: vec![(
                include_str!("../../snapshot_tests/view/dynamic_and_constant_width_views_row.scene.json"),
                DEFAULT_RESOLUTION,
            )],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/dynamic_and_constant_width_views_row_with_overflow",
            outputs: vec![(
                include_str!("../../snapshot_tests/view/dynamic_and_constant_width_views_row_with_overflow.scene.json"),
                DEFAULT_RESOLUTION,
            )],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_and_height_views_row",
            outputs: vec![(
                include_str!("../../snapshot_tests/view/constant_width_and_height_views_row.scene.json"),
                DEFAULT_RESOLUTION,
            )],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/view_with_relative_positioning_partially_covered_by_sibling",
            outputs: vec![(
                include_str!("../../snapshot_tests/view/view_with_relative_positioning_partially_covered_by_sibling.scene.json"),
                DEFAULT_RESOLUTION,
            )],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/view_with_relative_positioning_render_over_siblings",
            outputs: vec![(
                include_str!("../../snapshot_tests/view/view_with_relative_positioning_render_over_siblings.scene.json"),
                DEFAULT_RESOLUTION,
            )],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/root_view_with_background_color",
            outputs: vec![(
                include_str!("../../snapshot_tests/view/root_view_with_background_color.scene.json"),
                DEFAULT_RESOLUTION,
            )],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        }
    ])
}

pub fn base_snapshot_tests() -> Vec<TestCase> {
    Vec::from([TestCase {
        name: "simple_input_pass_through",
        outputs: vec![(
            include_str!("../../snapshot_tests/simple_input_pass_through.scene.json"),
            DEFAULT_RESOLUTION,
        )],
        inputs: vec![TestInput::new(1)],
        ..Default::default()
    }])
}
