use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StateGroupRow {
    pub state_group_id: Uuid,
    pub state_group_name: String,
    pub state_group_description: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

pub struct StateGroupQueries;

impl StateGroupQueries {
    pub async fn get_all(db: &PgPool) -> Result<Vec<StateGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            StateGroupRow,
            r#"SELECT state_group_id, state_group_name, 
                state_group_description, created_at, updated_at
	            FROM core.state_group"#
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_state_group_id(
        db: &PgPool,
        state_group_id: Uuid,
    ) -> Result<Option<StateGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            StateGroupRow,
            r#"SELECT state_group_id, state_group_name, state_group_description, created_at, updated_at
	            FROM core.state_group
                WHERE state_group_id = $1"#,
            state_group_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_state_group_name(
        db: &PgPool,
        state_group_name: &str,
    ) -> Result<Option<StateGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            StateGroupRow,
            r#"SELECT state_group_id, state_group_name, state_group_description, created_at, updated_at
	            FROM core.state_group
                WHERE state_group_name = $1"#,
            state_group_name
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_state_group_description(
        db: &PgPool,
        state_group_description: &str,
    ) -> Result<Option<StateGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            StateGroupRow,
            r#"SELECT state_group_id, state_group_name, state_group_description, created_at, updated_at
	            FROM core.state_group
                WHERE state_group_description = $1"#,
            state_group_description
        )
        .fetch_optional(db)
        .await
    }

    pub async fn create_state_group(
        db: &PgPool,
        state_group_name: &str,
        state_group_description: &str,
    ) -> Result<StateGroupRow, sqlx::Error> {
        sqlx::query_as!(
            StateGroupRow,
            r#"INSERT INTO core.state_group (state_group_name, state_group_description)
               VALUES ($1, $2)
               RETURNING state_group_id, state_group_name, state_group_description, created_at, updated_at"#,
            state_group_name, state_group_description
        )
        .fetch_one(db)
        .await
    }

    pub async fn update_state_group_name(
        db: &PgPool,
        state_group_id: Uuid,
        state_group_name: &str,
    ) -> Result<Option<StateGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            StateGroupRow,
            r#"UPDATE core.state_group 
               SET state_group_name = $2
               WHERE state_group_id = $1
               RETURNING state_group_id, state_group_name, state_group_description, created_at, updated_at"#,
            state_group_id,
            state_group_name
        )
        .fetch_optional(db)
        .await
    }

    pub async fn update_state_group_description(
        db: &PgPool,
        state_group_id: Uuid,
        state_group_description: &str,
    ) -> Result<Option<StateGroupRow>, sqlx::Error> {
        sqlx::query_as!(
            StateGroupRow,
            r#"UPDATE core.state_group 
               SET state_group_description = $2
               WHERE state_group_id = $1
               RETURNING state_group_id, state_group_name, state_group_description, created_at, updated_at"#,
            state_group_id,
            state_group_description
        )
        .fetch_optional(db)
        .await
    }

    pub async fn delete_state_group(db: &PgPool, state_group_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM core.state_group WHERE state_group_id = $1",
            state_group_id
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists(db: &PgPool, state_group_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.state_group WHERE state_group_id = $1
            )"#,
            state_group_id
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }
}