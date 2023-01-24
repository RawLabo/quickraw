use quickraw::{data, export};

#[test]
fn test_export() {
    let options = export::Options::new(data::GAMMA_SRGB, &data::XYZ2SRGB, false);
    let (image, width, height) = export::load_image_from_file("tests/sample0.ARW", options).unwrap();
    
    assert_eq!(
        "73011456 6048 4024",
        format!("{} {} {}", image.len(), width, height)
    )
}

#[test]
fn test_dng0() {
    let options = export::Options::new(data::GAMMA_SRGB, &data::XYZ2SRGB, false);
    let (image, width, height) = export::load_image_from_file("tests/sample1.dng", options).unwrap();
    println!("{} {} {}", image.len(), width, height);
}