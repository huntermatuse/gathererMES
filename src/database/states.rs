use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StateRow {
    pub state_id: Uuid,
    pub state_group_id: Uuid,
    pub state_description: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

pub struct StateRowQueries;

impl StateRowQueries {
    pub async fn get_all(db: &PgPool) -> Result<Vec<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, 
                state_description, created_at, updated_at
	            FROM core.state;"#
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_state_group_id(
        db: &PgPool,
        state_group_id: Uuid,
    ) -> Result<Option<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, state_description,
             created_at, updated_at
	            FROM core.state
                WHERE state_group_id = $1"#,
            state_group_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_state_id(
        db: &PgPool,
        state_id: Uuid,
    ) -> Result<Option<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, state_description,
             created_at, updated_at
	            FROM core.state
                WHERE state_id = $1"#,
            state_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_state_description(
        db: &PgPool,
        state_description: &str,
    ) -> Result<Option<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, state_description,
             created_at, updated_at
	            FROM core.state
                WHERE state_description = $1"#,
            state_description
        )
        .fetch_optional(db)
        .await
    }

    pub async fn create_mode(
        db: &PgPool,
        state_group_id: Uuid,
        state_description: &str,
    ) -> Result<StateRow, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"INSERT INTO core.mode (state_group_id, state_group_name)
               VALUES ($1, $2)
               RETURNING state_id, state_group_id, state_description,
                created_at, updated_at"#,
            state_group_id, state_description
        )
        .fetch_one(db)
        .await
    }

    pub async fn update_state_description(
        db: &PgPool,
        state_id: Uuid,
        state_description: &str,
    ) -> Result<Option<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"UPDATE core.state 
               SET state_description = $2
               WHERE state_id = $1
               RETURNING state_id, state_group_id, state_description,
                created_at, updated_at"#,
            state_id,
            state_description
        )
        .fetch_optional(db)
        .await
    }

    pub async fn update_state_group(
        db: &PgPool,
        state_id: Uuid,
        state_group_id: Uuid,
    ) -> Result<Option<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"UPDATE core.state 
               SET state_group_id = $2
               WHERE state_id = $1
               RETURNING state_id, state_group_id, state_description,
                created_at, updated_at"#,
            state_id,
            state_group_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn delete_state_group(db: &PgPool, state_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM core.state WHERE state_id = $1",
            state_id
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists(db: &PgPool, state_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.state WHERE state_id = $1
            )"#,
            state_id
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }
}