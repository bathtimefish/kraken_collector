use tonic::Response;
use kraken::kraken_service_client::KrakenServiceClient;
use kraken::{ KrakenRequest, KrakenResponse };
use http::Uri;


use crate::config::GrpcCfg;

pub mod kraken {
  tonic::include_proto!("kraken");
}

pub async fn send(config: &GrpcCfg, collector_name:&str, content_type:&str, metadata: &str, payload: &[u8]) -> Result<Response<KrakenResponse>, Box<dyn std::error::Error + Send + Sync>> {
  let dst = config.host.clone().parse::<Uri>().unwrap();
  let mut client = KrakenServiceClient::connect(dst).await?;
  let request = tonic::Request::new(KrakenRequest {
    collector_name: collector_name.to_string(),
    content_type: content_type.to_string(),
    metadata: metadata.to_string(),
    payload: payload.to_vec(),
  });
  let response = client.process_kraken_request(request).await?;
  Ok(response)
}
