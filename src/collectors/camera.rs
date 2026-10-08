use super::grpc;
use super::Collector;
use super::CollectorFactory;
use crate::config::CollectorCfg;
use image::codecs::jpeg::JpegEncoder;
use image::ExtendedColorType;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::{query, Camera};
use serde_json::json;
use std::thread;
use std::time::Duration;

/// Content type of the frames sent to the broker.
const FRAME_FORMAT: &str = "image/jpeg";

#[derive(Debug, serde::Serialize)]
struct MetaData {
    format: String,
    camera_name: String,
    width: u32,
    height: u32,
}

/// Encodes an RGB24 frame (3 bytes per pixel, row-major) as JPEG.
fn encode_jpeg(rgb: &[u8], width: u32, height: u32, quality: u8) -> anyhow::Result<Vec<u8>> {
    // the encoder panics on a buffer of the wrong size, so check it first
    let expected = width as usize * height as usize * 3;
    if rgb.len() != expected {
        anyhow::bail!(
            "the frame is {} bytes, expected {}x{}x3 = {}",
            rgb.len(),
            width,
            height,
            expected
        );
    }
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, quality.clamp(1, 100)).encode(
        rgb,
        width,
        height,
        ExtendedColorType::Rgb8,
    )?;
    Ok(out)
}

/// The name sent as `camera_name`: `KRKNC_CAMERA_NAME` if set, otherwise the camera's own name.
fn camera_name(configured: Option<&str>, detected: &str) -> String {
    match configured.map(str::trim).filter(|n| !n.is_empty()) {
        Some(name) => name.to_string(),
        None => detected.to_string(),
    }
}

pub struct CameraCollector {
    config: CollectorCfg,
}

pub struct CameraFactory {
    config: CollectorCfg,
}

impl CameraFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for CameraFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(CameraCollector {
            config: self.config.clone(),
        })
    }
}

impl Collector for CameraCollector {
    fn name(&self) -> &'static str {
        "camera"
    }

    fn is_enable(&self) -> bool {
        self.config.camera.enable
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        // Query available cameras to get camera info
        debug!("Querying available cameras...");
        let cameras = query(ApiBackend::Auto)
            .map_err(|e| anyhow::anyhow!("Failed to query cameras: {}", e))?;

        let camera_info = cameras
            .first()
            .ok_or_else(|| anyhow::anyhow!("No camera found"))?;

        debug!(
            "Found camera: {} (index: {:?})",
            camera_info.human_name(),
            camera_info.index()
        );

        // Open the same camera whose information was queried above
        let camera_index: CameraIndex = camera_info.index().clone();
        let requested_format =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        debug!("Initializing camera with index {:?}...", camera_index);
        let mut camera = Camera::new(camera_index, requested_format)
            .map_err(|e| anyhow::anyhow!("Failed to initialize camera: {}", e))?;

        debug!("Opening camera stream...");
        camera
            .open_stream()
            .map_err(|e| anyhow::anyhow!("Failed to open camera stream: {}", e))?;

        // Get camera format information after stream is opened
        let camera_format = camera.camera_format();

        let width = camera_format.width();
        let height = camera_format.height();

        debug!("Camera format: {}x{}", width, height);
        debug!("Camera initialized successfully, starting capture loop...");
        debug!(
            "Capture interval: {} seconds",
            self.config.camera.capture_interval_sec
        );

        // Store camera info for use in the loop
        let camera_name = camera_name(
            self.config.camera.name.as_deref(),
            &camera_info.human_name(),
        );
        let quality = self.config.camera.jpeg_quality;
        debug!("Camera name: {}, JPEG quality: {}", camera_name, quality);

        loop {
            // Skip buffered frames to get the most recent frame
            debug!("Capturing fresh frame...");
            for i in 0..3 {
                match camera.frame() {
                    Ok(_) => debug!("Skipped buffered frame {}", i + 1),
                    Err(e) => debug!("Error skipping frame {}: {}", i + 1, e),
                }
                // Small delay between frame reads
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            // Get the actual frame to process
            match camera.frame() {
                Ok(frame) => {
                    debug!("Fresh frame captured successfully");

                    // Decode frame to RGB format
                    match frame.decode_image::<RgbFormat>() {
                        Ok(decoded_image) => {
                            // The size of the decoded frame (it can differ from the requested format)
                            let (frame_width, frame_height) = decoded_image.dimensions();
                            let jpeg = match encode_jpeg(
                                decoded_image.as_raw(),
                                frame_width,
                                frame_height,
                                quality,
                            ) {
                                Ok(jpeg) => jpeg,
                                Err(e) => {
                                    error!("Failed to encode camera frame as JPEG: {}", e);
                                    thread::sleep(Duration::from_secs(
                                        self.config.camera.capture_interval_sec,
                                    ));
                                    continue;
                                }
                            };

                            let metadata = MetaData {
                                format: FRAME_FORMAT.to_string(),
                                camera_name: camera_name.clone(),
                                width: frame_width,
                                height: frame_height,
                            };
                            let meta_json = json!(metadata);

                            let sent = grpc::send(
                                &self.config.grpc,
                                "camera",
                                FRAME_FORMAT,
                                &serde_json::to_string(&meta_json).unwrap(),
                                &jpeg,
                            )
                            .await;

                            match sent {
                                Ok(_) => debug!("Camera frame sent to grpc server"),
                                Err(e) => error!("Failed to send camera frame to grpc: {:?}", e),
                            }
                        }
                        Err(e) => {
                            error!("Failed to decode camera frame: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to capture camera frame: {}", e);
                }
            }

            thread::sleep(Duration::from_secs(self.config.camera.capture_interval_sec));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(width: u32, height: u32) -> Vec<u8> {
        let mut rgb = Vec::with_capacity((width * height * 3) as usize);
        for y in 0..height {
            for x in 0..width {
                rgb.extend_from_slice(&[(x * 255 / width) as u8, (y * 255 / height) as u8, 128]);
            }
        }
        rgb
    }

    #[test]
    fn frames_are_encoded_as_jpeg_of_the_same_size() {
        let rgb = gradient(64, 48);
        let jpeg = encode_jpeg(&rgb, 64, 48, 85).unwrap();
        assert_eq!(&jpeg[..3], &[0xff, 0xd8, 0xff]);
        assert!(jpeg.len() < rgb.len());
        let decoded = image::load_from_memory(&jpeg).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (64, 48));
    }

    #[test]
    fn a_lower_quality_gives_a_smaller_jpeg() {
        let rgb = gradient(320, 240);
        let high = encode_jpeg(&rgb, 320, 240, 95).unwrap();
        let low = encode_jpeg(&rgb, 320, 240, 30).unwrap();
        assert!(low.len() < high.len());
        // out-of-range qualities are clamped instead of failing
        assert!(encode_jpeg(&rgb, 320, 240, 0).is_ok());
    }

    #[test]
    fn a_wrong_buffer_size_is_an_error() {
        assert!(encode_jpeg(&[0; 10], 64, 48, 85).is_err());
    }

    #[test]
    fn the_configured_name_replaces_the_detected_one() {
        assert_eq!(camera_name(Some("dock"), "USB Camera"), "dock");
        assert_eq!(camera_name(Some("  "), "USB Camera"), "USB Camera");
        assert_eq!(camera_name(None, "USB Camera"), "USB Camera");
    }
}
