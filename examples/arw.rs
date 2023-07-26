use std::env;

fn main() {
    let args = env::args().collect::<Box<_>>();
    let args = args.into_iter().map(|x| x.as_str()).collect::<Box<_>>();

    let (image_u16, w, h) = quickraw::extract_image("examples/samples/sample1.ARW").unwrap();

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
