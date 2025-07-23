use anyhow::Context;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;

mod config;
mod database;
mod http;
mod models;
mod services;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load .env file if it exists
    dotenv::dotenv().ok();

    // init the logger
    env_logger::init();

    // parse configuration from cli args and environment
    let config: Config = Config::parse();
    config.validate()?;

    // shared connection pool for sqlx shared across the application
    let db = PgPoolOptions::new()
        .max_connections(config.pool_size)
        .connect(&config.database_url.to_string())
        .await
        .context("could not connect to database_url")?;

    // run migrations (idk if we need this, keeping in for now)
    // sqlx::migrate!().run(&db).await?;

    // create services
    // let equipment_type_service = EquipmentTypeService::new(db.clone());

    // start both http and gRPC servers concurrently or in parallel
    tokio::try_join!(
        start_http_server(config, db.clone()),
        // start_grpc_server(equipment_type_service)
    )?;

    Ok(())
}

async fn start_http_server(config: Config, db: sqlx::PgPool) -> anyhow::Result<()> {
    println!("Starting HTTP server...");
    http::serve(config, db).await
}

// async fn start_grpc_server(equipment_type_service: EquipmentTypeService) -> anyhow::Result<()> {
//     use crate::grpc::equipment_types::{
//         EquipmentTypesGrpcService,
//         equipment_type_proto::equipment_types_server::EquipmentTypesServer,
//     };
//     use tonic::transport::Server;

//     let equipment_types_service = EquipmentTypesGrpcService::new(equipment_type_service);

//     let addr = "0.0.0.0:50051".parse()?;
//     println!("gRPC server listening on http://{}", addr);

//     Server::builder()
//         .add_service(EquipmentTypesServer::new(equipment_types_service))
//         .serve(addr)
//         .await
//         .context("gRPC server error")
// }
