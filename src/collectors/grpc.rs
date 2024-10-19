use tonic::Response;
use kraken::kraken_message_client::KrakenMessageClient;
use kraken::{ KrakenMessageRequest, KrakenMessageResponse };
use http::Uri;


use crate::config::GrpcCfg;

pub mod kraken {
  tonic::include_proto!("kraken");
}

pub async fn send(config: &GrpcCfg, payload: &str, provider: &str) -> Result<Response<KrakenMessageResponse>, Box<dyn std::error::Error>> {
  let dst = config.host.clone().parse::<Uri>().unwrap();
  let mut client = KrakenMessageClient::connect(dst).await?;
  let request = tonic::Request::new(KrakenMessageRequest {
    kind: "collector".to_string(),
    provider: provider.to_string(),
    payload: payload.to_string(),
  });
  let response = client.send(request).await?;
  Ok(response)
}