use crate::config::Config;
use crate::services::equipment_type_service::EquipmentTypeService;
use crate::services::mode_group_service::ModeGroupService;
use crate::services::mode_service::ModeService;
use anyhow::Context;
use axum::{Extension, Router, routing::get_service};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::services::ServeFile;
use tower_http::trace::TraceLayer;

pub mod equipment_types;
pub mod mode;
pub mod mode_groups;
pub mod response;

pub mod date_format {
    use serde::{self, Serializer};
    use time::OffsetDateTime;

    pub fn serialize<S>(date: &Option<OffsetDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(dt) => {
                let s = dt
                    .format(&time::format_description::well_known::Rfc3339)
                    .map_err(serde::ser::Error::custom)?;
                serializer.serialize_str(&s)
            }
            None => serializer.serialize_none(),
        }
    }
}

// TODO: the compiler says these are not being used?
#[derive(Clone)]
pub struct ApiContext {
    #[allow(dead_code)] // pasing these for now to suppress warnings
    pub config: Arc<Config>,
    #[allow(dead_code)] // pasing these for now to suppress warnings
    pub db: PgPool,
}

pub async fn serve(config: Config, db: PgPool) -> anyhow::Result<()> {
    let equipment_type_service = EquipmentTypeService::new(db.clone());
    let mode_group_service = ModeGroupService::new(db.clone());
    let mode_service = ModeService::new(db.clone());

    let app = api_router()
        .layer(
            ServiceBuilder::new()
                .layer(Extension(ApiContext {
                    // ApiContext is right here? not sure what the issue is
                    config: Arc::new(config),
                    db,
                }))
                .layer(Extension(equipment_type_service))
                .layer(Extension(mode_group_service))
                .layer(Extension(mode_service))
                .layer(TraceLayer::new_for_http()),
        )
        .fallback(response::handler_404);

    let listener = TcpListener::bind("0.0.0.0:19080")
        .await
        .context("failed to bind to address")?;

    println!("HTTP server listening on http://0.0.0.0:19080");

    axum::serve(listener, app)
        .await
        .context("error running HTTP server")
}

