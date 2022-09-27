use anyhow::{Context, Result};
use core::panic;
use quickraw::{
    data, export::Export, DemosaicingMethod, ExportError, Input, Output, OutputType, BENCH_FLAG,
};
use rayon::prelude::*;
use std::{env, fs, mem, path::Path};

#[derive(Clone)]
enum Switch {
    On,
    Off,
    Only,
}

#[derive(Clone)]
struct Options<'a> {
    inputs: Vec<String>,
    output_dir: Option<&'a str>,
    output_type: OutputType,
    jpeg_quality: u8,
    demosaicing_method: DemosaicingMethod,
    color_space: [f32; 9],
    gamma: [f32; 2],
    auto_crop: bool,
    auto_rotate: bool,
    thumbnail: Switch,
    exif_info: Switch,
}

fn option_handler<'a>(option_slice: &[&'a str], result: &mut Options<'a>) {
    match option_slice {
        ["--inputs", files] | ["-i", files] => {
            result.inputs = files.split(',').map(|p| p.to_owned()).collect();
        }
        ["--input-dir", dir] | ["-id", dir] => {
            result.inputs = match fs::read_dir(dir) {
                Ok(paths) => paths
                    .filter_map(|x| x.ok())
                    .filter_map(|x| x.path().into_os_string().into_string().ok())
                    .collect(),
                Err(_) => panic!("Invalid directory: {:?}", dir),
            }
        }
        ["--output-dir", dir] | ["-od", dir] => {
            result.output_dir = Some(dir);
        }
        ["--output-type", t] | ["-ot", t] => {
            result.output_type = match *t {
                "jpeg" => OutputType::Image8(".jpg".to_owned()),
                "tiff8" => OutputType::Image8(".tif".to_owned()),
                _ => OutputType::Image16(".tif".to_owned()),
            }
        }
        ["--jpeg-quality", q] | ["-jq", q] => {
            result.jpeg_quality = q.parse::<u8>().unwrap_or(92);
        }
        ["--bench"] | ["-b"] => {
            env::set_var(BENCH_FLAG, "1");
        }
        ["--demosaicing", method] | ["-d", method] => {
            result.demosaicing_method = match *method {
                "none" => DemosaicingMethod::None,
                "super" => DemosaicingMethod::SuperPixel,
                _ => DemosaicingMethod::Linear,
            };
        }
        ["--color-space", color_space] | ["-cs", color_space] => {
            result.color_space = match *color_space {
                "raw" => data::XYZ2RAW,
                "srgb" => data::XYZ2SRGB,
                "adobergb" => data::XYZ2ADOBE_RGB,
                matrix => {
                    let c = matrix
                        .split(',')
                        .filter_map(|x| x.parse::<f32>().ok())
                        .collect::<Vec<f32>>();

                    match c.try_into() {
                        Ok(x) => x,
                        Err(_) => result.color_space,
                    }
                }
            }
        }
        ["--gamma", gamma] | ["-g", gamma] => {
            let g = gamma
                .split(',')
                .filter_map(|x| x.parse::<f32>().ok())
                .collect::<Vec<f32>>();
            result.gamma = match g.try_into() {
                Ok(x) => x,
                Err(_) => result.gamma,
            }
        }
        ["--auto-crop"] | ["-ac"] => {
            result.auto_crop = true;
        }
        ["--auto-rotate"] | ["-ar"] => {
            result.auto_rotate = true;
        }
        ["--thumbnail", value] | ["-t", value] => {
            result.thumbnail = match *value {
                "on" => Switch::On,
                "only" => Switch::Only,
                _ => Switch::Off,
            }
        }
        ["--exif-info", value] | ["-ei", value] => {
            result.exif_info = match *value {
                "on" => Switch::On,
                "only" => Switch::Only,
                _ => Switch::Off,
            }
        }
        _ => panic!("Invalid options: {:?}", option_slice),
    }
}

fn merge_path(path: &str, output_dir: Option<&str>) -> String {
    match output_dir {
        Some(od) => {
            od.to_owned()
                + Path::new(path)
                    .file_name()
                    .and_then(|x| x.to_str())
                    .expect("Invalid file name")
        }
        None => path.to_owned(),
    }
}

fn merge_output_type(
    path: &str,
    output_dir: Option<&str>,
    prev_output_type: OutputType,
) -> OutputType {
    let merged_path = merge_path(path, output_dir);

    match &prev_output_type {
        OutputType::Image8(surfix) => OutputType::Image8(merged_path + surfix),
        OutputType::Image16(surfix) => OutputType::Image16(merged_path + surfix),
        _ => {
            panic!("Invalid output type");
        }
    }
}

