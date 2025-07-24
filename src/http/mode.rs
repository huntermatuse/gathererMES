use crate::http::date_format;
use crate::http::response::ApiResponse;
use crate::services::mode_service::{Mode, ModeService};
use axum::{
    Json, Router,
    extract::{Extension, Path, Query},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{error, info};
use uuid::Uuid;

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/modes", get(get_all_modes).post(create_mode))
        .route("/api/v1/modes/count", get(get_modes_count))
        .route("/api/v1/modes/{id}", get(get_mode_by_id))
        .route("/api/v1/modes/delete/{id}", post(delete_mode))
}

#[derive(Deserialize)]
pub struct CreateModeRequest {
    pub mode_group_id: Uuid,
    pub mode_description: String,
}

#[derive(Serialize)]
pub struct ModeResponse {
    pub mode_id: Uuid,
    pub mode_group_id: Uuid,
    pub mode_description: String,
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
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    pub mode_group_id: Option<Uuid>,
    pub search: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total_count: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Serialize)]
pub struct CountResponse {
    pub count: i64,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    50
}

impl From<Mode> for ModeResponse {
    fn from(mode: Mode) -> Self {
        Self {
            mode_id: mode.mode_id,
            mode_group_id: mode.mode_group_id,
            mode_description: mode.mode_description,
            created_at: mode.created_at,
            updated_at: mode.updated_at,
        }
    }
}

async fn get_all_modes(
    Extension(service): Extension<ModeService>,
    Query(query): Query<PaginationQuery>,
) -> Json<ApiResponse<PaginatedResponse<ModeResponse>>> {
    let offset = (query.page - 1) * query.per_page;

    let result = service
        .search_with_filters(
            query.mode_group_id,
            query.search.as_deref(),
            offset,
            query.per_page,
        )
        .await;

    match result {
        Ok((modes, total_count)) => {
            let response = PaginatedResponse {
                data: modes.into_iter().map(ModeResponse::from).collect(),
                total_count,
                page: query.page,
                per_page: query.per_page,
                total_pages: (total_count + query.per_page - 1) / query.per_page,
            };

            info!(
                "Retrieved {} modes (page {}/{})",
                response.data.len(),
                response.page,
                response.total_pages
            );

            Json(ApiResponse::success(response))
        }
        Err(e) => {
            error!("Failed to get modes: {}", e);
            Json(ApiResponse::error_str("Failed to retrieve modes"))
        }
    }
}

async fn get_mode_by_id(
    Extension(service): Extension<ModeService>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<ModeResponse>> {
    match service.get_by_id(id).await {
        Ok(mode) => Json(ApiResponse::success(ModeResponse::from(mode))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Json(ApiResponse::error_str("Mode not found"))
            } else {
                error!("Failed to fetch mode {}: {}", id, e);
                Json(ApiResponse::error_str("Failed to retrieve mode"))
            }
        }
    }
}

async fn create_mode(
    Extension(service): Extension<ModeService>,
    Json(payload): Json<CreateModeRequest>,
) -> Json<ApiResponse<ModeResponse>> {
    match service
        .create(payload.mode_group_id, &payload.mode_description)
        .await
    {
        Ok(mode) => {
            info!("Created mode: {}", mode.mode_description);
            Json(ApiResponse::success(ModeResponse::from(mode)))
        }
        Err(e) => {
            error!("Failed to create mode: {}", e);
            Json(ApiResponse::error_str("Failed to create mode"))
        }
    }
}

async fn delete_mode(
    Extension(service): Extension<ModeService>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<()>> {
    match service.delete(id).await {
        Ok(_) => {
            info!("Deleted mode: {}", id);
            Json(ApiResponse::success(()))
        }
        Err(e) => {
            if e.to_string().contains("not found") {
                Json(ApiResponse::error_str("Mode not found"))
            } else {
                error!("Failed to delete mode {}: {}", id, e);
                Json(ApiResponse::error_str("Failed to delete mode"))
            }
        }
    }
}

async fn get_modes_count(
    Extension(service): Extension<ModeService>,
) -> Json<ApiResponse<CountResponse>> {
    match service.count().await {
        Ok(count) => Json(ApiResponse::success(CountResponse { count })),
        Err(e) => {
            error!("Failed to get modes count: {}", e);
            Json(ApiResponse::error_str("Failed to get modes count"))
        }
    }
}
