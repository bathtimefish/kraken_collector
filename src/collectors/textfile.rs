use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use serde_json::json;
use notify::{RecursiveMode, EventKind};
use notify_types::event::*;
use notify_debouncer_full::new_debouncer;
use anyhow::{Result, Context, anyhow, bail};
use super::Collector;
use super::CollectorFactory;
use super::grpc;
use crate::config::{CollectorCfg, GrpcCfg};

#[derive(Clone, Debug)]
struct TfcConfig {
    grpc: GrpcCfg,
    target_file_path: PathBuf,
    monitor_dir_path: PathBuf,
    monitoring_mode: MonitoringMode,
    interval_sec: u64,
    file_options: FileOptions,
    cleanup_options: CleanupOptions,
}

#[derive(Clone, Debug)]
enum MonitoringMode {
    TimeInterval,
    EventDriven,
}

#[derive(Clone, Debug)]
struct FileOptions {
    allow_create: bool,
    allow_modify: bool,
}

#[derive(Clone, Debug)]
struct CleanupOptions {
    remove_created_file_after_read: bool,
    remove_files_except_modified_after_read: bool,
    remove_all_files_after_read: bool,
    remove_all_folders: bool,
}

fn check_file_validity(file_path: &Path) -> Result<()> {
    if !file_path.exists() {
        bail!("Target file not found: {}", file_path.display());
    }
    if !file_path.is_file() {
        bail!("Input data was not file path: {}", file_path.display());
    }
    Ok(())
}

fn read_file_content(file_path: &Path) -> Result<String> {
    std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))
}

fn is_hidden(path: &Path) -> Result<bool> {
    #[cfg(unix)]
    {
        let file_name = path.file_name()
            .ok_or_else(|| anyhow!("Cannot get filename from path: {}", path.display()))?;
        Ok(file_name.to_string_lossy().starts_with('.'))
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Cannot get metadata for path: {}", path.display()))?;
        Ok((metadata.file_attributes() & 0x2) != 0)
    }
    #[cfg(not(any(unix, windows)))]
    {
        Ok(false)
    }
}

// ファイル操作関数
async fn read_file_from_path(path: &Path, event_type: &str, config: &TfcConfig) -> Result<(), anyhow::Error> {
    if is_hidden(path)? {
        debug!("Hidden file was not opened: {:?}", path);
        bail!("Hidden file");
    }
    
    let result = read_file_content(path)?;
    debug!("Event Type: {:?}", event_type);
    debug!("Content excerpt: {:.10}", result);

    // send to Kraken Broker
    let meta_json = json!({});
    let sent = grpc::send(
        &config.grpc,
        "textfile",
        "text/plain",
        &serde_json::to_string(&meta_json).unwrap(),
        &result.as_bytes(),
    ).await;

    match sent {
        Ok(_) => {
            debug!("File content sent to Kraken Broker");
            Ok(())
        },
        Err(e) => {
            Err(anyhow::Error::msg(format!("Failed to send file content: {}", e)))
        },
    }
}

enum CleanupStrategy {
    AllFiles,
    AllFolders,
    AllExcept(PathBuf),
    Everything,
}

fn clean_directory(folder_path: &Path, strategy: CleanupStrategy) -> Result<()> {
    use CleanupStrategy::*;
    
    fn normalize_path(path: &Path) -> Result<PathBuf> {
        std::fs::canonicalize(path)
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))
    }
    
    let normalized_except_path = match &strategy {
        AllExcept(except_path) => {
            let normalized = normalize_path(except_path)?;
            Some(normalized)
        },
        _ => None,
    };
    
    for entry in std::fs::read_dir(folder_path)
        .with_context(|| format!("Failed to read directory: {}", folder_path.display()))? {
        
        let entry = entry.with_context(|| "Failed to get directory entry")?;
        let path = entry.path();
        
        match &strategy {
            AllFiles => {
                if path.is_file() && !is_hidden(&path)? {
                    std::fs::remove_file(&path)
                        .with_context(|| format!("Failed to remove file: {}", path.display()))?;
                    debug!("Deleted file: {:?}", &path);
                }
            },
            AllFolders => {
                if path.is_dir() {
                    std::fs::remove_dir_all(&path)
                        .with_context(|| format!("Failed to remove folder: {}", path.display()))?;
                    debug!("Deleted folder: {:?}", &path);
                }
            },
            AllExcept(_) => {
                if path.is_file() && !is_hidden(&path)? {
                    let normalized_path = normalize_path(&path)?;
                    debug!("Compare normalize: {:?} vs {:?}", normalized_path, normalized_except_path.as_ref().unwrap());
                    
                    if normalized_path != *normalized_except_path.as_ref().unwrap() {
                        std::fs::remove_file(&path)
                            .with_context(|| format!("Failed to remove file: {}", path.display()))?;
                        debug!("Deleted file: {:?}", &path);
                    } else {
                        debug!("Keeped file: {:?}", &path);
                    }
                }
            },
            Everything => {
                if path.is_file() && !is_hidden(&path)? {
                    std::fs::remove_file(&path)
                        .with_context(|| format!("Failed to remove file: {}", path.display()))?;
                    debug!("Deleted file: {:?}", &path);
                } else if path.is_dir() {
                    std::fs::remove_dir_all(&path)
                        .with_context(|| format!("Failed to remove folder: {}", path.display()))?;
                    debug!("Deleted folder: {:?}", &path);
                }
            }
        }
    }
    Ok(())
}

