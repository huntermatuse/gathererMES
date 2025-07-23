use crate::services::equipment_type_service::EquipmentTypeService;
use crate::http::response::ApiResponse;
use crate::http::date_format;
use axum::{
    extract::{Extension, Path},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use tracing::error;

// equipment type endpoints
pub fn router() -> Router {
    // by having each module responsible for setting up its own routing,
    // it makes the root module a lot cleaner.
    Router::new()
        .route(
            "/api/v1/equipment-types",
            get(get_all_equipment_types).post(create_equipment_type),
        )
        .route(
            "/api/v1/equipment-types/{id}",
            get(get_equipment_type_by_id)
                .put(update_equipment_type)
        )
        .route(
            "/api/v1/equipment-types/delete/{id}", 
            post(delete_equipment_type)
        )
}

// request/response dtos
#[derive(Serialize, Deserialize)]
pub struct EquipmentTypeResponse {
    pub type_id: Uuid,
    pub type_name: String,
    #[serde(
        serialize_with = "date_format::serialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub created_at: Option<OffsetDateTime>,
    #[serde(
        serialize_with = "date_format::serialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Deserialize)]
pub struct CreateEquipmentTypeRequest {
    pub type_name: String,
}

#[derive(Deserialize)]
pub struct UpdateEquipmentTypeRequest {
    pub type_name: String,
}

// service model -> response model
impl From<crate::services::equipment_type_service::EquipmentType> for EquipmentTypeResponse {
    fn from(equipment_type: crate::services::equipment_type_service::EquipmentType) -> Self {
        Self {
            type_id: equipment_type.type_id,
            type_name: equipment_type.type_name,
            created_at: equipment_type.created_at,
            updated_at: equipment_type.updated_at,
        }
    }
}

// handler functions for http endpoints
async fn get_all_equipment_types(
    Extension(service): Extension<EquipmentTypeService>,
) -> Json<ApiResponse<Vec<EquipmentTypeResponse>>> {
    match service.get_all().await {
        Ok(equipment_types) => {
            let response: Vec<EquipmentTypeResponse> = equipment_types
                .into_iter()
                .map(EquipmentTypeResponse::from)
                .collect();
            
            Json(ApiResponse::success(response))
        }
        Err(e) => {
            error!("Failed to get equipment types: {}", e);
            Json(ApiResponse::error_str("Failed to retrieve equipment types"))
        }
    }
}

async fn get_equipment_type_by_id(
    Extension(service): Extension<EquipmentTypeService>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<EquipmentTypeResponse>> {
    match service.get_by_id(id).await {
        Ok(equipment_type) => {
            Json(ApiResponse::success(EquipmentTypeResponse::from(equipment_type)))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Json(ApiResponse::error_str("Equipment type not found"))
            } else {
                error!("Failed to get equipment type {}: {}", id, e);
                Json(ApiResponse::error_str("Failed to retrieve equipment type"))
            }
        }
    }
}

async fn create_equipment_type(
    Extension(service): Extension<EquipmentTypeService>,
    Json(request): Json<CreateEquipmentTypeRequest>,
) -> Json<ApiResponse<EquipmentTypeResponse>> {
    match service.create(request.type_name).await {
        Ok(equipment_type) => {
            Json(ApiResponse::success(EquipmentTypeResponse::from(equipment_type)))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("already exists") {
                Json(ApiResponse::error_str("Equipment type name already exists"))
            } else if error_msg.contains("cannot be empty") || error_msg.contains("cannot exceed") {
                Json(ApiResponse::error(format!("Invalid input: {}", error_msg)))
            } else {
                error!("Failed to create equipment type: {}", e);
                Json(ApiResponse::error_str("Failed to create equipment type"))
            }
        }
    }
}

async fn update_equipment_type(
    Extension(service): Extension<EquipmentTypeService>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateEquipmentTypeRequest>,
) -> Json<ApiResponse<EquipmentTypeResponse>> {
    match service.update(id, request.type_name).await {
        Ok(equipment_type) => {
            Json(ApiResponse::success(EquipmentTypeResponse::from(equipment_type)))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Json(ApiResponse::error_str("Equipment type not found"))
            } else if error_msg.contains("already exists") {
                Json(ApiResponse::error_str("Equipment type name already exists"))
            } else if error_msg.contains("cannot be empty") || error_msg.contains("cannot exceed") {
                Json(ApiResponse::error(format!("Invalid input: {}", error_msg)))
            } else {
                error!("Failed to update equipment type {}: {}", id, e);
                Json(ApiResponse::error_str("Failed to update equipment type"))
            }
        }
    }
}

async fn delete_equipment_type(
    Extension(service): Extension<EquipmentTypeService>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<()>> {
    match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Json(ApiResponse::error_str("Equipment type not found"))
            } else {
                error!("Failed to delete equipment type {}: {}", id, e);
                Json(ApiResponse::error_str("Failed to delete equipment type"))
            }
        }
    }
}