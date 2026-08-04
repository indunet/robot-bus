//! Convert `sensor_msgs/Image` pixel buffers to AprilTag grayscale images.

use anyhow::{bail, Result};
use apriltag::Image as AprilImage;

use crate::sensor_msgs::msg::v1::Image;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelEncoding {
    Rgb8,
    Bgr8,
    Mono8,
}

impl PixelEncoding {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "rgb8" => Ok(Self::Rgb8),
            "bgr8" => Ok(Self::Bgr8),
            "mono8" | "8uc1" => Ok(Self::Mono8),
            other => bail!("unsupported image encoding {other:?}; expected rgb8, bgr8, or mono8"),
        }
    }

    fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgb8 | Self::Bgr8 => 3,
            Self::Mono8 => 1,
        }
    }
}

/// Build a single-channel AprilTag image from a ROS-style Image.
pub fn image_to_gray(image: &Image) -> Result<AprilImage> {
    let enc = PixelEncoding::parse(&image.encoding)?;
    let width = image.width as usize;
    let height = image.height as usize;
    if width == 0 || height == 0 {
        bail!("image width/height must be non-zero");
    }

    let bpp = enc.bytes_per_pixel();
    let min_step = width * bpp;
    let step = image.step as usize;
    if step < min_step {
        bail!(
            "image step {step} too small for width {width} encoding {}",
            image.encoding
        );
    }
    let expected = step * height;
    if image.data.len() < expected {
        bail!(
            "image data len {} < expected {} (step*height)",
            image.data.len(),
            expected
        );
    }

    // AprilTag prefers stride aligned to ~96 bytes; use width when already aligned.
    let mut gray = AprilImage::zeros_with_stride(width, height, width)
        .map_err(|e| anyhow::anyhow!("allocate AprilTag image: {e}"))?;
    let out_stride = gray.stride();
    let out = gray.as_slice_mut();

    match enc {
        PixelEncoding::Mono8 => {
            for row in 0..height {
                let src = &image.data[row * step..row * step + width];
                let dst = &mut out[row * out_stride..row * out_stride + width];
                dst.copy_from_slice(src);
            }
        }
        PixelEncoding::Rgb8 => {
            for row in 0..height {
                let src_off = row * step;
                let dst_off = row * out_stride;
                for col in 0..width {
                    let i = src_off + col * 3;
                    let r = image.data[i] as u16;
                    let g = image.data[i + 1] as u16;
                    let b = image.data[i + 2] as u16;
                    // ITU-R BT.601 luma.
                    out[dst_off + col] = ((77 * r + 150 * g + 29 * b) >> 8) as u8;
                }
            }
        }
        PixelEncoding::Bgr8 => {
            for row in 0..height {
                let src_off = row * step;
                let dst_off = row * out_stride;
                for col in 0..width {
                    let i = src_off + col * 3;
                    let b = image.data[i] as u16;
                    let g = image.data[i + 1] as u16;
                    let r = image.data[i + 2] as u16;
                    out[dst_off + col] = ((77 * r + 150 * g + 29 * b) >> 8) as u8;
                }
            }
        }
    }

    Ok(gray)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_encodings() {
        assert_eq!(PixelEncoding::parse("rgb8").unwrap(), PixelEncoding::Rgb8);
        assert_eq!(PixelEncoding::parse("BGR8").unwrap(), PixelEncoding::Bgr8);
        assert_eq!(PixelEncoding::parse("mono8").unwrap(), PixelEncoding::Mono8);
        assert!(PixelEncoding::parse("rgba8").is_err());
    }

    #[test]
    fn mono8_roundtrip_dims() {
        let image = Image {
            header: None,
            height: 2,
            width: 3,
            encoding: "mono8".into(),
            is_bigendian: false,
            step: 3,
            data: vec![1, 2, 3, 4, 5, 6],
        };
        let gray = image_to_gray(&image).unwrap();
        assert_eq!(gray.width(), 3);
        assert_eq!(gray.height(), 2);
        assert_eq!(&gray.as_slice()[..3], &[1, 2, 3]);
    }
}
