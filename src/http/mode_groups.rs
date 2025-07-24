use crate::http::date_format;
use crate::http::response::ApiResponse;
use crate::services::mode_group_service::ModeGroupService;
use axum::{
    Json, Router,
    extract::{Extension, Path, Query},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{error, info};
use uuid::Uuid;

// mode group endpoints
pub fn router() -> Router {
    Router::new()
        .route(
            "/api/v1/mode-groups",
            get(get_all_mode_groups).post(create_mode_group),
        )
        .route("/api/v1/mode-groups/bulk", post(bulk_create_mode_groups))
        .route("/api/v1/mode-groups/count", get(get_mode_groups_count))
        .route("/api/v1/mode-groups/{id}", get(get_mode_group_by_id))
        .route(
            "/api/v1/mode-groups/update-name/{id}",
            post(update_mode_group_name),
        )
        .route(
            "/api/v1/mode-groups/update-description/{id}",
            post(update_mode_group_description),
        )
        .route("/api/v1/mode-groups/delete/{id}", post(delete_mode_group))
        .route(
            "/api/v1/mode-groups/exists/{id}",
            get(check_mode_group_exists),
        )
        .route("/api/v1/mode-groups/by-name", get(get_mode_group_by_name))
        .route(
            "/api/v1/mode-groups/by-description",
            get(get_mode_group_by_description),
        )
}

// request/response dtos
#[derive(Serialize, Deserialize)]
pub struct ModeGroupResponse {
    pub mode_group_id: Uuid,
    pub mode_group_name: String,
    pub mode_group_description: String,
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
pub struct CreateModeGroupRequest {
    pub mode_group_name: String,
    pub mode_group_description: String,
}

#[derive(Deserialize)]
pub struct UpdateModeGroupNameRequest {
    pub mode_group_name: String,
}

#[derive(Deserialize)]
pub struct UpdateModeGroupDescriptionRequest {
    pub mode_group_description: String,
}

#[derive(Deserialize)]
pub struct BulkCreateModeGroupRequest {
    pub mode_groups: Vec<BulkModeGroupItem>,
}

#[derive(Deserialize)]
pub struct BulkModeGroupItem {
    pub mode_group_name: String,
    pub mode_group_description: String,
}

#[derive(Serialize)]
pub struct BulkCreateModeGroupResponse {
    pub created: Vec<ModeGroupResponse>,
    pub created_count: usize,
    pub total_requested: usize,
}

#[derive(Serialize)]
pub struct CountResponse {
    pub count: i64,
}

#[derive(Serialize)]
pub struct ExistsResponse {
    pub exists: bool,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total_count: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

#[derive(Deserialize)]
pub struct NameQuery {
    pub name: String,
}

#[derive(Deserialize)]
pub struct DescriptionQuery {
    pub description: String,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    50
}

// service model -> response model
impl From<crate::services::mode_group_service::ModeGroup> for ModeGroupResponse {
    fn from(mode_group: crate::services::mode_group_service::ModeGroup) -> Self {
        Self {
            mode_group_id: mode_group.mode_group_id,
            mode_group_name: mode_group.mode_group_name,
            mode_group_description: mode_group.mode_group_description,
            created_at: mode_group.created_at,
            updated_at: mode_group.updated_at,
        }
    }
}

// handler functions for http endpoints
async fn get_all_mode_groups(
    Extension(service): Extension<ModeGroupService>,
    Query(pagination): Query<PaginationQuery>,
) -> Json<ApiResponse<PaginatedResponse<ModeGroupResponse>>> {
    // Convert 1-based page to 0-based offset
    let offset = (pagination.page - 1) * pagination.per_page;

    match service.get_paginated(offset, pagination.per_page).await {
        Ok((mode_groups, total_count)) => {
            let response: Vec<ModeGroupResponse> = mode_groups
                .into_iter()
                .map(ModeGroupResponse::from)
                .collect();

            let total_pages = (total_count + pagination.per_page - 1) / pagination.per_page;

            let paginated_response = PaginatedResponse {
                data: response,
                total_count,
                page: pagination.page,
                per_page: pagination.per_page,
                total_pages,
            };

            info!(
                "Retrieved {} mode groups (page {}/{}, total: {})",
                paginated_response.data.len(),
                pagination.page,
                total_pages,
                total_count
            );

            Json(ApiResponse::success(paginated_response))
        }
        Err(e) => {
            error!("Failed to get mode groups: {}", e);
            Json(ApiResponse::error_str("Failed to retrieve mode groups"))
        }
    }
}

async fn get_mode_group_by_id(
    Extension(service): Extension<ModeGroupService>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<ModeGroupResponse>> {
    match service.get_by_id(id).await {
        Ok(mode_group) => {
            info!("Retrieved mode group: {}", mode_group.mode_group_name);
            Json(ApiResponse::success(ModeGroupResponse::from(mode_group)))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Json(ApiResponse::error_str("Mode group not found"))
            } else {
                error!("Failed to get mode group {}: {}", id, e);
                Json(ApiResponse::error_str("Failed to retrieve mode group"))
            }
        }
    }
}

async fn get_mode_group_by_name(
    Extension(service): Extension<ModeGroupService>,
    Query(query): Query<NameQuery>,
) -> Json<ApiResponse<ModeGroupResponse>> {
    if query.name.trim().is_empty() {
        return Json(ApiResponse::error_str("Mode group name cannot be empty"));
    }

    match service.get_by_name(&query.name).await {
        Ok(Some(mode_group)) => {
            info!(
                "Retrieved mode group by name: {}",
                mode_group.mode_group_name
            );
            Json(ApiResponse::success(ModeGroupResponse::from(mode_group)))
        }
        Ok(None) => Json(ApiResponse::error_str("Mode group not found")),
        Err(e) => {
            error!("Failed to get mode group by name '{}': {}", query.name, e);
            Json(ApiResponse::error_str("Failed to retrieve mode group"))
        }
    }
}

async fn get_mode_group_by_description(
    Extension(service): Extension<ModeGroupService>,
    Query(query): Query<DescriptionQuery>,
) -> Json<ApiResponse<ModeGroupResponse>> {
    if query.description.trim().is_empty() {
        return Json(ApiResponse::error_str(
            "Mode group description cannot be empty",
        ));
    }

    match service.get_by_description(&query.description).await {
        Ok(Some(mode_group)) => {
            info!(
                "Retrieved mode group by description: {}",
                mode_group.mode_group_name
            );
            Json(ApiResponse::success(ModeGroupResponse::from(mode_group)))
        }
        Ok(None) => Json(ApiResponse::error_str("Mode group not found")),
        Err(e) => {
            error!("Failed to get mode group by description: {}", e);
            Json(ApiResponse::error_str("Failed to retrieve mode group"))
        }
    }
}

async fn create_mode_group(
    Extension(service): Extension<ModeGroupService>,
    Json(request): Json<CreateModeGroupRequest>,
) -> Json<ApiResponse<ModeGroupResponse>> {
    match service
        .create(&request.mode_group_name, &request.mode_group_description)
        .await
    {
        Ok(mode_group) => {
            info!("Created mode group: {}", mode_group.mode_group_name);
            Json(ApiResponse::success(ModeGroupResponse::from(mode_group)))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("already exists") {
                Json(ApiResponse::error_str("Mode group name already exists"))
            } else if error_msg.contains("cannot be empty") || error_msg.contains("cannot exceed") {
                Json(ApiResponse::error(format!("Invalid input: {}", error_msg)))
            } else {
                error!("Failed to create mode group: {}", e);
                Json(ApiResponse::error_str("Failed to create mode group"))
            }
        }
    }
}

async fn update_mode_group_name(
    Extension(service): Extension<ModeGroupService>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateModeGroupNameRequest>,
) -> Json<ApiResponse<ModeGroupResponse>> {
    match service.update_name(id, &request.mode_group_name).await {
        Ok(mode_group) => {
            info!(
                "Updated mode group name {}: {}",
                id, mode_group.mode_group_name
            );
            Json(ApiResponse::success(ModeGroupResponse::from(mode_group)))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Json(ApiResponse::error_str("Mode group not found"))
            } else if error_msg.contains("already exists") {
                Json(ApiResponse::error_str("Mode group name already exists"))
            } else if error_msg.contains("cannot be empty") || error_msg.contains("cannot exceed") {
                Json(ApiResponse::error(format!("Invalid input: {}", error_msg)))
            } else {
                error!("Failed to update mode group name {}: {}", id, e);
                Json(ApiResponse::error_str("Failed to update mode group name"))
            }
        }
    }
}

async fn update_mode_group_description(
    Extension(service): Extension<ModeGroupService>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateModeGroupDescriptionRequest>,
) -> Json<ApiResponse<ModeGroupResponse>> {
    match service
        .update_description(id, &request.mode_group_description)
        .await
    {
        Ok(mode_group) => {
            info!("Updated mode group description {}", id);
            Json(ApiResponse::success(ModeGroupResponse::from(mode_group)))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Json(ApiResponse::error_str("Mode group not found"))
            } else if error_msg.contains("cannot be empty") || error_msg.contains("cannot exceed") {
                Json(ApiResponse::error(format!("Invalid input: {}", error_msg)))
            } else {
                error!("Failed to update mode group description {}: {}", id, e);
                Json(ApiResponse::error_str(
                    "Failed to update mode group description",
                ))
            }
        }
    }
}

async fn delete_mode_group(
    Extension(service): Extension<ModeGroupService>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<()>> {
    match service.delete(id).await {
        Ok(()) => {
            info!("Deleted mode group: {}", id);
            Json(ApiResponse::success(()))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                Json(ApiResponse::error_str("Mode group not found"))
            } else if error_msg.contains("in use") {
                Json(ApiResponse::error_str(
                    "Mode group is in use and cannot be deleted",
                ))
            } else {
                error!("Failed to delete mode group {}: {}", id, e);
                Json(ApiResponse::error_str("Failed to delete mode group"))
            }
        }
    }
}

async fn bulk_create_mode_groups(
    Extension(service): Extension<ModeGroupService>,
    Json(request): Json<BulkCreateModeGroupRequest>,
) -> Json<ApiResponse<BulkCreateModeGroupResponse>> {
    if request.mode_groups.is_empty() {
        return Json(ApiResponse::error_str("No mode groups provided"));
    }

    if request.mode_groups.len() > 100 {
        return Json(ApiResponse::error_str(
            "Cannot create more than 100 mode groups at once",
        ));
    }

    let mode_group_tuples: Vec<(&str, &str)> = request
        .mode_groups
        .iter()
        .map(|item| {
            (
                item.mode_group_name.as_str(),
                item.mode_group_description.as_str(),
            )
        })
        .collect();

    match service.bulk_create(mode_group_tuples).await {
        Ok(mode_groups) => {
            let response: Vec<ModeGroupResponse> = mode_groups
                .into_iter()
                .map(ModeGroupResponse::from)
                .collect();

            let bulk_response = BulkCreateModeGroupResponse {
                created_count: response.len(),
                total_requested: request.mode_groups.len(),
                created: response,
            };

            info!(
                "Bulk created {}/{} mode groups",
                bulk_response.created_count, bulk_response.total_requested
            );

            Json(ApiResponse::success(bulk_response))
        }
        Err(e) => {
            error!("Failed to bulk create mode groups: {}", e);
            Json(ApiResponse::error_str("Failed to create mode groups"))
        }
    }
}

async fn get_mode_groups_count(
    Extension(service): Extension<ModeGroupService>,
) -> Json<ApiResponse<CountResponse>> {
    match service.count().await {
        Ok(count) => {
            info!("Total mode groups count: {}", count);
            Json(ApiResponse::success(CountResponse { count }))
        }
        Err(e) => {
            error!("Failed to get mode groups count: {}", e);
            Json(ApiResponse::error_str("Failed to get mode groups count"))
        }
    }
}

async fn check_mode_group_exists(
    Extension(service): Extension<ModeGroupService>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<ExistsResponse>> {
    match service.exists(id).await {
        Ok(exists) => Json(ApiResponse::success(ExistsResponse { exists })),
        Err(e) => {
            error!("Failed to check if mode group {} exists: {}", id, e);
            Json(ApiResponse::error_str(
                "Failed to check mode group existence",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Extension,
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use sqlx::PgPool;
    use tower::ServiceExt;

    // Helper function to create test service
    fn create_test_service(pool: PgPool) -> ModeGroupService {
        ModeGroupService::new(pool)
    }

    #[sqlx::test]
    async fn test_create_mode_group_endpoint(pool: PgPool) -> sqlx::Result<()> {
        let service = create_test_service(pool);
        let app = router().layer(Extension(service));

        let request_body = json!({
            "mode_group_name": "Test HTTP Group",
            "mode_group_description": "Test HTTP Description"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/mode-groups")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_mode_group_by_id_endpoint(pool: PgPool) -> sqlx::Result<()> {
        let service = create_test_service(pool);

        // Create a test mode group first
        let created = service
            .create("Test Get Group", "Test Get Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let app = router().layer(Extension(service));

        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/v1/mode-groups/{}", created.mode_group_id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_name_endpoint(pool: PgPool) -> sqlx::Result<()> {
        let service = create_test_service(pool);

        // Create a test mode group first
        let created = service
            .create("Original Name", "Original Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let app = router().layer(Extension(service));

        let request_body = json!({
            "mode_group_name": "Updated Name"
        });

        let request = Request::builder()
            .method("POST")
            .uri(&format!(
                "/api/v1/mode-groups/update-name/{}",
                created.mode_group_id
            ))
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_bulk_create_mode_groups_endpoint(pool: PgPool) -> sqlx::Result<()> {
        let service = create_test_service(pool);
        let app = router().layer(Extension(service));

        let request_body = json!({
            "mode_groups": [
                {
                    "mode_group_name": "Bulk Group 1",
                    "mode_group_description": "Bulk Description 1"
                },
                {
                    "mode_group_name": "Bulk Group 2",
                    "mode_group_description": "Bulk Description 2"
                }
            ]
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/mode-groups/bulk")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_mode_groups_count_endpoint(pool: PgPool) -> sqlx::Result<()> {
        let service = create_test_service(pool);
        let app = router().layer(Extension(service));

        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/mode-groups/count")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_mode_group_endpoint(pool: PgPool) -> sqlx::Result<()> {
        let service = create_test_service(pool);

        // Create a test mode group first
        let created = service
            .create("To Delete", "Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let app = router().layer(Extension(service));

        let request = Request::builder()
            .method("POST")
            .uri(&format!(
                "/api/v1/mode-groups/delete/{}",
                created.mode_group_id
            ))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_pagination_endpoint(pool: PgPool) -> sqlx::Result<()> {
        let service = create_test_service(pool);

        // Create some test data
        for i in 1..=5 {
            let _ = service
                .create(
                    &format!("Pagination Group {}", i),
                    &format!("Description {}", i),
                )
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        }

        let app = router().layer(Extension(service));

        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/mode-groups?page=1&per_page=3")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }
}
