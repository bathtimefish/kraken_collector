use std::net::SocketAddr;
use bytes::{Buf, Bytes};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};

#[path = "./support/mod.rs"]
mod support;
use support::TokioIo;

use crate::config::{CollectorCfg, GrpcCfg};

use super::{Collector, CollectorFactory};
use super::grpc;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn post_webhook(req: Request<IncomingBody>, grpc_config: Arc<Mutex<GrpcCfg>>) -> Result<Response<BoxBody>, anyhow::Error> {
    let whole_body = req.collect().await?.aggregate();
    let body: serde_json::Value = serde_json::from_reader(whole_body.reader())?;
    debug!("POST /webhook: {}", &body);
    let json_bytes = serde_json::to_vec(&body)?;

    let grpc_config = grpc_config.lock().await;
    let sent = grpc::send(
        &*grpc_config,
        "webhook",
        "application/json",
        "{}",
        &json_bytes
    ).await;

    match sent {
        Ok(msg) => debug!("Sent message to grpc server: {:?}", msg),
        Err(msg) => error!("Failed to send to grpc: {:?}", msg),
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full(Bytes::from(r#"{"status": "POST_OK"}"#)))
        .unwrap();
    Ok(response)
}

async fn handle_request(req: Request<IncomingBody>, grpc_config: Arc<Mutex<GrpcCfg>>) -> Result<Response<BoxBody>, anyhow::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full("OK"))),
        (&Method::POST, "/webhook") => Ok(post_webhook(req, grpc_config.clone()).await.unwrap()),
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full("Not Found"))
                .unwrap();
            Ok(response)
        }
    }
}

pub struct Webhook {
    pub config: CollectorCfg,
}

pub struct WebhookFactory {
    pub config: CollectorCfg,
}

impl WebhookFactory {
    pub fn new(config: CollectorCfg) -> Self {
        Self { config }
    }
}

impl CollectorFactory for WebhookFactory {
    fn create(&self) -> Box<dyn Collector> {
        Box::new(Webhook{ config: self.config.clone() })
    }
}

impl Collector for Webhook {
    fn name(&self) -> &'static str {
        "webhook"
    }

    fn is_enable(&self) -> bool {
        self.config.webhook.enable
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let config = self.config.webhook.clone();
        let grpc_config = Arc::new(Mutex::new(self.config.grpc.clone()));  // Arcでラップ
        let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
        let listener = TcpListener::bind(&addr).await?;
        debug!("Webhook server is listening on http://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let grpc_config = grpc_config.clone();  // Arc をクローンして共有参照
            tokio::task::spawn(async move {
                let service = service_fn(
                    move |req| handle_request(req, grpc_config.clone())  // grpc_config をクローンして渡す
                );
                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    error!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}