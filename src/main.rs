use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post},
};
use chrono::Utc;
use std::net::SocketAddr;

mod api;
mod handlers;
mod models;

use api::response::{ApiResponse, ErrorResponse, SuccessResponse};
use handlers::core::{EquipmentTypeStore, init_equipment_type_store};
use models::core::{Equipment, EquipmentTypes, ModeGroups, Modes};

// app state to hold our data stores
#[derive(Clone)]
pub struct AppState {
    pub equipment_types_store: EquipmentTypeStore,
}

async fn health_check() -> Json<SuccessResponse<&'static str>> {
    Json(SuccessResponse::new("healthy"))
}

// updated equipment types handlers using CRUD
async fn get_equipment_types(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<EquipmentTypes>>> {
    match EquipmentTypes::get_all(&state.equipment_types_store) {
        Ok(equipment_types) => Json(ApiResponse::success(equipment_types)),
        Err(error) => Json(ApiResponse::error(error)),
    }
}

async fn get_equipment_type_by_id(
    State(state): State<AppState>,
    Path(id): Path<i8>,
) -> Json<ApiResponse<Option<EquipmentTypes>>> {
    // TODO: check if the provided id is i8 and is in the range of eqType IDs
    match EquipmentTypes::get_by_id(&state.equipment_types_store, id) {
        Ok(equipment_type) => Json(ApiResponse::success(equipment_type)),
        Err(error) => Json(ApiResponse::error(error)),
    }
}

// For create, you'd need to extract JSON from the request body
// This is a simplified version - you'd typically use a CreateEquipmentTypeRequest struct
async fn create_equipment_type(
    State(state): State<AppState>,
    // In real implementation, you'd extract JSON body here
    // Json(request): Json<CreateEquipmentTypeRequest>
) -> Json<ApiResponse<EquipmentTypes>> {
    // Hardcoded for example - replace with actual request data
    let type_name = "New Equipment Type".to_string();

    match EquipmentTypes::create(&state.equipment_types_store, type_name) {
        Ok(new_equipment_type) => Json(ApiResponse::success(new_equipment_type)),
        Err(error) => Json(ApiResponse::error(error)),
    }
}

async fn update_equipment_type(
    State(state): State<AppState>,
    Path(id): Path<i8>,
    // In real implementation: Json(request): Json<UpdateEquipmentTypeRequest>
) -> Json<ApiResponse<Option<EquipmentTypes>>> {
    // Hardcoded for example
    let new_name = "Updated Equipment Type".to_string();

    match EquipmentTypes::update(&state.equipment_types_store, id, new_name) {
        Ok(updated_equipment_type) => Json(ApiResponse::success(updated_equipment_type)),
        Err(error) => Json(ApiResponse::error(error)),
    }
}

async fn delete_equipment_type(
    State(state): State<AppState>,
    Path(id): Path<i8>,
) -> Json<ApiResponse<bool>> {
    match EquipmentTypes::delete(&state.equipment_types_store, id) {
        Ok(deleted) => Json(ApiResponse::success(deleted)),
        Err(error) => Json(ApiResponse::error(error)),
    }
}

// Keep your existing handlers for other endpoints (for now)
async fn get_equipment() -> Json<ApiResponse<Vec<Equipment>>> {
    let eqs = vec![
        Equipment {
            equipment_id: 1,
            equipment_name: "eq_Test_1".to_string(),
            equipment_type_id: 1,
            equipment_parent_id: None,
            equipment_enabled: true,
            equipment_metadata: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        },
        Equipment {
            equipment_id: 2,
            equipment_name: "eq_Test_2".to_string(),
            equipment_type_id: 2,
            equipment_parent_id: Some(1),
            equipment_enabled: true,
            equipment_metadata: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        },
    ];
    Json(ApiResponse::success(eqs))
}

async fn get_mode_groups() -> Json<ApiResponse<Vec<ModeGroups>>> {
    let default_mode_group = vec![ModeGroups {
        mode_group_id: 1,
        mode_group_name: "GathererMES_Default".to_string(),
        mode_group_description: Some("A default group that should work for most cases".to_string()),
    }];
    Json(ApiResponse::success(default_mode_group))
}

async fn get_modes() -> Json<ApiResponse<Vec<Modes>>> {
    let modes = vec![
        Modes {
            mode_id: 1,
            mode_group_id: 1,
            mode_code: 0,
            mode_description: Some("Disabled".to_string()),
        },
        Modes {
            mode_id: 2,
            mode_group_id: 1,
            mode_code: 1,
            mode_description: Some("Running".to_string()),
        },
        Modes {
            mode_id: 3,
            mode_group_id: 1,
            mode_code: 2,
            mode_description: Some("Change Over".to_string()),
        },
    ];
    Json(ApiResponse::success(modes))
}

async fn not_implemented() -> Json<ApiResponse<ErrorResponse>> {
    Json(ApiResponse::error_str(
        "this endpoint has not been completed yet",
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize your data stores
    let app_state = AppState {
        equipment_types_store: init_equipment_type_store(),
    };

    let app = Router::new()
        // Updated equipment type endpoints with actual CRUD
        .route("/equipment-types", post(create_equipment_type))
        .route("/equipment-types", get(get_equipment_types))
        .route("/equipment-types/{id}", get(get_equipment_type_by_id))
        .route("/equipment-types/{id}", post(update_equipment_type))
        .route("/equipment-types/{id}/delete", post(delete_equipment_type))
        // equipment endpoints
        .route("/equipment", get(get_equipment))
        .route("/equipment/{id}", get(not_implemented))
        // mode group endpoints
        .route("/mode-groups", post(not_implemented)) //create
        .route("/mode-groups", get(get_mode_groups)) //read
        .route("/mode-groups/{id}", get(not_implemented)) //read single
        .route("/mode-groups/{id}", post(not_implemented)) //update
        .route("/mode-groups/{id}/delete", post(not_implemented)) //delete
        // modes endpoints
        .route("/modes", post(not_implemented)) // create
        .route("/modes", get(get_modes)) // read
        .route("/modes/{id}", get(not_implemented)) // read single
        .route("/modes/update", post(not_implemented)) // update
        .route("/modes/delete/{id}", post(not_implemented)) // delete
        // Other endpoints...
        .route("/state-groups", get(not_implemented))
        .route("/state-groups/{id}", get(not_implemented))
        .route("/states", get(not_implemented))
        .route("/states/{id}", get(not_implemented))
        .route("/products", get(not_implemented))
        .route("/products/{id}", get(not_implemented))
        .route("/work-orders", get(not_implemented))
        .route("/work-orders/{id}", get(not_implemented))
        .route("/jobs", get(not_implemented))
        .route("/jobs/{id}", get(not_implemented))
        .route("/status", get(health_check))
        .with_state(app_state); // Pass the state to all handlers

    let addr = SocketAddr::from(([0, 0, 0, 0], 16002));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
