use anyhow::{Context, anyhow};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, error, instrument};
use uuid::Uuid;

const MAX_DESC_LEN: usize = 2048;

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
    /// Validates and sanitizes mode description input
    fn validate_mode_description(mode_description: &str) -> anyhow::Result<String> {
        let trimmed = mode_description.trim().to_string();

        if trimmed.is_empty() {
            return Err(anyhow!("mode_description cannot be empty"));
        }

        if trimmed.len() > MAX_DESC_LEN {
            return Err(anyhow!(
                "mode_description exceeds max length of {} characters",
                MAX_DESC_LEN
            ));
        }

        Ok(trimmed)
    }

    pub async fn get_all(db: &PgPool) -> Result<Vec<ModeRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeRow,
            r#"SELECT mode_id, mode_group_id, 
                mode_description, created_at, updated_at
	            FROM core.mode
                ORDER BY mode_description"#
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_mode_group_id(
        db: &PgPool,
        mode_group_id: Uuid,
    ) -> Result<Vec<ModeRow>, sqlx::Error> {
        sqlx::query_as!(
            ModeRow,
            r#"SELECT mode_id, mode_group_id, mode_description,
             created_at, updated_at
	            FROM core.mode
                WHERE mode_group_id = $1
                ORDER BY mode_description"#,
            mode_group_id
        )
        .fetch_all(db)
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

    #[instrument(skip(db), fields(group_id = %mode_group_id, description = %mode_description))]
    pub async fn create_mode(
        db: &PgPool,
        mode_group_id: Uuid,
        mode_description: &str,
    ) -> anyhow::Result<ModeRow> {
        let validated_description = Self::validate_mode_description(mode_description)?;

        // Check if mode_group_id exists
        let group_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM core.mode_group WHERE mode_group_id = $1)",
            mode_group_id
        )
        .fetch_one(db)
        .await
        .context("Failed to check if mode_group exists")?;

        if !group_exists.unwrap_or(false) {
            error!("Rejected: mode_group_id {} does not exist", mode_group_id);
            return Err(anyhow!("mode_group_id '{}' does not exist", mode_group_id));
        }

        // Check for duplicate description within the same mode group
        if let Some(_) = sqlx::query_scalar!(
            "SELECT 1 FROM core.mode WHERE mode_group_id = $1 AND mode_description = $2",
            mode_group_id,
            validated_description
        )
        .fetch_optional(db)
        .await
        .context("Failed to check for duplicate mode_description in mode_group")?
        {
            error!(
                "Rejected: duplicate mode_description '{}' in mode_group {}",
                validated_description, mode_group_id
            );
            return Err(anyhow!(
                "mode_description '{}' already exists in this mode group",
                validated_description
            ));
        }

        debug!(
            "Creating mode with description '{}' in group {}",
            validated_description, mode_group_id
        );
        let result = sqlx::query_as!(
            ModeRow,
            r#"INSERT INTO core.mode (mode_group_id, mode_description)
               VALUES ($1, $2)
               RETURNING mode_id, mode_group_id, mode_description,
                created_at, updated_at"#,
            mode_group_id,
            validated_description
        )
        .fetch_one(db)
        .await
        .with_context(|| {
            format!(
                "Failed to create mode with description '{}' in group {}",
                validated_description, mode_group_id
            )
        })?;

        debug!(?result, "Successfully created mode");
        Ok(result)
    }

    #[instrument(skip(db), fields(id = %mode_id, description = %mode_description))]
    pub async fn update_mode_description(
        db: &PgPool,
        mode_id: Uuid,
        mode_description: &str,
    ) -> anyhow::Result<Option<ModeRow>> {
        let validated_description = Self::validate_mode_description(mode_description)?;

        // Get the current mode to check for duplicate in same group
        let current_mode = sqlx::query!(
            "SELECT mode_group_id FROM core.mode WHERE mode_id = $1",
            mode_id
        )
        .fetch_optional(db)
        .await
        .context("Failed to fetch current mode")?;

        if let Some(current) = current_mode {
            // Check for duplicate description within the same mode group (excluding current record)
            if let Some(_) = sqlx::query_scalar!(
                "SELECT 1 FROM core.mode WHERE mode_group_id = $1 AND mode_description = $2 AND mode_id != $3",
                current.mode_group_id,
                validated_description,
                mode_id
            )
            .fetch_optional(db)
            .await
            .context("Failed to check for duplicate mode_description")?
            {
                return Err(anyhow!(
                    "mode_description '{}' already exists in this mode group",
                    validated_description
                ));
            }
        }

        debug!(
            "Updating mode {} to description '{}'",
            mode_id, validated_description
        );
        let result = sqlx::query_as!(
            ModeRow,
            r#"UPDATE core.mode 
               SET mode_description = $2, updated_at = NOW()
               WHERE mode_id = $1
               RETURNING mode_id, mode_group_id, mode_description,
                created_at, updated_at"#,
            mode_id,
            validated_description
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update mode description for id {}", mode_id))?;

        if result.is_some() {
            debug!("Successfully updated mode {}", mode_id);
        } else {
            debug!("Mode {} not found for update", mode_id);
        }

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %mode_id, new_group_id = %mode_group_id))]
    pub async fn update_mode_group(
        db: &PgPool,
        mode_id: Uuid,
        mode_group_id: Uuid,
    ) -> anyhow::Result<Option<ModeRow>> {
        // Check if new mode_group_id exists
        let group_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM core.mode_group WHERE mode_group_id = $1)",
            mode_group_id
        )
        .fetch_one(db)
        .await
        .context("Failed to check if mode_group exists")?;

        if !group_exists.unwrap_or(false) {
            return Err(anyhow!("mode_group_id '{}' does not exist", mode_group_id));
        }

        // Get current mode to check for conflicts
        let current_mode = sqlx::query!(
            "SELECT mode_description FROM core.mode WHERE mode_id = $1",
            mode_id
        )
        .fetch_optional(db)
        .await
        .context("Failed to fetch current mode")?;

        if let Some(current) = current_mode {
            // Check for duplicate description in the new mode group
            if let Some(_) = sqlx::query_scalar!(
                "SELECT 1 FROM core.mode WHERE mode_group_id = $1 AND mode_description = $2",
                mode_group_id,
                current.mode_description
            )
            .fetch_optional(db)
            .await
            .context("Failed to check for duplicate mode_description in new group")?
            {
                return Err(anyhow!(
                    "mode_description '{}' already exists in the target mode group",
                    current.mode_description
                ));
            }
        }

        debug!("Updating mode {} to group {}", mode_id, mode_group_id);
        let result = sqlx::query_as!(
            ModeRow,
            r#"UPDATE core.mode 
               SET mode_group_id = $2, updated_at = NOW()
               WHERE mode_id = $1
               RETURNING mode_id, mode_group_id, mode_description,
                created_at, updated_at"#,
            mode_id,
            mode_group_id
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update mode group for id {}", mode_id))?;

        if result.is_some() {
            debug!("Successfully updated mode {} group", mode_id);
        } else {
            debug!("Mode {} not found for update", mode_id);
        }

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %mode_id))]
    pub async fn delete_mode(db: &PgPool, mode_id: Uuid) -> anyhow::Result<bool> {
        // TODO: Add check to see if the mode is being used anywhere
        // This would prevent deletion of modes that are in use

        debug!("Deleting mode {}", mode_id);
        let result = sqlx::query!("DELETE FROM core.mode WHERE mode_id = $1", mode_id)
            .execute(db)
            .await
            .with_context(|| format!("Failed to delete mode with id {}", mode_id))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            debug!("Successfully deleted mode {}", mode_id);
        } else {
            debug!("Mode {} not found for deletion", mode_id);
        }

        Ok(deleted)
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

    /// Check if a mode description exists within a specific mode group
    pub async fn description_exists_in_group(
        db: &PgPool,
        mode_group_id: Uuid,
        mode_description: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.mode 
                WHERE mode_group_id = $1 AND mode_description = $2
            )"#,
            mode_group_id,
            mode_description.trim()
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Search modes by description (case-insensitive)
    pub async fn search_by_description(
        db: &PgPool,
        search_term: &str,
    ) -> Result<Vec<ModeRow>, sqlx::Error> {
        let pattern = format!("%{}%", search_term.trim().to_lowercase());

        sqlx::query_as!(
            ModeRow,
            r#"SELECT mode_id, mode_group_id, mode_description, created_at, updated_at
               FROM core.mode
               WHERE lower(mode_description::text COLLATE "C") LIKE $1
               ORDER BY mode_description"#,
            pattern
        )
        .fetch_all(db)
        .await
    }

    /// Get all modes for a specific mode group with validation
    pub async fn get_modes_for_group(
        db: &PgPool,
        mode_group_id: Uuid,
    ) -> anyhow::Result<Vec<ModeRow>> {
        // Check if mode_group exists first
        let group_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM core.mode_group WHERE mode_group_id = $1)",
            mode_group_id
        )
        .fetch_one(db)
        .await
        .context("Failed to check if mode_group exists")?;

        if !group_exists.unwrap_or(false) {
            return Err(anyhow!("mode_group_id '{}' does not exist", mode_group_id));
        }

        let modes = Self::get_by_mode_group_id(db, mode_group_id)
            .await
            .with_context(|| format!("Failed to fetch modes for group {}", mode_group_id))?;

        Ok(modes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Helper to create a test mode group for testing
    async fn create_test_mode_group(pool: &PgPool) -> sqlx::Result<Uuid> {
        let unique_name = format!("Test Group {}", Uuid::new_v4());
        let result = sqlx::query_scalar!(
            r#"INSERT INTO core.mode_group (mode_group_name, mode_group_description)
               VALUES ($1, 'Test Group Description')
               RETURNING mode_group_id"#,
            unique_name
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    // Helper to create a mode bypassing validation for test setup
    async fn create_mode_raw(
        pool: &PgPool,
        mode_group_id: Uuid,
        mode_description: &str,
    ) -> sqlx::Result<ModeRow> {
        sqlx::query_as!(
            ModeRow,
            r#"INSERT INTO core.mode (mode_group_id, mode_description)
               VALUES ($1, $2)
               RETURNING mode_id, mode_group_id, mode_description, created_at, updated_at"#,
            mode_group_id,
            mode_description
        )
        .fetch_one(pool)
        .await
    }

    #[sqlx::test]
    async fn test_create_mode(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let mode_description = "Test Mode Description";

        let mode = ModeRowQueries::create_mode(&pool, mode_group_id, mode_description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(mode.mode_group_id, mode_group_id);
        assert_eq!(mode.mode_description, mode_description);
        assert!(mode.mode_id != Uuid::nil());
        assert!(mode.created_at.is_some());

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_mode_validation_errors(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;

        // Test empty description
        let result = ModeRowQueries::create_mode(&pool, mode_group_id, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test whitespace-only description
        let result = ModeRowQueries::create_mode(&pool, mode_group_id, "   ").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test too long description
        let long_desc = "a".repeat(3000);
        let result = ModeRowQueries::create_mode(&pool, mode_group_id, &long_desc).await;
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
    async fn test_create_mode_invalid_group(pool: PgPool) -> sqlx::Result<()> {
        let invalid_group_id = Uuid::new_v4();

        let result =
            ModeRowQueries::create_mode(&pool, invalid_group_id, "Valid Description").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_mode_duplicate_description_in_group(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let mode_description = "Duplicate Description";

        // First creation should succeed
        let _first = ModeRowQueries::create_mode(&pool, mode_group_id, mode_description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Second creation with same description in same group should fail
        let result = ModeRowQueries::create_mode(&pool, mode_group_id, mode_description).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_mode_same_description_different_groups(pool: PgPool) -> sqlx::Result<()> {
        let group1_id = create_test_mode_group(&pool).await?;
        let group2_id = create_test_mode_group(&pool).await?;
        let mode_description = "Same Description";

        // Create mode in first group
        let _mode1 = ModeRowQueries::create_mode(&pool, group1_id, mode_description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Create mode with same description in second group - should succeed
        let _mode2 = ModeRowQueries::create_mode(&pool, group2_id, mode_description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        Ok(())
    }

    #[sqlx::test]
    async fn test_input_trimming(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;

        let created = ModeRowQueries::create_mode(&pool, mode_group_id, "  Trimmed Description  ")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created.mode_description, "Trimmed Description");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_all_modes(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;

        let mode1 = ModeRowQueries::create_mode(&pool, mode_group_id, "Mode 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let mode2 = ModeRowQueries::create_mode(&pool, mode_group_id, "Mode 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let all_modes = ModeRowQueries::get_all(&pool).await?;

        assert!(all_modes.len() >= 2);
        assert!(all_modes.iter().any(|m| m.mode_id == mode1.mode_id));
        assert!(all_modes.iter().any(|m| m.mode_id == mode2.mode_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_id(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let created = ModeRowQueries::create_mode(&pool, mode_group_id, "Test Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = ModeRowQueries::get_by_mode_id(&pool, created.mode_id).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.mode_id, created.mode_id);
        assert_eq!(found.mode_description, "Test Mode");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_id_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = ModeRowQueries::get_by_mode_id(&pool, random_id).await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_group_id(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;

        let mode1 = ModeRowQueries::create_mode(&pool, mode_group_id, "Mode 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let mode2 = ModeRowQueries::create_mode(&pool, mode_group_id, "Mode 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let modes = ModeRowQueries::get_by_mode_group_id(&pool, mode_group_id).await?;

        assert_eq!(modes.len(), 2);
        assert!(modes.iter().any(|m| m.mode_id == mode1.mode_id));
        assert!(modes.iter().any(|m| m.mode_id == mode2.mode_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_description(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let description = "Unique Mode Description";
        let created = ModeRowQueries::create_mode(&pool, mode_group_id, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = ModeRowQueries::get_by_mode_description(&pool, description).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.mode_id, created.mode_id);
        assert_eq!(found.mode_description, description);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_description(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let created = ModeRowQueries::create_mode(&pool, mode_group_id, "Original Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let new_description = "Updated Description";

        let updated =
            ModeRowQueries::update_mode_description(&pool, created.mode_id, new_description)
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.mode_id, created.mode_id);
        assert_eq!(updated.mode_description, new_description);
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_description_validation(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let created = ModeRowQueries::create_mode(&pool, mode_group_id, "Original Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test empty description
        let result = ModeRowQueries::update_mode_description(&pool, created.mode_id, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_description_duplicate(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let mode1 = ModeRowQueries::create_mode(&pool, mode_group_id, "First Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let mode2 = ModeRowQueries::create_mode(&pool, mode_group_id, "Second Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to update mode2 to have same description as mode1
        let result =
            ModeRowQueries::update_mode_description(&pool, mode2.mode_id, "First Description")
                .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group(pool: PgPool) -> sqlx::Result<()> {
        let group1_id = create_test_mode_group(&pool).await?;
        let group2_id = create_test_mode_group(&pool).await?;
        let created = ModeRowQueries::create_mode(&pool, group1_id, "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let updated = ModeRowQueries::update_mode_group(&pool, created.mode_id, group2_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.mode_id, created.mode_id);
        assert_eq!(updated.mode_group_id, group2_id);
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_invalid(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let created = ModeRowQueries::create_mode(&pool, mode_group_id, "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let invalid_group_id = Uuid::new_v4();

        let result =
            ModeRowQueries::update_mode_group(&pool, created.mode_id, invalid_group_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_conflict(pool: PgPool) -> sqlx::Result<()> {
        let group1_id = create_test_mode_group(&pool).await?;
        let group2_id = create_test_mode_group(&pool).await?;
        let description = "Same Description";

        // Create mode in group1
        let mode1 = ModeRowQueries::create_mode(&pool, group1_id, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Create mode in group2 with same description
        let _mode2 = ModeRowQueries::create_mode(&pool, group2_id, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to move mode1 to group2 - should fail due to description conflict
        let result = ModeRowQueries::update_mode_group(&pool, mode1.mode_id, group2_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_mode(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let created = ModeRowQueries::create_mode(&pool, mode_group_id, "To Delete")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let deleted = ModeRowQueries::delete_mode(&pool, created.mode_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(deleted);

        // Verify it's gone
        let found = ModeRowQueries::get_by_mode_id(&pool, created.mode_id).await?;
        assert!(found.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_mode_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let deleted = ModeRowQueries::delete_mode(&pool, random_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(!deleted);

        Ok(())
    }

    #[sqlx::test]
    async fn test_exists(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let created = ModeRowQueries::create_mode(&pool, mode_group_id, "Exists Test")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let exists = ModeRowQueries::exists(&pool, created.mode_id).await?;
        assert!(exists);

        let random_id = Uuid::new_v4();
        let not_exists = ModeRowQueries::exists(&pool, random_id).await?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_description_exists_in_group(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let description = "Exists Test Description";
        let _created = ModeRowQueries::create_mode(&pool, mode_group_id, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test exact match
        let exists =
            ModeRowQueries::description_exists_in_group(&pool, mode_group_id, description).await?;
        assert!(exists);

        // Test non-existent
        let not_exists =
            ModeRowQueries::description_exists_in_group(&pool, mode_group_id, "Does Not Exist")
                .await?;
        assert!(!not_exists);

        // Test different group
        let other_group_id = create_test_mode_group(&pool).await?;
        let not_exists_other =
            ModeRowQueries::description_exists_in_group(&pool, other_group_id, description).await?;
        assert!(!not_exists_other);

        Ok(())
    }

    #[sqlx::test]
    async fn test_search_by_description(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;

        // Create test data
        let _pump = ModeRowQueries::create_mode(&pool, mode_group_id, "Water Pump Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _motor = ModeRowQueries::create_mode(&pool, mode_group_id, "Electric Motor Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _valve = ModeRowQueries::create_mode(&pool, mode_group_id, "Control Valve Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Search for "pump"
        let pump_results = ModeRowQueries::search_by_description(&pool, "pump").await?;
        assert!(pump_results.len() >= 1);
        assert!(
            pump_results
                .iter()
                .any(|m| m.mode_description.contains("Pump"))
        );

        // Search for "motor"
        let motor_results = ModeRowQueries::search_by_description(&pool, "MOTOR").await?;
        assert!(motor_results.len() >= 1);
        assert!(
            motor_results
                .iter()
                .any(|m| m.mode_description.contains("Motor"))
        );

        // Search for non-existent term
        let empty_results = ModeRowQueries::search_by_description(&pool, "NonExistent").await?;
        assert!(empty_results.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_modes_for_group(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;

        let mode1 = ModeRowQueries::create_mode(&pool, mode_group_id, "Mode 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let mode2 = ModeRowQueries::create_mode(&pool, mode_group_id, "Mode 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let modes = ModeRowQueries::get_modes_for_group(&pool, mode_group_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(modes.len(), 2);
        assert!(modes.iter().any(|m| m.mode_id == mode1.mode_id));
        assert!(modes.iter().any(|m| m.mode_id == mode2.mode_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_modes_for_group_invalid(pool: PgPool) -> sqlx::Result<()> {
        let invalid_group_id = Uuid::new_v4();

        let result = ModeRowQueries::get_modes_for_group(&pool, invalid_group_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_preserves_timestamps(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_id = create_test_mode_group(&pool).await?;
        let created = ModeRowQueries::create_mode(&pool, mode_group_id, "Test Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let original_created_at = created.created_at;

        // Short delay to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let updated =
            ModeRowQueries::update_mode_description(&pool, created.mode_id, "Updated Mode")
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
    async fn test_multiple_operations_sequence(pool: PgPool) -> sqlx::Result<()> {
        // Test a sequence of operations to ensure they work together

        let group1_id = create_test_mode_group(&pool).await?;
        let group2_id = create_test_mode_group(&pool).await?;

        // Create
        let created = ModeRowQueries::create_mode(&pool, group1_id, "Sequence Test Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Verify it exists
        let exists = ModeRowQueries::exists(&pool, created.mode_id).await?;
        assert!(exists);

        // Verify description exists in group
        let desc_exists =
            ModeRowQueries::description_exists_in_group(&pool, group1_id, "Sequence Test Mode")
                .await?;
        assert!(desc_exists);

        // Update description
        let updated_desc = ModeRowQueries::update_mode_description(
            &pool,
            created.mode_id,
            "Updated Sequence Mode",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated_desc.is_some());

        // Update group
        let updated_group = ModeRowQueries::update_mode_group(&pool, created.mode_id, group2_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated_group.is_some());

        // Verify final state
        let final_state = ModeRowQueries::get_by_mode_id(&pool, created.mode_id).await?;
        assert!(final_state.is_some());
        let final_state = final_state.unwrap();
        assert_eq!(final_state.mode_description, "Updated Sequence Mode");
        assert_eq!(final_state.mode_group_id, group2_id);

        // Search should find it
        let search_results = ModeRowQueries::search_by_description(&pool, "sequence").await?;
        assert!(search_results.iter().any(|m| m.mode_id == created.mode_id));

        // Delete
        let deleted = ModeRowQueries::delete_mode(&pool, created.mode_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(deleted);

        // Verify it no longer exists
        let exists_after_delete = ModeRowQueries::exists(&pool, created.mode_id).await?;
        assert!(!exists_after_delete);

        Ok(())
    }
}
