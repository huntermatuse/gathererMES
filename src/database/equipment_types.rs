use sqlx::PgPool;
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EquipmentTypeRow {
    pub type_id: Uuid,
    pub type_name: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

pub struct EquipmentTypeQueries;

impl EquipmentTypeQueries {
    pub async fn get_all(db: &PgPool) -> Result<Vec<EquipmentTypeRow>, sqlx::Error> {
        sqlx::query_as!(
            EquipmentTypeRow,
            r#"SELECT type_id, type_name, created_at, updated_at
               FROM core.equipment_type
               ORDER BY type_id"#
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_id(
        db: &PgPool,
        type_id: Uuid,
    ) -> Result<Option<EquipmentTypeRow>, sqlx::Error> {
        sqlx::query_as!(
            EquipmentTypeRow,
            r#"SELECT type_id, type_name, created_at, updated_at
               FROM core.equipment_type
               WHERE type_id = $1"#,
            type_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn create(db: &PgPool, type_name: &str) -> Result<EquipmentTypeRow, sqlx::Error> {
        sqlx::query_as!(
            EquipmentTypeRow,
            r#"INSERT INTO core.equipment_type (type_id, type_name)
               VALUES (gen_random_uuid(), $1)
               RETURNING type_id, type_name, created_at, updated_at"#,
            type_name
        )
        .fetch_one(db)
        .await
    }

    pub async fn update(
        db: &PgPool,
        type_id: Uuid,
        type_name: &str,
    ) -> Result<Option<EquipmentTypeRow>, sqlx::Error> {
        sqlx::query_as!(
            EquipmentTypeRow,
            r#"UPDATE core.equipment_type 
               SET type_name = $2, updated_at = NOW()
               WHERE type_id = $1
               RETURNING type_id, type_name, created_at, updated_at"#,
            type_id,
            type_name
        )
        .fetch_optional(db)
        .await
    }

    pub async fn delete(db: &PgPool, type_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM core.equipment_type WHERE type_id = $1",
            type_id
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
