use anyhow::{Context, anyhow};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, error, instrument};
use uuid::Uuid;

const MAX_TYPE_NAME_LEN: usize = 255;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EquipmentTypeRow {
    pub type_id: Uuid,
    pub type_name: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

pub struct EquipmentTypeQueries;

impl EquipmentTypeQueries {
    /// Validates and sanitizes type name input
    fn validate_type_name(type_name: &str) -> anyhow::Result<String> {
        let trimmed = type_name.trim().to_string();

        if trimmed.is_empty() {
            return Err(anyhow!("type_name cannot be empty"));
        }

        if trimmed.len() > MAX_TYPE_NAME_LEN {
            return Err(anyhow!(
                "type_name exceeds max length of {} characters",
                MAX_TYPE_NAME_LEN
            ));
        }

        Ok(trimmed)
    }

    pub async fn get_all(db: &PgPool) -> Result<Vec<EquipmentTypeRow>, sqlx::Error> {
        sqlx::query_as!(
            EquipmentTypeRow,
            r#"SELECT type_id, type_name, created_at, updated_at
               FROM core.equipment_type
               ORDER BY type_name"#
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
        type_name: &str,
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

    #[instrument(skip(db), fields(name = %type_name))]
    pub async fn create(db: &PgPool, type_name: &str) -> anyhow::Result<EquipmentTypeRow> {
        let validated_name = Self::validate_type_name(type_name)?;

        // Check for duplicate name
        if let Some(_) = sqlx::query_scalar!(
            "SELECT 1 FROM core.equipment_type WHERE type_name = $1",
            validated_name
        )
        .fetch_optional(db)
        .await
        .context("Failed to check for duplicate type_name")?
        {
            error!("Rejected: duplicate type_name '{}'", validated_name);
            return Err(anyhow!("type_name '{}' already exists", validated_name));
        }

        debug!("Creating equipment type '{}'", validated_name);
        let result = sqlx::query_as!(
            EquipmentTypeRow,
            r#"INSERT INTO core.equipment_type (type_name)
               VALUES ($1)
               RETURNING type_id, type_name, created_at, updated_at"#,
            validated_name
        )
        .fetch_one(db)
        .await
        .with_context(|| {
            format!(
                "Failed to create equipment type with name '{}'",
                validated_name
            )
        })?;

        debug!(?result, "Successfully created equipment type");
        Ok(result)
    }

    #[instrument(skip(db), fields(id = %type_id, name = %type_name))]
    pub async fn update(
        db: &PgPool,
        type_id: Uuid,
        type_name: &str,
    ) -> anyhow::Result<Option<EquipmentTypeRow>> {
        let validated_name = Self::validate_type_name(type_name)?;

        // Check for duplicate name (excluding current record)
        if let Some(_) = sqlx::query_scalar!(
            "SELECT 1 FROM core.equipment_type WHERE type_name = $1 AND type_id != $2",
            validated_name,
            type_id
        )
        .fetch_optional(db)
        .await
        .context("Failed to check for duplicate type_name")?
        {
            return Err(anyhow!("type_name '{}' already exists", validated_name));
        }

        debug!(
            "Updating equipment type {} to '{}'",
            type_id, validated_name
        );
        let result = sqlx::query_as!(
            EquipmentTypeRow,
            r#"UPDATE core.equipment_type 
               SET type_name = $2, updated_at = NOW()
               WHERE type_id = $1
               RETURNING type_id, type_name, created_at, updated_at"#,
            type_id,
            validated_name
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update equipment type for id {}", type_id))?;

        if result.is_some() {
            debug!("Successfully updated equipment type {}", type_id);
        } else {
            debug!("Equipment type {} not found for update", type_id);
        }

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %type_id))]
    pub async fn delete(db: &PgPool, type_id: Uuid) -> anyhow::Result<bool> {
        // TODO: Add check to see if the equipment_type is being used anywhere
        // This would prevent deletion of types that are in use

        debug!("Deleting equipment type {}", type_id);
        let result = sqlx::query!(
            "DELETE FROM core.equipment_type WHERE type_id = $1",
            type_id
        )
        .execute(db)
        .await
        .with_context(|| format!("Failed to delete equipment type with id {}", type_id))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            debug!("Successfully deleted equipment type {}", type_id);
        } else {
            debug!("Equipment type {} not found for deletion", type_id);
        }

        Ok(deleted)
    }

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

    /// Check if an equipment type name exists (case-insensitive)
    pub async fn name_exists(db: &PgPool, type_name: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
            SELECT 1 FROM core.equipment_type WHERE type_name = $1
        )"#,
            type_name.trim()
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// get equipment types that contain the search term (case-insensitive)
    pub async fn search_by_name(
        db: &PgPool,
        search_term: &str,
    ) -> Result<Vec<EquipmentTypeRow>, sqlx::Error> {
        let pattern = format!("%{}%", search_term.trim().to_lowercase());

        sqlx::query_as!(
            EquipmentTypeRow,
            r#"SELECT type_id, type_name, created_at, updated_at
           FROM core.equipment_type
           WHERE lower(type_name::text COLLATE "C") LIKE $1
           ORDER BY type_name"#,
            pattern
        )
        .fetch_all(db)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // helper to create equipment type bypassing validation for test setup
    async fn create_equipment_type_raw(
        pool: &PgPool,
        type_name: &str,
    ) -> sqlx::Result<EquipmentTypeRow> {
        sqlx::query_as!(
            EquipmentTypeRow,
            r#"INSERT INTO core.equipment_type (type_name)
               VALUES ($1)
               RETURNING type_id, type_name, created_at, updated_at"#,
            type_name
        )
        .fetch_one(pool)
        .await
    }

    #[sqlx::test]
    async fn test_create_equipment_type(pool: PgPool) -> sqlx::Result<()> {
        let type_name = "Test Equipment Type";

        let equipment_type = EquipmentTypeQueries::create(&pool, type_name)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(equipment_type.type_name, type_name);
        assert!(equipment_type.type_id != Uuid::nil());
        assert!(equipment_type.created_at.is_some());

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_validation_errors(pool: PgPool) -> sqlx::Result<()> {
        // test empty name
        let result = EquipmentTypeQueries::create(&pool, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // test whitespace-only name
        let result = EquipmentTypeQueries::create(&pool, "   ").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // test too long name
        let long_name = "a".repeat(300);
        let result = EquipmentTypeQueries::create(&pool, &long_name).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds max length")
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_duplicate_name(pool: PgPool) -> sqlx::Result<()> {
        let type_name = "Duplicate Test Type";

        // first creation should succeed
        let _first = EquipmentTypeQueries::create(&pool, type_name)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // second creation with same name should fail
        let result = EquipmentTypeQueries::create(&pool, type_name).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_input_trimming(pool: PgPool) -> sqlx::Result<()> {
        // Test that inputs are properly trimmed
        let created = EquipmentTypeQueries::create(&pool, "  Trimmed Type  ")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created.type_name, "Trimmed Type");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_all_equipment_types(pool: PgPool) -> sqlx::Result<()> {
        // Creating some test data
        let type1 = EquipmentTypeQueries::create(&pool, "Type 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let type2 = EquipmentTypeQueries::create(&pool, "Type 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let all_types = EquipmentTypeQueries::get_all(&pool).await?;

        assert!(all_types.len() >= 2);
        assert!(all_types.iter().any(|t| t.type_id == type1.type_id));
        assert!(all_types.iter().any(|t| t.type_id == type2.type_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_id(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "Test Type")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

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
    async fn test_get_by_name(pool: PgPool) -> sqlx::Result<()> {
        let type_name = "Unique Type Name";
        let created = EquipmentTypeQueries::create(&pool, type_name)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = EquipmentTypeQueries::get_by_name(&pool, type_name).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.type_id, created.type_id);
        assert_eq!(found.type_name, type_name);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_name_not_found(pool: PgPool) -> sqlx::Result<()> {
        let result = EquipmentTypeQueries::get_by_name(&pool, "Non-existent Type").await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_equipment_type(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "Original Name")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let new_name = "Updated Name";

        let updated = EquipmentTypeQueries::update(&pool, created.type_id, new_name)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.type_id, created.type_id);
        assert_eq!(updated.type_name, new_name);
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_validation_errors(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "Original Name")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test empty name
        let result = EquipmentTypeQueries::update(&pool, created.type_id, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test whitespace-only name
        let result = EquipmentTypeQueries::update(&pool, created.type_id, "   ").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test too long name
        let long_name = "a".repeat(300);
        let result = EquipmentTypeQueries::update(&pool, created.type_id, &long_name).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds max length")
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_duplicate_name(pool: PgPool) -> sqlx::Result<()> {
        let _first = EquipmentTypeQueries::create(&pool, "First Type")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let second = EquipmentTypeQueries::create(&pool, "Second Type")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to update second type's name to match first type's name
        let result = EquipmentTypeQueries::update(&pool, second.type_id, "First Type").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_nonexistent(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = EquipmentTypeQueries::update(&pool, random_id, "New Name")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_preserves_timestamps(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "Test Type")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let original_created_at = created.created_at;

        // Short delay to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let updated = EquipmentTypeQueries::update(&pool, created.type_id, "Updated Type")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();

        // created_at should remain the same
        assert_eq!(updated.created_at, original_created_at);

        // updated_at should be different (newer)
        if let (Some(original_updated), Some(new_updated)) =
            (created.updated_at, updated.updated_at)
        {
            assert!(new_updated > original_updated);
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_equipment_type(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "To Delete")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let deleted = EquipmentTypeQueries::delete(&pool, created.type_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(deleted);

        // Verify it's gone
        let found = EquipmentTypeQueries::get_by_id(&pool, created.type_id).await?;
        assert!(found.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_nonexistent(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let deleted = EquipmentTypeQueries::delete(&pool, random_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(!deleted);

        Ok(())
    }

    #[sqlx::test]
    async fn test_exists(pool: PgPool) -> sqlx::Result<()> {
        let created = EquipmentTypeQueries::create(&pool, "Exists Test")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let exists = EquipmentTypeQueries::exists(&pool, created.type_id).await?;
        assert!(exists);

        let random_id = Uuid::new_v4();
        let not_exists = EquipmentTypeQueries::exists(&pool, random_id).await?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_name_exists(pool: PgPool) -> sqlx::Result<()> {
        let type_name = "Exists Test Type";
        let _created = EquipmentTypeQueries::create(&pool, type_name)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test exact match
        let exists = EquipmentTypeQueries::name_exists(&pool, type_name).await?;
        assert!(exists);

        // Test case insensitive match
        let exists_upper = EquipmentTypeQueries::name_exists(&pool, "EXISTS TEST TYPE").await?;
        assert!(exists_upper);

        // Test non-existent
        let not_exists = EquipmentTypeQueries::name_exists(&pool, "Does Not Exist").await?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_search_by_name(pool: PgPool) -> sqlx::Result<()> {
        // Create test data
        let _pump = EquipmentTypeQueries::create(&pool, "Water Pump")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _motor = EquipmentTypeQueries::create(&pool, "Electric Motor")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _valve = EquipmentTypeQueries::create(&pool, "Control Valve")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Search for "pump"
        let pump_results = EquipmentTypeQueries::search_by_name(&pool, "pump").await?;
        assert!(pump_results.len() >= 1);
        assert!(pump_results.iter().any(|t| t.type_name.contains("Pump")));

        // Search for "motor"
        let motor_results = EquipmentTypeQueries::search_by_name(&pool, "MOTOR").await?;
        assert!(motor_results.len() >= 1);
        assert!(motor_results.iter().any(|t| t.type_name.contains("Motor")));

        // Search for non-existent term
        let empty_results = EquipmentTypeQueries::search_by_name(&pool, "NonExistent").await?;
        assert!(empty_results.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn test_multiple_operations_sequence(pool: PgPool) -> sqlx::Result<()> {
        // Test a sequence of operations to ensure they work together

        // Create
        let created = EquipmentTypeQueries::create(&pool, "Sequence Test Type")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Verify it exists
        let exists = EquipmentTypeQueries::exists(&pool, created.type_id).await?;
        assert!(exists);

        // Verify name exists
        let name_exists = EquipmentTypeQueries::name_exists(&pool, "Sequence Test Type").await?;
        assert!(name_exists);

        // Update
        let updated = EquipmentTypeQueries::update(&pool, created.type_id, "Updated Sequence Type")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated.is_some());

        // Verify updated name
        let final_state = EquipmentTypeQueries::get_by_id(&pool, created.type_id).await?;
        assert!(final_state.is_some());
        assert_eq!(final_state.unwrap().type_name, "Updated Sequence Type");

        // Search should find it
        let search_results = EquipmentTypeQueries::search_by_name(&pool, "sequence").await?;
        assert!(search_results.iter().any(|t| t.type_id == created.type_id));

        // Delete
        let deleted = EquipmentTypeQueries::delete(&pool, created.type_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(deleted);

        // Verify it no longer exists
        let exists_after_delete = EquipmentTypeQueries::exists(&pool, created.type_id).await?;
        assert!(!exists_after_delete);

        Ok(())
    }
}
