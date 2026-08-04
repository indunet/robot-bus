//! Wrap the AprilTag C detector and map hits to bus messages.

use anyhow::{bail, Context, Result};
use apriltag::{Detector, DetectorBuilder, Family};

use super::config::DetectorConfig;
use super::convert::image_to_gray;
use crate::apriltag_msgs::msg::v1::{AprilTagDetection, AprilTagDetectionArray, Point};
use crate::sensor_msgs::msg::v1::Image;

/// Owns a configured AprilTag detector plus the family name for outbound messages.
pub struct TagDetector {
    inner: Detector,
    family: String,
}

impl TagDetector {
    pub fn new(cfg: &DetectorConfig) -> Result<Self> {
        let family: Family = cfg
            .family
            .parse()
            .map_err(|e| anyhow::anyhow!("unknown AprilTag family {:?}: {e}", cfg.family))?;

        let mut inner = DetectorBuilder::new()
            .add_family_bits(family, cfg.bits_corrected as usize)
            .build()
            .context("build AprilTag detector")?;

        inner.set_decimation(cfg.decimate as f32);
        inner.set_sigma(cfg.blur as f32);
        inner.set_refine_edges(cfg.refine_edges);
        // Typo in upstream API: shapening == sharpening.
        inner.set_shapening(cfg.sharpening);
        inner.set_thread_number(cfg.threads as u8);

        Ok(Self {
            inner,
            family: cfg.family.clone(),
        })
    }

    /// Detect tags in `image` and build an `AprilTagDetectionArray` (header copied from input).
    pub fn detect_array(&mut self, image: &Image) -> Result<AprilTagDetectionArray> {
        let gray = image_to_gray(image)?;
        let hits = self.inner.detect(&gray);

        let detections = hits
            .iter()
            .map(|d| self.to_msg(d))
            .collect::<Result<Vec<_>>>()?;

        Ok(AprilTagDetectionArray {
            header: image.header.clone(),
            detections,
        })
    }

    fn to_msg(&self, d: &apriltag::Detection) -> Result<AprilTagDetection> {
        let center = d.center();
        let corners = d.corners();
        let h = d.homography();
        if h.nrows() != 3 || h.ncols() != 3 {
            bail!(
                "expected 3x3 homography, got {}x{}",
                h.nrows(),
                h.ncols()
            );
        }
        let homography = h.data().to_vec();
        if homography.len() != 9 {
            bail!("homography data len {} != 9", homography.len());
        }

        let id = i32::try_from(d.id()).context("tag id does not fit i32")?;
        let hamming = i32::try_from(d.hamming()).context("hamming does not fit i32")?;

        Ok(AprilTagDetection {
            family: self.family.clone(),
            id,
            hamming,
            // Upstream AprilTag no longer exposes a separate "goodness"; leave 0.
            goodness: 0.0,
            decision_margin: d.decision_margin(),
            centre: Some(Point {
                x: center[0],
                y: center[1],
            }),
            corners: corners
                .iter()
                .map(|c| Point { x: c[0], y: c[1] })
                .collect(),
            homography,
        })
    }
}
