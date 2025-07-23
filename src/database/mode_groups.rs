use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModeGroupRow {
    pub mode_group_id: Uuid,
    pub mode_group_name: String,
    pub mode_group_description: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

pub struct ModeGroupQueries;

impl ModeGroupQueries {
    pub async fn get_all(db: &PgPool) -> Result<Vec<ModeGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeGroupRow,
            r#"SELECT mode_group_id, mode_group_name, 
                mode_group_description, created_at, updated_at
	            FROM core.mode_group;"#
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_mode_group_id(
        db: &PgPool,
        mode_group_id: Uuid,
    ) -> Result<Option<ModeGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeGroupRow,
            r#"SELECT mode_group_id, mode_group_name, mode_group_description, created_at, updated_at
	            FROM core.mode_group
                WHERE mode_group_id = $1"#,
            mode_group_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_mode_group_name(
        db: &PgPool,
        mode_group_name: &str,
    ) -> Result<Option<ModeGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeGroupRow,
            r#"SELECT mode_group_id, mode_group_name, mode_group_description, created_at, updated_at
	            FROM core.mode_group
                WHERE mode_group_name = $1"#,
            mode_group_name
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_mode_group_description(
        db: &PgPool,
        mode_group_description: &str,
    ) -> Result<Option<ModeGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeGroupRow,
            r#"SELECT mode_group_id, mode_group_name, mode_group_description, created_at, updated_at
	            FROM core.mode_group
                WHERE mode_group_description = $1"#,
            mode_group_description
        )
        .fetch_optional(db)
        .await
    }

    pub async fn create_mode_group(
        db: &PgPool,
        mode_group_name: &str,
        mode_group_description: &str,
    ) -> Result<ModeGroupRow, sqlx::Error> {
        sqlx::query_as!(
            ModeGroupRow,
            r#"INSERT INTO core.mode_group (mode_group_name, mode_group_description)
               VALUES ($1, $2)
               RETURNING mode_group_id, mode_group_name, mode_group_description, created_at, updated_at"#,
            mode_group_name, mode_group_description
        )
        .fetch_one(db)
        .await
    }

    pub async fn update_mode_group_name(
        db: &PgPool,
        mode_group_id: Uuid,
        mode_group_name: &str,
    ) -> Result<Option<ModeGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeGroupRow,
            r#"UPDATE core.mode_group 
               SET mode_group_name = $2
               WHERE mode_group_id = $1
               RETURNING mode_group_id, mode_group_name, mode_group_description, created_at, updated_at"#,
            mode_group_id,
            mode_group_name
        )
        .fetch_optional(db)
        .await
    }

    pub async fn update_mode_group_description(
        db: &PgPool,
        mode_group_id: Uuid,
        mode_group_description: &str,
    ) -> Result<Option<ModeGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeGroupRow,
            r#"UPDATE core.mode_group 
               SET mode_group_description = $2
               WHERE mode_group_id = $1
               RETURNING mode_group_id, mode_group_name, mode_group_description, created_at, updated_at"#,
            mode_group_id,
            mode_group_description
        )
        .fetch_optional(db)
        .await
    }

    pub async fn delete_mode_group(db: &PgPool, mode_group_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM core.mode_group WHERE mode_group_id = $1",
            mode_group_id
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists(db: &PgPool, mode_group_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.mode_group WHERE mode_group_id = $1
            )"#,
            mode_group_id
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }
}