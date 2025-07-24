use anyhow::{Context, anyhow};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, error, instrument};
use uuid::Uuid;

const MAX_NAME_LEN: usize = 255;
const MAX_DESC_LEN: usize = 2048;

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
	            FROM core.state_group
                ORDER BY state_group_name"#
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

    /// validates input strings for state group operations
    fn validate_input(name: &str, description: &str) -> anyhow::Result<(String, String)> {
        let name = name.trim().to_string();
        let desc = description.trim().to_string();

        if name.is_empty() {
            return Err(anyhow!("state_group_name cannot be empty"));
        }

        if desc.is_empty() {
            return Err(anyhow!("state_group_description cannot be empty"));
        }

        if name.len() > MAX_NAME_LEN {
            return Err(anyhow!(
                "state_group_name exceeds max length of {} characters",
                MAX_NAME_LEN
            ));
        }

        if desc.len() > MAX_DESC_LEN {
            return Err(anyhow!(
                "state_group_description exceeds max length of {} characters",
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

    #[instrument(skip(db), fields(name = %state_group_name))]
    pub async fn create_state_group(
        db: &PgPool,
        state_group_name: &str,
        state_group_description: &str,
    ) -> anyhow::Result<StateGroupRow> {
        let (name, desc) = Self::validate_input(state_group_name, state_group_description)?;

        // check for duplicate name
        if let Some(_) = sqlx::query_scalar!(
            "SELECT 1 FROM core.state_group WHERE state_group_name = $1",
            name
        )
        .fetch_optional(db)
        .await
        .context("Failed to check for duplicate state_group_name")?
        {
            error!("Rejected: duplicate state_group_name '{}'", name);
            return Err(anyhow!("state_group_name '{}' already exists", name));
        }

        debug!("Inserting state group '{}'", name);
        let result = sqlx::query_as!(
            StateGroupRow,
            r#"
            INSERT INTO core.state_group (state_group_name, state_group_description)
            VALUES ($1, $2)
            RETURNING state_group_id, state_group_name, state_group_description, created_at, updated_at
            "#,
            name,
            desc
        )
        .fetch_one(db)
        .await
        .with_context(|| format!("Failed to insert state group with name '{}'", name))?;

        debug!(?result, "Successfully inserted state group");
        Ok(result)
    }

    #[instrument(skip(db), fields(id = %state_group_id, name = %state_group_name))]
    pub async fn update_state_group_name(
        db: &PgPool,
        state_group_id: Uuid,
        state_group_name: &str,
    ) -> anyhow::Result<Option<StateGroupRow>> {
        let name = Self::validate_field("state_group_name", state_group_name, MAX_NAME_LEN)?;

        // check for duplicate name (excluding current record)
        if let Some(_) = sqlx::query_scalar!(
            "SELECT 1 FROM core.state_group WHERE state_group_name = $1 AND state_group_id != $2",
            name,
            state_group_id
        )
        .fetch_optional(db)
        .await
        .context("Failed to check for duplicate state_group_name")?
        {
            return Err(anyhow!("state_group_name '{}' already exists", name));
        }

        let result = sqlx::query_as!(
            StateGroupRow,
            r#"UPDATE core.state_group 
               SET state_group_name = $2, updated_at = NOW()
               WHERE state_group_id = $1
               RETURNING state_group_id, state_group_name, state_group_description, created_at, updated_at"#,
            state_group_id,
            name
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update state group name for id {}", state_group_id))?;

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %state_group_id))]
    pub async fn update_state_group_description(
        db: &PgPool,
        state_group_id: Uuid,
        state_group_description: &str,
    ) -> anyhow::Result<Option<StateGroupRow>> {
        let desc = Self::validate_field(
            "state_group_description",
            state_group_description,
            MAX_DESC_LEN,
        )?;

        let result = sqlx::query_as!(
            StateGroupRow,
            r#"UPDATE core.state_group 
               SET state_group_description = $2, updated_at = NOW()
               WHERE state_group_id = $1
               RETURNING state_group_id, state_group_name, state_group_description, created_at, updated_at"#,
            state_group_id,
            desc
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update state group description for id {}", state_group_id))?;

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %state_group_id))]
    pub async fn delete_state_group(
        db: &PgPool,
        state_group_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
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

    /// Check if a state group name exists (case-sensitive)
    pub async fn name_exists(db: &PgPool, state_group_name: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.state_group WHERE state_group_name = $1
            )"#,
            state_group_name.trim()
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Search state groups by name (case-insensitive)
    pub async fn search_by_name(
        db: &PgPool,
        search_term: &str,
    ) -> Result<Vec<StateGroupRow>, sqlx::Error> {
        let pattern = format!("%{}%", search_term.trim().to_lowercase());

        sqlx::query_as!(
            StateGroupRow,
            r#"SELECT state_group_id, state_group_name, state_group_description, created_at, updated_at
               FROM core.state_group
               WHERE lower(state_group_name::text COLLATE "C") LIKE $1
               ORDER BY state_group_name"#,
            pattern
        )
        .fetch_all(db)
        .await
    }

    /// Search state groups by description (case-insensitive)
    pub async fn search_by_description(
        db: &PgPool,
        search_term: &str,
    ) -> Result<Vec<StateGroupRow>, sqlx::Error> {
        let pattern = format!("%{}%", search_term.trim().to_lowercase());

        sqlx::query_as!(
            StateGroupRow,
            r#"SELECT state_group_id, state_group_name, state_group_description, created_at, updated_at
               FROM core.state_group
               WHERE lower(state_group_description::text COLLATE "C") LIKE $1
               ORDER BY state_group_name"#,
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

    // helper to create state group bypassing validation for test setup
    async fn create_state_group_raw(
        pool: &PgPool,
        state_group_name: &str,
        state_group_description: &str,
    ) -> sqlx::Result<StateGroupRow> {
        let result = sqlx::query_as!(
            StateGroupRow,
            r#"INSERT INTO core.state_group (state_group_name, state_group_description)
               VALUES ($1, $2)
               RETURNING state_group_id, state_group_name, state_group_description, created_at, updated_at"#,
            state_group_name, state_group_description
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    #[sqlx::test]
    async fn test_create_state_group(pool: PgPool) -> sqlx::Result<()> {
        let state_group_name = "Test State Group";
        let state_group_description = "Test State Group Description";

        let new_state_group =
            StateGroupQueries::create_state_group(&pool, state_group_name, state_group_description)
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(new_state_group.state_group_name, state_group_name);
        assert_eq!(
            new_state_group.state_group_description,
            state_group_description
        );
        assert!(new_state_group.state_group_id != Uuid::nil());
        assert!(new_state_group.created_at.is_some());
        Ok(())
    }

    #[sqlx::test]
    async fn test_create_state_group_validation_errors(pool: PgPool) -> sqlx::Result<()> {
        // test empty name
        let result = StateGroupQueries::create_state_group(&pool, "", "Valid description").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // test empty description
        let result = StateGroupQueries::create_state_group(&pool, "Valid name", "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // test whitespace-only inputs
        let result = StateGroupQueries::create_state_group(&pool, "   ", "Valid description").await;
        assert!(result.is_err());

        let result = StateGroupQueries::create_state_group(&pool, "Valid name", "   ").await;
        assert!(result.is_err());

        // test too long name
        let long_name = "a".repeat(300);
        let result =
            StateGroupQueries::create_state_group(&pool, &long_name, "Valid description").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds max length")
        );

        // test too long description
        let long_desc = "a".repeat(3000);
        let result = StateGroupQueries::create_state_group(&pool, "Valid name", &long_desc).await;
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
    async fn test_create_state_group_duplicate_name(pool: PgPool) -> sqlx::Result<()> {
        let group_name = "Duplicate Test State Group";

        // first creation should succeed
        let _first = StateGroupQueries::create_state_group(&pool, group_name, "Description 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // second creation with same name should fail
        let result =
            StateGroupQueries::create_state_group(&pool, group_name, "Description 2").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_all_state_groups(pool: PgPool) -> sqlx::Result<()> {
        let state_group_1 = StateGroupQueries::create_state_group(
            &pool,
            "Test State Group 1",
            "Test State Group 1 Description",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let state_group_2 = StateGroupQueries::create_state_group(
            &pool,
            "Test State Group 2",
            "Test State Group 2 Description",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let all_state_groups = StateGroupQueries::get_all(&pool).await?;

        assert!(all_state_groups.len() >= 2);
        assert!(
            all_state_groups
                .iter()
                .any(|e| e.state_group_id == state_group_1.state_group_id)
        );
        assert!(
            all_state_groups
                .iter()
                .any(|e| e.state_group_id == state_group_2.state_group_id)
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_group_id(pool: PgPool) -> sqlx::Result<()> {
        let created =
            StateGroupQueries::create_state_group(&pool, "Test State Group", "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = StateGroupQueries::get_by_state_group_id(&pool, created.state_group_id).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.state_group_id, created.state_group_id);
        assert_eq!(found.state_group_name, "Test State Group");
        assert_eq!(found.state_group_description, "Test Description");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_group_id_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = StateGroupQueries::get_by_state_group_id(&pool, random_id).await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_group_name(pool: PgPool) -> sqlx::Result<()> {
        let group_name = "Unique Test State Group Name";
        let created = StateGroupQueries::create_state_group(&pool, group_name, "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = StateGroupQueries::get_by_state_group_name(&pool, group_name).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.state_group_id, created.state_group_id);
        assert_eq!(found.state_group_name, group_name);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_group_name_not_found(pool: PgPool) -> sqlx::Result<()> {
        let result =
            StateGroupQueries::get_by_state_group_name(&pool, "Non-existent State Group").await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_group_description(pool: PgPool) -> sqlx::Result<()> {
        let description = "Unique Test State Description";
        let created = StateGroupQueries::create_state_group(&pool, "Test State Group", description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = StateGroupQueries::get_by_state_group_description(&pool, description).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.state_group_id, created.state_group_id);
        assert_eq!(found.state_group_description, description);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_group_description_not_found(pool: PgPool) -> sqlx::Result<()> {
        let result = StateGroupQueries::get_by_state_group_description(
            &pool,
            "Non-existent State Description",
        )
        .await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_group_name(pool: PgPool) -> sqlx::Result<()> {
        let created =
            StateGroupQueries::create_state_group(&pool, "Original State Name", "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let new_name = "Updated State Name";

        let updated =
            StateGroupQueries::update_state_group_name(&pool, created.state_group_id, new_name)
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.state_group_id, created.state_group_id);
        assert_eq!(updated.state_group_name, new_name);
        assert_eq!(
            updated.state_group_description,
            created.state_group_description
        );
        // updated_at should be different
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_group_name_validation(pool: PgPool) -> sqlx::Result<()> {
        let created =
            StateGroupQueries::create_state_group(&pool, "Original State Name", "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test empty name
        let result =
            StateGroupQueries::update_state_group_name(&pool, created.state_group_id, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test whitespace-only name
        let result =
            StateGroupQueries::update_state_group_name(&pool, created.state_group_id, "   ").await;
        assert!(result.is_err());

        // Test too long name
        let long_name = "a".repeat(300);
        let result =
            StateGroupQueries::update_state_group_name(&pool, created.state_group_id, &long_name)
                .await;
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
    async fn test_update_state_group_name_duplicate(pool: PgPool) -> sqlx::Result<()> {
        let first =
            StateGroupQueries::create_state_group(&pool, "First State Group", "First Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let second = StateGroupQueries::create_state_group(
            &pool,
            "Second State Group",
            "Second Description",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to update second group's name to match first group's name
        let result = StateGroupQueries::update_state_group_name(
            &pool,
            second.state_group_id,
            "First State Group",
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_group_name_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = StateGroupQueries::update_state_group_name(&pool, random_id, "New Name")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_group_description(pool: PgPool) -> sqlx::Result<()> {
        let created = StateGroupQueries::create_state_group(
            &pool,
            "Test State Group",
            "Original Description",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let new_description = "Updated State Description";

        let updated = StateGroupQueries::update_state_group_description(
            &pool,
            created.state_group_id,
            new_description,
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.state_group_id, created.state_group_id);
        assert_eq!(updated.state_group_description, new_description);
        assert_eq!(updated.state_group_name, created.state_group_name);
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_group_description_validation(pool: PgPool) -> sqlx::Result<()> {
        let created = StateGroupQueries::create_state_group(
            &pool,
            "Test State Group",
            "Original Description",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test empty description
        let result =
            StateGroupQueries::update_state_group_description(&pool, created.state_group_id, "")
                .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test whitespace-only description
        let result =
            StateGroupQueries::update_state_group_description(&pool, created.state_group_id, "   ")
                .await;
        assert!(result.is_err());

        // Test too long description
        let long_desc = "a".repeat(3000);
        let result = StateGroupQueries::update_state_group_description(
            &pool,
            created.state_group_id,
            &long_desc,
        )
        .await;
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
    async fn test_update_state_group_description_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result =
            StateGroupQueries::update_state_group_description(&pool, random_id, "New Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_state_group(pool: PgPool) -> sqlx::Result<()> {
        let created =
            StateGroupQueries::create_state_group(&pool, "To Delete State", "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let deleted = StateGroupQueries::delete_state_group(&pool, created.state_group_id).await?;
        assert!(deleted);

        // Verify it's gone
        let found = StateGroupQueries::get_by_state_group_id(&pool, created.state_group_id).await?;
        assert!(found.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_state_group_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let deleted = StateGroupQueries::delete_state_group(&pool, random_id).await?;

        assert!(!deleted);

        Ok(())
    }

    #[sqlx::test]
    async fn test_exists(pool: PgPool) -> sqlx::Result<()> {
        let created =
            StateGroupQueries::create_state_group(&pool, "Exists Test State", "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let exists = StateGroupQueries::exists(&pool, created.state_group_id).await?;
        assert!(exists);

        let random_id = Uuid::new_v4();
        let not_exists = StateGroupQueries::exists(&pool, random_id).await?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_name_exists(pool: PgPool) -> sqlx::Result<()> {
        let state_group_name = "Exists Test State Group";
        let _created =
            StateGroupQueries::create_state_group(&pool, state_group_name, "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test exact match
        let exists = StateGroupQueries::name_exists(&pool, state_group_name).await?;
        assert!(exists);

        // Test non-existent
        let not_exists = StateGroupQueries::name_exists(&pool, "Does Not Exist State").await?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_search_by_name(pool: PgPool) -> sqlx::Result<()> {
        // Create test data
        let _pump =
            StateGroupQueries::create_state_group(&pool, "Pump State Group", "Pump related states")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _motor = StateGroupQueries::create_state_group(
            &pool,
            "Motor State Group",
            "Motor related states",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _valve = StateGroupQueries::create_state_group(
            &pool,
            "Valve State Group",
            "Valve related states",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Search for "pump"
        let pump_results = StateGroupQueries::search_by_name(&pool, "pump").await?;
        assert!(pump_results.len() >= 1);
        assert!(
            pump_results
                .iter()
                .any(|g| g.state_group_name.contains("Pump"))
        );

        // Search for "motor"
        let motor_results = StateGroupQueries::search_by_name(&pool, "MOTOR").await?;
        assert!(motor_results.len() >= 1);
        assert!(
            motor_results
                .iter()
                .any(|g| g.state_group_name.contains("Motor"))
        );

        // Search for non-existent term
        let empty_results = StateGroupQueries::search_by_name(&pool, "NonExistent").await?;
        assert!(empty_results.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn test_search_by_description(pool: PgPool) -> sqlx::Result<()> {
        // Create test data
        let _pump =
            StateGroupQueries::create_state_group(&pool, "Group A", "Pump related operations")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _motor =
            StateGroupQueries::create_state_group(&pool, "Group B", "Motor control operations")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _valve =
            StateGroupQueries::create_state_group(&pool, "Group C", "Valve positioning operations")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Search for "operations"
        let operations_results =
            StateGroupQueries::search_by_description(&pool, "operations").await?;
        assert!(operations_results.len() >= 3);

        // Search for "pump"
        let pump_results = StateGroupQueries::search_by_description(&pool, "PUMP").await?;
        assert!(pump_results.len() >= 1);
        assert!(
            pump_results
                .iter()
                .any(|g| g.state_group_description.contains("Pump"))
        );

        // Search for non-existent term
        let empty_results = StateGroupQueries::search_by_description(&pool, "NonExistent").await?;
        assert!(empty_results.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn test_input_trimming(pool: PgPool) -> sqlx::Result<()> {
        // Test that inputs are properly trimmed
        let created = StateGroupQueries::create_state_group(
            &pool,
            "  Trimmed State Name  ",
            "  Trimmed State Description  ",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created.state_group_name, "Trimmed State Name");
        assert_eq!(created.state_group_description, "Trimmed State Description");

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_preserves_timestamps(pool: PgPool) -> sqlx::Result<()> {
        let created =
            StateGroupQueries::create_state_group(&pool, "Test State Group", "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let original_created_at = created.created_at;

        // Short delay to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let updated = StateGroupQueries::update_state_group_name(
            &pool,
            created.state_group_id,
            "Updated State Name",
        )
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
        let created = StateGroupQueries::create_state_group(
            &pool,
            "Sequence Test State",
            "Original State Description",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Verify it exists
        let exists = StateGroupQueries::exists(&pool, created.state_group_id).await?;
        assert!(exists);

        // Verify name exists
        let name_exists = StateGroupQueries::name_exists(&pool, "Sequence Test State").await?;
        assert!(name_exists);

        // Update name
        let updated_name = StateGroupQueries::update_state_group_name(
            &pool,
            created.state_group_id,
            "Updated State Name",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated_name.is_some());

        // Update description
        let updated_desc = StateGroupQueries::update_state_group_description(
            &pool,
            created.state_group_id,
            "Updated State Description",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated_desc.is_some());

        // Verify final state
        let final_state =
            StateGroupQueries::get_by_state_group_id(&pool, created.state_group_id).await?;
        assert!(final_state.is_some());
        let final_state = final_state.unwrap();
        assert_eq!(final_state.state_group_name, "Updated State Name");
        assert_eq!(
            final_state.state_group_description,
            "Updated State Description"
        );

        // Search should find it
        let search_results = StateGroupQueries::search_by_name(&pool, "updated").await?;
        assert!(
            search_results
                .iter()
                .any(|g| g.state_group_id == created.state_group_id)
        );

        // Delete
        let deleted = StateGroupQueries::delete_state_group(&pool, created.state_group_id).await?;
        assert!(deleted);

        // Verify it no longer exists
        let exists_after_delete = StateGroupQueries::exists(&pool, created.state_group_id).await?;
        assert!(!exists_after_delete);

        Ok(())
    }
}
