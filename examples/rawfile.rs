use std::{env, fs::File, io};

fn open_file_by_type(x: &str) -> io::Result<File> {
    let prefix = "examples/samples/";
    let f = match x {
        "arw" | "arw0" => "sample0.ARW",
        "arw1" => "sample1.ARW",
        "dng0" => "dng/uncompressed-bayer.dng",
        "dng1" => "dng/uncompressed-rgb.dng",
        "dng2" => "dng/lossless-compressed-bayer.dng",
        "dng3" => "dng/lossless-compressed-rgb.dng",
        "dng4" => "dng/lossy-compressed.dng",
        path => path,
    };
    let path = format!("{prefix}{f}");
    println!("Processing raw file: {}", path);
    File::open(path)
}

fn main() {
    let args = env::args().collect::<Box<_>>();
    let args = args.into_iter().map(|x| x.as_str()).collect::<Box<_>>();

    // get file type
    if let Some(t) = args.get(1) {
        let sample_file = open_file_by_type(t).unwrap();
        let (image_u16, w, h) = quickraw::extract_image::<3>(
            sample_file,
            quickraw::color_data::GAMMA_SRGB,
            &quickraw::color_data::XYZ2SRGB,
        )
        .unwrap();

        if let Some(&"dump") = args.get(2) {
            println!("Dumping image to dump.ppm");
            let image_beu8: Box<_> = image_u16
                .into_iter()
                .flat_map(|v| v.to_be_bytes())
                .collect();
            let mut ppm_header = format!("P6 {w} {h} 65535\n").as_bytes().to_vec();
            ppm_header.extend(&image_beu8[..]);
            std::fs::write("dump.ppm", ppm_header).unwrap();
        }
    } else {
        eprintln!("Please specify file type like arw, dng, etc.");
    }
}
