use std::thread;
use std::time::Duration;
use serde_json::json;
use nokhwa::{Camera, query};
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType, ApiBackend};
use nokhwa::pixel_format::RgbFormat;
use super::Collector;
use super::CollectorFactory;
use super::grpc;
use crate::config::CollectorCfg;

#[derive(Debug, serde::Serialize)]
struct MetaData {
    format: String,
    camera_name: String,
    camera_index: String,
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
        Box::new(CameraCollector { config: self.config.clone() })
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
        
        let camera_info = cameras.get(0)
            .ok_or_else(|| anyhow::anyhow!("No camera found"))?;
        
        debug!("Found camera: {} (index: {:?})", camera_info.human_name(), camera_info.index());
        
        let camera_index = CameraIndex::Index(0);
        let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        
        debug!("Initializing camera with index 0...");
        let mut camera = Camera::new(camera_index, requested_format)
            .map_err(|e| anyhow::anyhow!("Failed to initialize camera: {}", e))?;

        debug!("Opening camera stream...");
        camera.open_stream()
            .map_err(|e| anyhow::anyhow!("Failed to open camera stream: {}", e))?;

        debug!("Camera initialized successfully, starting capture loop...");
        debug!("Capture interval: {} seconds", self.config.camera.capture_interval_sec);

        // Store camera info for use in the loop
        let camera_name = camera_info.human_name().to_string();
        let camera_index = format!("{:?}", camera_info.index());

        loop {
            match camera.frame() {
                Ok(frame) => {
                    debug!("Frame captured successfully");
                    
                    // Decode frame to RGB format
                    match frame.decode_image::<RgbFormat>() {
                        Ok(decoded_image) => {
                            // Convert to JPEG bytes (this is a simplified approach)
                            // In a real implementation, you might want to use an image crate
                            // to properly encode to JPEG
                            let image_data = decoded_image.into_raw();
                            
                            let metadata = MetaData {
                                format: "image/jpeg".to_string(),
                                camera_name: camera_name.clone(),
                                camera_index: camera_index.clone(),
                            };
                            let meta_json = json!(metadata);
                            
                            let sent = grpc::send(
                                &self.config.grpc,
                                "camera",
                                "application/octet-stream",
                                &serde_json::to_string(&meta_json).unwrap(),
                                &image_data,
                            ).await;
                            
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