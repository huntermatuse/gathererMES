use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

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

    pub async fn get_by_name(
        db: &PgPool,
        type_name: String,
    ) -> Result<Option<EquipmentTypeRow>, sqlx::Error> {
        sqlx::query_as!(
            EquipmentTypeRow,
            r#"SELECT type_id, type_name, created_at, updated_at
               FROM core.equipment_type
               WHERE type_name = $1"#,
            type_name
        )
        .fetch_optional(db)
        .await
    }

    pub async fn create(db: &PgPool, type_name: &str) -> Result<EquipmentTypeRow, sqlx::Error> {
        sqlx::query_as!(
            EquipmentTypeRow,
            r#"INSERT INTO core.equipment_type (type_name)
               VALUES ($1)
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

    // TODO: logic to see if the equipment_type is being used anywhere
    pub async fn delete(db: &PgPool, type_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM core.equipment_type WHERE type_id = $1",
            type_id
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    #[allow(dead_code)] // using this in the test currently
    pub async fn exists(db: &PgPool, type_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.equipment_type WHERE type_id = $1
            )"#,
            type_id
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // TODO: Test to see if updated_at column is working on update

    #[sqlx::test]
    async fn test_create_equipment_type(pool: PgPool) -> sqlx::Result<()> {
        let type_name = "Test Equipment Type";

        let equipment_type = EquipmentTypeQueries::create(&pool, type_name).await?;
        dbg!(&equipment_type);
        assert_eq!(equipment_type.type_name, type_name);
        assert!(equipment_type.type_id != Uuid::nil());
        assert!(equipment_type.created_at.is_some());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_all_equipment_types(pool: PgPool) -> sqlx::Result<()> {
        // creating some test data
        let type1 = EquipmentTypeQueries::create(&pool, "Type 1").await?;
        let type2 = EquipmentTypeQueries::create(&pool, "Type 2").await?;

        let all_types = EquipmentTypeQueries::get_all(&pool).await?;

        assert!(all_types.len() >= 2);
        assert!(all_types.iter().any(|t| t.type_id == type1.type_id));
        assert!(all_types.iter().any(|t| t.type_id == type2.type_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_id(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "Test Type").await?;

        let found = EquipmentTypeQueries::get_by_id(&pool, created.type_id).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.type_id, created.type_id);
        assert_eq!(found.type_name, "Test Type");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_id_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = EquipmentTypeQueries::get_by_id(&pool, random_id).await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_equipment_type(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "Original Name").await?;
        let new_name = "Updated Name";

        let updated = EquipmentTypeQueries::update(&pool, created.type_id, new_name).await?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.type_id, created.type_id);
        assert_eq!(updated.type_name, new_name);
        assert_ne!(updated.updated_at, created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_nonexistent(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = EquipmentTypeQueries::update(&pool, random_id, "New Name").await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_equipment_type(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "To Delete").await?;

        let deleted = EquipmentTypeQueries::delete(&pool, created.type_id).await?;
        assert!(deleted);

        // verify it's gone
        let found = EquipmentTypeQueries::get_by_id(&pool, created.type_id).await?;
        assert!(found.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_nonexistent(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let deleted = EquipmentTypeQueries::delete(&pool, random_id).await?;

        assert!(!deleted);

        Ok(())
    }

    #[sqlx::test]
    async fn test_unique_constraint(pool: PgPool) -> sqlx::Result<()> {
        let type_name = "Unique Name";

        // First creation should succeed
        let _first = EquipmentTypeQueries::create(&pool, type_name).await?;

        // Second creation with same name should fail
        let result = EquipmentTypeQueries::create(&pool, type_name).await;
        assert!(result.is_err());

        // Verify it's a constraint error
        if let Err(sqlx::Error::Database(db_err)) = result {
            assert!(db_err.constraint().is_some());
        } else {
            panic!("Expected database constraint error");
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_exists(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "Exists Test").await?;

        let exists = EquipmentTypeQueries::exists(&pool, created.type_id).await?;
        assert!(exists);

        let random_id = Uuid::new_v4();
        let not_exists = EquipmentTypeQueries::exists(&pool, random_id).await?;
        assert!(!not_exists);

        Ok(())
    }
}
