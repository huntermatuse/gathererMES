use anyhow::{Context, anyhow};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, error, instrument};
use uuid::Uuid;

const MAX_DESC_LEN: usize = 2048;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StateRow {
    pub state_id: Uuid,
    pub state_group_id: Uuid,
    pub state_code: i32,
    pub state_description: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

pub struct StateRowQueries;

impl StateRowQueries {
    /// Validates and sanitizes state description input
    fn validate_state_description(state_description: &str) -> anyhow::Result<String> {
        let trimmed = state_description.trim().to_string();

        if trimmed.is_empty() {
            return Err(anyhow!("state_description cannot be empty"));
        }

        if trimmed.len() > MAX_DESC_LEN {
            return Err(anyhow!(
                "state_description exceeds max length of {} characters",
                MAX_DESC_LEN
            ));
        }

        Ok(trimmed)
    }

    /// Validates state code input
    fn validate_state_code(state_code: i32) -> anyhow::Result<i32> {
        if state_code < 0 {
            return Err(anyhow!("state_code cannot be negative"));
        }

        Ok(state_code)
    }

    pub async fn get_all(db: &PgPool) -> Result<Vec<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, state_code,
                state_description, created_at, updated_at
	            FROM core.state
                ORDER BY state_code, state_description"#
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_state_group_id(
        db: &PgPool,
        state_group_id: Uuid,
    ) -> Result<Vec<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, state_code, 
             state_description, created_at, updated_at
	            FROM core.state
                WHERE state_group_id = $1
                ORDER BY state_code, state_description"#,
            state_group_id
        )
        .fetch_all(db)
        .await
    }

    pub async fn get_by_state_id(
        db: &PgPool,
        state_id: Uuid,
    ) -> Result<Option<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, state_code,
             state_description, created_at, updated_at
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
            r#"SELECT state_id, state_group_id, state_code,
             state_description, created_at, updated_at
	            FROM core.state
                WHERE state_description = $1"#,
            state_description
        )
        .fetch_optional(db)
        .await
    }

    pub async fn get_by_state_code_and_group(
        db: &PgPool,
        state_group_id: Uuid,
        state_code: i32,
    ) -> Result<Option<StateRow>, sqlx::Error> {
        sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, state_code,
             state_description, created_at, updated_at
	            FROM core.state
                WHERE state_group_id = $1 AND state_code = $2"#,
            state_group_id,
            state_code
        )
        .fetch_optional(db)
        .await
    }

    #[instrument(skip(db), fields(group_id = %state_group_id, code = %state_code, description = %state_description))]
    pub async fn create_state(
        db: &PgPool,
        state_group_id: Uuid,
        state_code: i32,
        state_description: &str,
    ) -> anyhow::Result<StateRow> {
        let validated_description = Self::validate_state_description(state_description)?;
        let validated_code = Self::validate_state_code(state_code)?;

        // Check if state_group_id exists
        let group_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM core.state_group WHERE state_group_id = $1)",
            state_group_id
        )
        .fetch_one(db)
        .await
        .context("Failed to check if state_group exists")?;

        if !group_exists.unwrap_or(false) {
            error!("Rejected: state_group_id {} does not exist", state_group_id);
            return Err(anyhow!(
                "state_group_id '{}' does not exist",
                state_group_id
            ));
        }

        // Check for duplicate state_code within the same state group
        if let Some(_) = sqlx::query_scalar!(
            "SELECT 1 FROM core.state WHERE state_group_id = $1 AND state_code = $2",
            state_group_id,
            validated_code
        )
        .fetch_optional(db)
        .await
        .context("Failed to check for duplicate state_code in state_group")?
        {
            error!(
                "Rejected: duplicate state_code '{}' in state_group {}",
                validated_code, state_group_id
            );
            return Err(anyhow!(
                "state_code '{}' already exists in this state group",
                validated_code
            ));
        }

        // Check for duplicate description within the same state group
        if let Some(_) = sqlx::query_scalar!(
            "SELECT 1 FROM core.state WHERE state_group_id = $1 AND state_description = $2",
            state_group_id,
            validated_description
        )
        .fetch_optional(db)
        .await
        .context("Failed to check for duplicate state_description in state_group")?
        {
            error!(
                "Rejected: duplicate state_description '{}' in state_group {}",
                validated_description, state_group_id
            );
            return Err(anyhow!(
                "state_description '{}' already exists in this state group",
                validated_description
            ));
        }

        debug!(
            "Creating state with code {} and description '{}' in group {}",
            validated_code, validated_description, state_group_id
        );
        let result = sqlx::query_as!(
            StateRow,
            r#"INSERT INTO core.state (state_group_id, state_code, state_description)
               VALUES ($1, $2, $3)
               RETURNING state_id, state_group_id, state_code,
             state_description, created_at, updated_at"#,
            state_group_id,
            validated_code,
            validated_description
        )
        .fetch_one(db)
        .await
        .with_context(|| {
            format!(
                "Failed to create state with code {} and description '{}' in group {}",
                validated_code, validated_description, state_group_id
            )
        })?;

        debug!(?result, "Successfully created state");
        Ok(result)
    }

    #[instrument(skip(db), fields(id = %state_id, description = %state_description))]
    pub async fn update_state_description(
        db: &PgPool,
        state_id: Uuid,
        state_description: &str,
    ) -> anyhow::Result<Option<StateRow>> {
        let validated_description = Self::validate_state_description(state_description)?;

        // Get the current state to check for duplicate in same group
        let current_state = sqlx::query!(
            "SELECT state_group_id FROM core.state WHERE state_id = $1",
            state_id
        )
        .fetch_optional(db)
        .await
        .context("Failed to fetch current state")?;

        if let Some(current) = current_state {
            // Check for duplicate description within the same state group (excluding current record)
            if let Some(_) = sqlx::query_scalar!(
                "SELECT 1 FROM core.state WHERE state_group_id = $1 AND state_description = $2 AND state_id != $3",
                current.state_group_id,
                validated_description,
                state_id
            )
            .fetch_optional(db)
            .await
            .context("Failed to check for duplicate state_description")?
            {
                return Err(anyhow!(
                    "state_description '{}' already exists in this state group",
                    validated_description
                ));
            }
        }

        debug!(
            "Updating state {} to description '{}'",
            state_id, validated_description
        );
        let result = sqlx::query_as!(
            StateRow,
            r#"UPDATE core.state 
               SET state_description = $2, updated_at = NOW()
               WHERE state_id = $1
               RETURNING state_id, state_group_id, state_code,
             state_description, created_at, updated_at"#,
            state_id,
            validated_description
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update state description for id {}", state_id))?;

        if result.is_some() {
            debug!("Successfully updated state {}", state_id);
        } else {
            debug!("State {} not found for update", state_id);
        }

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %state_id, code = %state_code))]
    pub async fn update_state_code(
        db: &PgPool,
        state_id: Uuid,
        state_code: i32,
    ) -> anyhow::Result<Option<StateRow>> {
        let validated_code = Self::validate_state_code(state_code)?;

        // Get the current state to check for duplicate in same group
        let current_state = sqlx::query!(
            "SELECT state_group_id FROM core.state WHERE state_id = $1",
            state_id
        )
        .fetch_optional(db)
        .await
        .context("Failed to fetch current state")?;

        if let Some(current) = current_state {
            // Check for duplicate code within the same state group (excluding current record)
            if let Some(_) = sqlx::query_scalar!(
                "SELECT 1 FROM core.state WHERE state_group_id = $1 AND state_code = $2 AND state_id != $3",
                current.state_group_id,
                validated_code,
                state_id
            )
            .fetch_optional(db)
            .await
            .context("Failed to check for duplicate state_code")?
            {
                return Err(anyhow!(
                    "state_code '{}' already exists in this state group",
                    validated_code
                ));
            }
        }

        debug!("Updating state {} to code '{}'", state_id, validated_code);
        let result = sqlx::query_as!(
            StateRow,
            r#"UPDATE core.state 
               SET state_code = $2, updated_at = NOW()
               WHERE state_id = $1
               RETURNING state_id, state_group_id, state_code,
             state_description, created_at, updated_at"#,
            state_id,
            validated_code
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update state code for id {}", state_id))?;

        if result.is_some() {
            debug!("Successfully updated state {}", state_id);
        } else {
            debug!("State {} not found for update", state_id);
        }

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %state_id, new_group_id = %state_group_id))]
    pub async fn update_state_group(
        db: &PgPool,
        state_id: Uuid,
        state_group_id: Uuid,
    ) -> anyhow::Result<Option<StateRow>> {
        // Check if new state_group_id exists
        let group_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM core.state_group WHERE state_group_id = $1)",
            state_group_id
        )
        .fetch_one(db)
        .await
        .context("Failed to check if state_group exists")?;

        if !group_exists.unwrap_or(false) {
            return Err(anyhow!(
                "state_group_id '{}' does not exist",
                state_group_id
            ));
        }

        // Get current state to check for conflicts
        let current_state = sqlx::query!(
            "SELECT state_code, state_description FROM core.state WHERE state_id = $1",
            state_id
        )
        .fetch_optional(db)
        .await
        .context("Failed to fetch current state")?;

        if let Some(current) = current_state {
            // Check for duplicate code in the new state group
            if let Some(_) = sqlx::query_scalar!(
                "SELECT 1 FROM core.state WHERE state_group_id = $1 AND state_code = $2",
                state_group_id,
                current.state_code
            )
            .fetch_optional(db)
            .await
            .context("Failed to check for duplicate state_code in new group")?
            {
                return Err(anyhow!(
                    "state_code '{}' already exists in the target state group",
                    current.state_code
                ));
            }

            // Check for duplicate description in the new state group
            if let Some(_) = sqlx::query_scalar!(
                "SELECT 1 FROM core.state WHERE state_group_id = $1 AND state_description = $2",
                state_group_id,
                current.state_description
            )
            .fetch_optional(db)
            .await
            .context("Failed to check for duplicate state_description in new group")?
            {
                return Err(anyhow!(
                    "state_description '{}' already exists in the target state group",
                    current.state_description
                ));
            }
        }

        debug!("Updating state {} to group {}", state_id, state_group_id);
        let result = sqlx::query_as!(
            StateRow,
            r#"UPDATE core.state 
               SET state_group_id = $2, updated_at = NOW()
               WHERE state_id = $1
               RETURNING state_id, state_group_id, state_code,
             state_description, created_at, updated_at"#,
            state_id,
            state_group_id
        )
        .fetch_optional(db)
        .await
        .with_context(|| format!("Failed to update state group for id {}", state_id))?;

        if result.is_some() {
            debug!("Successfully updated state {} group", state_id);
        } else {
            debug!("State {} not found for update", state_id);
        }

        Ok(result)
    }

    #[instrument(skip(db), fields(id = %state_id))]
    pub async fn delete_state(db: &PgPool, state_id: Uuid) -> anyhow::Result<bool> {
        // TODO: Add check to see if the state is being used anywhere
        // This would prevent deletion of states that are in use

        debug!("Deleting state {}", state_id);
        let result = sqlx::query!("DELETE FROM core.state WHERE state_id = $1", state_id)
            .execute(db)
            .await
            .with_context(|| format!("Failed to delete state with id {}", state_id))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            debug!("Successfully deleted state {}", state_id);
        } else {
            debug!("State {} not found for deletion", state_id);
        }

        Ok(deleted)
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

    /// Check if a state code exists within a specific state group
    pub async fn code_exists_in_group(
        db: &PgPool,
        state_group_id: Uuid,
        state_code: i32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.state 
                WHERE state_group_id = $1 AND state_code = $2
            )"#,
            state_group_id,
            state_code
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Check if a state description exists within a specific state group
    pub async fn description_exists_in_group(
        db: &PgPool,
        state_group_id: Uuid,
        state_description: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM core.state 
                WHERE state_group_id = $1 AND state_description = $2
            )"#,
            state_group_id,
            state_description.trim()
        )
        .fetch_one(db)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Search states by description (case-insensitive)
    pub async fn search_by_description(
        db: &PgPool,
        search_term: &str,
    ) -> Result<Vec<StateRow>, sqlx::Error> {
        let pattern = format!("%{}%", search_term.trim().to_lowercase());

        sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, state_code, state_description, created_at, updated_at
               FROM core.state
               WHERE lower(state_description::text COLLATE "C") LIKE $1
               ORDER BY state_code, state_description"#,
            pattern
        )
        .fetch_all(db)
        .await
    }

    /// Get all states for a specific state group with validation
    pub async fn get_states_for_group(
        db: &PgPool,
        state_group_id: Uuid,
    ) -> anyhow::Result<Vec<StateRow>> {
        // Check if state_group exists first
        let group_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM core.state_group WHERE state_group_id = $1)",
            state_group_id
        )
        .fetch_one(db)
        .await
        .context("Failed to check if state_group exists")?;

        if !group_exists.unwrap_or(false) {
            return Err(anyhow!(
                "state_group_id '{}' does not exist",
                state_group_id
            ));
        }

        let states = Self::get_by_state_group_id(db, state_group_id)
            .await
            .with_context(|| format!("Failed to fetch states for group {}", state_group_id))?;

        Ok(states)
    }

    /// Get states by code range within a specific group
    pub async fn get_states_by_code_range(
        db: &PgPool,
        state_group_id: Uuid,
        min_code: i32,
        max_code: i32,
    ) -> anyhow::Result<Vec<StateRow>> {
        if min_code > max_code {
            return Err(anyhow!("min_code cannot be greater than max_code"));
        }

        // Check if state_group exists first
        let group_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM core.state_group WHERE state_group_id = $1)",
            state_group_id
        )
        .fetch_one(db)
        .await
        .context("Failed to check if state_group exists")?;

        if !group_exists.unwrap_or(false) {
            return Err(anyhow!(
                "state_group_id '{}' does not exist",
                state_group_id
            ));
        }

        let states = sqlx::query_as!(
            StateRow,
            r#"SELECT state_id, state_group_id, state_code, state_description, created_at, updated_at
               FROM core.state
               WHERE state_group_id = $1 AND state_code >= $2 AND state_code <= $3
               ORDER BY state_code"#,
            state_group_id,
            min_code,
            max_code
        )
        .fetch_all(db)
        .await
        .with_context(|| format!("Failed to fetch states by code range for group {}", state_group_id))?;

        Ok(states)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Helper to create a test state group for testing with unique name
    async fn create_test_state_group(pool: &PgPool) -> sqlx::Result<Uuid> {
        let unique_name = format!("Test State Group {}", Uuid::new_v4());
        let result = sqlx::query_scalar!(
            r#"INSERT INTO core.state_group (state_group_name, state_group_description)
               VALUES ($1, 'Test State Group Description')
               RETURNING state_group_id"#,
            unique_name
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    // Helper to create a state bypassing validation for test setup
    async fn create_state_raw(
        pool: &PgPool,
        state_group_id: Uuid,
        state_code: i32,
        state_description: &str,
    ) -> sqlx::Result<StateRow> {
        sqlx::query_as!(
            StateRow,
            r#"INSERT INTO core.state (state_group_id, state_code, state_description)
               VALUES ($1, $2, $3)
               RETURNING state_id, state_group_id, state_code, state_description, created_at, updated_at"#,
            state_group_id,
            state_code,
            state_description
        )
        .fetch_one(pool)
        .await
    }

    #[sqlx::test]
    async fn test_create_state(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let state_code = 100;
        let state_description = "Test State Description";

        let state =
            StateRowQueries::create_state(&pool, state_group_id, state_code, state_description)
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(state.state_group_id, state_group_id);
        assert_eq!(state.state_code, state_code);
        assert_eq!(state.state_description, state_description);
        assert!(state.state_id != Uuid::nil());
        assert!(state.created_at.is_some());

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_state_validation_errors(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;

        // Test empty description
        let result = StateRowQueries::create_state(&pool, state_group_id, 100, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test whitespace-only description
        let result = StateRowQueries::create_state(&pool, state_group_id, 100, "   ").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test too long description
        let long_desc = "a".repeat(3000);
        let result = StateRowQueries::create_state(&pool, state_group_id, 100, &long_desc).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds max length")
        );

        // Test negative state code
        let result =
            StateRowQueries::create_state(&pool, state_group_id, -1, "Valid Description").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot be negative")
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_state_invalid_group(pool: PgPool) -> sqlx::Result<()> {
        let invalid_group_id = Uuid::new_v4();

        let result =
            StateRowQueries::create_state(&pool, invalid_group_id, 100, "Valid Description").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_state_duplicate_code_in_group(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let state_code = 100;

        // First creation should succeed
        let _first =
            StateRowQueries::create_state(&pool, state_group_id, state_code, "First Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Second creation with same code in same group should fail
        let result =
            StateRowQueries::create_state(&pool, state_group_id, state_code, "Second Description")
                .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_state_duplicate_description_in_group(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let state_description = "Duplicate Description";

        // First creation should succeed
        let _first = StateRowQueries::create_state(&pool, state_group_id, 100, state_description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Second creation with same description in same group should fail
        let result =
            StateRowQueries::create_state(&pool, state_group_id, 200, state_description).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_state_same_values_different_groups(pool: PgPool) -> sqlx::Result<()> {
        let group1_id = create_test_state_group(&pool).await?;
        let group2_id = create_test_state_group(&pool).await?;
        let state_code = 100;
        let state_description = "Same Description";

        // Create state in first group
        let _state1 =
            StateRowQueries::create_state(&pool, group1_id, state_code, state_description)
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Create state with same code and description in second group - should succeed
        let _state2 =
            StateRowQueries::create_state(&pool, group2_id, state_code, state_description)
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        Ok(())
    }

    #[sqlx::test]
    async fn test_input_trimming(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;

        let created =
            StateRowQueries::create_state(&pool, state_group_id, 100, "  Trimmed Description  ")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created.state_description, "Trimmed Description");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_all_states(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;

        let state1 = StateRowQueries::create_state(&pool, state_group_id, 100, "State 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let state2 = StateRowQueries::create_state(&pool, state_group_id, 200, "State 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let all_states = StateRowQueries::get_all(&pool).await?;

        assert!(all_states.len() >= 2);
        assert!(all_states.iter().any(|s| s.state_id == state1.state_id));
        assert!(all_states.iter().any(|s| s.state_id == state2.state_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_id(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let created = StateRowQueries::create_state(&pool, state_group_id, 100, "Test State")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = StateRowQueries::get_by_state_id(&pool, created.state_id).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.state_id, created.state_id);
        assert_eq!(found.state_code, 100);
        assert_eq!(found.state_description, "Test State");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_id_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let result = StateRowQueries::get_by_state_id(&pool, random_id).await?;

        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_group_id(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;

        let state1 = StateRowQueries::create_state(&pool, state_group_id, 100, "State 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let state2 = StateRowQueries::create_state(&pool, state_group_id, 200, "State 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let states = StateRowQueries::get_by_state_group_id(&pool, state_group_id).await?;

        assert_eq!(states.len(), 2);
        assert!(states.iter().any(|s| s.state_id == state1.state_id));
        assert!(states.iter().any(|s| s.state_id == state2.state_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_description(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let description = "Unique State Description";
        let created = StateRowQueries::create_state(&pool, state_group_id, 100, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = StateRowQueries::get_by_state_description(&pool, description).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.state_id, created.state_id);
        assert_eq!(found.state_description, description);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_state_code_and_group(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let state_code = 100;
        let created =
            StateRowQueries::create_state(&pool, state_group_id, state_code, "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found =
            StateRowQueries::get_by_state_code_and_group(&pool, state_group_id, state_code).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.state_id, created.state_id);
        assert_eq!(found.state_code, state_code);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_description(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let created =
            StateRowQueries::create_state(&pool, state_group_id, 100, "Original Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let new_description = "Updated Description";

        let updated =
            StateRowQueries::update_state_description(&pool, created.state_id, new_description)
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.state_id, created.state_id);
        assert_eq!(updated.state_description, new_description);
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_description_validation(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let created =
            StateRowQueries::create_state(&pool, state_group_id, 100, "Original Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test empty description
        let result = StateRowQueries::update_state_description(&pool, created.state_id, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_description_duplicate(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let state1 = StateRowQueries::create_state(&pool, state_group_id, 100, "First Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let state2 =
            StateRowQueries::create_state(&pool, state_group_id, 200, "Second Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to update state2 to have same description as state1
        let result =
            StateRowQueries::update_state_description(&pool, state2.state_id, "First Description")
                .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_code(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let created = StateRowQueries::create_state(&pool, state_group_id, 100, "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let new_code = 200;

        let updated = StateRowQueries::update_state_code(&pool, created.state_id, new_code)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.state_id, created.state_id);
        assert_eq!(updated.state_code, new_code);
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_code_validation(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let created = StateRowQueries::create_state(&pool, state_group_id, 100, "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test negative code
        let result = StateRowQueries::update_state_code(&pool, created.state_id, -1).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot be negative")
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_code_duplicate(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let state1 = StateRowQueries::create_state(&pool, state_group_id, 100, "First Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let state2 =
            StateRowQueries::create_state(&pool, state_group_id, 200, "Second Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to update state2 to have same code as state1
        let result = StateRowQueries::update_state_code(&pool, state2.state_id, 100).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_group(pool: PgPool) -> sqlx::Result<()> {
        let group1_id = create_test_state_group(&pool).await?;
        let group2_id = create_test_state_group(&pool).await?;
        let created = StateRowQueries::create_state(&pool, group1_id, 100, "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let updated = StateRowQueries::update_state_group(&pool, created.state_id, group2_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.state_id, created.state_id);
        assert_eq!(updated.state_group_id, group2_id);
        assert!(updated.updated_at != created.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_group_invalid(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let created = StateRowQueries::create_state(&pool, state_group_id, 100, "Test Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let invalid_group_id = Uuid::new_v4();

        let result =
            StateRowQueries::update_state_group(&pool, created.state_id, invalid_group_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_group_code_conflict(pool: PgPool) -> sqlx::Result<()> {
        let group1_id = create_test_state_group(&pool).await?;
        let group2_id = create_test_state_group(&pool).await?;
        let state_code = 100;

        // Create state in group1
        let state1 = StateRowQueries::create_state(&pool, group1_id, state_code, "Description 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Create state in group2 with same code
        let _state2 = StateRowQueries::create_state(&pool, group2_id, state_code, "Description 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to move state1 to group2 - should fail due to code conflict
        let result = StateRowQueries::update_state_group(&pool, state1.state_id, group2_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_state_group_description_conflict(pool: PgPool) -> sqlx::Result<()> {
        let group1_id = create_test_state_group(&pool).await?;
        let group2_id = create_test_state_group(&pool).await?;
        let description = "Same Description";

        // Create state in group1
        let state1 = StateRowQueries::create_state(&pool, group1_id, 100, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Create state in group2 with same description
        let _state2 = StateRowQueries::create_state(&pool, group2_id, 200, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to move state1 to group2 - should fail due to description conflict
        let result = StateRowQueries::update_state_group(&pool, state1.state_id, group2_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_state(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let created = StateRowQueries::create_state(&pool, state_group_id, 100, "To Delete")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let deleted = StateRowQueries::delete_state(&pool, created.state_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(deleted);

        // Verify it's gone
        let found = StateRowQueries::get_by_state_id(&pool, created.state_id).await?;
        assert!(found.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_state_not_found(pool: PgPool) -> sqlx::Result<()> {
        let random_id = Uuid::new_v4();
        let deleted = StateRowQueries::delete_state(&pool, random_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(!deleted);

        Ok(())
    }

    #[sqlx::test]
    async fn test_exists(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let created = StateRowQueries::create_state(&pool, state_group_id, 100, "Exists Test")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let exists = StateRowQueries::exists(&pool, created.state_id).await?;
        assert!(exists);

        let random_id = Uuid::new_v4();
        let not_exists = StateRowQueries::exists(&pool, random_id).await?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_code_exists_in_group(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let state_code = 100;
        let _created =
            StateRowQueries::create_state(&pool, state_group_id, state_code, "Test Description")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test exact match
        let exists =
            StateRowQueries::code_exists_in_group(&pool, state_group_id, state_code).await?;
        assert!(exists);

        // Test non-existent
        let not_exists = StateRowQueries::code_exists_in_group(&pool, state_group_id, 999).await?;
        assert!(!not_exists);

        // Test different group
        let other_group_id = create_test_state_group(&pool).await?;
        let not_exists_other =
            StateRowQueries::code_exists_in_group(&pool, other_group_id, state_code).await?;
        assert!(!not_exists_other);

        Ok(())
    }

    #[sqlx::test]
    async fn test_description_exists_in_group(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let description = "Exists Test Description";
        let _created = StateRowQueries::create_state(&pool, state_group_id, 100, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Test exact match
        let exists =
            StateRowQueries::description_exists_in_group(&pool, state_group_id, description)
                .await?;
        assert!(exists);

        // Test non-existent
        let not_exists =
            StateRowQueries::description_exists_in_group(&pool, state_group_id, "Does Not Exist")
                .await?;
        assert!(!not_exists);

        // Test different group
        let other_group_id = create_test_state_group(&pool).await?;
        let not_exists_other =
            StateRowQueries::description_exists_in_group(&pool, other_group_id, description)
                .await?;
        assert!(!not_exists_other);

        Ok(())
    }

    #[sqlx::test]
    async fn test_search_by_description(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;

        // Create test data
        let _pump = StateRowQueries::create_state(&pool, state_group_id, 100, "Water Pump State")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _motor =
            StateRowQueries::create_state(&pool, state_group_id, 200, "Electric Motor State")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _valve =
            StateRowQueries::create_state(&pool, state_group_id, 300, "Control Valve State")
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Search for "pump"
        let pump_results = StateRowQueries::search_by_description(&pool, "pump").await?;
        assert!(pump_results.len() >= 1);
        assert!(
            pump_results
                .iter()
                .any(|s| s.state_description.contains("Pump"))
        );

        // Search for "motor"
        let motor_results = StateRowQueries::search_by_description(&pool, "MOTOR").await?;
        assert!(motor_results.len() >= 1);
        assert!(
            motor_results
                .iter()
                .any(|s| s.state_description.contains("Motor"))
        );

        // Search for non-existent term
        let empty_results = StateRowQueries::search_by_description(&pool, "NonExistent").await?;
        assert!(empty_results.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_states_for_group(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;

        let state1 = StateRowQueries::create_state(&pool, state_group_id, 100, "State 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let state2 = StateRowQueries::create_state(&pool, state_group_id, 200, "State 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let states = StateRowQueries::get_states_for_group(&pool, state_group_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(states.len(), 2);
        assert!(states.iter().any(|s| s.state_id == state1.state_id));
        assert!(states.iter().any(|s| s.state_id == state2.state_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_states_for_group_invalid(pool: PgPool) -> sqlx::Result<()> {
        let invalid_group_id = Uuid::new_v4();

        let result = StateRowQueries::get_states_for_group(&pool, invalid_group_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_states_by_code_range(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;

        let _state1 = StateRowQueries::create_state(&pool, state_group_id, 100, "State 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _state2 = StateRowQueries::create_state(&pool, state_group_id, 200, "State 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _state3 = StateRowQueries::create_state(&pool, state_group_id, 300, "State 3")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Get states in range 150-250
        let states = StateRowQueries::get_states_by_code_range(&pool, state_group_id, 150, 250)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(states.len(), 1);
        assert_eq!(states[0].state_code, 200);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_states_by_code_range_validation(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;

        // Test invalid range (min > max)
        let result =
            StateRowQueries::get_states_by_code_range(&pool, state_group_id, 200, 100).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot be greater than")
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_preserves_timestamps(pool: PgPool) -> sqlx::Result<()> {
        let state_group_id = create_test_state_group(&pool).await?;
        let created = StateRowQueries::create_state(&pool, state_group_id, 100, "Test State")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let original_created_at = created.created_at;

        // Short delay to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let updated =
            StateRowQueries::update_state_description(&pool, created.state_id, "Updated State")
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

        let group1_id = create_test_state_group(&pool).await?;
        let group2_id = create_test_state_group(&pool).await?;

        // Create
        let created = StateRowQueries::create_state(&pool, group1_id, 100, "Sequence Test State")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Verify it exists
        let exists = StateRowQueries::exists(&pool, created.state_id).await?;
        assert!(exists);

        // Verify code exists in group
        let code_exists = StateRowQueries::code_exists_in_group(&pool, group1_id, 100).await?;
        assert!(code_exists);

        // Verify description exists in group
        let desc_exists =
            StateRowQueries::description_exists_in_group(&pool, group1_id, "Sequence Test State")
                .await?;
        assert!(desc_exists);

        // Update description
        let updated_desc = StateRowQueries::update_state_description(
            &pool,
            created.state_id,
            "Updated Sequence State",
        )
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated_desc.is_some());

        // Update code
        let updated_code = StateRowQueries::update_state_code(&pool, created.state_id, 200)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated_code.is_some());

        // Update group
        let updated_group = StateRowQueries::update_state_group(&pool, created.state_id, group2_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(updated_group.is_some());

        // Verify final state
        let final_state = StateRowQueries::get_by_state_id(&pool, created.state_id).await?;
        assert!(final_state.is_some());
        let final_state = final_state.unwrap();
        assert_eq!(final_state.state_description, "Updated Sequence State");
        assert_eq!(final_state.state_code, 200);
        assert_eq!(final_state.state_group_id, group2_id);

        // Search should find it
        let search_results = StateRowQueries::search_by_description(&pool, "sequence").await?;
        assert!(
            search_results
                .iter()
                .any(|s| s.state_id == created.state_id)
        );

        // Delete
        let deleted = StateRowQueries::delete_state(&pool, created.state_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(deleted);

        // Verify it no longer exists
        let exists_after_delete = StateRowQueries::exists(&pool, created.state_id).await?;
        assert!(!exists_after_delete);

        Ok(())
    }
}
