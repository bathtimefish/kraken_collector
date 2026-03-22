use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use serde_json::json;
use super::Collector;
use super::CollectorFactory;
use super::grpc;
use crate::config::CollectorCfg;
use bjig_controller::BjigController;
use tokio::time::sleep;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, serde::Serialize)]
struct MetaData {
    bjig: String,
}

#[derive(Debug, serde::Serialize)]
struct ReconnectResult {
    status: String,
    timestamp: String,
    steps: ReconnectSteps,
    message: String,
}

#[derive(Debug, serde::Serialize)]
struct ReconnectSteps {
    connect: String,
    stabilize: String,
}

pub struct Bjig {
    config: CollectorCfg,
}

pub struct BjigFactory {
    config: CollectorCfg,
}

impl BjigFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for BjigFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Bjig { config: self.config.clone() })
    }
}

impl Collector for Bjig {
    fn name(&self) -> &'static str {
        "bjig"
    }

    fn is_enable(&self) -> bool {
        self.config.bjig.enable
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let device_path = self.config.bjig.device_path.clone();
        let cli_bin_path = self.config.bjig.cli_bin_path.clone();
        let timeout_sec = self.config.bjig.data_timeout_sec;

        info!("Starting BraveJIG collector on device: {}", device_path);
        info!("Using bjig binary: {}", cli_bin_path);
        info!("Data timeout: {} seconds", timeout_sec);

        // Create BjigController
        let bjig = BjigController::new(&cli_bin_path)?
            .with_port(&device_path)
            .with_baud(115200);

        // Shared state for timeout monitoring
        let last_data_time = Arc::new(Mutex::new(Instant::now()));
        let last_data_time_clone = Arc::clone(&last_data_time);

        // Shared state for action processing (debounce + cooldown)
        let action_in_progress = Arc::new(AtomicBool::new(false));
        let last_action_time = Arc::new(Mutex::new(None::<Instant>));
        let action_cooldown_sec = self.config.bjig.action_cooldown_sec;

        // Create channel for action requests
        let (action_tx, mut action_rx) = mpsc::channel::<String>(10);

        // Clone config for potential future use in monitor restart
        let _device_path_action = device_path.clone();
        let _cli_bin_path_action = cli_bin_path.clone();

        // gRPC config clone
        let grpc_config = self.config.grpc.clone();

        // Clone for action processing task (before callback takes ownership)
        let action_in_progress_action = action_in_progress.clone();
        let last_action_time_action = last_action_time.clone();

        // Connect router and start monitor with callback and handle
        info!("Connecting router and starting monitor...");
        let handle = bjig.monitor()
            .connect_with_callback_and_handle(move |line| {
                // Clone for async block
                let last_data_time_update = last_data_time_clone.clone();
                let line_owned = line.to_string();
                let grpc_config_inner = grpc_config.clone();
                let action_in_progress_clone = action_in_progress.clone();
                let last_action_time_clone = last_action_time.clone();
                let cooldown_duration = Duration::from_secs(action_cooldown_sec);
                let action_tx_clone = action_tx.clone();

                tokio::spawn(async move {
                    // Update last data time
                    *last_data_time_update.lock().await = Instant::now();

                    // Send JSON data to gRPC
                    let metadata = MetaData {
                        bjig: "data".to_string(),
                    };
                    let meta_json = json!(metadata);
                    match grpc::send(
                        &grpc_config_inner,
                        "bjig",
                        "application/json",
                        &serde_json::to_string(&meta_json).unwrap(),
                        line_owned.as_bytes(),
                    )
                    .await
                    {
                        Ok(response) => {
                            debug!("Sent sensor data to gRPC");

                            // Check if response is for bjig action
                            let kraken_response = response.into_inner();
                            if kraken_response.collector_name == "bjig" && !kraken_response.payload.is_empty() {
                                warn!("Received bjig action command from broker");

                                // Check if action is already in progress (debounce)
                                if action_in_progress_clone.load(Ordering::SeqCst) {
                                    warn!("Action already in progress, ignoring this action request");
                                    return;
                                }

                                // Check cooldown period
                                let should_process = {
                                    let last_time_guard = last_action_time_clone.lock().await;
                                    match *last_time_guard {
                                        Some(last_time) => {
                                            let elapsed = last_time.elapsed();
                                            if elapsed < cooldown_duration {
                                                warn!("Action cooldown active ({:.1}s < {}s), ignoring this action request",
                                                    elapsed.as_secs_f64(), cooldown_duration.as_secs());
                                                false
                                            } else {
                                                true
                                            }
                                        }
                                        None => true,
                                    }
                                };

                                if !should_process {
                                    return;
                                }

                                // Set action in progress flag
                                action_in_progress_clone.store(true, Ordering::SeqCst);
                                info!("Processing bjig action (debounce + cooldown passed)");

                                // Parse response payload and send to action processing task
                                match String::from_utf8(kraken_response.payload.clone()) {
                                    Ok(payload_str) => {
                                        info!("Action payload: {}", payload_str);

                                        // Send action request to processing task
                                        if let Err(e) = action_tx_clone.try_send(payload_str) {
                                            error!("Failed to send action request to processing task: {:?}", e);
                                            // Clear flags on error
                                            *last_action_time_clone.lock().await = Some(Instant::now());
                                            action_in_progress_clone.store(false, Ordering::SeqCst);
                                        } else {
                                            // Flags will be cleared by action processing task
                                            info!("Action request queued for processing");
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to parse response payload as UTF-8: {:?}", e);
                                        // Clear flags on error
                                        *last_action_time_clone.lock().await = Some(Instant::now());
                                        action_in_progress_clone.store(false, Ordering::SeqCst);
                                    }
                                }
                            }
                        }
                        Err(e) => error!("Failed to send to gRPC: {:?}", e),
                    }
                });

                // Continue monitoring
                Ok(true)
            })
            .await?;

        // Clone handle for action processing task
        let handle_clone = Arc::new(Mutex::new(handle));
        let handle_action = handle_clone.clone();

        // Spawn action processing task
        tokio::spawn(async move {
            while let Some(payload_str) = action_rx.recv().await {
                info!("Action processing task received action request");

                // Pause monitor
                match handle_action.lock().await.pause().await {
                    Ok(_) => info!("Monitor paused for action processing"),
                    Err(e) => {
                        error!("Failed to pause monitor: {:?}", e);
                        // Clear flags and continue
                        *last_action_time_action.lock().await = Some(Instant::now());
                        action_in_progress_action.store(false, Ordering::SeqCst);
                        continue;
                    }
                }

                debug!("Action payload: {}", payload_str);

                // TODO: Process action based on payload
                // This will involve parsing the JSON payload and sending commands to the router
                // Example implementation:
                // match serde_json::from_str::<ActionCommand>(&payload_str) {
                //     Ok(action) => {
                //         // Execute router command based on action
                //         // let result = bjig.router().execute_action(action).await;
                //     }
                //     Err(e) => error!("Failed to parse action payload: {:?}", e),
                // }
                debug!("Action processing not yet implemented");

                // Resume monitor
                match handle_action.lock().await.resume().await {
                    Ok(_) => info!("Monitor resumed after action processing"),
                    Err(e) => error!("Failed to resume monitor: {:?}", e),
                }

                // Clear flags
                *last_action_time_action.lock().await = Some(Instant::now());
                action_in_progress_action.store(false, Ordering::SeqCst);
                info!("Action processing completed");
            }
        });

        // Spawn timeout monitoring task
        let last_data_time_monitor = last_data_time.clone();
        let device_path_monitor = device_path.clone();
        let cli_bin_path_monitor = cli_bin_path.clone();
        let grpc_config_monitor = self.config.grpc.clone();

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(10)).await;

                let elapsed = last_data_time_monitor.lock().await.elapsed();
                if elapsed > Duration::from_secs(timeout_sec) {
                    warn!("Data timeout detected ({} seconds), initiating router reconnect...", elapsed.as_secs());

                    // Execute reconnect flow
                    let reconnect_result = execute_reconnect_flow(
                        &cli_bin_path_monitor,
                        &device_path_monitor,
                    ).await;

                    // Send reconnect result to gRPC
                    let metadata = MetaData {
                        bjig: "alert".to_string(),
                    };
                    let meta_json = json!(metadata);

                    match grpc::send(
                        &grpc_config_monitor,
                        "bjig",
                        "application/json",
                        &serde_json::to_string(&meta_json).unwrap(),
                        &serde_json::to_vec(&reconnect_result).unwrap(),
                    )
                    .await
                    {
                        Ok(_) => info!("Sent reconnect result to gRPC"),
                        Err(e) => error!("Failed to send reconnect result to gRPC: {:?}", e),
                    }

                    // Reset last data time
                    *last_data_time_monitor.lock().await = Instant::now();
                }
            }
        });

        // Keep monitor running indefinitely
        info!("Monitor running...");

        // Keep the main task alive while monitor runs
        loop {
            sleep(Duration::from_secs(60)).await;
            if !handle_clone.lock().await.is_running() {
                error!("Monitor stopped unexpectedly");
                break;
            }
        }

        Ok(())
    }
}

