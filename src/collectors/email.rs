use std::sync::Arc;
use bytes::Bytes;
use mailparse::{parse_mail, MailHeaderMap, ParsedMail};
use serde::Serialize;
use mailin_embedded::{Handler, Server, response::*};

use crate::config::{CollectorCfg, EmailCfg};
use super::{Collector, CollectorFactory};
use super::grpc;

// Email payload structure
#[derive(Serialize)]
struct EmailPayload {
    ipaddr: String,
    from: String,
    to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cc: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bcc: Vec<String>,
    subject: String,
    body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    html_body: Option<String>,
    timestamp: String,
    message_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<Attachment>,
}

#[derive(Serialize)]
struct Attachment {
    name: String,
    mime_type: String,
    size: usize,
    #[serde(serialize_with = "serialize_base64")]
    data: Vec<u8>,
}

fn serialize_base64<S>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
    serializer.serialize_str(&encoded)
}

// Task structure for async processing
#[derive(Clone)]
struct EmailTask {
    ip: String,
    from: String,
    to: Vec<String>,
    data: Bytes,
}

// SMTP Handler implementation
#[derive(Clone)]
struct EmailHandler {
    tx: tokio::sync::broadcast::Sender<EmailTask>,
    config: Arc<EmailCfg>,
}

impl EmailHandler {
    fn new(tx: tokio::sync::broadcast::Sender<EmailTask>, config: Arc<EmailCfg>) -> Self {
        Self { tx, config }
    }

    fn check_allowed_sender(&self, from: &str) -> bool {
        if self.config.allowed_senders.is_empty() {
            return true;
        }
        self.config.allowed_senders.iter().any(|allowed| from.contains(allowed))
    }
}

impl Handler for EmailHandler {
    fn helo(&mut self, _ip: std::net::IpAddr, _domain: &str) -> Response {
        OK
    }

    fn mail(&mut self, _ip: std::net::IpAddr, _domain: &str, from: &str) -> Response {
        if !self.check_allowed_sender(from) {
            warn!("Rejected email from unauthorized sender: {}", from);
            return Response::custom(550, "Sender not allowed".to_string());
        }
        OK
    }

    fn rcpt(&mut self, _to: &str) -> Response {
        OK
    }

    fn data_start(&mut self, _domain: &str, _from: &str, _is8bit: bool, _to: &[String]) -> Response {
        START_DATA
    }

    fn data(&mut self, _buf: &[u8]) -> std::io::Result<()> {
        Ok(())
    }

    fn data_end(&mut self) -> Response {
        OK
    }
}

// Custom handler to intercept full message
#[derive(Clone)]
struct EmailHandlerWrapper {
    inner: EmailHandler,
    buffer: Arc<tokio::sync::Mutex<Vec<u8>>>,
    from: Arc<tokio::sync::Mutex<String>>,
    to: Arc<tokio::sync::Mutex<Vec<String>>>,
    ip: Arc<tokio::sync::Mutex<String>>,
}

impl EmailHandlerWrapper {
    fn new(tx: tokio::sync::broadcast::Sender<EmailTask>, config: Arc<EmailCfg>) -> Self {
        Self {
            inner: EmailHandler::new(tx, config),
            buffer: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            from: Arc::new(tokio::sync::Mutex::new(String::new())),
            to: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            ip: Arc::new(tokio::sync::Mutex::new(String::from("unknown"))),
        }
    }
}

impl Handler for EmailHandlerWrapper {
    fn helo(&mut self, ip: std::net::IpAddr, domain: &str) -> Response {
        *self.ip.blocking_lock() = ip.to_string();
        self.inner.helo(ip, domain)
    }

    fn mail(&mut self, ip: std::net::IpAddr, domain: &str, from: &str) -> Response {
        *self.from.blocking_lock() = from.to_string();
        self.inner.mail(ip, domain, from)
    }

    fn rcpt(&mut self, to: &str) -> Response {
        self.to.blocking_lock().push(to.to_string());
        self.inner.rcpt(to)
    }

    fn data_start(&mut self, domain: &str, from: &str, is8bit: bool, to: &[String]) -> Response {
        self.buffer.blocking_lock().clear();
        self.inner.data_start(domain, from, is8bit, to)
    }

