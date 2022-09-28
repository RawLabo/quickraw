use quickraw::{Export, Input, Output, OutputType, DemosaicingMethod};

#[test]
fn test_export() {
    let demosaicing_method = DemosaicingMethod::Linear;
    let color_space = quickraw::data::XYZ2SRGB;
    let gamma = quickraw::data::GAMMA_SRGB;
    let output_type = OutputType::Raw16;
    let auto_crop = false;
    let auto_rotate = false;

    let export_job = Export::new(
        Input::ByFile("tests/sample0.ARW"),
        Output::new(
            demosaicing_method,
            color_space,
            gamma,
            output_type,
            auto_crop,
            auto_rotate,
        ),
    ).unwrap();

    let (image, width, height) = export_job.export_16bit_image();
    assert_eq!("73011456 6048 4024", format!("{} {} {}", image.len(), width, height))
}
