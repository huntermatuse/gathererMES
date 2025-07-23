use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EquipmentMetadata {
    #[allow(dead_code)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Equipment {
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub equipment_type_id: Uuid,
    pub equipment_parent_id: Option<Uuid>,
    pub equipment_enabled: bool,
    pub equipment_metadata: Option<serde_json::Value>,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

pub struct EquipmentQueries;

impl EquipmentQueries {
    pub async fn get_all(db: &PgPool) -> Result<Vec<Equipment>, sqlx::Error> {
        sqlx::query_as!(
            Equipment,
            r#"SELECT equipment_id, equipment_name, equipment_type_id, 
                      equipment_parent_id, equipment_enabled, 
                      equipment_metadata,
                      created_at, updated_at
               FROM core.equipment
               ORDER BY equipment_id"#
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_id(
        db: &PgPool,
        equipment_id: Uuid,
    ) -> Result<Option<Equipment>, sqlx::Error> {
        sqlx::query_as!(
            Equipment,
            r#"SELECT equipment_id, equipment_name, equipment_type_id, 
                      equipment_parent_id, equipment_enabled, 
                      equipment_metadata,
                      created_at, updated_at
               FROM core.equipment
               WHERE equipment_id = $1"#,
            equipment_id
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_name(
        db: &PgPool,
        equipment_name: String,
    ) -> Result<Option<Equipment>, sqlx::Error> {
        sqlx::query_as!(
            Equipment,
            r#"SELECT equipment_id, equipment_name, equipment_type_id, 
                      equipment_parent_id, equipment_enabled, 
                      equipment_metadata,
                      created_at, updated_at
               FROM core.equipment
               WHERE equipment_name = $1"#,
            equipment_name
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_type_id(
        db: &PgPool,
        equipment_type_id: Uuid,
    ) -> Result<Vec<Equipment>, sqlx::Error> {
        sqlx::query_as!(
            Equipment,
            r#"SELECT equipment_id, equipment_name, equipment_type_id, 
                      equipment_parent_id, equipment_enabled, 
                      equipment_metadata,
                      created_at, updated_at
               FROM core.equipment
               WHERE equipment_type_id = $1
               ORDER BY equipment_name"#,
            equipment_type_id
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_parent_id(
        db: &PgPool,
        equipment_parent_id: Option<Uuid>,
    ) -> Result<Vec<Equipment>, sqlx::Error> {
        sqlx::query_as!(
            Equipment,
            r#"SELECT equipment_id, equipment_name, equipment_type_id, 
                      equipment_parent_id, equipment_enabled, 
                      equipment_metadata,
                      created_at, updated_at
               FROM core.equipment
               WHERE equipment_parent_id = $1 OR (equipment_parent_id IS NULL AND $1 IS NULL)
               ORDER BY equipment_name"#,
            equipment_parent_id
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_enabled(db: &PgPool) -> Result<Vec<Equipment>, sqlx::Error> {
        sqlx::query_as!(
            Equipment,
            r#"SELECT equipment_id, equipment_name, equipment_type_id, 
                      equipment_parent_id, equipment_enabled, 
                      equipment_metadata,
                      created_at, updated_at
               FROM core.equipment
               WHERE equipment_enabled = true
               ORDER BY equipment_name"#
        )
        .fetch_all(db)
        .await
    }

    pub async fn create(
        db: &PgPool,
        equipment_name: &str,
        equipment_type_id: Uuid,
        equipment_parent_id: Option<Uuid>,
        equipment_enabled: Option<bool>,
        equipment_metadata: Option<&serde_json::Value>,
    ) -> Result<Equipment, sqlx::Error> {
        let enabled = equipment_enabled.unwrap_or(true);
        let default_metadata = serde_json::json!({});
        let metadata = equipment_metadata.unwrap_or(&default_metadata);

        sqlx::query_as!(
            Equipment,
            r#"INSERT INTO core.equipment (equipment_name, equipment_type_id, equipment_parent_id, equipment_enabled, equipment_metadata)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING equipment_id, equipment_name, equipment_type_id, 
                         equipment_parent_id, equipment_enabled, 
                         equipment_metadata,
                         created_at, updated_at"#,
            equipment_name,
            equipment_type_id,
            equipment_parent_id,
            enabled,
            metadata
        )
        .fetch_one(db)
        .await
    }

    pub async fn delete(db: &PgPool, equipment_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM core.equipment WHERE equipment_id = $1",
            equipment_id
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn exists(db: &PgPool, equipment_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.equipment WHERE equipment_id = $1
            )"#,
            equipment_id
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }

    pub async fn set_enabled(
        db: &PgPool,
        equipment_id: Uuid,
        enabled: bool,
    ) -> Result<Option<Equipment>, sqlx::Error> {
        sqlx::query_as!(
            Equipment,
            r#"UPDATE core.equipment 
               SET equipment_enabled = $2, updated_at = NOW()
               WHERE equipment_id = $1
               RETURNING equipment_id, equipment_name, equipment_type_id, 
                         equipment_parent_id, equipment_enabled, 
                         equipment_metadata,
                         created_at, updated_at"#,
            equipment_id,
            enabled
        )
        .fetch_optional(db)
        .await
    }

    pub async fn update_metadata(
        db: &PgPool,
        equipment_id: Uuid,
        metadata: &serde_json::Value,
    ) -> Result<Option<Equipment>, sqlx::Error> {
        sqlx::query_as!(
            Equipment,
            r#"UPDATE core.equipment 
               SET equipment_metadata = $2, updated_at = NOW()
               WHERE equipment_id = $1
               RETURNING equipment_id, equipment_name, equipment_type_id, 
                         equipment_parent_id, equipment_enabled, 
                         equipment_metadata,
                         created_at, updated_at"#,
            equipment_id,
            metadata
        )
        .fetch_optional(db)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use serde_json::json;

    // helper to create a test equipment type
    async fn create_test_equipment_type(pool: &PgPool, name: &str) -> sqlx::Result<Uuid> {
        let result = sqlx::query_scalar!(
            "INSERT INTO core.equipment_type (type_name) VALUES ($1) RETURNING type_id",
            name
        )
        .fetch_one(pool)
        .await?;
        
        Ok(result)
    }

    #[sqlx::test]
    async fn test_create_equipment(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        let equipment_name = "Test Equipment";

        let equipment = EquipmentQueries::create(
            &pool,
            equipment_name,
            type_id,
            None,
            None,
            None,
        ).await?;

        assert_eq!(equipment.equipment_name, equipment_name);
        assert_eq!(equipment.equipment_type_id, type_id);
        assert!(equipment.equipment_parent_id.is_none());
        assert!(equipment.equipment_enabled);
        assert!(equipment.equipment_id != Uuid::nil());
        assert!(equipment.created_at.is_some());

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_equipment_with_parent(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        
        // Create parent equipment
        let parent = EquipmentQueries::create(
            &pool,
            "Parent Equipment",
            type_id,
            None,
            None,
            None,
        ).await?;

        // Create child equipment
        let child = EquipmentQueries::create(
            &pool,
            "Child Equipment",
            type_id,
            Some(parent.equipment_id),
            None,
            None,
        ).await?;

        assert_eq!(child.equipment_parent_id, Some(parent.equipment_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_equipment_with_metadata(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        let metadata = json!({"config": "test", "value": 42});

        let equipment = EquipmentQueries::create(
            &pool,
            "Test Equipment",
            type_id,
            None,
            None,
            Some(&metadata),
        ).await?;

        assert!(equipment.equipment_metadata.is_some());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_all_equipment(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        
        let eq1 = EquipmentQueries::create(&pool, "Equipment 1", type_id, None, None, None).await?;
        let eq2 = EquipmentQueries::create(&pool, "Equipment 2", type_id, None, None, None).await?;

        let all_equipment = EquipmentQueries::get_all(&pool).await?;

        assert!(all_equipment.len() >= 2);
        assert!(all_equipment.iter().any(|e| e.equipment_id == eq1.equipment_id));
        assert!(all_equipment.iter().any(|e| e.equipment_id == eq2.equipment_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_id(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        let created = EquipmentQueries::create(&pool, "Test Equipment", type_id, None, None, None).await?;

        let found = EquipmentQueries::get_by_id(&pool, created.equipment_id).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.equipment_id, created.equipment_id);
        assert_eq!(found.equipment_name, "Test Equipment");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_id_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = EquipmentQueries::get_by_id(&pool, random_id).await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_type_id(pool: PgPool) -> sqlx::Result<()> {
        let type_id1 = create_test_equipment_type(&pool, "Type 1").await?;
        let type_id2 = create_test_equipment_type(&pool, "Type 2").await?;

        let eq1 = EquipmentQueries::create(&pool, "Equipment 1", type_id1, None, None, None).await?;
        let _eq2 = EquipmentQueries::create(&pool, "Equipment 2", type_id2, None, None, None).await?;
        let eq3 = EquipmentQueries::create(&pool, "Equipment 3", type_id1, None, None, None).await?;

        let type1_equipment = EquipmentQueries::get_by_type_id(&pool, type_id1).await?;

        assert_eq!(type1_equipment.len(), 2);
        assert!(type1_equipment.iter().any(|e| e.equipment_id == eq1.equipment_id));
        assert!(type1_equipment.iter().any(|e| e.equipment_id == eq3.equipment_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_parent_id(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        
        let parent = EquipmentQueries::create(&pool, "Parent", type_id, None, None, None).await?;
        let child1 = EquipmentQueries::create(&pool, "Child 1", type_id, Some(parent.equipment_id), None, None).await?;
        let child2 = EquipmentQueries::create(&pool, "Child 2", type_id, Some(parent.equipment_id), None, None).await?;
        let _orphan = EquipmentQueries::create(&pool, "Orphan", type_id, None, None, None).await?;

        let children = EquipmentQueries::get_by_parent_id(&pool, Some(parent.equipment_id)).await?;

        assert_eq!(children.len(), 2);
        assert!(children.iter().any(|e| e.equipment_id == child1.equipment_id));
        assert!(children.iter().any(|e| e.equipment_id == child2.equipment_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_enabled(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        
        let enabled = EquipmentQueries::create(&pool, "Enabled", type_id, None, Some(true), None).await?;
        let disabled = EquipmentQueries::create(&pool, "Disabled", type_id, None, Some(false), None).await?;

        let enabled_equipment = EquipmentQueries::get_enabled(&pool).await?;

        assert!(enabled_equipment.iter().any(|e| e.equipment_id == enabled.equipment_id));
        assert!(!enabled_equipment.iter().any(|e| e.equipment_id == disabled.equipment_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_set_enabled(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        let created = EquipmentQueries::create(&pool, "Test Equipment", type_id, None, Some(true), None).await?;

        let updated = EquipmentQueries::set_enabled(&pool, created.equipment_id, false).await?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert!(!updated.equipment_enabled);
        assert_ne!(updated.updated_at, created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_metadata(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        let created = EquipmentQueries::create(&pool, "Test Equipment", type_id, None, None, None).await?;
        let new_metadata = json!({"updated": true, "version": 2});

        let updated = EquipmentQueries::update_metadata(&pool, created.equipment_id, &new_metadata).await?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert!(updated.equipment_metadata.is_some());
        assert_ne!(updated.updated_at, created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_equipment(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        let created = EquipmentQueries::create(&pool, "To Delete", type_id, None, None, None).await?;

        let deleted = EquipmentQueries::delete(&pool, created.equipment_id).await?;
        assert!(deleted);

        // Verify it's gone
        let found = EquipmentQueries::get_by_id(&pool, created.equipment_id).await?;
        assert!(found.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_nonexistent(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let deleted = EquipmentQueries::delete(&pool, random_id).await?;

        assert!(!deleted);

        Ok(())
    }

    #[sqlx::test]
    async fn test_exists(pool: PgPool) -> sqlx::Result<()> {
        let type_id = create_test_equipment_type(&pool, "Test Type").await?;
        let created = EquipmentQueries::create(&pool, "Exists Test", type_id, None, None, None).await?;

        let exists = EquipmentQueries::exists(&pool, created.equipment_id).await?;
        assert!(exists);

        let random_id = Uuid::new_v4();
        let not_exists = EquipmentQueries::exists(&pool, random_id).await?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_foreign_key_constraint(pool: PgPool) -> sqlx::Result<()> {
        let random_type_id = Uuid::new_v4();

        // Should fail due to foreign key constraint
        let result = EquipmentQueries::create(&pool, "Invalid", random_type_id, None, None, None).await;
        assert!(result.is_err());

        if let Err(sqlx::Error::Database(db_err)) = result {
            assert!(db_err.constraint().is_some());
        } else {
            panic!("Expected database constraint error");
        }

        Ok(())
    }
}