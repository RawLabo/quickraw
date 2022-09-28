use std::{
    fs::File,
    io::{BufWriter, Write},
};

use image::{
    codecs::jpeg, imageops, ColorType, EncodableLayout, ImageBuffer, ImageEncoder, ImageFormat,
    ImageResult, Rgb,
};

use super::*;
use crate::raw::{Orientation, RawImage};

pub struct Export {
    color_conversion: ColorConversion,
    raw_image: RawImage,
    output: Output,
}

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

    fn _export_thumbnail_data(buffer: &[u8]) -> Result<(&[u8], Orientation), ExportError> {
        let (thumbnail, orientation) = decode::get_thumbnail(buffer)?;
        Ok((thumbnail, orientation))
    }
    pub fn export_thumbnail_to_file(
        input_path: &str,
        output_path: &str,
    ) -> Result<(), ExportError> {
        let buffer = decode::get_buffer_from_file(input_path)?;
        let (thumbnail, orientation) = decode::get_thumbnail(buffer.as_slice())?;

        match orientation {
            Orientation::Horizontal => {
                let mut f = File::create(output_path)
                    .map_err(|_| ExportError::ErrorWhenExportingFile(output_path.to_owned()))?;
                f.write_all(thumbnail)
                    .map_err(|_| ExportError::ErrorWhenExportingFile(output_path.to_owned()))?;
            }
            _ => {
                let img = image::load_from_memory(thumbnail).map_err(|_| {
                    ExportError::CannotReadThumbnail(thumbnail.len(), input_path.to_owned())
                })?;

                let img = match orientation {
                    Orientation::Rotate90 => imageops::rotate90(&img),
                    Orientation::Rotate180 => imageops::rotate180(&img),
                    Orientation::Rotate270 => imageops::rotate270(&img),
                    _ => img.to_rgba8(),
                };

                img.save(output_path)
                    .map_err(|_| ExportError::ErrorWhenExportingFile(output_path.to_owned()))?;
            }
        }

        Ok(())
    }

    #[attrs::bench(demosaicing_with_postprocess)]
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

    pub fn export_exif_info_directly(input: Input) -> Result<String, RawFileReadingError> {
        let buffer = match input {
            Input::ByFile(file) => decode::get_buffer_from_file(file)?,
            Input::ByBuffer(buffer) => buffer,
        };

        decode::get_exif_info(&buffer)?
            .stringify_all()
            .map_err(|err| quickexif::parsed_info::Error::from(err).into())
    }

    #[attrs::bench(writing_file)]
    fn write_to_file<T: 'static + image::Primitive>(
        &self,
        path: &String,
        (data, width, height): (Vec<T>, usize, usize),
        quality: u8,
    ) -> Result<(), ExportError>
    where
        [T]: image::EncodableLayout,
    {
        let len = data.len();
        let mut image = ImageBuffer::<Rgb<T>, Vec<T>>::from_raw(width as u32, height as u32, data)
            .ok_or_else(|| {
                ExportError::ImageBufferError(stringify!(T).to_owned(), len, width, height)
            })?;

        let image = match (&self.raw_image.crop, self.output.auto_crop) {
            (Some(c), true) => imageops::crop(&mut image, c.x, c.y, c.width, c.height).to_image(),
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
            .map_err(|_| ExportError::ErrorWhenExportingFile(path.to_owned()))?;

        Ok(())
    }

    pub fn export_to_file(&self, quality: u8) -> Result<(), ExportError> {
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