    fn data(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.buffer.blocking_lock().extend_from_slice(buf);
        self.inner.data(buf)
    }

    fn data_end(&mut self) -> Response {
        let data = self.buffer.blocking_lock().clone();
        let from = self.from.blocking_lock().clone();
        let to = self.to.blocking_lock().clone();
        let ip = self.ip.blocking_lock().clone();

        // Check message size
        if data.len() > self.inner.config.max_message_size {
            warn!("Email exceeds max message size: {} bytes", data.len());
            return Response::custom(552, "Message size exceeds limit".to_string());
        }

        // Convert to Bytes
        let data_bytes = Bytes::from(data);

        // Create task
        let task = EmailTask {
            ip,
            from,
            to: to.clone(),
            data: data_bytes,
        };

        // Send to worker pool
        if let Err(e) = self.inner.tx.send(task) {
            error!("Failed to send email task to worker pool: {}", e);
            return Response::custom(451, "Temporary failure, please retry".to_string());
        }

        // Clear state for next message
        self.to.blocking_lock().clear();

        debug!("Email accepted");
        Response::custom(250, "OK".to_string())
    }
}

// Background worker for processing emails
async fn process_email_worker(
    mut rx: tokio::sync::broadcast::Receiver<EmailTask>,
    grpc_config: Arc<crate::config::GrpcCfg>,
    email_config: Arc<EmailCfg>,
) {
    while let Ok(task) = rx.recv().await {
        if let Err(e) = process_single_email(&task, &grpc_config, &email_config).await {
            error!("Failed to process email: {}", e);
        }
    }
}

// Process a single email
async fn process_single_email(
    task: &EmailTask,
    grpc_config: &crate::config::GrpcCfg,
    email_config: &EmailCfg,
) -> Result<(), anyhow::Error> {
    // Parse email
    let parsed = parse_mail(&task.data)?;

    // Extract metadata
    let timestamp = chrono::Utc::now().to_rfc3339();
    let message_id = parsed
        .headers
        .get_first_value("Message-ID")
        .unwrap_or_default();
    let subject = parsed
        .headers
        .get_first_value("Subject")
        .unwrap_or_default();

    // Extract body parts
    let (body, html_body) = extract_body_parts(&parsed);

    // Extract recipients
    let to = extract_addresses(&parsed, "To");
    let cc = extract_addresses(&parsed, "Cc");
    let bcc = extract_addresses(&parsed, "Bcc");

    // Extract attachments
    let attachments = extract_attachments(&parsed, email_config.max_attachment_size)?;

    // Build payload
    let payload = EmailPayload {
        ipaddr: task.ip.clone(),
        from: task.from.clone(),
        to: if !to.is_empty() { to } else { task.to.clone() },
        cc,
        bcc,
        subject,
        body,
        html_body,
        timestamp,
        message_id,
        attachments,
    };

    // Serialize to JSON
    let json_bytes = serde_json::to_vec(&payload)?;

    debug!("Sending email payload to gRPC (size: {} bytes)", json_bytes.len());

    // Send to gRPC
    match grpc::send(
        grpc_config,
        "email",
        "application/json",
        "{}",
        &json_bytes,
    )
    .await {
        Ok(_) => {
            debug!("Email successfully processed and sent to broker");
            Ok(())
        }
        Err(e) => {
            error!("Failed to send email to gRPC: {:?}", e);
            Err(anyhow::anyhow!("gRPC send failed: {:?}", e))
        }
    }
}

// Extract text and HTML body parts
fn extract_body_parts(parsed: &ParsedMail) -> (String, Option<String>) {
    let mut text_body = String::new();
    let mut html_body = None;

    if parsed.subparts.is_empty() {
        // Simple message without parts
        if let Ok(body) = parsed.get_body() {
            text_body = body;
        }
    } else {
        // Multipart message
        for part in &parsed.subparts {
            let content_type = &part.ctype.mimetype;
            if content_type == "text/plain" && text_body.is_empty() {
                if let Ok(body) = part.get_body() {
                    text_body = body;
                }
            } else if content_type == "text/html" && html_body.is_none() {
                if let Ok(body) = part.get_body() {
                    html_body = Some(body);
                }
            }
        }
    }

    (text_body, html_body)
}

