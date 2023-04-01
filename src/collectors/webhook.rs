use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, StatusCode, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::result::Result;

use crate::config::CollectorConfig;

use super::Collector;
use super::grpc;

async fn post_webhook(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await;
    let body = String::from_utf8(body_bytes.unwrap().to_vec()).unwrap();
    info!("POST /webhook: {}", &body);
    let sent = grpc::send(&body.as_str(), &"webhook").await;
    match sent {
        Ok(msg) => info!("Sent message to grpc server: {:?}", msg),
        Err(msg) => error!("Failed to send to grpc: {:?}", msg),
    }
    let mut response = Response::new(Body::from("{ \"status\": \"POST_OK\"}"));
    *response.status_mut() = StatusCode::OK;
    Ok(response)
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from("{ \"status\": \"OK\"}"))),
        (&Method::POST, "/webhook") => Ok(post_webhook(req).await.unwrap()),
        _ => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
        }
    }
}

pub struct Webhook {
    pub config: CollectorConfig,
}

impl Collector for Webhook {
    fn name(&self) -> &str {
        "webhook"
    }
    #[tokio::main(flavor = "current_thread")]
    async fn start(&self) -> Result<(), anyhow::Error> {
        let port = self.config.webhook.port;
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let make_svc = make_service_fn(|_| {
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    handle_request(req)
                }))
            }
        });
        let server = Server::bind(&addr).serve(make_svc);
        debug!("Webhook server was started that is listening on http://{}", addr);
        if let Err(e) = server.await {
            error!("Webhook server had an error: {}", e);
        }
        Ok(())
    }
}