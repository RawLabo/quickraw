use std::{env, fs::File};

fn main() {
    let args = env::args().collect::<Box<_>>();
    let args = args.into_iter().map(|x| x.as_str()).collect::<Box<_>>();

    let sample_file = File::open("examples/samples/sample0.ARW").unwrap();
    let (image_u16, w, h) = quickraw::extract_image::<3>(
        sample_file,
        quickraw::color_data::GAMMA_SRGB,
        &quickraw::color_data::XYZ2SRGB,
    )
    .unwrap();

    match &args[..] {
        [_, "dump"] => {
            let image_beu8: Box<_> = image_u16
                .into_iter()
                .flat_map(|v| v.to_be_bytes())
                .collect();
            let mut ppm_header = format!("P6 {w} {h} 65535\n").as_bytes().to_vec();
            ppm_header.extend(&image_beu8[..]);
            std::fs::write("dump.ppm", ppm_header).unwrap();
        }
        _ => {}
    }
}
