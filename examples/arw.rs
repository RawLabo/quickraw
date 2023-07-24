use std::{fs, time::Instant};

fn main() {
    let t = Instant::now();
    let (image_u16, w, h) = quickraw::extract_image("examples/samples/sample1.ARW").unwrap();
    println!("time elapsed: {:?}", Instant::now() - t);

    let image_beu8 : Vec<_> = image_u16.into_iter().flat_map(|v| v.to_be_bytes()).collect();
    let mut ppm_header = format!("P6 {w} {h} 65535\n").as_bytes().to_vec();
    ppm_header.extend(image_beu8);
    // fs::write("sample.ppm", ppm_header).unwrap();
}