use tonic::{transport::Server, Request, Response, Status};
use kraken::kraken_message_server::{KrakenMessage, KrakenMessageServer};
use kraken::{KrakenMessageRequest, KrakenMessageResponse};

pub mod kraken {
  tonic::include_proto!("kraken");
}

#[derive(Debug, Default)]
pub struct KrakenBroker {}

#[tonic::async_trait]
impl KrakenMessage for KrakenBroker {
  async fn send(
    &self,
    request: Request<KrakenMessageRequest>,
  ) -> Result<Response<KrakenMessageResponse>, Status> {
    println!("Got a request: {:?}", request);
    println!("{:?}", request.into_inner().payload);
    let reply = kraken::KrakenMessageResponse {
      status: 1,
    };
    Ok(Response::new(reply))
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let addr = "[::1]:50051".parse()?;
  let broker = KrakenBroker::default();
  Server::builder()
    .add_service(KrakenMessageServer::new(broker))
    .serve(addr)
    .await?;
  Ok(())
}