// Extract email addresses from header
fn extract_addresses(parsed: &ParsedMail, header_name: &str) -> Vec<String> {
    parsed
        .headers
        .get_all_values(header_name)
        .into_iter()
        .flat_map(|addr_list| {
            addr_list
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .collect()
}

// Extract attachments with size limit
fn extract_attachments(
    parsed: &ParsedMail,
    max_size: usize,
) -> Result<Vec<Attachment>, anyhow::Error> {
    let mut attachments = Vec::new();

    debug!("Email has {} top-level subparts", parsed.subparts.len());

    for (i, part) in parsed.subparts.iter().enumerate() {
        debug!("Part {}: mimetype={}, subparts={}",
            i, part.ctype.mimetype, part.subparts.len());

        if let Some(attachment) = extract_single_attachment(part, max_size)? {
            attachments.push(attachment);
        }

        // Recursively check nested parts
        for nested_part in &part.subparts {
            if let Some(attachment) = extract_single_attachment(nested_part, max_size)? {
                attachments.push(attachment);
            }
        }
    }

    Ok(attachments)
}

fn extract_single_attachment(
    part: &ParsedMail,
    max_size: usize,
) -> Result<Option<Attachment>, anyhow::Error> {
    // Get filename from content-disposition
    let content_disposition = part.get_content_disposition();
    let filename = content_disposition
        .params
        .get("filename")
        .map(|s| s.as_str())
        .unwrap_or("");

    // Skip if no filename (likely inline text/html body, not attachment)
    if filename.is_empty() {
        return Ok(None);
    }

    let filename = filename.to_string();

    // Get raw body data
    let data = part.get_body_raw()?.to_vec();

    // Check size limit
    if data.len() > max_size {
        warn!(
            "Attachment '{}' exceeds max size ({} > {}), skipping",
            filename,
            data.len(),
            max_size
        );
        return Ok(None);
    }

    debug!(
        "Extracted attachment: {} ({} bytes, {})",
        filename,
        data.len(),
        part.ctype.mimetype
    );

    Ok(Some(Attachment {
        name: filename,
        mime_type: part.ctype.mimetype.clone(),
        size: data.len(),
        data,
    }))
}

// Email Collector
pub struct Email {
    config: CollectorCfg,
}

pub struct EmailFactory {
    config: CollectorCfg,
}

impl EmailFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for EmailFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Email {
            config: self.config.clone(),
        })
    }
}

impl Collector for Email {
    fn name(&self) -> &'static str {
        "email"
    }

    fn is_enable(&self) -> bool {
        self.config.email.enable
    }

    fn start(&self) -> Result<(), anyhow::Error> {
        let email_config = Arc::new(self.config.email.clone());
        let grpc_config = Arc::new(self.config.grpc.clone());

        // Create Tokio runtime for async tasks
        let rt = tokio::runtime::Runtime::new()?;

        // Create worker pool with broadcast channel
        let (tx, _rx) = tokio::sync::broadcast::channel(1000);

        // Spawn single worker thread (must be exactly 1 to avoid duplicate messages to broker)
        let rx_worker = tx.subscribe();
        let grpc_clone = grpc_config.clone();
        let email_clone = email_config.clone();

        rt.spawn(async move {
            debug!("Email worker thread started");
            process_email_worker(rx_worker, grpc_clone, email_clone).await;
        });

        // Create SMTP handler
        let handler = EmailHandlerWrapper::new(tx, email_config.clone());

        // Build server address
        let addr = format!("{}:{}", email_config.host_addr, email_config.smtp_port);

        // Create SMTP server
        let mut server = Server::new(handler);
        server.with_addr(&addr)
            .map_err(|e| anyhow::anyhow!("Failed to set SMTP listen address: {}", e))?;

        info!(
            "Email SMTP server starting on {} (max_size: {} bytes)",
            addr, email_config.max_message_size
        );

        // Start server (blocking) - mailin_embedded uses its own threading
        server.serve()
            .map_err(|e| anyhow::anyhow!("SMTP server error: {}", e))?;

        Ok(())
    }
}
