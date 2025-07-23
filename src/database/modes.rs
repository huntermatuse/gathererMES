use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModeRow {
    pub mode_id: Uuid,
    pub mode_group_id: Uuid,
    pub mode_description: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

pub struct ModeRowQueries;

impl ModeRowQueries {
    pub async fn get_all(db: &PgPool) -> Result<Vec<ModeRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeRow,
            r#"SELECT mode_id, mode_group_id, 
                mode_description, created_at, updated_at
	            FROM core.mode_group;"#
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_mode_group_id(
        db: &PgPool,
        mode_group_id: Uuid,
    ) -> Result<Option<ModeRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeRow,
            r#"SELECT mode_id, mode_group_id, mode_description,
             created_at, updated_at
	            FROM core.mode
                WHERE mode_group_id = $1"#,
            mode_group_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_mode_id(
        db: &PgPool,
        mode_id: Uuid,
    ) -> Result<Option<ModeRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeRow,
            r#"SELECT mode_id, mode_group_id, mode_description,
             created_at, updated_at
	            FROM core.mode
                WHERE mode_id = $1"#,
            mode_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_mode_description(
        db: &PgPool,
        mode_description: &str,
    ) -> Result<Option<ModeRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeRow,
            r#"SELECT mode_id, mode_group_id, mode_description,
             created_at, updated_at
	            FROM core.mode
                WHERE mode_description = $1"#,
            mode_description
        )
        .fetch_optional(db)
        .await
    }

    pub async fn create_mode(
        db: &PgPool,
        mode_group_id: Uuid,
        mode_description: &str,
    ) -> Result<ModeRow, sqlx::Error> {
        sqlx::query_as!(
            ModeRow,
            r#"INSERT INTO core.mode (mode_group_id, mode_group_name)
               VALUES ($1, $2)
               RETURNING mode_id, mode_group_id, mode_description,
                created_at, updated_at"#,
            mode_group_id, mode_description
        )
        .fetch_one(db)
        .await
    }

    pub async fn update_mode_description(
        db: &PgPool,
        mode_id: Uuid,
        mode_description: &str,
    ) -> Result<Option<ModeRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeRow,
            r#"UPDATE core.mode 
               SET mode_description = $2
               WHERE mode_id = $1
               RETURNING mode_id, mode_group_id, mode_description,
                created_at, updated_at"#,
            mode_id,
            mode_description
        )
        .fetch_optional(db)
        .await
    }

    pub async fn update_mode_group(
        db: &PgPool,
        mode_id: Uuid,
        mode_group_id: Uuid,
    ) -> Result<Option<ModeRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeRow,
            r#"UPDATE core.mode 
               SET mode_group_id = $2
               WHERE mode_id = $1
               RETURNING mode_id, mode_group_id, mode_description,
                created_at, updated_at"#,
            mode_id,
            mode_group_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn delete_mode_group(db: &PgPool, mode_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM core.mode WHERE mode_id = $1",
            mode_id
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists(db: &PgPool, mode_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.mode WHERE mode_id = $1
            )"#,
            mode_id
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }
}