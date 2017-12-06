//! Reftest image helper library.
#![warn(missing_docs)]

extern crate base64;
extern crate image;

use image::GenericImage;
use std::cmp;


/// A reference image for tests.
pub struct ReftestImage {
    /// The RGBA8 byte data of the image.
    pub data: Vec<u8>,
    /// Width and height of the image.
    pub size: (u32, u32),
}

impl From<image::DynamicImage> for ReftestImage {
    fn from(image: image::DynamicImage) -> Self {
        let size = image.dimensions();
        ReftestImage {
            data: image.to_rgba().into_raw(),
            size,
        }
    }
}

/// Comparison result
pub enum ReftestImageComparison {
    /// Images match perfectly.
    Equal,
    /// Images don't match.
    NotEqual {
        /// Maximum difference in the pixel value.
        max_difference: usize,
        /// Count of different pixels.
        count_different: usize,
    },
}

impl ReftestImage {
    /// Compare this reftest image with another one.
    pub fn compare(&self, other: &ReftestImage) -> ReftestImageComparison {
        assert_eq!(self.size, other.size);
        assert_eq!(self.data.len(), other.data.len());
        assert_eq!(self.data.len() % 4, 0);

        let mut count = 0;
        let mut max = 0;

        for (a, b) in self.data.chunks(4).zip(other.data.chunks(4)) {
            if a != b {
                let pixel_max = a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| (*x as isize - *y as isize).abs() as usize)
                    .max()
                    .unwrap();

                count += 1;
                max = cmp::max(max, pixel_max);
            }
        }

        if count != 0 {
            ReftestImageComparison::NotEqual {
                max_difference: max,
                count_different: count,
            }
        } else {
            ReftestImageComparison::Equal
        }
    }

    /// Convert this image into a base64 data URI.
    pub fn into_data_uri(self) -> String {
        let (width, height) = self.size;

        let mut buffer = image::RgbaImage
            ::from_raw(width, height, self.data)
            .expect("bug: unable to construct image buffer");

        // flip image vertically (texture is upside down)
        buffer = image::imageops::flip_vertical(&buffer);

        let mut png: Vec<u8> = vec![];
        image::png::PNGEncoder::new(&mut png)
            .encode(&buffer, width, height, image::ColorType::RGBA(8))
            .expect("Unable to encode PNG!");

        let png_base64 = base64::encode(&png);
        format!("data:image/png;base64,{}", png_base64)
    }
}