fn api_router() -> Router {
    Router::new()
        .route(
            "/favicon.ico",
            get_service(ServeFile::new("static/favicon.ico")),
        )
        // the order in which i made these
        .merge(equipment_types::router())
        .merge(mode_groups::router())
        .merge(mode::router())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use std::fs;
    use tempfile::TempDir;
    use tower::ServiceExt; // for `oneshot`

    // Best approach: Create app with minimal setup
    async fn create_test_app() -> Router {
        // For routing tests, we can create the router without the database layer
        // Since we're only testing that routes return 404, we don't need real database

        let config = Config {
            database_url: "postgresql://test:test@localhost/test".parse().unwrap(),
            pool_size: 1,
            bind_address: "127.0.0.1:0".to_string(),
            log_level: "error".to_string(),
        };

        // Create the router structure without actually connecting to database
        // Most security tests only care about routing, not database operations
        Router::new()
            .route(
                "/favicon.ico",
                get_service(ServeFile::new("static/favicon.ico")),
            )
            // If you have API routes that don't immediately need database, add them:
            // .merge(core::router())
            .layer(Extension(Arc::new(config))) // Just pass config, not full ApiContext
            .fallback(response::handler_404)
    }

    // For tests that need actual files
    async fn create_test_app_with_files() -> (Router, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let static_dir = temp_dir.path().join("static");
        fs::create_dir_all(&static_dir).expect("Failed to create static dir");
        fs::write(static_dir.join("favicon.ico"), b"test favicon")
            .expect("Failed to write favicon");

        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

        let app = create_test_app().await;

        std::env::set_current_dir(original_dir).expect("Failed to restore directory");

        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_favicon_is_accessible() {
        let (app, _temp_dir) = create_test_app_with_files().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/favicon.ico")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_common_fuzzing_attacks() {
        let app = create_test_app().await;

        let fuzzing_paths = vec![
            "/.vscode/sftp.json",
            "/config.yml",
            "/config.xml",
            "/etc/ssl/private/server.key",
            "/config.php",
            "/.svn/wc.db",
            "/.ssh/id_cd25519",
            "/.env.production",
            "/.aws/credentials",
            "/phpinfo.php",
            "/.git/HEAD",
            "/backup.tar.gz",
            "/user_secrets.yml",
            "/docker-compose.yml",
            "/.env",
            "/api/.env",
            "/wp-admin/setup-config.php",
            "/_vti_pvt/service.pwd",
            "/backup.zip",
            "/server.key",
            "/config.json",
            "/.ssh/id_ecdsa",
            "/database_backup.sql",
            "/database.sql",
            "/backup.sql",
            "/.ssh/id_rsa",
            "/dump.sql",
            "/cloud-config.yml",
            "/web.config",
            "/config.yaml",
            "/wp-config.php",
            "/config/production.json",
            "/feed",
            "/db/schema.rb",
            "/settings.py",
            "/secrets.json",
        ];

        for path in fuzzing_paths {
            let request = Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Fuzzing attack path {} should return 404",
                path
            );
        }
    }

    #[tokio::test]
    async fn test_git_directory_not_accessible() {
        let app = create_test_app().await;

        let git_paths = vec![
            "/.git",
            "/.git/",
            "/.git/config",
            "/.git/HEAD",
            "/.git/../.git/config",
            "/static/../.git/config",
        ];

        for path in git_paths {
            let request = Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Path {} should not be accessible",
                path
            );
        }
    }

    #[tokio::test]
    async fn test_env_file_not_accessible() {
        let app = create_test_app().await;

        let env_paths = vec![
            "/.env",
            "/.env.local",
            "/.env.production",
            "/static/../.env",
        ];

        for path in env_paths {
            let request = Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Environment file {} should not be accessible",
                path
            );
        }
    }

    #[tokio::test]
    async fn test_directory_traversal_attempts() {
        let app = create_test_app().await;

        let traversal_paths = vec![
            "/../etc/passwd",
            "/static/../../etc/passwd",
            "/../.git/config",
            "/static/../.git/config",
            "/../.env",
            "/static/../../../../etc/shadow",
            "/favicon.ico/../.git/config",
            "/favicon.ico/../../.env",
        ];

        for path in traversal_paths {
            let request = Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Directory traversal path {} should not be accessible",
                path
            );
        }
    }

    #[tokio::test]
    async fn test_ssh_and_crypto_files() {
        let app = create_test_app().await;

        let crypto_paths = vec![
            "/.ssh/id_rsa",
            "/.ssh/id_ecdsa",
            "/.ssh/id_ed25519",
            "/.ssh/id_cd25519",
            "/.ssh/authorized_keys",
            "/server.key",
            "/private.key",
            "/.aws/credentials",
            "/etc/ssl/private/server.key",
        ];

        for path in crypto_paths {
            let request = Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Cryptographic file {} should not be accessible",
                path
            );
        }
    }

    #[tokio::test]
    async fn test_post_requests_with_malicious_paths() {
        let app = create_test_app().await;

        let post_paths = vec![
            "/hello.world",
            "/.env",
            "/config.php",
            "/admin",
            "/.git/config",
        ];

        for path in post_paths {
            let request = Request::builder()
                .method(Method::POST)
                .uri(path)
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            assert!(
                response.status() == StatusCode::NOT_FOUND
                    || response.status() == StatusCode::METHOD_NOT_ALLOWED,
                "POST to {} should return 404 or 405, got {}",
                path,
                response.status()
            );
        }
    }

    #[tokio::test]
    async fn test_different_http_methods() {
        let app = create_test_app().await;

        let methods = vec![
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::HEAD,
            Method::OPTIONS,
        ];

        for method in methods {
            let request = Request::builder()
                .method(method.clone())
                .uri("/.env")
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            assert!(
                response.status() == StatusCode::NOT_FOUND
                    || response.status() == StatusCode::METHOD_NOT_ALLOWED,
                "{} to /.env should return 404 or 405, got {}",
                method,
                response.status()
            );
        }
    }
}
