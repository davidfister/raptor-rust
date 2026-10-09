use std::sync::Arc;

use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

use rust_raptor::{
    generate_network,
    load_schedule,
    raptor,
    RaptorNetwork,
};

pub mod routing {
    tonic::include_proto!("routing");
}

use routing::routing_service_server::{
    RoutingService,
    RoutingServiceServer,
};

use routing::{
    Request as RouteRequest,
    Response as RouteResponse,
};


#[derive(Clone)]
struct MyRoutingService {
    network: Arc<RaptorNetwork>,
}


#[tonic::async_trait]
impl RoutingService for MyRoutingService {
    async fn get_route(
        &self,
        request: Request<RouteRequest>,
    ) -> Result<Response<RouteResponse>, Status> {

        let request = request.into_inner();
        let network = Arc::clone(&self.network);
        let start = request.start_stop_id as usize;
        let destination = request.destination_stop_id as usize;
        let rounds = request.rounds as usize;

        let duration = tokio::task::spawn_blocking(move || {
            raptor(
                &network,
                start,
                destination,
                rounds,
            )
        })
            .await
            .map_err(|e| {
                Status::internal(format!(
                    "RAPTOR task failed: {}",
                    e
                ))
            })?;

        Ok(Response::new(RouteResponse {
            duration_seconds: duration as u64,
        }))
    }
}


#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Load your existing schedule
    let schedule = load_schedule(
        "data/berlin-v7.1-10pct_transitSchedule.xml"
    );

    // Build RAPTOR network once
    let network = generate_network(&schedule);

    let service = MyRoutingService {
        network: Arc::new(network),
    };

    let address = "0.0.0.0:50052".parse()?;

    println!(
        "RAPTOR server listening on {}",
        address
    );

    Server::builder()
        .add_service(
            RoutingServiceServer::new(service)
        )
        .serve(address)
        .await?;

    Ok(())
}