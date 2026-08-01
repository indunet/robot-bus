//! Convert `sensor_msgs/Image` pixel buffers to planar YUV420.

use anyhow::{bail, Result};
use ffmpeg_next::format;
use ffmpeg_next::frame::Video as VideoFrame;
use ffmpeg_next::software::scaling;

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

    fn ffmpeg_pixel(self) -> format::Pixel {
        match self {
            Self::Rgb8 => format::Pixel::RGB24,
            Self::Bgr8 => format::Pixel::BGR24,
            Self::Mono8 => format::Pixel::GRAY8,
        }
    }

    fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgb8 | Self::Bgr8 => 3,
            Self::Mono8 => 1,
        }
    }
}

/// Build a YUV420P frame from a ROS-style Image (optionally scaling to `out_w`×`out_h`).
pub fn image_to_yuv420(image: &Image, out_w: u32, out_h: u32) -> Result<VideoFrame> {
    let enc = PixelEncoding::parse(&image.encoding)?;
    let width = image.width;
    let height = image.height;
    if width == 0 || height == 0 {
        bail!("image width/height must be non-zero");
    }

    let bpp = enc.bytes_per_pixel();
    let min_step = width as usize * bpp;
    let step = image.step as usize;
    if step < min_step {
        bail!(
            "image step {step} too small for width {width} encoding {}",
            image.encoding
        );
    }
    let expected = step * height as usize;
    if image.data.len() < expected {
        bail!(
            "image data len {} < expected {} (step*height)",
            image.data.len(),
            expected
        );
    }

    let mut src = VideoFrame::new(enc.ffmpeg_pixel(), width, height);
    {
        let dst_stride = src.stride(0);
        let dst = src.data_mut(0);
        for row in 0..height as usize {
            let src_off = row * step;
            let dst_off = row * dst_stride;
            dst[dst_off..dst_off + min_step]
                .copy_from_slice(&image.data[src_off..src_off + min_step]);
        }
    }

    let mut yuv = VideoFrame::new(format::Pixel::YUV420P, out_w, out_h);
    let mut context = scaling::Context::get(
        enc.ffmpeg_pixel(),
        width,
        height,
        format::Pixel::YUV420P,
        out_w,
        out_h,
        scaling::Flags::BILINEAR,
    )?;
    context.run(&src, &mut yuv)?;
    Ok(yuv)
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
}
