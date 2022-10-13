//! Contains all the functions needed to export image.
//!
use super::*;
use crate::raw::{Orientation, RawImage};

/// Errors cover issues during raw reading and image exporting.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot export the image.")]
    RawFileReadingError(#[from] RawFileReadingError),
    #[error("Cannot create the export object for the file: '{0}'")]
    InvalidFileForNewExport(String),
    #[error("Cannot export image to the file: '{0}'")]
    ErrorWhenExportingFile(String),
    #[error("The {0} image data(len:{1}, width:{2}, height:{3}) is invalid for ImageBuffer.")]
    ImageBufferError(String, usize, usize, usize),
    #[error("Cannot understand the thumbnail image data(len: {0}) for the file: '{1}'")]
    CannotReadThumbnail(usize, String),
}

/// Exports the rendered result.
pub struct Export {
    color_conversion: ColorConversion,
    raw_image: RawImage,
    output: Output,
}

impl Export {
    fn cast_identity(x: u16) -> u16 {
        x
    }
    fn cast_u16_u8(x: u16) -> u8 {
        (x / 256) as u8
    }
    pub fn new(input: Input, output: Output) -> Result<Self, RawFileReadingError> {
        let raw_image = match input {
            Input::ByFile(file) => decode::new_image_from_file(file),
            Input::ByBuffer(buffer) => decode::new_image_from_buffer(buffer),
        }?;

        let color_conversion = ColorConversion::new(&raw_image, output.color_space, output.gamma);

        Ok(Export {
            color_conversion,
            raw_image,
            output,
        })
    }

    /// Exports u8 array reference of thumbnail data and the orientation.
    pub fn export_thumbnail_data(buffer: &[u8]) -> Result<(&[u8], Orientation), Error> {
        let (thumbnail, orientation) = decode::get_thumbnail(buffer)?;
        Ok((thumbnail, orientation))
    }

    /// Exports 16bit image RGB data with width and height.
    pub fn export_16bit_image(&self) -> (Vec<u16>, usize, usize) {
        self.export_image_data(Self::cast_identity)
    }
    /// Exports 8bit image RGB data with width and height.
    pub fn export_8bit_image(&self) -> (Vec<u8>, usize, usize) {
        self.export_image_data(Self::cast_u16_u8)
    }

    #[cfg_attr(
        not(feature = "wasm-bindgen"),
        fn_util::bench(demosaicing_with_postprocess)
    )]
    fn export_image_data<T>(&self, cast_fn: fn(u16) -> T) -> (Vec<T>, usize, usize) {
        match self.output.demosaicing_method {
            DemosaicingMethod::None => self
                .raw_image
                .no_demosaic_render(&self.color_conversion, cast_fn),
            DemosaicingMethod::SuperPixel => self
                .raw_image
                .super_pixel_render(&self.color_conversion, cast_fn),
            DemosaicingMethod::Linear => self
                .raw_image
                .linear_render(&self.color_conversion, cast_fn),
        }
    }

    /// Exports the parsed EXIF data from input
    pub fn export_exif_info(input: Input) -> Result<quickexif::ParsedInfo, RawFileReadingError> {
        let buffer = match input {
            Input::ByFile(file) => decode::get_buffer_from_file(file)?,
            Input::ByBuffer(buffer) => buffer,
        };

        decode::get_exif_info(&buffer)
    }

    /// Print all the parsed EXIF data from input
    pub fn print_exif_info(input: Input) -> Result<String, RawFileReadingError> {
        Export::export_exif_info(input)?
            .stringify_all()
            .map_err(|err| quickexif::parsed_info::Error::from(err).into())
    }
}

/// Enables image rotation and different image types output available.
#[cfg(feature = "image")]
pub mod image_export {
    use super::*;
    use std::io::Write;
    use std::{fs::File, io::BufWriter};

    use image::{
        codecs::jpeg, imageops, ColorType, EncodableLayout, ImageBuffer, ImageEncoder, ImageFormat,
        ImageResult, Rgb,
    };

