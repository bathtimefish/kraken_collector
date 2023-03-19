use tonic::Response;
use kraken::kraken_message_client::KrakenMessageClient;
use kraken::{ KrakenMessageRequest, KrakenMessageResponse };

pub mod kraken {
  tonic::include_proto!("kraken");
}

pub async fn send(s: &str, provider: &str) -> Result<Response<KrakenMessageResponse>, Box<dyn std::error::Error>> {
  let mut client = KrakenMessageClient::connect("http://[::1]:50051").await?;
  let request = tonic::Request::new(KrakenMessageRequest {
    kind: "collector".to_string(),
    provider: provider.to_string(),
    payload: s.to_string(),
  });
  let response = client.send(request).await?;
  Ok(response)
}