fn export_by_file(file: &str, options: Options) -> Result<()> {
    match options.thumbnail {
        Switch::Only | Switch::On => {
            let path = merge_path(file, options.output_dir) + ".thumbnail.jpg";
            Export::export_thumbnail_to_file(file, path.as_str())
                .with_context(|| ExportError::InvalidFileForNewExport(file.to_owned()))?;

            if matches!(options.thumbnail, Switch::Only) {
                return Ok(());
            }
        }
        Switch::Off => {}
    };

    match options.exif_info {
        Switch::Only | Switch::On => {
            let info = Export::export_exif_info_directly(Input::ByFile(file))
                .with_context(|| ExportError::InvalidFileForNewExport(file.to_owned()))?;
            println!("\nExif info for '{file}':\n{info}");

            if matches!(options.exif_info, Switch::Only) {
                return Ok(());
            }
        }
        Switch::Off => {}
    };

    let output = Output {
        demosaicing_method: options.demosaicing_method,
        color_space: options.color_space,
        gamma: options.gamma,
        auto_crop: options.auto_crop,
        auto_rotate: options.auto_rotate,
        output_type: merge_output_type(file, options.output_dir, options.output_type),
    };
    let export = Export::new(Input::ByFile(file), output)
        .with_context(|| ExportError::InvalidFileForNewExport(file.to_owned()))?;

    export.export_to_file(options.jpeg_quality)?;
    Ok(())
}

fn export_by_options(mut options: Options) -> Result<()> {
    let inputs = mem::take(&mut options.inputs);

    match inputs.as_slice() {
        [file] => {
            export_by_file(file.as_str(), options)?;
        }
        files => {
            files
                .par_iter()
                .map(|file| -> Result<&str> {
                    let options_cloned = options.clone();
                    export_by_file(file.as_str(), options_cloned)?;
                    Ok(file.as_str())
                })
                .for_each(|result| match result {
                    Ok(f) => {
                        if !matches!(options.exif_info, Switch::Only) {
                            println!("Finished: {}", f);
                        }
                    }
                    Err(e) => {
                        eprintln!("\nError: {}", e);
                        e.chain().skip(1).for_each(|item| {
                            eprintln!("Caused by:\n  {}", item);
                        })
                    }
                });
        }
    };

    Ok(())
}

#[attrs::bench(total_process)]
fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let args = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

    match &args[1..] {
        ["-v"] | ["--version"] => {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
        ["-h"] | ["--help"] | [] => {
            println!("{}", HELP);
        }
        input_options => {
            env::remove_var(BENCH_FLAG);
            let mut result = Options {
                inputs: vec![],
                output_dir: None,
                output_type: OutputType::Image16(".tif".to_owned()),
                jpeg_quality: 92,
                demosaicing_method: DemosaicingMethod::Linear,
                color_space: data::XYZ2SRGB,
                gamma: [0.45, 4.5],
                auto_crop: false,
                auto_rotate: false,
                thumbnail: Switch::Off,
                exif_info: Switch::Off,
            };

            input_options.iter().for_each(|&option| {
                let option_vec = option.split('=').collect::<Vec<&str>>();
                option_handler(option_vec.as_slice(), &mut result);
            });

            export_by_options(result)?;
        }
    }

    Ok(())
}

static HELP: &str = "===== quickraw: A digital still camera raw file tool library =====

quickraw [-v | --version] 
             Desc: Print the version number.
         
         [--inputs(-i)=<raw_files>]
             Desc: Set the input raw files. Multiple files can use all cpu cores.
             Please split files with comma. Like: path/to/file1,path/to/file2,path/to/file3
 
         [--input-dir(-id)=<directory>]
             Desc: Set the input directory. All files in the directory will be processed.
 
         [--output-dir(-od)=<directory>]
             Default: input files' directory
             Desc: Set the output directory.
         
         [--thumbnail(-t)=<thumbnail_option>]
             Values: on | off | only
             Default: off
             Desc: A switch for thumbnail export. 
                  'only' means no raw image will be exported.
         
         [--exif-info(-ei)=<exif_info_option>]
             Values: on | off | only
             Default: off
             Desc: Display the collected exif information.
                   'only' means no raw image will be exported.
 
         [--bench(-b)]
             Default: no --bench
             Desc: Having --bench flag means true for benchmark some key processes.
 

 Raw image output options:
 
         [--output-type(-ot)=<file_type>]
             Values: jpeg | tiff8 | tiff16
             Default: tiff16
             Desc: Set the output image type.
 
         [--jpeg-quality(-jq)=<jpeg_quality>]
             Values: 0 - 100
             Default: 92
             Desc: Set the quality value for JPEG output. 
 
         [--demosaicing(-d)=<demosaicing_method>]
             Values: linear | none | super
             Default: linear
             Desc: Set the demosaicing method.
 
         [--color-space(-cs)=<color_space>]
             Values: raw | srgb | adobergb | 9_float_numbers_split_with_comma
             Default: srgb
             Desc: Set the output color space matrix.
                   You can use predefined color space or set your like this:
                   2.0413,-0.5649,-0.3446,-0.9692,1.8760,0.0415,0.0134,-0.1183,1.0154
 
         [--gamma(-g)=<power>,<linear_coefficient>]
             Default: 0.45,4.5
             Desc: Set the gamma power and linear coefficient.
 
         [--auto-crop(-ac)]
             Default: no --auto-crop
             Desc: Having --auto-crop flag means true for image auto cropping.
 
         [--auto-rotate(-ar)]
             Default: no --auto-rotate
             Desc: Having --auto-rotate flag means true for image auto rotation.
";
