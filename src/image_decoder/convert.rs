//! Convert decoded FFmpeg video frames to `sensor_msgs/Image` pixel buffers.

use anyhow::{bail, Result};
use ffmpeg_next::format;
use ffmpeg_next::frame::Video as VideoFrame;
use ffmpeg_next::software::scaling;

use super::config::OutputEncoding;
use crate::builtin_interfaces::msg::v1::Time;
use crate::sensor_msgs::msg::v1::Image;
use crate::std_msgs::msg::v1::Header;

impl OutputEncoding {
    fn ffmpeg_pixel(self) -> format::Pixel {
        match self {
            Self::Rgb8 => format::Pixel::RGB24,
            Self::Bgr8 => format::Pixel::BGR24,
        }
    }
}

/// Scale/convert a decoded frame to a packed Image (`rgb8` / `bgr8`).
pub fn frame_to_image(
    frame: &VideoFrame,
    encoding: OutputEncoding,
    frame_id: &str,
    stamp: Option<Time>,
) -> Result<Image> {
    let width = frame.width();
    let height = frame.height();
    if width == 0 || height == 0 {
        bail!("decoded frame size is zero");
    }

    let src_fmt = frame.format();
    if src_fmt == format::Pixel::None {
        bail!("decoded frame has unknown pixel format");
    }

    let dst_fmt = encoding.ffmpeg_pixel();
    let mut rgb = VideoFrame::new(dst_fmt, width, height);
    let mut context = scaling::Context::get(
        src_fmt,
        width,
        height,
        dst_fmt,
        width,
        height,
        scaling::Flags::BILINEAR,
    )?;
    context.run(frame, &mut rgb)?;

    let step = width.saturating_mul(3);
    let mut data = vec![0u8; (step as usize).saturating_mul(height as usize)];
    {
        let src_stride = rgb.stride(0);
        let src = rgb.data(0);
        let row_bytes = step as usize;
        for row in 0..height as usize {
            let src_off = row * src_stride;
            let dst_off = row * row_bytes;
            data[dst_off..dst_off + row_bytes]
                .copy_from_slice(&src[src_off..src_off + row_bytes]);
        }
    }

    Ok(Image {
        header: Some(Header {
            stamp,
            frame_id: frame_id.to_string(),
        }),
        height,
        width,
        encoding: encoding.as_str().into(),
        is_bigendian: false,
        step,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_encoding_ffmpeg_pixel() {
        assert_eq!(OutputEncoding::Rgb8.ffmpeg_pixel(), format::Pixel::RGB24);
        assert_eq!(OutputEncoding::Bgr8.ffmpeg_pixel(), format::Pixel::BGR24);
    }
}