// Dispatch event
async fn dispatch_event(config: &TfcConfig, path: &Path, event_type: &str) -> Result<()> {
    debug!("Processing event: {}", event_type);
    
    // Read file content
    let _ = read_file_from_path(path, event_type, config).await 
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    // Determine cleanup strategy 
    match event_type {
        "create" => {
            debug!("Created file has closed.");
            
            if config.cleanup_options.remove_all_files_after_read {
                debug!("Remove all files in {:?}", config.monitor_dir_path);
                
                thread::sleep(Duration::from_secs(1));
                
                clean_directory(
                    &config.monitor_dir_path,
                    if config.cleanup_options.remove_all_folders {
                        CleanupStrategy::Everything
                    } else {
                        CleanupStrategy::AllFiles
                    }
                )?;
            } else if config.cleanup_options.remove_created_file_after_read {
                // Delete created file after reading
                thread::sleep(Duration::from_secs(1));
                debug!("Remove created file after reading {:?}", path);
                std::fs::remove_file(path)
                    .with_context(|| format!("Failed to remove file: {}", path.display()))?;
            }
        },
        "modify" => {
            debug!("Modified file has closed.");
            if config.cleanup_options.remove_all_files_after_read {
                // Delete all files after reading
                thread::sleep(Duration::from_secs(1));
                debug!("Remove all files in {:?}", config.monitor_dir_path);
                
                clean_directory(
                    &config.monitor_dir_path,
                    if config.cleanup_options.remove_all_folders {
                        CleanupStrategy::Everything
                    } else {
                        CleanupStrategy::AllFiles
                    }
                )?;
            } else if config.cleanup_options.remove_files_except_modified_after_read {
                // Delete all files except the modified file
                debug!("Remove all files except modified file");
                clean_directory(
                    &config.monitor_dir_path,
                    CleanupStrategy::AllExcept(path.to_path_buf())
                )?;
            }
        },
        _ => {
            info!("Unknown event type: {}", event_type);
        }
    }
    
    Ok(())
}

// Monitor by time interval
fn monitor_by_time_interval(config: &TfcConfig) -> Result<()> {
    // Check if the target file is valid 
    check_file_validity(&config.target_file_path)
        .with_context(|| format!("Invalid target file: {}", config.target_file_path.display()))?;
        
    debug!("Start to monitor file by time interval: {}", config.target_file_path.display());
    debug!("Interval: {} seconds", config.interval_sec);
    
    // main loop 
    loop {
        match read_file_content(&config.target_file_path) {
            Ok(content) => {
                debug!("Detected file content:");
                debug!("---\n{}\n---", content);
            }
            Err(err) => {
                error!("Failed to read the file: {}", err);
            }
        }
        thread::sleep(Duration::from_secs(config.interval_sec));
    }
}

