use anyhow::{Context, anyhow};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, error, instrument};
use uuid::Uuid;

const MAX_NAME_LEN: usize = 255;
const MAX_DESC_LEN: usize = 2048;

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

    /// validates input strings for mode group operations
    fn validate_input(name: &str, description: &str) -> anyhow::Result<(String, String)> {
        let name = name.trim().to_string();
        let desc = description.trim().to_string();

        if name.is_empty() {
            return Err(anyhow!("mode_group_name cannot be empty"));
        }

        if desc.is_empty() {
            return Err(anyhow!("mode_group_description cannot be empty"));
        }

        if name.len() > MAX_NAME_LEN {
            return Err(anyhow!(
                "mode_group_name exceeds max length of {} characters",
                MAX_NAME_LEN
            ));
        }

        if desc.len() > MAX_DESC_LEN {
            return Err(anyhow!(
                "mode_group_description exceeds max length of {} characters",
                MAX_DESC_LEN
            ));
        }

        Ok((name, desc))
    }

    /// validates a single field name or description
    fn validate_field(field_name: &str, value: &str, max_len: usize) -> anyhow::Result<String> {
        let trimmed = value.trim().to_string();

        if trimmed.is_empty() {
            return Err(anyhow!("{} cannot be empty", field_name));
        }

        if trimmed.len() > max_len {
            return Err(anyhow!(
                "{} exceeds max length of {} characters",
                field_name,
                max_len
            ));
        }

        Ok(trimmed)
    }

    #[instrument(skip(db), fields(name = %mode_group_name))]
    pub async fn create_mode_group(
        db: &PgPool,
        mode_group_name: &str,
        mode_group_description: &str,
    ) -> anyhow::Result<ModeGroupRow> {
        let (name, desc) = Self::validate_input(mode_group_name, mode_group_description)?;

        // check for duplicate name
        if let Some(_) = sqlx::query_scalar!(
            "SELECT 1 FROM core.mode_group WHERE mode_group_name = $1",
            name
        )
        .fetch_optional(db)
        .await
        .context("Failed to check for duplicate mode_group_name")?
        {
            error!("Rejected: duplicate mode_group_name '{}'", name);
            return Err(anyhow!("mode_group_name '{}' already exists", name));
        }

        debug!("Inserting mode group '{}'", name);
        let result = sqlx::query_as!(
            ModeGroupRow,
            r#"
            INSERT INTO core.mode_group (mode_group_name, mode_group_description)
            VALUES ($1, $2)
            RETURNING mode_group_id, mode_group_name, mode_group_description, created_at, updated_at
            "#,
            name,
            desc
        )
        .fetch_one(db)
        .await
        .with_context(|| format!("Failed to insert mode group with name '{}'", name))?;

        debug!(?result, "Successfully inserted mode group");
        Ok(result)
    }

    #[instrument(skip(db), fields(id = %mode_group_id, name = %mode_group_name))]
    pub async fn update_mode_group_name(
        db: &PgPool,
        mode_group_id: Uuid,
        mode_group_name: &str,
    ) -> anyhow::Result<Option<ModeGroupRow>> {
        let name = Self::validate_field("mode_group_name", mode_group_name, MAX_NAME_LEN)?;

        // check for duplicate name (excluding current record)
        if let Some(_) = sqlx::query_scalar!(
            "SELECT 1 FROM core.mode_group WHERE mode_group_name = $1 AND mode_group_id != $2",
            name,
            mode_group_id
        )
        .fetch_optional(db)
        .await
        .context("Failed to check for duplicate mode_group_name")?
        {
            return Err(anyhow!("mode_group_name '{}' already exists", name));
        }

        let result = sqlx::query_as!(
            ModeGroupRow,
            r#"UPDATE core.mode_group 
               SET mode_group_name = $2, updated_at = NOW()
               WHERE mode_group_id = $1
               RETURNING mode_group_id, mode_group_name, mode_group_description, created_at, updated_at"#,
            mode_group_id,
            name
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update mode group name for id {}", mode_group_id))?;

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %mode_group_id))]
    pub async fn update_mode_group_description(
        db: &PgPool,
        mode_group_id: Uuid,
        mode_group_description: &str,
    ) -> anyhow::Result<Option<ModeGroupRow>> {
        let desc = Self::validate_field(
            "mode_group_description",
            mode_group_description,
            MAX_DESC_LEN,
        )?;

        let result = sqlx::query_as!(
            ModeGroupRow,
            r#"UPDATE core.mode_group 
               SET mode_group_description = $2, updated_at = NOW()
               WHERE mode_group_id = $1
               RETURNING mode_group_id, mode_group_name, mode_group_description, created_at, updated_at"#,
            mode_group_id,
            desc
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update mode group description for id {}", mode_group_id))?;

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %mode_group_id))]
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // helper to create a test group bypassing validation for test setup
    async fn create_test_mode_group_raw(
        pool: &PgPool,
        mode_group_name: &str,
        mode_group_description: &str,
    ) -> sqlx::Result<ModeGroupRow> {
        let result = sqlx::query_as!(
            ModeGroupRow,
            r#"INSERT INTO core.mode_group (mode_group_name, mode_group_description)
               VALUES ($1, $2)
               RETURNING mode_group_id, mode_group_name, mode_group_description, created_at, updated_at"#,
            mode_group_name, mode_group_description
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    #[sqlx::test]
    async fn test_create_mode_group(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_name = "Test Group";
        let mode_group_description = "Test Group Description";

        let new_mode_group =
            ModeGroupQueries::create_mode_group(&pool, mode_group_name, mode_group_description)
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(new_mode_group.mode_group_name, mode_group_name);
        assert_eq!(
            new_mode_group.mode_group_description,
            mode_group_description
        );
        assert!(new_mode_group.mode_group_id != Uuid::nil());
        assert!(new_mode_group.created_at.is_some());
        Ok(())
    }

    #[sqlx::test]
    async fn test_create_mode_group_validation_errors(pool: PgPool) -> sqlx::Result<()> {
        // test empty name
        let result = ModeGroupQueries::create_mode_group(&pool, "", "Valid description").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // test empty description
        let result = ModeGroupQueries::create_mode_group(&pool, "Valid name", "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // test whitespace-only inputs
        let result = ModeGroupQueries::create_mode_group(&pool, "   ", "Valid description").await;
        assert!(result.is_err());

        let result = ModeGroupQueries::create_mode_group(&pool, "Valid name", "   ").await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_mode_group_duplicate_name(pool: PgPool) -> sqlx::Result<()> {
        let group_name = "Duplicate Test Group";

        // first creation should succeed
        let _first = ModeGroupQueries::create_mode_group(&pool, group_name, "Description 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // second creation with same name should fail
        let result = ModeGroupQueries::create_mode_group(&pool, group_name, "Description 2").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_all_mode_groups(pool: PgPool) -> sqlx::Result<()> {
        let mode_group_1 =
            ModeGroupQueries::create_mode_group(&pool, "Test Group 1", "Test Group 1 Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let mode_group_2 =
            ModeGroupQueries::create_mode_group(&pool, "Test Group 2", "Test Group 2 Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let all_mode_groups = ModeGroupQueries::get_all(&pool).await?;

        assert!(all_mode_groups.len() >= 2);
        assert!(
            all_mode_groups
                .iter()
                .any(|e| e.mode_group_id == mode_group_1.mode_group_id)
        );
        assert!(
            all_mode_groups
                .iter()
                .any(|e| e.mode_group_id == mode_group_2.mode_group_id)
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_group_id(pool: PgPool) -> sqlx::Result<()> {
        let created = ModeGroupQueries::create_mode_group(&pool, "Test Group", "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = ModeGroupQueries::get_by_mode_group_id(&pool, created.mode_group_id).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.mode_group_id, created.mode_group_id);
        assert_eq!(found.mode_group_name, "Test Group");
        assert_eq!(found.mode_group_description, "Test Description");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_group_id_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = ModeGroupQueries::get_by_mode_group_id(&pool, random_id).await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_group_name(pool: PgPool) -> sqlx::Result<()> {
        let group_name = "Unique Test Group Name";
        let created = ModeGroupQueries::create_mode_group(&pool, group_name, "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = ModeGroupQueries::get_by_mode_group_name(&pool, group_name).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.mode_group_id, created.mode_group_id);
        assert_eq!(found.mode_group_name, group_name);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_group_name_not_found(pool: PgPool) -> sqlx::Result<()> {
        let result = ModeGroupQueries::get_by_mode_group_name(&pool, "Non-existent Group").await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_group_description(pool: PgPool) -> sqlx::Result<()> {
        let description = "Unique Test Description";
        let created = ModeGroupQueries::create_mode_group(&pool, "Test Group", description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = ModeGroupQueries::get_by_mode_group_description(&pool, description).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.mode_group_id, created.mode_group_id);
        assert_eq!(found.mode_group_description, description);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_group_description_not_found(pool: PgPool) -> sqlx::Result<()> {
        let result =
            ModeGroupQueries::get_by_mode_group_description(&pool, "Non-existent Description")
                .await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_name(pool: PgPool) -> sqlx::Result<()> {
        let created =
            ModeGroupQueries::create_mode_group(&pool, "Original Name", "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let new_name = "Updated Name";

        let updated =
            ModeGroupQueries::update_mode_group_name(&pool, created.mode_group_id, new_name)
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.mode_group_id, created.mode_group_id);
        assert_eq!(updated.mode_group_name, new_name);
        assert_eq!(
            updated.mode_group_description,
            created.mode_group_description
        );
        // updated_at should be different
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_name_validation(pool: PgPool) -> sqlx::Result<()> {
        let created =
            ModeGroupQueries::create_mode_group(&pool, "Original Name", "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test empty name
        let result =
            ModeGroupQueries::update_mode_group_name(&pool, created.mode_group_id, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test whitespace-only name
        let result =
            ModeGroupQueries::update_mode_group_name(&pool, created.mode_group_id, "   ").await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_name_duplicate(pool: PgPool) -> sqlx::Result<()> {
        let first = ModeGroupQueries::create_mode_group(&pool, "First Group", "First Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let second =
            ModeGroupQueries::create_mode_group(&pool, "Second Group", "Second Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to update second group's name to match first group's name
        let result =
            ModeGroupQueries::update_mode_group_name(&pool, second.mode_group_id, "First Group")
                .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_name_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = ModeGroupQueries::update_mode_group_name(&pool, random_id, "New Name")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_description(pool: PgPool) -> sqlx::Result<()> {
        let created =
            ModeGroupQueries::create_mode_group(&pool, "Test Group", "Original Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let new_description = "Updated Description";

        let updated = ModeGroupQueries::update_mode_group_description(
            &pool,
            created.mode_group_id,
            new_description,
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.mode_group_id, created.mode_group_id);
        assert_eq!(updated.mode_group_description, new_description);
        assert_eq!(updated.mode_group_name, created.mode_group_name);
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_description_validation(pool: PgPool) -> sqlx::Result<()> {
        let created =
            ModeGroupQueries::create_mode_group(&pool, "Test Group", "Original Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test empty description
        let result =
            ModeGroupQueries::update_mode_group_description(&pool, created.mode_group_id, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group_description_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result =
            ModeGroupQueries::update_mode_group_description(&pool, random_id, "New Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_mode_group(pool: PgPool) -> sqlx::Result<()> {
        let created = ModeGroupQueries::create_mode_group(&pool, "To Delete", "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let deleted = ModeGroupQueries::delete_mode_group(&pool, created.mode_group_id).await?;
        assert!(deleted);

        // Verify it's gone
        let found = ModeGroupQueries::get_by_mode_group_id(&pool, created.mode_group_id).await?;
        assert!(found.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_mode_group_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let deleted = ModeGroupQueries::delete_mode_group(&pool, random_id).await?;

        assert!(!deleted);

        Ok(())
    }

    #[sqlx::test]
    async fn test_exists(pool: PgPool) -> sqlx::Result<()> {
        let created = ModeGroupQueries::create_mode_group(&pool, "Exists Test", "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let exists = ModeGroupQueries::exists(&pool, created.mode_group_id).await?;
        assert!(exists);

        let random_id = Uuid::new_v4();
        let not_exists = ModeGroupQueries::exists(&pool, random_id).await?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_input_trimming(pool: PgPool) -> sqlx::Result<()> {
        // Test that inputs are properly trimmed
        let created = ModeGroupQueries::create_mode_group(
            &pool,
            "  Trimmed Name  ",
            "  Trimmed Description  ",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created.mode_group_name, "Trimmed Name");
        assert_eq!(created.mode_group_description, "Trimmed Description");

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_preserves_timestamps(pool: PgPool) -> sqlx::Result<()> {
        let created = ModeGroupQueries::create_mode_group(&pool, "Test Group", "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let original_created_at = created.created_at;

        // Short delay to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let updated =
            ModeGroupQueries::update_mode_group_name(&pool, created.mode_group_id, "Updated Name")
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

        // Create
        let created =
            ModeGroupQueries::create_mode_group(&pool, "Sequence Test", "Original Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Verify it exists
        let exists = ModeGroupQueries::exists(&pool, created.mode_group_id).await?;
        assert!(exists);

        // Update name
        let updated_name =
            ModeGroupQueries::update_mode_group_name(&pool, created.mode_group_id, "Updated Name")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated_name.is_some());

        // Update description
        let updated_desc = ModeGroupQueries::update_mode_group_description(
            &pool,
            created.mode_group_id,
            "Updated Description",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated_desc.is_some());

        // Verify final state
        let final_state =
            ModeGroupQueries::get_by_mode_group_id(&pool, created.mode_group_id).await?;
        assert!(final_state.is_some());
        let final_state = final_state.unwrap();
        assert_eq!(final_state.mode_group_name, "Updated Name");
        assert_eq!(final_state.mode_group_description, "Updated Description");

        // Delete
        let deleted = ModeGroupQueries::delete_mode_group(&pool, created.mode_group_id).await?;
        assert!(deleted);

        // Verify it no longer exists
        let exists_after_delete = ModeGroupQueries::exists(&pool, created.mode_group_id).await?;
        assert!(!exists_after_delete);

        Ok(())
    }
}