    trait SaveWithQuality {
        fn save_with_quality(&self, path: &str, quality: u8) -> ImageResult<()>;
    }

    impl<T: 'static + image::Primitive> SaveWithQuality for ImageBuffer<Rgb<T>, Vec<T>>
    where
        [T]: image::EncodableLayout,
    {
        fn save_with_quality(&self, path: &str, quality: u8) -> ImageResult<()> {
            let format = ImageFormat::from_path(path)?;
            match format {
                ImageFormat::Jpeg => {
                    let fout = &mut BufWriter::new(File::create(path)?);
                    jpeg::JpegEncoder::new_with_quality(fout, quality).write_image(
                        self.as_bytes(),
                        self.width(),
                        self.height(),
                        ColorType::Rgb8,
                    )
                }
                _ => self.save(path),
            }
        }
    }

    impl Export {
        #[cfg_attr(not(feature = "wasm-bindgen"), fn_util::bench(writing_file))]
        fn write_to_file<T: 'static + image::Primitive>(
            &self,
            path: &String,
            (data, width, height): (Vec<T>, usize, usize),
            quality: u8,
        ) -> Result<(), Error>
        where
            [T]: image::EncodableLayout,
        {
            let len = data.len();
            let mut image =
                ImageBuffer::<Rgb<T>, Vec<T>>::from_raw(width as u32, height as u32, data)
                    .ok_or_else(|| {
                        Error::ImageBufferError(stringify!(T).to_owned(), len, width, height)
                    })?;

            let image = match (&self.raw_image.crop, self.output.auto_crop) {
                (Some(c), true) => {
                    imageops::crop(&mut image, c.x, c.y, c.width, c.height).to_image()
                }
                _ => image,
            };
            let image = match (&self.raw_image.orientation, self.output.auto_rotate) {
                (Orientation::Horizontal, _) | (_, false) => image,
                (Orientation::Rotate90, true) => imageops::rotate90(&image),
                (Orientation::Rotate180, true) => imageops::rotate180(&image),
                (Orientation::Rotate270, true) => imageops::rotate270(&image),
            };

            image
                .save_with_quality(path, quality)
                .map_err(|_| Error::ErrorWhenExportingFile(path.to_owned()))?;

            Ok(())
        }

        /// Exports the thumbnail image from raw to the path
        pub fn export_thumbnail_to_file(input_path: &str, output_path: &str) -> Result<(), Error> {
            let buffer = decode::get_buffer_from_file(input_path)?;
            let (thumbnail, orientation) = decode::get_thumbnail(buffer.as_slice())?;

            match orientation {
                Orientation::Horizontal => {
                    let mut f = File::create(output_path)
                        .map_err(|_| Error::ErrorWhenExportingFile(output_path.to_owned()))?;
                    f.write_all(thumbnail)
                        .map_err(|_| Error::ErrorWhenExportingFile(output_path.to_owned()))?;
                }
                _ => {
                    let img = image::load_from_memory(thumbnail).map_err(|_| {
                        Error::CannotReadThumbnail(thumbnail.len(), input_path.to_owned())
                    })?;

                    let img = match orientation {
                        Orientation::Rotate90 => imageops::rotate90(&img),
                        Orientation::Rotate180 => imageops::rotate180(&img),
                        Orientation::Rotate270 => imageops::rotate270(&img),
                        _ => img.to_rgba8(),
                    };

                    img.save(output_path)
                        .map_err(|_| Error::ErrorWhenExportingFile(output_path.to_owned()))?;
                }
            }

            Ok(())
        }

        /// Exports the rendered image to a Image file with specificed quality.
        pub fn export_image(&self, quality: u8) -> Result<(), Error> {
            match &self.output.output_type {
                OutputType::Image8(path) => {
                    let data = self.export_image_data(Export::cast_u16_u8);
                    self.write_to_file(path, data, quality)?;
                }
                OutputType::Image16(path) => {
                    let data = self.export_image_data(Export::cast_identity);
                    self.write_to_file(path, data, quality)?;
                }
                _ => {}
            }

            Ok(())
        }
    }
}