// Event-driven monitoring
fn monitor_by_dir_event(config: &TfcConfig) -> Result<()> {
    let mut current_event_type = "unknown".to_string();
    
    // Create a debouncer
    let (tx, rx) = std::sync::mpsc::channel();
    let debounce_interval = Duration::from_secs(config.interval_sec);
    
    let mut debouncer = new_debouncer(debounce_interval, None, tx)
        .with_context(|| "Failed to create file system debouncer")?;
        
    debouncer.watch(&config.monitor_dir_path, RecursiveMode::Recursive)
        .with_context(|| format!("Failed to watch directory: {}", config.monitor_dir_path.display()))?;
        
    debug!("Started event-driven monitoring for: {}", config.monitor_dir_path.display());
    
    // main loop
    for result in rx {
        match result {
            Ok(events) => events.iter().for_each(|event| {
                let kind = event.kind;
                let paths = event.paths.clone();
                
                match kind {
                    EventKind::Create(create_kind) => {
                        if !config.file_options.allow_create {
                            debug!("Create events not allowed by configuration");
                            return;
                        }
                        
                        match create_kind {
                            CreateKind::File => {
                                for path in &paths {
                                    if let Ok(false) = is_hidden(path) {
                                        debug!("Create File event detected");
                                        current_event_type = "create".to_string();
                                        debug!("Created file: {:?}", path);
                                        debug!("Event Type: {:?}", current_event_type);
                                        let _ = dispatch_event(config, path, &current_event_type); // ! async function
                                        current_event_type = "unknown".to_string();
                                    }
                                }
                            },
                            CreateKind::Folder => {
                                if config.cleanup_options.remove_all_folders {
                                    debug!("Remove all folders in {:?}", config.monitor_dir_path);
                                    if let Err(e) = clean_directory(&config.monitor_dir_path, CleanupStrategy::AllFolders) {
                                        error!("Failed to clean folders: {}", e);
                                    }
                                }
                            },
                            _ => {}
                        }
                    },
                    EventKind::Modify(modify_kind) => {
                        if !config.file_options.allow_modify {
                            debug!("Modify events not allowed by configuration");
                            return;
                        }
                        
                        match modify_kind {
                            ModifyKind::Data(_) => {
                                for path in &paths {
                                    if let Ok(false) = is_hidden(path) {
                                        debug!("Modify Data event detected");
                                        current_event_type = "modify".to_string();
                                        debug!("Modified file: {:?}", path);
                                        debug!("Event Type: {:?}", current_event_type);
                                        let _ = dispatch_event(config, path, &current_event_type); // ! async function
                                        current_event_type = "unknown".to_string();
                                    }
                                }
                            },
                            _ => {}
                        }
                    },
                    EventKind::Access(access_kind) => {
                        match access_kind {
                            AccessKind::Close(_) => {
                                thread::sleep(Duration::from_secs(1));
                                for path in &paths {
                                    if let Ok(false) = is_hidden(path) {
                                        let _ = dispatch_event(config, path, &current_event_type); // ! async function
                                        current_event_type = "unknown".to_string();
                                    }
                                }
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }),
            Err(errors) => errors.iter().for_each(|error| error!("Watch error: {:?}", error)),
        }
    }
    Ok(())
}

pub struct Textfile {
    config: CollectorCfg,
}

pub struct TextfileFactory {
    config: CollectorCfg,
}

impl TextfileFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for TextfileFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Textfile { config: self.config.clone() })
    }
}

impl Collector for Textfile {
    fn name(&self) -> &'static str {
        "textfile"
    }

    fn is_enable(&self) -> bool {
        self.config.text_file.enable
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let config = TfcConfig{
            grpc: self.config.grpc.clone(),
            target_file_path: PathBuf::from(self.config.text_file.target_file_path.clone()),
            monitor_dir_path: PathBuf::from(self.config.text_file.monitor_dir_path.clone()),
            monitoring_mode: if self.config.text_file.monitoring_mode == "time_interval" {
                MonitoringMode::TimeInterval
            } else {
                MonitoringMode::EventDriven
            },
            interval_sec: self.config.text_file.interval_sec,
            file_options: FileOptions {
                allow_create: self.config.text_file.allow_create,
                allow_modify: self.config.text_file.allow_modify,
            },
            cleanup_options: CleanupOptions {
                remove_created_file_after_read: self.config.text_file.remove_created,
                remove_files_except_modified_after_read: self.config.text_file.remove_except_modified,
                remove_all_files_after_read: self.config.text_file.remove_all_files,
                remove_all_folders: self.config.text_file.remove_all_folder,
            },
        };
        match config.monitoring_mode {
            MonitoringMode::TimeInterval => {
                monitor_by_time_interval(&config)
                    .with_context(|| "Failed to monitor by time interval")?;
            }
            MonitoringMode::EventDriven => {
                monitor_by_dir_event(&config)
                    .with_context(|| "Failed to monitor by directory event")?;
            }
        }
        Ok(())
    }
}