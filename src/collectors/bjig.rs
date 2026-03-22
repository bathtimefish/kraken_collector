use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use serde_json::json;
use super::Collector;
use super::CollectorFactory;
use super::grpc;
use crate::config::CollectorCfg;
use bjig_controller::{BjigController, MonitorHandle};
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
        let handle = start_bjig_monitor(
            &bjig,
            true,
            last_data_time_clone.clone(),
            grpc_config.clone(),
            action_in_progress.clone(),
            last_action_time.clone(),
            action_cooldown_sec,
            action_tx.clone(),
        ).await?;

        // Clone handle for action processing task
        let handle_clone = Arc::new(Mutex::new(Some(handle)));
        let handle_action = handle_clone.clone();
        let reconnect_in_progress = Arc::new(AtomicBool::new(false));

        // Spawn action processing task
        tokio::spawn(async move {
            while let Some(payload_str) = action_rx.recv().await {
                info!("Action processing task received action request");

                // Pause monitor
                let pause_result = {
                    let guard = handle_action.lock().await;
                    match guard.as_ref() {
                        Some(handle) => handle.pause().await,
                        None => Err(anyhow::anyhow!("Monitor handle unavailable during action processing").into()),
                    }
                };
                match pause_result {
                    Ok(_) => info!("Monitor paused for action processing"),
                    Err(e) => {
                        error!("Failed to pause monitor: {:?}", e);
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
                let resume_result = {
                    let guard = handle_action.lock().await;
                    match guard.as_ref() {
                        Some(handle) => handle.resume().await,
                        None => Err(anyhow::anyhow!("Monitor handle unavailable during action resume").into()),
                    }
                };
                match resume_result {
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
        let handle_timeout = handle_clone.clone();
        let reconnect_flag = reconnect_in_progress.clone();
        let action_in_progress_timeout = action_in_progress.clone();
        let last_action_time_timeout = last_action_time.clone();
        let action_tx_timeout = action_tx.clone();

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(10)).await;

                if reconnect_flag.load(Ordering::SeqCst) {
                    continue;
                }

                let elapsed = last_data_time_monitor.lock().await.elapsed();
                if elapsed > Duration::from_secs(timeout_sec) {
                    warn!("Data timeout detected ({} seconds), initiating router reconnect...", elapsed.as_secs());
                    reconnect_flag.store(true, Ordering::SeqCst);

                    if let Some(handle) = handle_timeout.lock().await.take() {
                        match handle.stop().await {
                            Ok(_) => info!("Stopped monitor before router reconnect"),
                            Err(e) => error!("Failed to stop monitor before reconnect: {:?}", e),
                        }
                    }

                    // Execute reconnect flow
                    let reconnect_result = execute_reconnect_flow(
                        &cli_bin_path_monitor,
                        &device_path_monitor,
                    ).await;

                    if reconnect_result.status == "success" {
                        match BjigController::new(&cli_bin_path_monitor) {
                            Ok(controller) => {
                                let controller = controller
                                    .with_port(&device_path_monitor)
                                    .with_baud(115200);

                                match start_bjig_monitor(
                                    &controller,
                                    false,
                                    last_data_time_monitor.clone(),
                                    grpc_config_monitor.clone(),
                                    action_in_progress_timeout.clone(),
                                    last_action_time_timeout.clone(),
                                    action_cooldown_sec,
                                    action_tx_timeout.clone(),
                                ).await {
                                    Ok(new_handle) => {
                                        *last_data_time_monitor.lock().await = Instant::now();
                                        *handle_timeout.lock().await = Some(new_handle);
                                        info!("Monitor restarted after router reconnect");
                                    }
                                    Err(e) => {
                                        error!("Failed to restart monitor after reconnect: {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to create BjigController for monitor restart: {:?}", e);
                            }
                        }
                    }

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

                    reconnect_flag.store(false, Ordering::SeqCst);
                }
            }
        });

        // Keep monitor running indefinitely
        info!("Monitor running...");

        // Keep the main task alive while monitor runs
        loop {
            sleep(Duration::from_secs(60)).await;
            if reconnect_in_progress.load(Ordering::SeqCst) {
                continue;
            }

            let is_running = {
                let guard = handle_clone.lock().await;
                guard.as_ref().map(|handle| handle.is_running()).unwrap_or(false)
            };
            if !is_running {
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
    const ROUTER_REBOOT_WAIT_SECS: u64 = 10;

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

    // Step 1: Stop router
    info!("Reconnect flow: Stopping router before restart...");
    match bjig.router().stop().await {
        Ok(result) => {
            if result.is_success() {
                steps.connect = "success".to_string();
                info!("Router stop succeeded");
            } else {
                steps.connect = "error".to_string();
                return ReconnectResult {
                    status: "error".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    steps,
                    message: format!("Router stop failed: {}", result.message),
                };
            }
        }
        Err(e) => {
            steps.connect = "error".to_string();
            return ReconnectResult {
                status: "error".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                steps,
                message: format!("Failed to stop router: {}", e),
            };
        }
    }

    // Step 2: Wait for router reboot to complete before starting it again.
    info!(
        "Reconnect flow: Waiting {} seconds for router reboot to complete...",
        ROUTER_REBOOT_WAIT_SECS
    );
    sleep(Duration::from_secs(ROUTER_REBOOT_WAIT_SECS)).await;

    // Step 3: Start router again
    info!("Reconnect flow: Starting router after reboot wait...");
    match bjig.router().start().await {
        Ok(result) => {
            if result.is_success() {
                steps.stabilize = "success".to_string();
                info!("Router start succeeded");
            } else {
                steps.stabilize = "error".to_string();
                return ReconnectResult {
                    status: "error".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    steps,
                    message: format!("Router start failed: {}", result.message),
                };
            }
        }
        Err(e) => {
            steps.stabilize = "error".to_string();
            return ReconnectResult {
                status: "error".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                steps,
                message: format!("Failed to start router: {}", e),
            };
        }
    }

    ReconnectResult {
        status: "success".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        steps,
        message: "Router restarted successfully".to_string(),
    }
}

async fn start_bjig_monitor(
    bjig: &BjigController,
    connect_first: bool,
    last_data_time: Arc<Mutex<Instant>>,
    grpc_config: crate::config::GrpcCfg,
    action_in_progress: Arc<AtomicBool>,
    last_action_time: Arc<Mutex<Option<Instant>>>,
    action_cooldown_sec: u64,
    action_tx: mpsc::Sender<String>,
) -> Result<MonitorHandle, anyhow::Error> {
    let callback = move |line: &str| {
            let last_data_time_update = last_data_time.clone();
            let line_owned = line.to_string();
            let grpc_config_inner = grpc_config.clone();
            let action_in_progress_clone = action_in_progress.clone();
            let last_action_time_clone = last_action_time.clone();
            let cooldown_duration = Duration::from_secs(action_cooldown_sec);
            let action_tx_clone = action_tx.clone();

            tokio::spawn(async move {
                *last_data_time_update.lock().await = Instant::now();

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

                        let kraken_response = response.into_inner();
                        if kraken_response.collector_name == "bjig" && !kraken_response.payload.is_empty() {
                            warn!("Received bjig action command from broker");

                            if action_in_progress_clone.load(Ordering::SeqCst) {
                                warn!("Action already in progress, ignoring this action request");
                                return;
                            }

                            let should_process = {
                                let last_time_guard = last_action_time_clone.lock().await;
                                match *last_time_guard {
                                    Some(last_time) => {
                                        let elapsed = last_time.elapsed();
                                        if elapsed < cooldown_duration {
                                            warn!(
                                                "Action cooldown active ({:.1}s < {}s), ignoring this action request",
                                                elapsed.as_secs_f64(),
                                                cooldown_duration.as_secs()
                                            );
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

                            action_in_progress_clone.store(true, Ordering::SeqCst);
                            info!("Processing bjig action (debounce + cooldown passed)");

                            match String::from_utf8(kraken_response.payload.clone()) {
                                Ok(payload_str) => {
                                    info!("Action payload: {}", payload_str);

                                    if let Err(e) = action_tx_clone.try_send(payload_str) {
                                        error!("Failed to send action request to processing task: {:?}", e);
                                        *last_action_time_clone.lock().await = Some(Instant::now());
                                        action_in_progress_clone.store(false, Ordering::SeqCst);
                                    } else {
                                        info!("Action request queued for processing");
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to parse response payload as UTF-8: {:?}", e);
                                    *last_action_time_clone.lock().await = Some(Instant::now());
                                    action_in_progress_clone.store(false, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                    Err(e) => error!("Failed to send to gRPC: {:?}", e),
                }
            });

            Ok(true)
        };

    let handle = if connect_first {
        bjig.monitor().connect_with_callback_and_handle(callback).await?
    } else {
        bjig.monitor().start_with_callback_and_handle(callback).await?
    };

    Ok(handle)
}
