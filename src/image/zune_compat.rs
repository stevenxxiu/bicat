/// Compatibility layer supporting both zune-image (fast) and image crate (fallback)
use {
    crate::errors::ProgramError,
    std::path::Path,
    zune_core::colorspace::ColorSpace,
};

impl From<zune_image::errors::ImageErrors> for ProgramError {
    fn from(err: zune_image::errors::ImageErrors) -> Self {
        ProgramError::ImageError {
            details: err.to_string(),
        }
    }
}

/// Image type that uses either zune-image (fast) or image crate (fallback)
#[derive(Clone)]
pub enum DynamicImage {
    /// Fast decoder (zune-image) - used for JPEG, PNG, BMP, etc.
    Zune(zune_image::image::Image),
}

impl DynamicImage {
    pub fn from_path(path: &Path) -> Result<Self, ProgramError> {
        let img = zune_image::image::Image::open(path)?;
        let nb_components = img.colorspace().num_components();
        if nb_components < 3 {
            // Current implementation of the module requires an RGB image and
            // zune panics if we try to convert an image with less than 3 channels to RGB
            // (when calling Frame::flatten)
            return Err(ProgramError::ImageError {
                details: format!(
                    "Unsupported color space with {nb_components} components in image: {path:?}"
                ),
            });
        }
        Ok(Self::Zune(img))
    }

    pub fn from_rgba8(
        width: u32,
        height: u32,
        data: Vec<u8>,
    ) -> Result<Self, ProgramError> {
        let expected_len = (width as usize) * (height as usize) * 4;
        if data.len() != expected_len {
            return Err(ProgramError::Internal {
                details: format!(
                    "Invalid RGBA data length: expected {}, got {}",
                    expected_len,
                    data.len()
                ),
            });
        }

        let img = zune_image::image::Image::from_u8(
            &data,
            width as usize,
            height as usize,
            ColorSpace::RGBA,
        );
        Ok(Self::Zune(img))
    }

    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::Zune(img) => {
                let dims = img.dimensions();
                (dims.0 as u32, dims.1 as u32)
            }
        }
    }

    pub fn resize(
        &self,
        max_width: u32,
        max_height: u32,
    ) -> Result<Self, ProgramError> {
        match self {
            Self::Zune(img) => {
                let (width, height) = self.dimensions();

                if width <= max_width && height <= max_height {
                    return Ok(self.clone());
                }

                // Calculate new dimensions maintaining aspect ratio
                let ratio =
                    (max_width as f32 / width as f32).min(max_height as f32 / height as f32);
                let new_width = (width as f32 * ratio) as usize;
                let new_height = (height as f32 * ratio) as usize;

                // Use bilinear resize
                let colorspace = img.colorspace();
                let frames = img.frames_ref();
                if let Some(frame) = frames.first() {
                    let src_data: Vec<u8> = frame.flatten(colorspace);
                    let dst_data = resize_bilinear(
                        &src_data,
                        width as usize,
                        height as usize,
                        new_width,
                        new_height,
                        colorspace.num_components(),
                    );

                    let img = zune_image::image::Image::from_u8(
                        &dst_data, new_width, new_height, colorspace,
                    );
                    Ok(Self::Zune(img))
                } else {
                    Ok(self.clone())
                }
            }
        }
    }

    pub fn as_rgb8(&self) -> Option<RgbImage> {
        match self {
            Self::Zune(img) => {
                if img.colorspace() == ColorSpace::RGB {
                    Some(RgbImage::Zune(img.clone()))
                } else {
                    None
                }
            }
        }
    }

    pub fn as_rgba8(&self) -> Option<RgbaImage> {
        match self {
            Self::Zune(img) => {
                if img.colorspace() == ColorSpace::RGBA {
                    Some(RgbaImage::Zune(img.clone()))
                } else {
                    None
                }
            }
        }
    }

    pub fn to_rgb8(&self) -> RgbImage {
        match self {
            Self::Zune(img) => {
                let mut img = img.clone();

                // Convert to RGB if needed
                if img.colorspace() != ColorSpace::RGB {
                    let frames = img.frames_ref();
                    if let Some(frame) = frames.first() {
                        // beware that zune panics on next line if the image has less than 3 channels
                        let data: Vec<u8> = frame.flatten(ColorSpace::RGB);
                        let (w, h) = img.dimensions();
                        img = zune_image::image::Image::from_u8(&data, w, h, ColorSpace::RGB);
                    }
                }

                RgbImage::Zune(img)
            }
        }
    }
}

fn resize_bilinear(
    src: &[u8],
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize,
    channels: usize,
) -> Vec<u8> {
    let mut dst = vec![0u8; dst_width * dst_height * channels];

    let x_ratio = (src_width - 1) as f32 / dst_width.max(1) as f32;
    let y_ratio = (src_height - 1) as f32 / dst_height.max(1) as f32;

    for dst_y in 0..dst_height {
        for dst_x in 0..dst_width {
            // Calculate the source coordinates (floating point)
            let src_x_f = dst_x as f32 * x_ratio;
            let src_y_f = dst_y as f32 * y_ratio;

            // Get the four surrounding pixels
            let x0 = src_x_f.floor() as usize;
            let y0 = src_y_f.floor() as usize;
            let x1 = (x0 + 1).min(src_width - 1);
            let y1 = (y0 + 1).min(src_height - 1);

            // Calculate the fractional parts (weights)
            let x_frac = src_x_f - x0 as f32;
            let y_frac = src_y_f - y0 as f32;

            // Bilinear interpolation weights
            let w00 = (1.0 - x_frac) * (1.0 - y_frac);
            let w10 = x_frac * (1.0 - y_frac);
            let w01 = (1.0 - x_frac) * y_frac;
            let w11 = x_frac * y_frac;

            // Calculate pixel indices
            let idx00 = (y0 * src_width + x0) * channels;
            let idx10 = (y0 * src_width + x1) * channels;
            let idx01 = (y1 * src_width + x0) * channels;
            let idx11 = (y1 * src_width + x1) * channels;

            let dst_idx = (dst_y * dst_width + dst_x) * channels;

            // Interpolate each channel
            for c in 0..channels {
                let p00 = src[idx00 + c] as f32;
                let p10 = src[idx10 + c] as f32;
                let p01 = src[idx01 + c] as f32;
                let p11 = src[idx11 + c] as f32;

                let value = p00 * w00 + p10 * w10 + p01 * w01 + p11 * w11;
                dst[dst_idx + c] = value.round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    dst
}

pub enum RgbImage {
    Zune(zune_image::image::Image),
}

impl RgbImage {
    pub fn as_raw(&self) -> Vec<u8> {
        match self {
            Self::Zune(img) => {
                let frames = img.frames_ref();
                if let Some(frame) = frames.first() {
                    frame.flatten(ColorSpace::RGB)
                } else {
                    Vec::new()
                }
            }
        }
    }
}

pub enum RgbaImage {
    Zune(zune_image::image::Image),
}

impl RgbaImage {
    pub fn as_raw(&self) -> Vec<u8> {
        match self {
            Self::Zune(img) => {
                let frames = img.frames_ref();
                if let Some(frame) = frames.first() {
                    frame.flatten(ColorSpace::RGBA)
                } else {
                    Vec::new()
                }
            }
        }
    }
}
