use tonic::{transport::Server, Request, Response, Status};
use kraken::kraken_service_server::{KrakenService, KrakenServiceServer};
use kraken::{KrakenRequest, KrakenResponse};

pub mod kraken {
  tonic::include_proto!("kraken");
}

#[derive(Debug, Default)]
pub struct KrakenBroker {}

#[tonic::async_trait]
impl KrakenService for KrakenBroker {
  async fn process_kraken_request(
    &self,
    request: Request<KrakenRequest>,
  ) -> Result<Response<KrakenResponse>, Status> {
    println!("Got a request: {:?}", request);
    let payload = request.get_ref().payload.clone();
    println!("{:?}", payload);
    let reply = kraken::KrakenResponse {
      collector_name: "example_collector".to_string(),
      content_type: "example_type".to_string(),
      metadata: "example_metadata".to_string(),
      payload,
    };
    Ok(Response::new(reply))
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let addr = "[::1]:50051".parse()?;
  let broker = KrakenBroker::default();
  Server::builder()
    .add_service(KrakenServiceServer::new(broker))
    .serve(addr)
    .await?;
  Ok(())
}