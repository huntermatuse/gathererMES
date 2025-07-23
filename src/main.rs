use anyhow::Context;
use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post},
};
use chrono::Utc;
use clap::Parser;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use sqlx::postgres::PgPoolOptions;

mod api;
mod handlers;
mod models;
mod config;

use api::response::{ApiResponse, ErrorResponse, SuccessResponse};
use crate::{api::core::get_equipment_types, config::Config};
use handlers::core::{
    EquipmentTypeStore, ModeGroupStore, init_equipment_type_store, init_mode_group_store,
};
use models::core::{Equipment, EquipmentTypes, ModeGroups, Modes};

// app state to hold our data stores
#[derive(Clone)]
pub struct AppState {
    pub equipment_types_store: EquipmentTypeStore,
}

async fn health_check() -> Json<SuccessResponse<&'static str>> {
    Json(SuccessResponse::new("healthy"))
}

async fn not_implemented() -> Json<ApiResponse<ErrorResponse>> {
    Json(ApiResponse::error_str(
        "this endpoint has not been completed yet",
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load .env file if it exists break if it doesn't exist
    dotenv::dotenv().ok();
    
    // init the logger
    env_logger::init();
    
    // parse configuration from CLI args and environment
    let config: Config = config::Config::parse();

    // Initialize your data stores
    let app_state = AppState {
        equipment_types_store: init_equipment_type_store(),
    };

    // shared connection pool for sqlx shared across the application.
    let db = PgPoolOptions::new()
        // TODO: update this comment from launchbadge
        // The default connection limit for a Postgres server is 100 connections, minus 3 for superusers.
        // Since we're using the default superuser we don't have to worry about this too much,
        // although we should leave some connections available for manual access.
        //
        // If you're deploying your application with multiple replicas, then the total
        // across all replicas should not exceed the Postgres connection limit.
        .max_connections(config.pool_size)
        .connect(&config.database_url.to_string())
        .await
        .context("could not connect to database_url")?;

    // this embeds database migrations in the application binary so we can ensure the database
    // is migrated correctly on startup
    sqlx::migrate!().run(&db).await?;

    // Finally, we spin up our API.
    // http::serve(config, db).await?;

    // endpoints for each of the action groups.
    // action groups are broken into the two parts of the static data api.
    // more on these types can be found in the models::core and
    // models::operations respectively.
    // - Core
    //     - Equipment Types
    //     - Equipment
    //     - Mode Groups
    //     - Mode(s)
    //     - State Groups
    //     - State(s)
    // - Operations
    //     - Products
    //     - Work Orders
    //     - Jobs
    //
    let app = Router::new()
        //
        // core schema
        //
        // equipment type endpoints
        .route(
            "/api/v1/equipment-types",
            get(get_equipment_types)
                .post(not_implemented)
        )
        .route(
            "/api/v1/equipment-types/{id}",
            get(not_implemented)
                .post(not_implemented)
        )
        .route(
            "/api/v1/equipment-types/{id}/delete", 
            post(not_implemented)
        )
        //
        // equipment endpoints
        .route("/equipment", get(not_implemented))
        .route("/equipment/{id}", get(not_implemented))
        //
        // mode group endpoints
        .route("/mode-groups", post(not_implemented))
        .route("/mode-groups", get(not_implemented))
        .route("/mode-groups/{id}", get(not_implemented))
        .route("/mode-groups/{id}", post(not_implemented))
        .route("/mode-groups/{id}/delete", post(not_implemented))
        //
        // mode endpoints
        .route("/modes", post(not_implemented))
        .route("/modes", get(not_implemented))
        .route("/modes/{id}", get(not_implemented))
        .route("/modes/update", post(not_implemented))
        .route("/modes/delete/{id}", post(not_implemented))
        //
        // state group endpoints
        .route("/state-groups", get(not_implemented))
        .route("/state-groups/{id}", get(not_implemented))
        //
        // state endpoints
        .route("/states", get(not_implemented))
        .route("/states/{id}", get(not_implemented))
        //
        // operations schema
        //
        // product endpoints
        .route("/products", get(not_implemented))
        .route("/products/{id}", get(not_implemented))
        //
        // work order endpoints
        .route("/work-orders", get(not_implemented))
        .route("/work-orders/{id}", get(not_implemented))
        //
        // job endpoints
        .route("/jobs", get(not_implemented))
        .route("/jobs/{id}", get(not_implemented))
        //
        // api endpoints
        //
        // api status endpoint
        .route("/status", get(health_check))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 16002));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