/// Execute router reconnect flow
async fn execute_reconnect_flow(
    cli_bin_path: &str,
    device_path: &str,
) -> ReconnectResult {
    let mut steps = ReconnectSteps {
        connect: "pending".to_string(),
        stabilize: "pending".to_string(),
    };

    let bjig = match BjigController::new(cli_bin_path) {
        Ok(controller) => controller.with_port(device_path).with_baud(115200),
        Err(e) => {
            return ReconnectResult {
                status: "error".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                steps,
                message: format!("Failed to create BjigController: {}", e),
            };
        }
    };

    // Step 1: Connect router
    info!("Reconnect flow: Connecting router...");
    match bjig.router().connect().await {
        Ok(result) => {
            if result.is_success() {
                steps.connect = "success".to_string();
                info!("Router connect succeeded");
            } else {
                steps.connect = "error".to_string();
                return ReconnectResult {
                    status: "error".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    steps,
                    message: format!("Router connect failed: {}", result.message),
                };
            }
        }
        Err(e) => {
            steps.connect = "error".to_string();
            return ReconnectResult {
                status: "error".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                steps,
                message: format!("Failed to connect router: {}", e),
            };
        }
    }

    // Step 2: Wait for connection state to stabilize before reporting success
    info!("Reconnect flow: Waiting 3 seconds for connection to stabilize...");
    sleep(Duration::from_secs(3)).await;
    steps.stabilize = "success".to_string();

    ReconnectResult {
        status: "success".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        steps,
        message: "Router reconnected successfully".to_string(),
    }
}
